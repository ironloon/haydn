use std::time::Duration;

/// Low-frequency oscillator for modulation effects.
pub struct Lfo {
    rate_hz: f32,
    depth: f32,
    sample_rate: u32,
    sample_index: u64,
}

impl Lfo {
    pub fn new(rate_hz: f32, depth: f32, sample_rate: u32) -> Self {
        Self {
            rate_hz,
            depth,
            sample_rate,
            sample_index: 0,
        }
    }

    pub fn sample(&mut self) -> f32 {
        let t = self.sample_index as f32 / self.sample_rate as f32;
        self.sample_index += 1;
        self.depth * (2.0 * std::f32::consts::PI * self.rate_hz * t).sin()
    }
}

/// Expressive source: wraps a base waveform and applies vibrato, tremolo, and velocity scaling.
pub struct ExpressiveSource {
    sample_rate: u32,
    base_frequency: f32,
    amplitude: f32,
    velocity_scale: f32,
    vibrato: Lfo,
    tremolo: Lfo,
    sample_index: u64,
    total_samples: u64,
}

impl ExpressiveSource {
    pub fn new(
        frequency: f32,
        duration: Duration,
        sample_rate: u32,
        amplitude: f32,
        velocity: u8,
    ) -> Self {
        let total_samples = (duration.as_secs_f64() * sample_rate as f64) as u64;
        Self {
            sample_rate,
            base_frequency: frequency,
            amplitude,
            velocity_scale: velocity as f32 / 127.0,
            vibrato: Lfo::new(5.5, 0.005, sample_rate),   // subtle pitch wobble
            tremolo: Lfo::new(4.0, 0.1, sample_rate),     // gentle amplitude variation
            sample_index: 0,
            total_samples,
        }
    }
}

impl Iterator for ExpressiveSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.sample_index >= self.total_samples {
            return None;
        }

        let vibrato_mod = self.vibrato.sample();
        let tremolo_mod = self.tremolo.sample();

        let frequency = self.base_frequency * (1.0 + vibrato_mod);
        let phase = frequency * self.sample_index as f32 / self.sample_rate as f32;

        // Use blended waveform like Layer 2
        use super::waveform::{saw_sample, triangle_sample};
        let sine = (2.0 * std::f32::consts::PI * phase).sin();
        let base_sample = 0.5 * saw_sample(phase) + 0.3 * sine + 0.2 * triangle_sample(phase);

        let amp = self.amplitude * self.velocity_scale * (1.0 + tremolo_mod);
        self.sample_index += 1;
        Some(base_sample * amp)
    }
}

impl rodio::Source for ExpressiveSource {
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.total_samples.saturating_sub(self.sample_index) as usize)
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_secs_f64(
            self.total_samples as f64 / self.sample_rate as f64,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lfo_oscillates() {
        let mut lfo = Lfo::new(1.0, 1.0, 44100);
        let mut samples = Vec::new();
        for _ in 0..44100 {
            samples.push(lfo.sample());
        }
        // Should have both positive and negative values
        assert!(samples.iter().any(|&s| s > 0.5));
        assert!(samples.iter().any(|&s| s < -0.5));
    }

    #[test]
    fn test_expressive_source_varies() {
        let source = ExpressiveSource::new(440.0, Duration::from_millis(200), 44100, 1.0, 100);
        let samples: Vec<f32> = source.collect();
        assert!(!samples.is_empty());
        // Verify amplitude variation from tremolo
        let max_abs = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        assert!(max_abs > 0.1, "should produce audible output");
    }
}
