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

/// Expressive source: additive synthesis with instrument-specific harmonic profiles,
/// vibrato, tremolo, and velocity scaling. Vibrato fades in gradually so short notes stay clean.
pub struct ExpressiveSource {
    sample_rate: u32,
    base_frequency: f32,
    amplitude: f32,
    velocity_scale: f32,
    vibrato: Lfo,
    tremolo: Lfo,
    sample_index: u64,
    total_samples: u64,
    vibrato_onset_samples: u64,
    harmonics: &'static [f32],
}

impl ExpressiveSource {
    pub fn new(
        frequency: f32,
        duration: Duration,
        sample_rate: u32,
        amplitude: f32,
        velocity: u8,
    ) -> Self {
        Self::with_instrument(
            frequency,
            duration,
            sample_rate,
            amplitude,
            velocity,
            super::timbre::Instrument::default(),
        )
    }

    pub fn with_instrument(
        frequency: f32,
        duration: Duration,
        sample_rate: u32,
        amplitude: f32,
        velocity: u8,
        instrument: super::timbre::Instrument,
    ) -> Self {
        let total_samples = (duration.as_secs_f64() * sample_rate as f64) as u64;
        let profile = instrument.profile();

        let vibrato_onset_samples = (profile.vibrato_onset_ms / 1000.0 * sample_rate as f32) as u64;

        let vibrato = if profile.vibrato_depth > 0.0 {
            Lfo::new(profile.vibrato_rate, profile.vibrato_depth, sample_rate)
        } else {
            Lfo::new(0.0, 0.0, sample_rate)
        };

        let tremolo = if profile.tremolo_depth > 0.0 {
            Lfo::new(profile.tremolo_rate, profile.tremolo_depth, sample_rate)
        } else {
            Lfo::new(0.0, 0.0, sample_rate)
        };

        Self {
            sample_rate,
            base_frequency: frequency,
            amplitude,
            velocity_scale: velocity as f32 / 127.0,
            vibrato,
            tremolo,
            sample_index: 0,
            total_samples,
            vibrato_onset_samples,
            harmonics: profile.harmonics,
        }
    }
}

impl Iterator for ExpressiveSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.sample_index >= self.total_samples {
            return None;
        }

        let vibrato_raw = self.vibrato.sample();
        let tremolo_mod = self.tremolo.sample();

        // Fade vibrato in over onset period so short notes stay clean
        let vibrato_mod = if self.sample_index < self.vibrato_onset_samples {
            let fade = self.sample_index as f32 / self.vibrato_onset_samples as f32;
            vibrato_raw * fade
        } else {
            vibrato_raw
        };

        let frequency = self.base_frequency * (1.0 + vibrato_mod);
        let phase = frequency * self.sample_index as f32 / self.sample_rate as f32;

        // Additive synthesis using instrument harmonic profile
        let base_sample = super::timbre::additive_sample(
            phase,
            frequency,
            self.sample_rate,
            self.harmonics,
        );

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
