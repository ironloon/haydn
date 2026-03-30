use super::{freq_to_midi, PitchDetector, PitchEstimate};

/// McLeod NSDF pitch detection algorithm (McLeod & Wyvill, 2005).
///
/// Uses the Normalized Square Difference Function for robust fundamental
/// frequency estimation with better octave accuracy than raw autocorrelation.
pub struct McLeodDetector {
    window_size: usize,
    clarity_threshold: f32,
    nsdf: Vec<f32>,
}

impl McLeodDetector {
    pub fn new(window_size: usize, clarity_threshold: f32) -> Self {
        Self {
            window_size,
            clarity_threshold,
            nsdf: vec![0.0; window_size],
        }
    }
}

impl PitchDetector for McLeodDetector {
    fn detect(&mut self, samples: &[f32], sample_rate: u32) -> Option<PitchEstimate> {
        let len = samples.len().min(self.window_size);
        if len < self.window_size {
            return None;
        }

        // Early bailout: check RMS
        let rms = {
            let sum_sq: f32 = samples[..len].iter().map(|s| s * s).sum();
            (sum_sq / len as f32).sqrt()
        };
        if rms < 0.01 {
            return None;
        }

        let w = self.window_size;

        // Compute NSDF: nsdf(τ) = 2·r(τ) / m(τ)
        // where r(τ) = Σ x[j]·x[j+τ], m(τ) = Σ (x[j]² + x[j+τ]²)
        for tau in 0..w {
            let mut acf = 0.0f32; // autocorrelation r(τ)
            let mut m = 0.0f32; // energy normalization m(τ)
            for j in 0..(w - tau) {
                acf += samples[j] * samples[j + tau];
                m += samples[j] * samples[j] + samples[j + tau] * samples[j + tau];
            }
            self.nsdf[tau] = if m > 0.0 { 2.0 * acf / m } else { 0.0 };
        }

        // Find key maxima: positive peaks after zero-crossings from negative to positive
        // Track the first peak above clarity_threshold
        let mut best_tau: Option<usize> = None;
        let mut best_val: f32 = 0.0;
        let mut was_negative = false;
        let mut in_positive_region = false;
        let mut local_max_tau = 0usize;
        let mut local_max_val = 0.0f32;

        // Start from tau=1 to skip the trivial peak at tau=0
        for tau in 1..w {
            if self.nsdf[tau] < 0.0 {
                // If we were in a positive region and found a peak, record it
                if in_positive_region && local_max_val > best_val {
                    best_tau = Some(local_max_tau);
                    best_val = local_max_val;
                    // If this peak is above clarity threshold, accept it immediately
                    if best_val >= self.clarity_threshold {
                        break;
                    }
                }
                was_negative = true;
                in_positive_region = false;
                local_max_val = 0.0;
            } else {
                if was_negative {
                    // Zero crossing from negative to positive — start new positive region
                    in_positive_region = true;
                    local_max_val = 0.0;
                    was_negative = false;
                }
                if in_positive_region && self.nsdf[tau] > local_max_val {
                    local_max_val = self.nsdf[tau];
                    local_max_tau = tau;
                }
            }
        }

        // Handle case where we ended in a positive region
        if in_positive_region && local_max_val > best_val {
            best_tau = Some(local_max_tau);
            best_val = local_max_val;
        }

        let tau_best = best_tau?;
        let confidence = best_val;

        if confidence < 0.3 {
            return None;
        }

        // Parabolic interpolation
        let refined_tau = if tau_best > 1 && tau_best < w - 1 {
            let s0 = self.nsdf[tau_best - 1];
            let s1 = self.nsdf[tau_best];
            let s2 = self.nsdf[tau_best + 1];
            let denom = s0 - 2.0 * s1 + s2;
            if denom.abs() > f32::EPSILON {
                tau_best as f32 - 0.5 * (s0 - s2) / denom
            } else {
                tau_best as f32
            }
        } else {
            tau_best as f32
        };

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
        "mcleod"
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

    #[test]
    fn mcleod_detects_a4_440hz() {
        let samples = generate_sine(440.0, 44100, 2048);
        let mut detector = McLeodDetector::new(2048, 0.7);
        let result = detector.detect(&samples, 44100);
        let est = result.expect("should detect A4");
        assert_eq!(est.midi_note, 69, "A4 should be MIDI 69");
        assert!(est.confidence > 0.8, "confidence should be high for pure sine");
    }

    #[test]
    fn mcleod_detects_c4_261hz() {
        let samples = generate_sine(261.63, 44100, 2048);
        let mut detector = McLeodDetector::new(2048, 0.7);
        let result = detector.detect(&samples, 44100);
        let est = result.expect("should detect C4");
        assert_eq!(est.midi_note, 60, "C4 should be MIDI 60");
    }

    #[test]
    fn mcleod_detects_e2_82hz() {
        let samples = generate_sine(82.41, 44100, 2048);
        let mut detector = McLeodDetector::new(2048, 0.7);
        let result = detector.detect(&samples, 44100);
        let est = result.expect("should detect E2");
        assert_eq!(est.midi_note, 40, "E2 should be MIDI 40");
    }

    #[test]
    fn mcleod_silence_returns_none() {
        let samples = vec![0.0f32; 2048];
        let mut detector = McLeodDetector::new(2048, 0.7);
        assert!(detector.detect(&samples, 44100).is_none());
    }

    #[test]
    fn mcleod_harmonic_detects_fundamental() {
        let samples = generate_harmonic(440.0, 44100, 2048);
        let mut detector = McLeodDetector::new(2048, 0.7);
        let result = detector.detect(&samples, 44100);
        let est = result.expect("should detect fundamental of harmonic signal");
        assert_eq!(est.midi_note, 69, "should detect A4 fundamental, not harmonic");
    }
}
