/// Instrument presets with characteristic harmonic profiles, ADSR shapes,
/// and expression parameters. Each instrument produces a distinct timbre
/// through additive synthesis — summing sine waves at integer multiples
/// of the fundamental, weighted by the instrument's overtone series.

/// Available instrument presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Instrument {
    Piano,
    Strings,
    Flute,
    Organ,
    Clarinet,
    Trumpet,
}

impl Default for Instrument {
    fn default() -> Self {
        Instrument::Piano
    }
}

/// Complete timbral profile for an instrument.
pub struct InstrumentProfile {
    /// Relative amplitudes for harmonics 1 (fundamental), 2, 3, ...
    /// The synthesis engine normalizes these so the sum doesn't clip.
    pub harmonics: &'static [f32],
    /// ADSR envelope parameters
    pub attack_ms: f32,
    pub decay_ms: f32,
    pub sustain_level: f32,
    pub release_ms: f32,
    /// Vibrato: rate in Hz, depth as fraction of frequency, onset delay in ms
    pub vibrato_rate: f32,
    pub vibrato_depth: f32,
    pub vibrato_onset_ms: f32,
    /// Tremolo: rate in Hz, depth as amplitude fraction
    pub tremolo_rate: f32,
    pub tremolo_depth: f32,
}

impl Instrument {
    pub fn profile(self) -> InstrumentProfile {
        match self {
            Instrument::Piano => InstrumentProfile {
                // Piano: strong fundamental, harmonics decay roughly as 1/n
                // with slight inharmonicity in upper partials
                harmonics: &[1.0, 0.5, 0.33, 0.25, 0.15, 0.10, 0.06, 0.04],
                attack_ms: 5.0,
                decay_ms: 150.0,
                sustain_level: 0.3,
                release_ms: 300.0,
                vibrato_rate: 0.0,
                vibrato_depth: 0.0,
                vibrato_onset_ms: 0.0,
                tremolo_rate: 0.0,
                tremolo_depth: 0.0,
            },
            Instrument::Strings => InstrumentProfile {
                // Strings: rich overtones, both even and odd, with slow bow decay
                harmonics: &[1.0, 0.7, 0.45, 0.35, 0.25, 0.18, 0.12, 0.08, 0.05, 0.03],
                attack_ms: 80.0,
                decay_ms: 60.0,
                sustain_level: 0.85,
                release_ms: 200.0,
                vibrato_rate: 5.0,
                vibrato_depth: 0.003,
                vibrato_onset_ms: 300.0,
                tremolo_rate: 0.0,
                tremolo_depth: 0.0,
            },
            Instrument::Flute => InstrumentProfile {
                // Flute: dominated by fundamental, very weak upper harmonics
                // (nearly sinusoidal with a hint of 2nd and 3rd)
                harmonics: &[1.0, 0.15, 0.06, 0.02],
                attack_ms: 40.0,
                decay_ms: 30.0,
                sustain_level: 0.9,
                release_ms: 100.0,
                vibrato_rate: 5.5,
                vibrato_depth: 0.003,
                vibrato_onset_ms: 200.0,
                tremolo_rate: 0.0,
                tremolo_depth: 0.0,
            },
            Instrument::Organ => InstrumentProfile {
                // Organ: specific "stops" — strong odd harmonics like a
                // principal pipe, with almost no decay
                harmonics: &[1.0, 0.6, 0.0, 0.5, 0.0, 0.3, 0.0, 0.2],
                attack_ms: 15.0,
                decay_ms: 10.0,
                sustain_level: 0.95,
                release_ms: 50.0,
                vibrato_rate: 0.0,
                vibrato_depth: 0.0,
                vibrato_onset_ms: 0.0,
                tremolo_rate: 6.0,
                tremolo_depth: 0.02,
            },
            Instrument::Clarinet => InstrumentProfile {
                // Clarinet: closed cylindrical bore produces predominantly odd harmonics.
                // Even harmonics are strongly suppressed.
                harmonics: &[1.0, 0.0, 0.75, 0.0, 0.5, 0.0, 0.14, 0.0, 0.5, 0.0, 0.12, 0.0, 0.17],
                attack_ms: 30.0,
                decay_ms: 40.0,
                sustain_level: 0.85,
                release_ms: 80.0,
                vibrato_rate: 0.0,
                vibrato_depth: 0.0,
                vibrato_onset_ms: 0.0,
                tremolo_rate: 0.0,
                tremolo_depth: 0.0,
            },
            Instrument::Trumpet => InstrumentProfile {
                // Trumpet: bright, strong higher harmonics from conical bore + bell
                harmonics: &[1.0, 0.8, 0.65, 0.55, 0.4, 0.3, 0.2, 0.12, 0.07],
                attack_ms: 20.0,
                decay_ms: 50.0,
                sustain_level: 0.8,
                release_ms: 100.0,
                vibrato_rate: 5.0,
                vibrato_depth: 0.002,
                vibrato_onset_ms: 400.0,
                tremolo_rate: 0.0,
                tremolo_depth: 0.0,
            },
        }
    }
}

/// Generate one sample of additive synthesis at the given phase and sample rate.
///
/// Sums sine waves at harmonic multiples of the fundamental, weighted by the
/// profile's amplitude coefficients. Automatically limits harmonics that would
/// exceed the Nyquist frequency to prevent aliasing.
pub fn additive_sample(phase: f32, frequency: f32, sample_rate: u32, harmonics: &[f32]) -> f32 {
    let nyquist = sample_rate as f32 / 2.0;
    let two_pi = 2.0 * std::f32::consts::PI;

    let mut sum = 0.0f32;
    let mut weight_sum = 0.0f32;

    for (i, &amp) in harmonics.iter().enumerate() {
        let harmonic_num = (i + 1) as f32;
        let harmonic_freq = frequency * harmonic_num;
        // Skip harmonics above Nyquist to prevent aliasing
        if harmonic_freq >= nyquist {
            break;
        }
        sum += amp * (two_pi * harmonic_num * phase).sin();
        weight_sum += amp;
    }

    if weight_sum > 0.0 {
        sum / weight_sum
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_instruments_have_profiles() {
        let instruments = [
            Instrument::Piano,
            Instrument::Strings,
            Instrument::Flute,
            Instrument::Organ,
            Instrument::Clarinet,
            Instrument::Trumpet,
        ];
        for inst in instruments {
            let p = inst.profile();
            assert!(!p.harmonics.is_empty(), "{:?} has empty harmonics", inst);
            assert!(p.harmonics[0] > 0.0, "{:?} fundamental is zero", inst);
        }
    }

    #[test]
    fn test_additive_sample_in_range() {
        let profile = Instrument::Piano.profile();
        for i in 0..1000 {
            let phase = 440.0 * i as f32 / 44100.0;
            let s = additive_sample(phase, 440.0, 44100, profile.harmonics);
            assert!(s >= -1.1 && s <= 1.1, "sample out of range: {}", s);
        }
    }

    #[test]
    fn test_nyquist_filtering() {
        // Very high frequency should produce near-silence as harmonics are filtered
        let harmonics = &[0.1, 0.1, 0.1, 0.1, 1.0]; // most energy in 5th harmonic
        let freq = 10000.0; // 5th harmonic = 50kHz > 22050 Nyquist
        let s = additive_sample(0.25, freq, 44100, harmonics);
        // Only lower harmonics survive, so the sample should be much quieter
        // than if all harmonics were included
        assert!(s.abs() < 0.5, "expected quiet signal, got {}", s);
    }

    #[test]
    fn test_clarinet_odd_harmonics_only() {
        let profile = Instrument::Clarinet.profile();
        // Even harmonics (indices 1, 3, 5, 7...) should be zero
        for (i, &amp) in profile.harmonics.iter().enumerate() {
            let harmonic_num = i + 1;
            if harmonic_num % 2 == 0 {
                assert_eq!(amp, 0.0, "clarinet harmonic {} should be zero", harmonic_num);
            }
        }
    }

    #[test]
    fn test_piano_no_vibrato() {
        let p = Instrument::Piano.profile();
        assert_eq!(p.vibrato_depth, 0.0);
        assert_eq!(p.vibrato_rate, 0.0);
    }

    #[test]
    fn test_instruments_differ() {
        // Different instruments should produce different samples at the same phase
        let phase = 440.0 * 100.0 / 44100.0;
        let piano = additive_sample(phase, 440.0, 44100, Instrument::Piano.profile().harmonics);
        let clarinet = additive_sample(phase, 440.0, 44100, Instrument::Clarinet.profile().harmonics);
        let flute = additive_sample(phase, 440.0, 44100, Instrument::Flute.profile().harmonics);

        assert!((piano - clarinet).abs() > 0.001, "piano and clarinet should differ");
        assert!((piano - flute).abs() > 0.001, "piano and flute should differ");
    }
}
