use std::time::Duration;

/// Convert MIDI note number to frequency in Hz.
pub fn midi_to_freq(midi_note: u8) -> f32 {
    440.0 * 2f32.powf((midi_note as f32 - 69.0) / 12.0)
}

/// A sine wave audio source for rodio.
pub struct SineSource {
    sample_rate: u32,
    frequency: f32,
    amplitude: f32,
    sample_index: u64,
    total_samples: u64,
}

impl SineSource {
    pub fn new(frequency: f32, duration: Duration, sample_rate: u32, amplitude: f32) -> Self {
        let total_samples = (duration.as_secs_f64() * sample_rate as f64) as u64;
        Self {
            sample_rate,
            frequency,
            amplitude,
            sample_index: 0,
            total_samples,
        }
    }
}

impl Iterator for SineSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.sample_index >= self.total_samples {
            return None;
        }
        let t = self.sample_index as f32 / self.sample_rate as f32;
        let sample = self.amplitude * (2.0 * std::f32::consts::PI * self.frequency * t).sin();
        self.sample_index += 1;
        Some(sample)
    }
}

impl rodio::Source for SineSource {
    fn current_frame_len(&self) -> Option<usize> {
        let remaining = self.total_samples.saturating_sub(self.sample_index);
        Some(remaining as usize)
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

/// Silence source for rests.
pub struct SilenceSource {
    sample_rate: u32,
    sample_index: u64,
    total_samples: u64,
}

impl SilenceSource {
    pub fn new(duration: Duration, sample_rate: u32) -> Self {
        let total_samples = (duration.as_secs_f64() * sample_rate as f64) as u64;
        Self {
            sample_rate,
            sample_index: 0,
            total_samples,
        }
    }
}

impl Iterator for SilenceSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.sample_index >= self.total_samples {
            return None;
        }
        self.sample_index += 1;
        Some(0.0)
    }
}

impl rodio::Source for SilenceSource {
    fn current_frame_len(&self) -> Option<usize> {
        let remaining = self.total_samples.saturating_sub(self.sample_index);
        Some(remaining as usize)
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
    fn test_midi_to_freq_a4() {
        let freq = midi_to_freq(69);
        assert!((freq - 440.0).abs() < 0.01);
    }

    #[test]
    fn test_midi_to_freq_c4() {
        let freq = midi_to_freq(60);
        assert!((freq - 261.63).abs() < 0.1);
    }

    #[test]
    fn test_sine_source_produces_samples() {
        let source = SineSource::new(440.0, Duration::from_millis(100), 44100, 1.0);
        let samples: Vec<f32> = source.collect();
        assert_eq!(samples.len(), 4410);
        // First sample should be ~0 (sin(0))
        assert!(samples[0].abs() < 0.01);
        // Should contain non-zero samples
        assert!(samples.iter().any(|&s| s.abs() > 0.5));
    }

    #[test]
    fn test_silence_source() {
        let source = SilenceSource::new(Duration::from_millis(100), 44100);
        let samples: Vec<f32> = source.collect();
        assert_eq!(samples.len(), 4410);
        assert!(samples.iter().all(|&s| s == 0.0));
    }
}
