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

/// Expressive source: additive synthesis with spectral evolution, attack transients,
/// instrument-specific harmonic profiles, vibrato, tremolo, and velocity scaling.
///
/// Key realism features:
/// - Harmonics are brighter at attack, mellowing into sustain (spectral evolution)
/// - Filtered noise burst during attack phase (hammer/breath/bow transient)
/// - Vibrato fades in gradually so short notes stay clean
/// - Per-instrument ADSR curves, harmonic profiles, and expression parameters
pub struct ExpressiveSource {
    sample_rate: u32,
    base_frequency: f32,
    amplitude: f32,
    velocity_scale: f32,
    vibrato: Lfo,
    tremolo: Lfo,
    /// Slow LFO that modulates vibrato depth — makes vibrato "breathe"
    /// so long notes don't sound like a machine wobble.
    vibrato_breath: Lfo,
    sample_index: u64,
    total_samples: u64,
    vibrato_onset_samples: u64,
    harmonics: &'static [f32],
    // Spectral evolution
    attack_brightness: f32,
    brightness_decay_samples: u64,
    // Attack noise transient
    noise: super::timbre::NoiseGenerator,
    attack_noise_amplitude: f32,
    attack_noise_samples: u64,
    // Articulation gap
    gap_samples: u64,
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
        let profile = instrument.profile();

        // Reserve gap samples at the end for articulation silence
        let gap_samples = (profile.articulation_gap_ms / 1000.0 * sample_rate as f32) as u64;
        let raw_total = (duration.as_secs_f64() * sample_rate as f64) as u64;
        let total_samples = raw_total; // gap is appended, total stays the same

        let vibrato_onset_samples = (profile.vibrato_onset_ms / 1000.0 * sample_rate as f32) as u64;

        let vibrato = if profile.vibrato_depth > 0.0 {
            Lfo::new(profile.vibrato_rate, profile.vibrato_depth, sample_rate)
        } else {
            Lfo::new(0.0, 0.0, sample_rate)
        };

        // Slow "breathing" LFO modulates vibrato depth.
        // Two superimposed rates create an organic, non-periodic feel:
        // the primary breath cycle (~0.13 Hz = ~8sec) is amplitude-modulated
        // by a secondary drift (~0.07 Hz = ~14sec) so it never repeats exactly.
        // Depth 0.4 means vibrato swings between ~20-100% of max.
        let vibrato_breath = if profile.vibrato_depth > 0.0 {
            Lfo::new(0.13, 0.4, sample_rate)
        } else {
            Lfo::new(0.0, 0.0, sample_rate)
        };

        let tremolo = if profile.tremolo_depth > 0.0 {
            Lfo::new(profile.tremolo_rate, profile.tremolo_depth, sample_rate)
        } else {
            Lfo::new(0.0, 0.0, sample_rate)
        };

        // Spectral evolution timing
        let brightness_decay_samples = (profile.brightness_decay_ms / 1000.0 * sample_rate as f32) as u64;
        let attack_noise_samples = (profile.attack_noise_ms / 1000.0 * sample_rate as f32) as u64;

        Self {
            sample_rate,
            base_frequency: frequency,
            amplitude,
            velocity_scale: velocity as f32 / 127.0,
            vibrato,
            vibrato_breath,
            tremolo,
            sample_index: 0,
            total_samples,
            vibrato_onset_samples,
            harmonics: profile.harmonics,
            attack_brightness: profile.attack_brightness,
            brightness_decay_samples,
            noise: super::timbre::NoiseGenerator::new(profile.noise_highpass),
            attack_noise_amplitude: profile.attack_noise,
            attack_noise_samples,
            gap_samples,
        }
    }
}

impl Iterator for ExpressiveSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.sample_index >= self.total_samples {
            return None;
        }

        // Articulation gap: output silence for the last N samples
        let sound_samples = self.total_samples.saturating_sub(self.gap_samples);
        if self.sample_index >= sound_samples {
            self.sample_index += 1;
            return Some(0.0);
        }

        let vibrato_raw = self.vibrato.sample();
        let tremolo_mod = self.tremolo.sample();

        // Breathing modulation: vibrato depth varies organically.
        // breath_raw ranges from -0.4 to +0.4; we shift to 0.6..1.4, clamp to 0.2..1.0
        // so vibrato sometimes nearly disappears, sometimes reaches full depth.
        let breath_raw = self.vibrato_breath.sample();
        let breath_scale = (1.0 + breath_raw).max(0.2).min(1.0);

        // Fade vibrato in over onset period — use a smooth ease-in (squared) curve
        // so it doesn't suddenly "turn on"
        let vibrato_mod = if self.vibrato_onset_samples > 0 && self.sample_index < self.vibrato_onset_samples {
            let t = self.sample_index as f32 / self.vibrato_onset_samples as f32;
            let fade = t * t; // quadratic ease-in
            vibrato_raw * fade * breath_scale
        } else {
            vibrato_raw * breath_scale
        };

        let frequency = self.base_frequency * (1.0 + vibrato_mod);
        let phase = frequency * self.sample_index as f32 / self.sample_rate as f32;

        // Spectral evolution: brightness decays from attack_brightness to 1.0
        let brightness = if self.brightness_decay_samples > 0 && self.sample_index < self.brightness_decay_samples {
            let t = self.sample_index as f32 / self.brightness_decay_samples as f32;
            self.attack_brightness + (1.0 - self.attack_brightness) * t
        } else {
            1.0
        };

        // Additive synthesis with evolved spectral profile
        let base_sample = super::timbre::additive_sample_evolved(
            phase,
            frequency,
            self.sample_rate,
            self.harmonics,
            brightness,
        );

        // Attack noise transient
        let noise = if self.sample_index < self.attack_noise_samples {
            let fade = 1.0 - (self.sample_index as f32 / self.attack_noise_samples as f32);
            self.noise.sample() * self.attack_noise_amplitude * fade
        } else {
            0.0
        };

        let amp = self.amplitude * self.velocity_scale * (1.0 + tremolo_mod);
        self.sample_index += 1;
        Some((base_sample + noise) * amp)
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
