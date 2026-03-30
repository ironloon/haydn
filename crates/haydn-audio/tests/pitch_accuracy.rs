use haydn_audio::pitch::{McLeodDetector, PitchDetector, YinDetector};

fn generate_sine(freq: f32, sample_rate: u32, num_samples: usize) -> Vec<f32> {
    (0..num_samples)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            (2.0 * std::f32::consts::PI * freq * t).sin()
        })
        .collect()
}

fn generate_harmonic(freq: f32, sample_rate: u32, num_samples: usize) -> Vec<f32> {
    (0..num_samples)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            let fundamental = (2.0 * std::f32::consts::PI * freq * t).sin();
            let second = 0.5 * (2.0 * std::f32::consts::PI * 2.0 * freq * t).sin();
            let third = 0.25 * (2.0 * std::f32::consts::PI * 3.0 * freq * t).sin();
            fundamental + second + third
        })
        .collect()
}

/// Test both algorithms at several frequencies across voice/guitar range.
#[test]
fn both_detectors_agree_on_common_notes() {
    let test_cases: &[(f32, u8, &str)] = &[
        (82.41, 40, "E2 (low guitar)"),
        (130.81, 48, "C3"),
        (220.0, 57, "A3"),
        (261.63, 60, "C4 (middle C)"),
        (329.63, 64, "E4"),
        (440.0, 69, "A4 (concert pitch)"),
        (523.25, 72, "C5"),
        (880.0, 81, "A5"),
        (1046.50, 84, "C6"),
    ];

    let mut yin = YinDetector::new(2048, 0.15);
    let mut mcleod = McLeodDetector::new(2048, 0.7);

    for &(freq, expected_midi, label) in test_cases {
        let samples = generate_sine(freq, 44100, 2048);

        let yin_result = yin.detect(&samples, 44100);
        let mcleod_result = mcleod.detect(&samples, 44100);

        let yin_est = yin_result.unwrap_or_else(|| panic!("YIN failed to detect {label}"));
        let mcleod_est =
            mcleod_result.unwrap_or_else(|| panic!("McLeod failed to detect {label}"));

        assert_eq!(
            yin_est.midi_note, expected_midi,
            "YIN: {label} should be MIDI {expected_midi}, got {}",
            yin_est.midi_note
        );
        assert_eq!(
            mcleod_est.midi_note, expected_midi,
            "McLeod: {label} should be MIDI {expected_midi}, got {}",
            mcleod_est.midi_note
        );
    }
}

/// Pure sine should give higher confidence than harmonic.
#[test]
fn pure_sine_higher_confidence_than_harmonic() {
    let pure = generate_sine(440.0, 44100, 2048);
    let harmonic = generate_harmonic(440.0, 44100, 2048);

    let mut yin = YinDetector::new(2048, 0.15);
    let pure_est = yin.detect(&pure, 44100).expect("should detect pure sine");
    let harm_est = yin.detect(&harmonic, 44100).expect("should detect harmonic");

    assert!(
        pure_est.confidence >= harm_est.confidence,
        "YIN: pure sine confidence ({}) should be >= harmonic confidence ({})",
        pure_est.confidence,
        harm_est.confidence
    );
}

/// Both detectors return None for silence.
#[test]
fn both_return_none_for_silence() {
    let silence = vec![0.0f32; 2048];

    let mut yin = YinDetector::new(2048, 0.15);
    let mut mcleod = McLeodDetector::new(2048, 0.7);

    assert!(yin.detect(&silence, 44100).is_none(), "YIN should return None for silence");
    assert!(mcleod.detect(&silence, 44100).is_none(), "McLeod should return None for silence");
}

/// Harmonic signal should still detect the fundamental, not a harmonic.
#[test]
fn harmonic_signal_detects_fundamental() {
    let samples = generate_harmonic(261.63, 44100, 2048);

    let mut yin = YinDetector::new(2048, 0.15);
    let mut mcleod = McLeodDetector::new(2048, 0.7);

    let yin_est = yin.detect(&samples, 44100).expect("YIN should detect C4 harmonic");
    let mcleod_est = mcleod.detect(&samples, 44100).expect("McLeod should detect C4 harmonic");

    assert_eq!(yin_est.midi_note, 60, "YIN should detect C4 fundamental");
    assert_eq!(mcleod_est.midi_note, 60, "McLeod should detect C4 fundamental");
}
