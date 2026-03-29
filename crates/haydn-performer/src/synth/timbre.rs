/// Instrument presets with characteristic harmonic profiles, ADSR shapes,
/// spectral evolution, attack transients, and expression parameters.
///
/// Each instrument produces a distinct timbre through additive synthesis —
/// summing sine waves at integer multiples of the fundamental, weighted by
/// the instrument's overtone series, with harmonics that evolve over the
/// note's lifetime (bright attack → mellow sustain).

/// Available instrument presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Instrument {
    Piano,
    Strings,
    Cello,
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
    /// These are the *sustain* harmonic levels.
    pub harmonics: &'static [f32],
    /// ADSR envelope parameters
    pub attack_ms: f32,
    pub decay_ms: f32,
    pub sustain_level: f32,
    pub release_ms: f32,
    /// ADSR curve steepness (1.0 = linear, 2.0+ = exponential)
    pub adsr_curve: f32,
    /// Vibrato: rate in Hz, depth as fraction of frequency, onset delay in ms
    pub vibrato_rate: f32,
    pub vibrato_depth: f32,
    pub vibrato_onset_ms: f32,
    /// Tremolo: rate in Hz, depth as amplitude fraction
    pub tremolo_rate: f32,
    pub tremolo_depth: f32,
    /// Spectral brightness multiplier at note attack (1.0 = same as sustain).
    /// Higher values make upper harmonics louder during attack/decay, then
    /// they fade to their sustain levels. This models the bright transient
    /// of a piano hammer or trumpet buzz.
    pub attack_brightness: f32,
    /// Duration in ms over which brightness fades from attack to sustain level
    pub brightness_decay_ms: f32,
    /// Attack noise: amplitude of filtered noise mixed into the attack phase.
    /// Models breathy/percussive onset (hammer thunk, bow scratch, breath).
    pub attack_noise: f32,
    /// Duration of noise burst in ms (only during attack/early decay)
    pub attack_noise_ms: f32,
    /// High-pass filter for noise: 0.0 = full band, 0.9 = mostly hiss.
    /// Piano hammer = low (~0.3), flute breath = high (~0.7).
    pub noise_highpass: f32,
    /// Inter-note gap in ms (tiny silence before non-legato notes).
    /// 0 = no gap (organ, legato). ~15-30ms for most instruments.
    pub articulation_gap_ms: f32,
}

impl Instrument {
    pub fn profile(self) -> InstrumentProfile {
        match self {
            Instrument::Piano => InstrumentProfile {
                // Piano: strong fundamental, harmonics decay roughly as 1/n
                harmonics: &[1.0, 0.5, 0.33, 0.25, 0.15, 0.10, 0.06, 0.04],
                attack_ms: 5.0,
                decay_ms: 200.0,
                sustain_level: 0.2,
                release_ms: 400.0,
                adsr_curve: 3.0,      // steep exponential — piano decays fast then rings
                vibrato_rate: 0.0,
                vibrato_depth: 0.0,
                vibrato_onset_ms: 0.0,
                tremolo_rate: 0.0,
                tremolo_depth: 0.0,
                attack_brightness: 3.0,  // hammer strike is very bright
                brightness_decay_ms: 80.0,
                attack_noise: 0.08,      // hammer thunk
                attack_noise_ms: 15.0,
                noise_highpass: 0.3,     // low, thumpy noise
                articulation_gap_ms: 10.0,
            },
            Instrument::Strings => InstrumentProfile {
                // Strings ensemble: rich overtones, slow bow
                harmonics: &[1.0, 0.7, 0.45, 0.35, 0.25, 0.18, 0.12, 0.08, 0.05, 0.03],
                attack_ms: 80.0,
                decay_ms: 60.0,
                sustain_level: 0.85,
                release_ms: 250.0,
                adsr_curve: 1.5,       // gentle curve — bowed entry
                vibrato_rate: 5.0,
                vibrato_depth: 0.0012,
                vibrato_onset_ms: 500.0,
                tremolo_rate: 0.0,
                tremolo_depth: 0.0,
                attack_brightness: 1.8,  // bow scratch brightens initial contact
                brightness_decay_ms: 120.0,
                attack_noise: 0.04,      // rosin/bow noise
                attack_noise_ms: 60.0,
                noise_highpass: 0.5,
                articulation_gap_ms: 5.0,  // legato-ish
            },
            Instrument::Cello => InstrumentProfile {
                // Cello: bowed string produces sawtooth-like waveform (all harmonics, ~1/n decay)
                // Strong low partials give warmth; gradual rolloff gives richness
                // Extended to 16 harmonics for the characteristic "body" of the cello
                harmonics: &[1.0, 0.90, 0.65, 0.50, 0.38, 0.28, 0.22, 0.17, 0.13, 0.10, 0.08, 0.06, 0.05, 0.04, 0.03, 0.025],
                attack_ms: 100.0,        // bow draw is slow and gradual
                decay_ms: 40.0,
                sustain_level: 0.92,     // very strong sustained bowed tone
                release_ms: 300.0,       // bow lift — gradual, not abrupt
                adsr_curve: 1.2,         // nearly linear — bow pressure is smooth
                vibrato_rate: 5.8,       // typical cello vibrato ~5.5-6Hz
                vibrato_depth: 0.0005,   // very narrow: ~±0.08 semitone — barely perceptible pitch shift
                vibrato_onset_ms: 800.0, // cellists wait almost a full second before adding vibrato
                tremolo_rate: 0.0,
                tremolo_depth: 0.0,
                attack_brightness: 1.3,  // very subtle — bow catches string gently
                brightness_decay_ms: 150.0,  // long mellow-out
                attack_noise: 0.025,     // very light rosin whisper
                attack_noise_ms: 80.0,   // spread over longer window
                noise_highpass: 0.25,    // low, warm rosin character
                articulation_gap_ms: 2.0,  // nearly seamless legato
            },
            Instrument::Flute => InstrumentProfile {
                // Flute: nearly sinusoidal
                harmonics: &[1.0, 0.15, 0.06, 0.02],
                attack_ms: 50.0,
                decay_ms: 30.0,
                sustain_level: 0.9,
                release_ms: 120.0,
                adsr_curve: 1.8,
                vibrato_rate: 5.5,
                vibrato_depth: 0.001,
                vibrato_onset_ms: 300.0,
                tremolo_rate: 0.0,
                tremolo_depth: 0.0,
                attack_brightness: 2.5,  // breathy onset is bright
                brightness_decay_ms: 60.0,
                attack_noise: 0.12,      // breath noise is prominent
                attack_noise_ms: 45.0,
                noise_highpass: 0.7,     // airy, hissy
                articulation_gap_ms: 20.0,
            },
            Instrument::Organ => InstrumentProfile {
                // Organ: pipe stops, almost no decay
                harmonics: &[1.0, 0.6, 0.0, 0.5, 0.0, 0.3, 0.0, 0.2],
                attack_ms: 20.0,
                decay_ms: 10.0,
                sustain_level: 0.95,
                release_ms: 60.0,
                adsr_curve: 1.2,        // nearly linear — pipes are consistent
                vibrato_rate: 0.0,
                vibrato_depth: 0.0,
                vibrato_onset_ms: 0.0,
                tremolo_rate: 6.0,
                tremolo_depth: 0.02,
                attack_brightness: 1.2,  // minimal brightness change
                brightness_decay_ms: 30.0,
                attack_noise: 0.03,      // pipe air chiff
                attack_noise_ms: 20.0,
                noise_highpass: 0.6,
                articulation_gap_ms: 0.0,  // organ is legato
            },
            Instrument::Clarinet => InstrumentProfile {
                // Clarinet: odd harmonics only
                harmonics: &[1.0, 0.0, 0.75, 0.0, 0.5, 0.0, 0.14, 0.0, 0.5, 0.0, 0.12, 0.0, 0.17],
                attack_ms: 35.0,
                decay_ms: 40.0,
                sustain_level: 0.85,
                release_ms: 90.0,
                adsr_curve: 2.0,
                vibrato_rate: 0.0,
                vibrato_depth: 0.0,
                vibrato_onset_ms: 0.0,
                tremolo_rate: 0.0,
                tremolo_depth: 0.0,
                attack_brightness: 2.0,  // reed buzz on attack
                brightness_decay_ms: 50.0,
                attack_noise: 0.06,      // reed noise
                attack_noise_ms: 25.0,
                noise_highpass: 0.4,
                articulation_gap_ms: 15.0,
            },
            Instrument::Trumpet => InstrumentProfile {
                // Trumpet: bright, strong higher harmonics
                harmonics: &[1.0, 0.8, 0.65, 0.55, 0.4, 0.3, 0.2, 0.12, 0.07],
                attack_ms: 25.0,
                decay_ms: 50.0,
                sustain_level: 0.8,
                release_ms: 120.0,
                adsr_curve: 2.5,        // punchy attack
                vibrato_rate: 5.0,
                vibrato_depth: 0.001,
                vibrato_onset_ms: 500.0,
                tremolo_rate: 0.0,
                tremolo_depth: 0.0,
                attack_brightness: 3.5,  // very bright brass buzz
                brightness_decay_ms: 100.0,
                attack_noise: 0.05,      // valve/buzz noise
                attack_noise_ms: 20.0,
                noise_highpass: 0.5,
                articulation_gap_ms: 25.0,
            },
        }
    }
}

/// Generate one sample of additive synthesis with spectral evolution.
///
/// `brightness` controls upper harmonic boost: 1.0 = use harmonics as-is,
/// >1.0 = boost higher harmonics (brighter). This models how real instruments
/// are spectrally richer at the moment of attack.
pub fn additive_sample_evolved(
    phase: f32,
    frequency: f32,
    sample_rate: u32,
    harmonics: &[f32],
    brightness: f32,
) -> f32 {
    let nyquist = sample_rate as f32 / 2.0;
    let two_pi = 2.0 * std::f32::consts::PI;

    let mut sum = 0.0f32;
    let mut weight_sum = 0.0f32;

    for (i, &amp) in harmonics.iter().enumerate() {
        let harmonic_num = (i + 1) as f32;
        let harmonic_freq = frequency * harmonic_num;
        if harmonic_freq >= nyquist {
            break;
        }
        // Scale higher harmonics by brightness factor:
        // harmonic 1 unaffected, harmonic N scaled by brightness^(log2(N))
        let brightness_scale = if i > 0 && brightness > 1.0 {
            brightness.powf((harmonic_num).ln() / 2.0_f32.ln())
        } else {
            1.0
        };
        let scaled_amp = amp * brightness_scale;
        sum += scaled_amp * (two_pi * harmonic_num * phase).sin();
        weight_sum += scaled_amp;
    }

    if weight_sum > 0.0 {
        sum / weight_sum
    } else {
        0.0
    }
}

/// Simple filtered noise generator for attack transients.
/// Uses a deterministic hash so it's reproducible per sample index.
pub struct NoiseGenerator {
    state: u32,
    highpass_prev: f32,
    highpass_coeff: f32,
    last_input: f32,
}

impl NoiseGenerator {
    pub fn new(highpass: f32) -> Self {
        Self {
            state: 0x12345678,
            highpass_prev: 0.0,
            highpass_coeff: highpass.clamp(0.0, 0.99),
            last_input: 0.0,
        }
    }

    pub fn sample(&mut self) -> f32 {
        // xorshift32 PRNG for fast white noise
        self.state ^= self.state << 13;
        self.state ^= self.state >> 17;
        self.state ^= self.state << 5;
        let white = (self.state as f32 / u32::MAX as f32) * 2.0 - 1.0;

        // Single-pole highpass: y[n] = alpha * (y[n-1] + x[n] - x[n-1])
        // highpass_coeff is alpha; higher alpha = more bass cut
        let output = self.highpass_coeff * (self.highpass_prev + white - self.last_input);
        self.highpass_prev = output;
        self.last_input = white;
        output
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
            Instrument::Cello,
            Instrument::Flute,
            Instrument::Organ,
            Instrument::Clarinet,
            Instrument::Trumpet,
        ];
        for inst in instruments {
            let p = inst.profile();
            assert!(!p.harmonics.is_empty(), "{:?} has empty harmonics", inst);
            assert!(p.harmonics[0] > 0.0, "{:?} fundamental is zero", inst);
            assert!(p.adsr_curve >= 1.0, "{:?} curve must be >= 1.0", inst);
            assert!(p.attack_brightness >= 1.0, "{:?} brightness must be >= 1.0", inst);
        }
    }

    #[test]
    fn test_additive_sample_in_range() {
        let profile = Instrument::Piano.profile();
        for i in 0..1000 {
            let phase = 440.0 * i as f32 / 44100.0;
            let s = additive_sample_evolved(phase, 440.0, 44100, profile.harmonics, 1.0);
            assert!(s >= -1.1 && s <= 1.1, "sample out of range: {}", s);
        }
    }

    #[test]
    fn test_brightness_makes_brighter() {
        let harmonics = &[1.0, 0.5, 0.3, 0.2, 0.1];
        // Sum squared differences over many phases to ensure brightness has an effect
        let mut diff_sum = 0.0f32;
        for i in 0..500 {
            let phase = 440.0 * i as f32 / 44100.0;
            let normal = additive_sample_evolved(phase, 440.0, 44100, harmonics, 1.0);
            let bright = additive_sample_evolved(phase, 440.0, 44100, harmonics, 3.0);
            diff_sum += (normal - bright).abs();
        }
        assert!(
            diff_sum > 0.1,
            "brightness should change the signal, total diff={}",
            diff_sum
        );
    }

    #[test]
    fn test_nyquist_filtering() {
        let harmonics = &[0.1, 0.1, 0.1, 0.1, 1.0];
        let freq = 10000.0;
        let s = additive_sample_evolved(0.25, freq, 44100, harmonics, 1.0);
        assert!(s.abs() < 0.5, "expected quiet signal, got {}", s);
    }

    #[test]
    fn test_clarinet_odd_harmonics_only() {
        let profile = Instrument::Clarinet.profile();
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
        let phase = 440.0 * 100.0 / 44100.0;
        let piano = additive_sample_evolved(phase, 440.0, 44100, Instrument::Piano.profile().harmonics, 1.0);
        let clarinet = additive_sample_evolved(phase, 440.0, 44100, Instrument::Clarinet.profile().harmonics, 1.0);
        let flute = additive_sample_evolved(phase, 440.0, 44100, Instrument::Flute.profile().harmonics, 1.0);

        assert!((piano - clarinet).abs() > 0.001, "piano and clarinet should differ");
        assert!((piano - flute).abs() > 0.001, "piano and flute should differ");
    }

    #[test]
    fn test_noise_generator_produces_output() {
        let mut ng = NoiseGenerator::new(0.5);
        let samples: Vec<f32> = (0..1000).map(|_| ng.sample()).collect();
        assert!(samples.iter().any(|&s| s > 0.01), "noise should have positive values");
        assert!(samples.iter().any(|&s| s < -0.01), "noise should have negative values");
    }

    #[test]
    fn test_noise_generator_varies_with_highpass() {
        // Different highpass settings should produce different spectral content
        let mut ng_low = NoiseGenerator::new(0.1);
        let mut ng_high = NoiseGenerator::new(0.9);
        let low_samples: Vec<f32> = (0..5000).map(|_| ng_low.sample()).collect();
        let high_samples: Vec<f32> = (0..5000).map(|_| ng_high.sample()).collect();

        // They should produce detectably different output
        let diff: f32 = low_samples.iter().zip(high_samples.iter())
            .map(|(a, b)| (a - b).abs())
            .sum();
        assert!(diff > 1.0, "different highpass settings should produce different noise");
    }
}
