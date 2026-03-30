/// Audio message events produced by the audio pipeline.
#[derive(Debug, Clone)]
pub enum AudioMsg {
    NoteOn { note: u8, confidence: f32 },
    NoteOff,
}

/// Configuration for audio capture and pitch detection.
#[derive(Debug, Clone)]
pub struct AudioConfig {
    pub noise_gate_db: f32,
    pub onset_threshold_db: f32,
    pub pitch_stability_cents: f32,
    pub min_note_ms: u32,
    pub confidence_threshold: f32,
    pub algorithm: String,
    pub window_size: usize,
    pub hop_size: usize,
    pub sample_rate: u32,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            noise_gate_db: -40.0,
            onset_threshold_db: 6.0,
            pitch_stability_cents: 50.0,
            min_note_ms: 80,
            confidence_threshold: 0.7,
            algorithm: "mcleod".to_string(),
            window_size: 2048,
            hop_size: 512,
            sample_rate: 44100,
        }
    }
}

/// Errors from the audio subsystem.
#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("audio device not found: {0}")]
    DeviceNotFound(String),
    #[error("audio stream error: {0}")]
    StreamError(String),
    #[error("audio config error: {0}")]
    ConfigError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audio_msg_note_on_constructs() {
        let msg = AudioMsg::NoteOn {
            note: 69,
            confidence: 0.95,
        };
        match msg {
            AudioMsg::NoteOn { note, confidence } => {
                assert_eq!(note, 69);
                assert!((confidence - 0.95).abs() < f32::EPSILON);
            }
            _ => panic!("expected NoteOn"),
        }
    }

    #[test]
    fn audio_msg_note_off_constructs() {
        let msg = AudioMsg::NoteOff;
        assert!(matches!(msg, AudioMsg::NoteOff));
    }

    #[test]
    fn audio_config_defaults() {
        let config = AudioConfig::default();
        assert!((config.noise_gate_db - (-40.0)).abs() < f32::EPSILON);
        assert!((config.onset_threshold_db - 6.0).abs() < f32::EPSILON);
        assert!((config.pitch_stability_cents - 50.0).abs() < f32::EPSILON);
        assert_eq!(config.min_note_ms, 80);
        assert!((config.confidence_threshold - 0.7).abs() < f32::EPSILON);
        assert_eq!(config.algorithm, "mcleod");
        assert_eq!(config.window_size, 2048);
        assert_eq!(config.hop_size, 512);
        assert_eq!(config.sample_rate, 44100);
    }
}
