use haydn_audio::{AudioConfig, AudioMsg, NoteStateMachine, PitchEstimate};

fn silence_frame() -> Vec<f32> {
    vec![0.0f32; 512]
}

fn loud_sine_frame(freq: f32, offset: usize) -> Vec<f32> {
    (0..512)
        .map(|i| {
            let t = (i + offset) as f32 / 44100.0;
            (2.0 * std::f32::consts::PI * freq * t).sin() * 0.5
        })
        .collect()
}

fn pitch_est(note: u8, confidence: f32) -> PitchEstimate {
    PitchEstimate {
        frequency_hz: 440.0,
        midi_note: note,
        cents_offset: 0.0,
        confidence,
    }
}

fn collect_events(
    sm: &mut NoteStateMachine,
    frames: &[Vec<f32>],
    pitch: Option<&PitchEstimate>,
) -> Vec<AudioMsg> {
    let mut events = Vec::new();
    for frame in frames {
        events.extend(sm.process_frame(frame, pitch));
    }
    events
}

#[test]
fn silence_produces_no_events() {
    let mut sm = NoteStateMachine::new(&AudioConfig::default());
    let frames: Vec<_> = (0..10).map(|_| silence_frame()).collect();
    let events = collect_events(&mut sm, &frames, None);
    assert!(events.is_empty(), "silence should produce no events");
}

#[test]
fn onset_produces_exactly_one_note_on() {
    let mut sm = NoteStateMachine::new(&AudioConfig::default());

    // Silence baseline
    let silence_frames: Vec<_> = (0..5).map(|_| silence_frame()).collect();
    let _ = collect_events(&mut sm, &silence_frames, None);

    // Loud frames with pitch
    let est = pitch_est(69, 0.9);
    let loud_frames: Vec<_> = (0..10).map(|i| loud_sine_frame(440.0, i * 512)).collect();
    let events = collect_events(&mut sm, &loud_frames, Some(&est));

    let note_ons: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, AudioMsg::NoteOn { .. }))
        .collect();
    assert_eq!(note_ons.len(), 1, "should get exactly one NoteOn");
}

#[test]
fn sustained_note_no_repeated_note_on() {
    let mut sm = NoteStateMachine::new(&AudioConfig::default());

    // Silence then 30 frames of same note
    let silence_frames: Vec<_> = (0..5).map(|_| silence_frame()).collect();
    let _ = collect_events(&mut sm, &silence_frames, None);

    let est = pitch_est(69, 0.9);
    let loud_frames: Vec<_> = (0..30).map(|i| loud_sine_frame(440.0, i * 512)).collect();
    let events = collect_events(&mut sm, &loud_frames, Some(&est));

    let note_on_count = events
        .iter()
        .filter(|e| matches!(e, AudioMsg::NoteOn { .. }))
        .count();
    assert_eq!(
        note_on_count, 1,
        "held note should not produce repeated NoteOn"
    );
}

#[test]
fn note_off_when_signal_drops() {
    let mut sm = NoteStateMachine::new(&AudioConfig::default());

    // Silence baseline
    let silence_frames: Vec<_> = (0..5).map(|_| silence_frame()).collect();
    let _ = collect_events(&mut sm, &silence_frames, None);

    // Note on
    let est = pitch_est(69, 0.9);
    let loud_frames: Vec<_> = (0..10).map(|i| loud_sine_frame(440.0, i * 512)).collect();
    let mut events = collect_events(&mut sm, &loud_frames, Some(&est));

    // Signal drops to silence
    let silence_after: Vec<_> = (0..5).map(|_| silence_frame()).collect();
    events.extend(collect_events(&mut sm, &silence_after, None));

    let note_off_count = events
        .iter()
        .filter(|e| matches!(e, AudioMsg::NoteOff))
        .count();
    assert!(
        note_off_count >= 1,
        "should produce NoteOff when signal drops"
    );
}

#[test]
fn two_notes_separated_by_silence() {
    let mut sm = NoteStateMachine::new(&AudioConfig {
        min_note_ms: 10,
        ..AudioConfig::default()
    });

    let mut all_events = Vec::new();

    // Silence baseline
    let silence: Vec<_> = (0..5).map(|_| silence_frame()).collect();
    all_events.extend(collect_events(&mut sm, &silence, None));

    // First note: A4
    let est_a = pitch_est(69, 0.9);
    let loud_a: Vec<_> = (0..10).map(|i| loud_sine_frame(440.0, i * 512)).collect();
    all_events.extend(collect_events(&mut sm, &loud_a, Some(&est_a)));

    // Silence gap
    let gap: Vec<_> = (0..10).map(|_| silence_frame()).collect();
    all_events.extend(collect_events(&mut sm, &gap, None));

    // Second note: C5
    let est_c = pitch_est(72, 0.9);
    let loud_c: Vec<_> = (0..10).map(|i| loud_sine_frame(523.25, i * 512)).collect();
    all_events.extend(collect_events(&mut sm, &loud_c, Some(&est_c)));

    let note_ons: Vec<_> = all_events
        .iter()
        .filter(|e| matches!(e, AudioMsg::NoteOn { .. }))
        .collect();
    assert!(
        note_ons.len() >= 2,
        "should get NoteOn for both notes, got {}",
        note_ons.len()
    );
}

#[test]
fn debounce_prevents_rapid_fire() {
    let mut sm = NoteStateMachine::new(&AudioConfig {
        min_note_ms: 500, // very long debounce
        ..AudioConfig::default()
    });

    let mut all_events = Vec::new();

    // Silence baseline
    let silence: Vec<_> = (0..5).map(|_| silence_frame()).collect();
    all_events.extend(collect_events(&mut sm, &silence, None));

    // First note
    let est = pitch_est(69, 0.9);
    let loud: Vec<_> = (0..5).map(|i| loud_sine_frame(440.0, i * 512)).collect();
    all_events.extend(collect_events(&mut sm, &loud, Some(&est)));

    // Quick silence + new onset within debounce window
    let quick_silence: Vec<_> = (0..2).map(|_| silence_frame()).collect();
    all_events.extend(collect_events(&mut sm, &quick_silence, None));

    let est2 = pitch_est(72, 0.9);
    let loud2: Vec<_> = (0..5).map(|i| loud_sine_frame(523.25, i * 512)).collect();
    all_events.extend(collect_events(&mut sm, &loud2, Some(&est2)));

    // With 500ms debounce and ~11.6ms per frame, 12 frames ≈ 139ms < 500ms
    // So second note should be blocked by debounce
    let note_ons: Vec<_> = all_events
        .iter()
        .filter(|e| matches!(e, AudioMsg::NoteOn { .. }))
        .collect();
    assert!(
        note_ons.len() <= 1,
        "debounce should prevent rapid-fire NoteOn events, got {}",
        note_ons.len()
    );
}
