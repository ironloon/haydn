use std::time::Duration;

/// Bandlimited sawtooth wave sample.
/// Uses additive synthesis (first 20 harmonics) to avoid aliasing.
pub fn saw_sample(phase: f32) -> f32 {
    let mut sum = 0.0f32;
    for k in 1..=20 {
        let sign = if k % 2 == 0 { -1.0 } else { 1.0 };
        sum += sign * (2.0 * std::f32::consts::PI * k as f32 * phase).sin() / k as f32;
    }
    sum * (2.0 / std::f32::consts::PI)
}

/// Bandlimited triangle wave sample.
/// Uses odd harmonics with 1/(2k+1)^2 rolloff.
pub fn triangle_sample(phase: f32) -> f32 {
    let mut sum = 0.0f32;
    for k in 0..10 {
        let sign = if k % 2 == 0 { 1.0 } else { -1.0 };
        let n = (2 * k + 1) as f32;
        sum += sign * (2.0 * std::f32::consts::PI * n * phase).sin() / (n * n);
    }
    sum * (8.0 / (std::f32::consts::PI * std::f32::consts::PI))
}

/// Bandlimited square wave sample.
/// Uses odd harmonics with 1/(2k+1) rolloff.
pub fn square_sample(phase: f32) -> f32 {
    let mut sum = 0.0f32;
    for k in 0..10 {
        let n = (2 * k + 1) as f32;
        sum += (2.0 * std::f32::consts::PI * n * phase).sin() / n;
    }
    sum * (4.0 / std::f32::consts::PI)
}

/// Blended waveform source: 50% saw + 30% sine + 20% triangle.
/// Produces a warm, rich, piano-like timbre.
pub struct WaveformSource {
    sample_rate: u32,
    frequency: f32,
    amplitude: f32,
    sample_index: u64,
    total_samples: u64,
}

impl WaveformSource {
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

impl Iterator for WaveformSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.sample_index >= self.total_samples {
            return None;
        }
        let phase = self.frequency * self.sample_index as f32 / self.sample_rate as f32;
        let sine = (2.0 * std::f32::consts::PI * phase).sin();
        let sample = self.amplitude * (0.5 * saw_sample(phase) + 0.3 * sine + 0.2 * triangle_sample(phase));
        self.sample_index += 1;
        Some(sample)
    }
}

impl rodio::Source for WaveformSource {
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
    fn test_saw_sample_range() {
        // Sawtooth should produce values roughly in -1..1
        for i in 0..100 {
            let phase = i as f32 / 100.0;
            let s = saw_sample(phase);
            assert!(s.abs() < 1.5, "saw sample out of range: {}", s);
        }
    }

    #[test]
    fn test_triangle_sample_range() {
        for i in 0..100 {
            let phase = i as f32 / 100.0;
            let s = triangle_sample(phase);
            assert!(s.abs() < 1.5, "triangle sample out of range: {}", s);
        }
    }

    #[test]
    fn test_square_sample_range() {
        for i in 0..100 {
            let phase = i as f32 / 100.0;
            let s = square_sample(phase);
            assert!(s.abs() < 1.5, "square sample out of range: {}", s);
        }
    }

    #[test]
    fn test_waveform_source_richer_than_sine() {
        use crate::synth::sine::SineSource;

        let sine = SineSource::new(440.0, Duration::from_millis(100), 44100, 1.0);
        let waveform = WaveformSource::new(440.0, Duration::from_millis(100), 44100, 1.0);

        let sine_samples: Vec<f32> = sine.collect();
        let waveform_samples: Vec<f32> = waveform.collect();

        // Waveform should have more harmonic energy (higher variance in sample differences)
        let sine_energy: f32 = sine_samples.windows(2).map(|w| (w[1] - w[0]).powi(2)).sum();
        let waveform_energy: f32 = waveform_samples.windows(2).map(|w| (w[1] - w[0]).powi(2)).sum();

        assert!(
            waveform_energy > sine_energy,
            "waveform should have more harmonic energy: sine={}, waveform={}",
            sine_energy,
            waveform_energy
        );
    }
}
