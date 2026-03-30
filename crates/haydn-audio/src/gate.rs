/// Noise gate — classifies audio frames as signal or silence based on RMS energy.
pub struct NoiseGate {
    threshold_db: f32,
}

impl NoiseGate {
    pub fn new(threshold_db: f32) -> Self {
        Self { threshold_db }
    }

    /// Compute the RMS energy of a sample buffer in decibels.
    pub fn rms_db(samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return f32::NEG_INFINITY;
        }
        let sum_sq: f32 = samples.iter().map(|s| s * s).sum();
        let rms = (sum_sq / samples.len() as f32).sqrt();
        if rms == 0.0 {
            f32::NEG_INFINITY
        } else {
            20.0 * rms.log10()
        }
    }

    /// Returns true if the signal is above the noise floor (gate is open).
    pub fn is_active(&self, samples: &[f32]) -> bool {
        Self::rms_db(samples) > self.threshold_db
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn silence_is_not_active() {
        let gate = NoiseGate::new(-40.0);
        let silence = vec![0.0f32; 512];
        assert!(!gate.is_active(&silence));
    }

    #[test]
    fn loud_sine_is_active() {
        let gate = NoiseGate::new(-40.0);
        let samples: Vec<f32> = (0..512)
            .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 44100.0).sin() * 0.5)
            .collect();
        assert!(gate.is_active(&samples));
    }

    #[test]
    fn very_quiet_noise_is_not_active() {
        let gate = NoiseGate::new(-40.0);
        let samples: Vec<f32> = (0..512).map(|i| ((i * 997) % 1000) as f32 * 0.0000001).collect();
        assert!(!gate.is_active(&samples));
    }

    #[test]
    fn rms_db_silence_is_neg_infinity() {
        assert_eq!(NoiseGate::rms_db(&[0.0; 512]), f32::NEG_INFINITY);
    }

    #[test]
    fn rms_db_half_amplitude_sine_reasonable() {
        let samples: Vec<f32> = (0..44100)
            .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 44100.0).sin() * 0.5)
            .collect();
        let db = NoiseGate::rms_db(&samples);
        // 0.5 amplitude sine has RMS ≈ 0.354, so dB ≈ 20*log10(0.354) ≈ -9.0
        assert!(db > -12.0 && db < -6.0, "expected ~-9dB, got {db}");
    }
}
