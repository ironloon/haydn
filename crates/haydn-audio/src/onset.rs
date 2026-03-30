use crate::gate::NoiseGate;

/// Result of processing a single audio frame through the onset detector.
pub struct OnsetResult {
    pub is_onset: bool,
    pub current_rms_db: f32,
}

/// Energy-based onset detection — detects sudden energy spikes above a running baseline.
pub struct OnsetDetector {
    threshold_db: f32,
    prev_rms_db: f32,
    running_avg_db: f32,
}

impl OnsetDetector {
    pub fn new(threshold_db: f32) -> Self {
        Self {
            threshold_db,
            prev_rms_db: f32::NEG_INFINITY,
            running_avg_db: -80.0,
        }
    }

    pub fn process_frame(&mut self, samples: &[f32]) -> OnsetResult {
        let current_db = NoiseGate::rms_db(samples);
        let delta = current_db - self.running_avg_db;
        let is_onset = delta > self.threshold_db && current_db > self.prev_rms_db;

        // Update running average (EMA with alpha=0.1), clamp to avoid -inf propagation
        let clamped_db = current_db.max(-80.0);
        self.running_avg_db = 0.9 * self.running_avg_db + 0.1 * clamped_db;

        self.prev_rms_db = current_db;

        OnsetResult {
            is_onset,
            current_rms_db: current_db,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn loud_sine_samples(n: usize) -> Vec<f32> {
        (0..n)
            .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 44100.0).sin() * 0.5)
            .collect()
    }

    #[test]
    fn detects_onset_from_silence_to_loud() {
        let mut onset = OnsetDetector::new(6.0);
        let silence = vec![0.0f32; 512];
        let loud = loud_sine_samples(512);

        // Feed several silence frames to establish baseline
        for _ in 0..5 {
            let result = onset.process_frame(&silence);
            assert!(!result.is_onset);
        }

        // First loud frame should trigger onset
        let result = onset.process_frame(&loud);
        assert!(result.is_onset, "should detect onset from silence to loud");
    }

    #[test]
    fn no_onset_during_sustained_signal() {
        let mut onset = OnsetDetector::new(6.0);
        let loud = loud_sine_samples(512);

        // Let baseline settle on loud signal
        for _ in 0..20 {
            onset.process_frame(&loud);
        }

        // Additional frames should NOT trigger onset (baseline has caught up)
        let result = onset.process_frame(&loud);
        assert!(
            !result.is_onset,
            "should NOT trigger onset during sustained signal"
        );
    }

    #[test]
    fn re_triggers_after_silence_gap() {
        let mut onset = OnsetDetector::new(6.0);
        let silence = vec![0.0f32; 512];
        let loud = loud_sine_samples(512);

        // First note
        for _ in 0..5 {
            onset.process_frame(&silence);
        }
        onset.process_frame(&loud);

        // Sustain then silence gap
        for _ in 0..10 {
            onset.process_frame(&loud);
        }
        for _ in 0..10 {
            onset.process_frame(&silence);
        }

        // Second note should trigger onset again
        let result = onset.process_frame(&loud);
        assert!(result.is_onset, "should re-trigger onset after silence gap");
    }
}
