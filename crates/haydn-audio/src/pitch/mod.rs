pub mod mcleod;
pub mod yin;

pub use mcleod::McLeodDetector;
pub use yin::YinDetector;

/// A single pitch detection result.
#[derive(Debug, Clone)]
pub struct PitchEstimate {
    pub frequency_hz: f32,
    pub midi_note: u8,
    pub cents_offset: f32,
    pub confidence: f32,
}

/// Trait for pitch detection algorithms.
pub trait PitchDetector: Send {
    fn detect(&mut self, samples: &[f32], sample_rate: u32) -> Option<PitchEstimate>;
    fn name(&self) -> &str;
}

/// Convert a frequency in Hz to a MIDI note number and cents offset.
///
/// Returns `(midi_note, cents_offset)` where cents_offset is in the range -50..+50.
pub fn freq_to_midi(frequency_hz: f32) -> (u8, f32) {
    let midi_float = 69.0 + 12.0 * (frequency_hz / 440.0).log2();
    let midi_note = midi_float.round() as u8;
    let cents_offset = (midi_float - midi_note as f32) * 100.0;
    (midi_note, cents_offset)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn freq_to_midi_a4() {
        let (note, cents) = freq_to_midi(440.0);
        assert_eq!(note, 69);
        assert!(cents.abs() < 1.0);
    }

    #[test]
    fn freq_to_midi_bb4() {
        let (note, cents) = freq_to_midi(466.16);
        assert_eq!(note, 70);
        assert!(cents.abs() < 2.0);
    }

    #[test]
    fn freq_to_midi_c4() {
        let (note, cents) = freq_to_midi(261.63);
        assert_eq!(note, 60);
        assert!(cents.abs() < 2.0);
    }

    #[test]
    fn freq_to_midi_low_e2() {
        let (note, cents) = freq_to_midi(82.41);
        assert_eq!(note, 40);
        assert!(cents.abs() < 2.0);
    }

    #[test]
    fn freq_to_midi_high_c6() {
        let (note, cents) = freq_to_midi(1046.50);
        assert_eq!(note, 84);
        assert!(cents.abs() < 2.0);
    }
}
