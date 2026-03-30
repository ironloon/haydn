use super::{freq_to_midi, PitchDetector, PitchEstimate};

/// YIN pitch detection algorithm (de Cheveigné & Kawahara, 2002).
///
/// Estimates fundamental frequency from the autocorrelation of a signal using
/// cumulative mean normalized difference to eliminate octave errors.
pub struct YinDetector {
    window_size: usize,
    threshold: f32,
    diff: Vec<f32>,
    cmnd: Vec<f32>,
}

impl YinDetector {
    pub fn new(window_size: usize, threshold: f32) -> Self {
        let half = window_size / 2;
        Self {
            window_size,
            threshold,
            diff: vec![0.0; half],
            cmnd: vec![0.0; half],
        }
    }
}

impl PitchDetector for YinDetector {
    fn detect(&mut self, samples: &[f32], sample_rate: u32) -> Option<PitchEstimate> {
        let len = samples.len().min(self.window_size);
        if len < self.window_size {
            return None;
        }

        // Early bailout: check RMS of input
        let rms = {
            let sum_sq: f32 = samples[..len].iter().map(|s| s * s).sum();
            (sum_sq / len as f32).sqrt()
        };
        if rms < 0.01 {
            return None;
        }

        let half = self.window_size / 2;

        // Step 1: Difference function
        // d(τ) = Σ_{j=0}^{W/2-1} (x[j] - x[j+τ])²
        self.diff[0] = 0.0;
        for tau in 1..half {
            let mut sum = 0.0f32;
            for j in 0..half {
                let delta = samples[j] - samples[j + tau];
                sum += delta * delta;
            }
            self.diff[tau] = sum;
        }

        // Step 2: Cumulative mean normalized difference (CMND)
        self.cmnd[0] = 1.0;
        let mut running_sum = 0.0f32;
        for tau in 1..half {
            running_sum += self.diff[tau];
            if running_sum == 0.0 {
                self.cmnd[tau] = 1.0;
            } else {
                self.cmnd[tau] = self.diff[tau] * tau as f32 / running_sum;
            }
        }

        // Step 3: Absolute threshold — find first τ ≥ 2 where cmnd[τ] < threshold
        let mut tau_best = None;
        for tau in 2..half {
            if self.cmnd[tau] < self.threshold {
                // Find the local minimum from here
                let mut best = tau;
                while best + 1 < half && self.cmnd[best + 1] < self.cmnd[best] {
                    best += 1;
                }
                tau_best = Some(best);
                break;
            }
        }

        // If no τ below threshold, find global minimum (τ ≥ 2)
        let tau_best = tau_best.unwrap_or_else(|| {
            let mut best = 2;
            for tau in 3..half {
                if self.cmnd[tau] < self.cmnd[best] {
                    best = tau;
                }
            }
            best
        });

        let confidence = 1.0 - self.cmnd[tau_best];
        if confidence < 0.3 {
            return None;
        }

        // Step 4: Parabolic interpolation
        let refined_tau = if tau_best > 1 && tau_best < half - 1 {
            let s0 = self.cmnd[tau_best - 1];
            let s1 = self.cmnd[tau_best];
            let s2 = self.cmnd[tau_best + 1];
            let denom = s0 - 2.0 * s1 + s2;
            if denom.abs() > f32::EPSILON {
                tau_best as f32 + 0.5 * (s0 - s2) / denom
            } else {
                tau_best as f32
            }
        } else {
            tau_best as f32
        };

        // Step 5: Frequency
        if refined_tau <= 0.0 {
            return None;
        }
        let f0 = sample_rate as f32 / refined_tau;

        if f0 < 20.0 || f0 > 5000.0 {
            return None;
        }

        let (midi_note, cents_offset) = freq_to_midi(f0);
        Some(PitchEstimate {
            frequency_hz: f0,
            midi_note,
            cents_offset,
            confidence,
        })
    }

    fn name(&self) -> &str {
        "yin"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generate_sine(freq: f32, sample_rate: u32, num_samples: usize) -> Vec<f32> {
        (0..num_samples)
            .map(|i| {
                let t = i as f32 / sample_rate as f32;
                (2.0 * std::f32::consts::PI * freq * t).sin()
            })
            .collect()
    }

    #[test]
    fn yin_detects_a4_440hz() {
        let samples = generate_sine(440.0, 44100, 2048);
        let mut detector = YinDetector::new(2048, 0.15);
        let result = detector.detect(&samples, 44100);
        let est = result.expect("should detect A4");
        assert_eq!(est.midi_note, 69, "A4 should be MIDI 69");
        assert!(est.confidence > 0.8, "confidence should be high for pure sine");
    }

    #[test]
    fn yin_detects_c4_261hz() {
        let samples = generate_sine(261.63, 44100, 2048);
        let mut detector = YinDetector::new(2048, 0.15);
        let result = detector.detect(&samples, 44100);
        let est = result.expect("should detect C4");
        assert_eq!(est.midi_note, 60, "C4 should be MIDI 60");
    }

    #[test]
    fn yin_detects_e2_82hz() {
        let samples = generate_sine(82.41, 44100, 2048);
        let mut detector = YinDetector::new(2048, 0.15);
        let result = detector.detect(&samples, 44100);
        let est = result.expect("should detect E2");
        assert_eq!(est.midi_note, 40, "E2 should be MIDI 40");
    }

    #[test]
    fn yin_silence_returns_none() {
        let samples = vec![0.0f32; 2048];
        let mut detector = YinDetector::new(2048, 0.15);
        assert!(detector.detect(&samples, 44100).is_none());
    }

    #[test]
    fn yin_low_noise_returns_none() {
        let samples: Vec<f32> = (0..2048)
            .map(|i| ((i * 997) % 1000) as f32 * 0.000001 - 0.0005)
            .collect();
        let mut detector = YinDetector::new(2048, 0.15);
        let result = detector.detect(&samples, 44100);
        assert!(
            result.is_none() || result.unwrap().confidence < 0.3,
            "should not detect pitch in very low noise"
        );
    }
}
