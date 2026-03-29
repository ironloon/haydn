/// Schroeder algorithmic reverb — simulates room acoustics.
///
/// Uses 4 parallel comb filters feeding into 2 series allpass filters.
/// This is a classic design that creates a dense, natural-sounding reverb
/// tail without loading external impulse response files.

/// Comb filter: delayed feedback loop that creates resonant echoes.
struct CombFilter {
    buffer: Vec<f32>,
    index: usize,
    feedback: f32,
    /// Simple low-pass damping in the feedback path (models air absorption)
    damp: f32,
    prev: f32,
}

impl CombFilter {
    fn new(delay_samples: usize, feedback: f32, damp: f32) -> Self {
        Self {
            buffer: vec![0.0; delay_samples],
            index: 0,
            feedback,
            damp,
            prev: 0.0,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let delayed = self.buffer[self.index];
        // Low-pass filter in feedback path: damp controls how much high freq is absorbed
        self.prev = delayed * (1.0 - self.damp) + self.prev * self.damp;
        self.buffer[self.index] = input + self.prev * self.feedback;
        self.index = (self.index + 1) % self.buffer.len();
        delayed
    }
}

/// Allpass filter: passes all frequencies equally but adds phase dispersion,
/// which thickens the reverb tail and prevents metallic ringing.
struct AllpassFilter {
    buffer: Vec<f32>,
    index: usize,
    feedback: f32,
}

impl AllpassFilter {
    fn new(delay_samples: usize, feedback: f32) -> Self {
        Self {
            buffer: vec![0.0; delay_samples],
            index: 0,
            feedback,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let delayed = self.buffer[self.index];
        let output = -input + delayed;
        self.buffer[self.index] = input + delayed * self.feedback;
        self.index = (self.index + 1) % self.buffer.len();
        output
    }
}

/// Room reverb processor.
///
/// Apply to a mono buffer in-place. Uses mutually-prime delay lengths
/// to avoid resonant peaks, and damping to simulate air absorption
/// (high frequencies die faster than lows, like a real room).
pub struct Reverb {
    combs: Vec<CombFilter>,
    allpasses: Vec<AllpassFilter>,
    wet: f32,
    dry: f32,
}

impl Reverb {
    /// Create a reverb tuned for the given sample rate.
    ///
    /// - `mix`: 0.0 = fully dry, 1.0 = fully wet. 0.15-0.25 is natural for most music.
    /// - `room_size`: 0.0 = small room, 1.0 = large hall. Scales feedback.
    /// - `damping`: 0.0 = bright reverb, 1.0 = very dark. 0.4-0.6 is natural.
    pub fn new(sample_rate: u32, mix: f32, room_size: f32, damping: f32) -> Self {
        let sr_scale = sample_rate as f32 / 44100.0;

        // Mutually-prime delay lengths in samples (at 44.1kHz, scaled for other rates).
        // These are classic Schroeder values adjusted for musicality.
        let comb_delays = [1557, 1617, 1491, 1422, 1277, 1356, 1188, 1116];
        let allpass_delays = [225, 556, 441, 341];

        let base_feedback = 0.7 + room_size.clamp(0.0, 1.0) * 0.28; // 0.70 to 0.98
        let damp = damping.clamp(0.0, 1.0);

        let combs = comb_delays
            .iter()
            .map(|&d| {
                let delay = ((d as f32) * sr_scale) as usize;
                CombFilter::new(delay, base_feedback, damp)
            })
            .collect();

        let allpasses = allpass_delays
            .iter()
            .map(|&d| {
                let delay = ((d as f32) * sr_scale) as usize;
                AllpassFilter::new(delay, 0.5)
            })
            .collect();

        Self {
            combs,
            allpasses,
            wet: mix.clamp(0.0, 1.0),
            dry: 1.0 - mix.clamp(0.0, 1.0),
        }
    }

    /// Concert hall preset — spacious with moderate damping.
    pub fn hall(sample_rate: u32) -> Self {
        Self::new(sample_rate, 0.18, 0.8, 0.45)
    }

    /// Process a mono buffer in-place, adding reverb.
    pub fn process(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            let input = *sample;

            // Sum parallel comb filter outputs
            let comb_sum: f32 = self.combs.iter_mut().map(|c| c.process(input)).sum();
            let mut wet = comb_sum / self.combs.len() as f32;

            // Series allpass filters
            for ap in &mut self.allpasses {
                wet = ap.process(wet);
            }

            *sample = input * self.dry + wet * self.wet;
        }
    }
}

/// Convert a mono buffer to interleaved stereo with subtle width.
///
/// Creates stereo by applying a tiny L/R delay difference (Haas effect)
/// and slight decorrelation, giving the impression of space without
/// making the sound feel off-center.
pub fn mono_to_stereo(mono: &[f32], sample_rate: u32) -> Vec<f32> {
    // Haas delay: ~0.6ms offset between L and R
    let delay_samples = (0.0006 * sample_rate as f64) as usize;
    let len = mono.len();
    let mut stereo = vec![0.0f32; len * 2];

    for i in 0..len {
        let left = mono[i];
        // Right channel is slightly delayed — creates width perception
        let right = if i >= delay_samples {
            mono[i - delay_samples]
        } else {
            0.0
        };

        // Slight cross-feed for natural stereo (not hard-panned)
        let cross = 0.15;
        stereo[i * 2] = left * (1.0 - cross) + right * cross;
        stereo[i * 2 + 1] = right * (1.0 - cross) + left * cross;
    }

    stereo
}

/// Soft saturation — warm analog-style limiting.
///
/// Instead of hard clipping at ±1.0, the signal is gently compressed
/// using a tanh curve. This adds subtle harmonic warmth and prevents
/// harsh digital distortion when notes stack up.
pub fn soft_saturate(buffer: &mut [f32], drive: f32) {
    let d = drive.max(1.0);
    for sample in buffer.iter_mut() {
        // tanh provides smooth saturation; normalize so small signals pass through
        *sample = (*sample * d).tanh();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reverb_adds_tail() {
        let mut reverb = Reverb::hall(44100);
        // Create an impulse followed by silence
        let mut buffer = vec![0.0f32; 44100];
        buffer[0] = 1.0;

        reverb.process(&mut buffer);

        // The reverb tail should have energy well after the impulse
        let late_energy: f32 = buffer[4410..8820].iter().map(|s| s * s).sum();
        assert!(late_energy > 0.0001, "reverb should produce a tail");
    }

    #[test]
    fn test_reverb_preserves_dry() {
        let mut reverb = Reverb::new(44100, 0.0, 0.5, 0.5); // 100% dry
        let mut buffer = vec![0.5; 100];
        reverb.process(&mut buffer);
        for &s in &buffer {
            assert!((s - 0.5).abs() < 0.01, "dry signal should be preserved");
        }
    }

    #[test]
    fn test_mono_to_stereo_doubles_length() {
        let mono = vec![0.5; 100];
        let stereo = mono_to_stereo(&mono, 44100);
        assert_eq!(stereo.len(), 200);
    }

    #[test]
    fn test_mono_to_stereo_left_right_differ() {
        let mono: Vec<f32> = (0..1000).map(|i| (i as f32 * 0.01).sin()).collect();
        let stereo = mono_to_stereo(&mono, 44100);
        // Due to Haas delay, L and R shouldn't be identical
        let mut differ_count = 0;
        for i in 0..1000 {
            if (stereo[i * 2] - stereo[i * 2 + 1]).abs() > 0.001 {
                differ_count += 1;
            }
        }
        assert!(differ_count > 100, "L and R should differ for width");
    }

    #[test]
    fn test_soft_saturate_bounds() {
        let mut buffer = vec![-2.0, -1.0, 0.0, 1.0, 2.0];
        soft_saturate(&mut buffer, 2.0);
        for &s in &buffer {
            assert!(s >= -1.0 && s <= 1.0, "saturated signal should be in [-1, 1], got {}", s);
        }
    }

    #[test]
    fn test_soft_saturate_compresses() {
        let mut buffer = vec![0.5, -0.5, 0.8];
        soft_saturate(&mut buffer, 2.0);
        // With drive=2.0, input 0.5 -> tanh(1.0) ≈ 0.76, which is less than 1.0
        for &s in &buffer {
            assert!(s.abs() < 1.0, "saturated signals should be bounded");
        }
    }
}
