/// ADSR envelope phases.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AdsrPhase {
    Attack,
    Decay,
    Sustain,
    Release,
    Done,
}

/// ADSR envelope generator that shapes note amplitude over time.
pub struct AdsrEnvelope {
    attack_samples: u64,
    decay_samples: u64,
    sustain_level: f32,
    release_start: u64,
    total_samples: u64,
    release_samples: u64,
    sample_index: u64,
}

impl AdsrEnvelope {
    /// Create a new ADSR envelope.
    ///
    /// - `attack_ms`: time to ramp from 0 to 1.0
    /// - `decay_ms`: time to ramp from 1.0 to sustain_level
    /// - `sustain_level`: amplitude during sustain (0.0-1.0)
    /// - `release_ms`: time to ramp from sustain_level to 0.0
    /// - `note_duration`: total note duration
    /// - `sample_rate`: audio sample rate
    pub fn new(
        attack_ms: f32,
        decay_ms: f32,
        sustain_level: f32,
        release_ms: f32,
        note_duration: std::time::Duration,
        sample_rate: u32,
    ) -> Self {
        let sr = sample_rate as f64;
        let attack_samples = (attack_ms as f64 / 1000.0 * sr) as u64;
        let decay_samples = (decay_ms as f64 / 1000.0 * sr) as u64;
        let release_samples = (release_ms as f64 / 1000.0 * sr) as u64;
        let total_samples = (note_duration.as_secs_f64() * sr) as u64;

        let release_start = total_samples.saturating_sub(release_samples);

        Self {
            attack_samples,
            decay_samples,
            sustain_level,
            release_start,
            total_samples,
            release_samples,
            sample_index: 0,
        }
    }

    /// Default piano-like envelope: attack=10ms, decay=100ms, sustain=0.7, release=200ms
    pub fn piano(note_duration: std::time::Duration, sample_rate: u32) -> Self {
        Self::new(10.0, 100.0, 0.7, 200.0, note_duration, sample_rate)
    }

    /// Create an ADSR envelope from an instrument profile.
    pub fn from_profile(
        profile: &super::timbre::InstrumentProfile,
        note_duration: std::time::Duration,
        sample_rate: u32,
    ) -> Self {
        Self::new(
            profile.attack_ms,
            profile.decay_ms,
            profile.sustain_level,
            profile.release_ms,
            note_duration,
            sample_rate,
        )
    }

    /// Get current envelope amplitude and advance by one sample.
    pub fn amplitude(&mut self) -> f32 {
        if self.sample_index >= self.total_samples {
            return 0.0;
        }

        let idx = self.sample_index;
        self.sample_index += 1;

        // Release phase takes priority if we've passed the release start point
        if idx >= self.release_start {
            let release_progress = (idx - self.release_start) as f32 / self.release_samples.max(1) as f32;
            return self.sustain_level * (1.0 - release_progress).max(0.0);
        }

        // Attack phase
        if idx < self.attack_samples {
            return idx as f32 / self.attack_samples.max(1) as f32;
        }

        // Decay phase
        let decay_start = self.attack_samples;
        if idx < decay_start + self.decay_samples {
            let decay_progress = (idx - decay_start) as f32 / self.decay_samples.max(1) as f32;
            return 1.0 - (1.0 - self.sustain_level) * decay_progress;
        }

        // Sustain phase
        self.sustain_level
    }

    pub fn phase_at(&self) -> AdsrPhase {
        if self.sample_index >= self.total_samples {
            AdsrPhase::Done
        } else if self.sample_index >= self.release_start {
            AdsrPhase::Release
        } else if self.sample_index < self.attack_samples {
            AdsrPhase::Attack
        } else if self.sample_index < self.attack_samples + self.decay_samples {
            AdsrPhase::Decay
        } else {
            AdsrPhase::Sustain
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_envelope_starts_at_zero() {
        let mut env = AdsrEnvelope::piano(Duration::from_millis(500), 44100);
        // First sample should be 0 (start of attack)
        let first = env.amplitude();
        assert!(first < 0.01, "envelope should start near 0, got {}", first);
    }

    #[test]
    fn test_envelope_reaches_peak() {
        let mut env = AdsrEnvelope::new(10.0, 100.0, 0.7, 200.0, Duration::from_millis(500), 44100);
        // Skip through attack phase (10ms = 441 samples)
        let mut peak = 0.0f32;
        for _ in 0..500 {
            let a = env.amplitude();
            peak = peak.max(a);
        }
        assert!(peak > 0.95, "should reach near 1.0 peak, got {}", peak);
    }

    #[test]
    fn test_envelope_sustains() {
        let mut env = AdsrEnvelope::new(10.0, 100.0, 0.7, 200.0, Duration::from_millis(1000), 44100);
        // Skip to sustain phase (after attack + decay = ~110ms = ~4851 samples)
        for _ in 0..6000 {
            env.amplitude();
        }
        let sustain_val = env.amplitude();
        assert!(
            (sustain_val - 0.7).abs() < 0.05,
            "sustain should be ~0.7, got {}",
            sustain_val
        );
    }

    #[test]
    fn test_envelope_releases_to_zero() {
        let mut env = AdsrEnvelope::piano(Duration::from_millis(500), 44100);
        // Consume all samples
        let total = (0.5 * 44100.0) as usize;
        for _ in 0..total {
            env.amplitude();
        }
        let final_val = env.amplitude();
        assert!(final_val < 0.01, "should release to ~0, got {}", final_val);
    }
}
