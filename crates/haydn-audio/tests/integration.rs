use haydn_audio::pitch::mcleod::McLeodDetector;
use haydn_audio::pitch::PitchDetector;
use haydn_audio::state_machine::NoteStateMachine;
use haydn_audio::types::{AudioConfig, AudioMsg};

/// Run the analysis pipeline on a buffer of samples, returning all AudioMsg events.
/// Simulates what the analysis thread in capture.rs does (without cpal/ringbuf).
fn run_pipeline(samples: &[f32], config: &AudioConfig) -> Vec<AudioMsg> {
    let mut detector = McLeodDetector::new(config.window_size, config.confidence_threshold);
    let mut state_machine = NoteStateMachine::new(config);
    let mut analysis_buffer = vec![0.0f32; config.window_size];
    let mut events = Vec::new();

    let mut offset = 0;
    while offset + config.hop_size <= samples.len() {
        // Shift buffer left by hop_size
        analysis_buffer.copy_within(config.hop_size.., 0);
        let fill_start = config.window_size - config.hop_size;
        let src = &samples[offset..offset + config.hop_size];
        analysis_buffer[fill_start..].copy_from_slice(src);

        let pitch = detector.detect(&analysis_buffer, config.sample_rate);
        let frame_events = state_machine.process_frame(
            &analysis_buffer[fill_start..],
            pitch.as_ref(),
        );
        events.extend(frame_events);
        offset += config.hop_size;
    }
    events
}

/// Generate a sine wave of given frequency and duration in samples.
fn generate_sine(freq: f32, num_samples: usize, sample_rate: u32, amplitude: f32) -> Vec<f32> {
    (0..num_samples)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            (2.0 * std::f32::consts::PI * freq * t).sin() * amplitude
        })
        .collect()
}

/// Generate a note sequence: alternating silence and sine tones.
fn generate_note_sequence(
    notes: &[(f32, f32)], // (frequency_hz, duration_seconds)
    silence_secs: f32,
    sample_rate: u32,
) -> Vec<f32> {
    let silence_len = (silence_secs * sample_rate as f32) as usize;
    let silence = vec![0.0f32; silence_len];
    let mut result = silence.clone(); // leading silence

    for (freq, dur) in notes {
        let tone_len = (*dur * sample_rate as f32) as usize;
        result.extend(generate_sine(*freq, tone_len, sample_rate, 0.5));
        result.extend(&silence);
    }
    result
}

#[test]
fn test_silence_produces_no_events() {
    let config = AudioConfig::default();
    let samples = vec![0.0f32; 44100 * 2]; // 2 seconds of silence
    let events = run_pipeline(&samples, &config);
    assert!(events.is_empty(), "silence should produce no events, got {events:?}");
}

#[test]
fn test_single_note_produces_note_on_and_off() {
    let config = AudioConfig::default();
    let samples = generate_note_sequence(&[(440.0, 1.0)], 0.5, config.sample_rate);
    let events = run_pipeline(&samples, &config);

    let note_ons: Vec<_> = events.iter().filter(|e| matches!(e, AudioMsg::NoteOn { .. })).collect();
    let note_offs: Vec<_> = events.iter().filter(|e| matches!(e, AudioMsg::NoteOff)).collect();

    assert_eq!(note_ons.len(), 1, "expected exactly 1 NoteOn, got {note_ons:?}");
    assert_eq!(note_offs.len(), 1, "expected exactly 1 NoteOff, got {note_offs:?}");

    match note_ons[0] {
        AudioMsg::NoteOn { note, confidence } => {
            assert_eq!(*note, 69, "expected MIDI note 69 (A4), got {note}");
            assert!(*confidence > 0.5, "confidence should be high for clean sine");
        }
        _ => unreachable!(),
    }
}

#[test]
fn test_two_notes_sequential() {
    let config = AudioConfig::default();
    let samples = generate_note_sequence(
        &[(440.0, 0.5), (523.25, 0.5)],
        0.3,
        config.sample_rate,
    );
    let events = run_pipeline(&samples, &config);

    let note_ons: Vec<_> = events.iter().filter(|e| matches!(e, AudioMsg::NoteOn { .. })).collect();
    let note_offs: Vec<_> = events.iter().filter(|e| matches!(e, AudioMsg::NoteOff)).collect();

    assert!(note_ons.len() >= 2, "expected at least 2 NoteOn events, got {note_ons:?}");
    assert!(note_offs.len() >= 2, "expected at least 2 NoteOff events, got {note_offs:?}");

    // First note should be A4 (69), second should be C5 (72)
    match note_ons[0] {
        AudioMsg::NoteOn { note, .. } => assert_eq!(*note, 69, "first note should be A4"),
        _ => unreachable!(),
    }
    match note_ons[1] {
        AudioMsg::NoteOn { note, .. } => assert_eq!(*note, 72, "second note should be C5"),
        _ => unreachable!(),
    }
}

#[test]
fn test_sustained_note_no_duplicates() {
    let config = AudioConfig::default();
    let samples = generate_sine(440.0, 44100 * 3, config.sample_rate, 0.5); // 3 seconds
    let events = run_pipeline(&samples, &config);

    let note_ons: Vec<_> = events.iter().filter(|e| matches!(e, AudioMsg::NoteOn { .. })).collect();
    assert_eq!(note_ons.len(), 1, "sustained note should produce exactly 1 NoteOn, got {note_ons:?}");
}

#[test]
fn test_low_frequency_e2() {
    let config = AudioConfig::default();
    // E2 = 82.41 Hz, needs longer duration for reliable detection at low freq
    let samples = generate_note_sequence(&[(82.41, 1.5)], 0.5, config.sample_rate);
    let events = run_pipeline(&samples, &config);

    let note_ons: Vec<_> = events.iter().filter(|e| matches!(e, AudioMsg::NoteOn { .. })).collect();
    assert!(!note_ons.is_empty(), "should detect low E2 note");

    match note_ons[0] {
        AudioMsg::NoteOn { note, .. } => {
            assert_eq!(*note, 40, "E2 should be MIDI note 40, got {note}");
        }
        _ => unreachable!(),
    }
}

#[test]
fn test_noise_below_gate() {
    let config = AudioConfig::default();
    // Very low amplitude random-ish noise (well below -40dB gate)
    let samples: Vec<f32> = (0..44100)
        .map(|i| {
            // Deterministic pseudo-noise using simple hash
            let x = ((i as u32).wrapping_mul(2654435761) >> 16) as f32 / 65536.0;
            (x - 0.5) * 0.0002 // amplitude ~0.0001, ≈ -80dB
        })
        .collect();
    let events = run_pipeline(&samples, &config);
    assert!(events.is_empty(), "noise below gate should produce no events, got {events:?}");
}
