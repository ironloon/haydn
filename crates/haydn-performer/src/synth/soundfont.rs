use crate::synth::SynthBackend;
use crate::types::{NoteSequence, ScoreEvent};
use std::path::PathBuf;
use std::sync::Arc;

/// SoundFont-based synthesizer using rustysynth.
///
/// Loads a .sf2 file and renders note sequences with realistic instrument timbres.
pub struct SoundFontSynth {
    sf2_path: PathBuf,
    program: Option<u8>,
}

impl SoundFontSynth {
    pub fn new(sf2_path: PathBuf) -> Self {
        Self { sf2_path, program: None }
    }

    /// Create a SoundFontSynth with GM program number mapped from instrument.
    pub fn with_instrument(sf2_path: PathBuf, instrument: super::timbre::Instrument) -> Self {
        let program = match instrument {
            super::timbre::Instrument::Piano => 0,       // Acoustic Grand Piano
            super::timbre::Instrument::Strings => 48,    // String Ensemble 1
            super::timbre::Instrument::Cello => 42,      // Cello
            super::timbre::Instrument::Flute => 73,      // Flute
            super::timbre::Instrument::Organ => 19,      // Church Organ
            super::timbre::Instrument::Clarinet => 71,   // Clarinet
            super::timbre::Instrument::Trumpet => 56,    // Trumpet
        };
        Self { sf2_path, program: Some(program) }
    }
}

impl SynthBackend for SoundFontSynth {
    fn synthesize(
        &self,
        sequence: &NoteSequence,
        sample_rate: u32,
    ) -> Box<dyn rodio::Source<Item = f32> + Send> {
        let mut file = match std::fs::File::open(&self.sf2_path) {
            Ok(f) => f,
            Err(_) => {
                eprintln!(
                    "SoundFont file not found: {}. Download a GM SoundFont \
                     (e.g., TimGM6mb.sf2 or FluidR3_GM.sf2) and pass with --soundfont.",
                    self.sf2_path.display()
                );
                // Return silence
                let silence: Vec<f32> = Vec::new();
                return Box::new(rodio::buffer::SamplesBuffer::new(2, sample_rate, silence));
            }
        };

        let sound_font = match rustysynth::SoundFont::new(&mut file) {
            Ok(sf) => Arc::new(sf),
            Err(e) => {
                eprintln!("Failed to load SoundFont: {}", e);
                let silence: Vec<f32> = Vec::new();
                return Box::new(rodio::buffer::SamplesBuffer::new(2, sample_rate, silence));
            }
        };

        let settings = rustysynth::SynthesizerSettings::new(sample_rate as i32);
        let mut synth = match rustysynth::Synthesizer::new(&sound_font, &settings) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to create SoundFont synthesizer: {}", e);
                let silence: Vec<f32> = Vec::new();
                return Box::new(rodio::buffer::SamplesBuffer::new(2, sample_rate, silence));
            }
        };

        // Send GM program change if instrument mapping is set
        if let Some(pg) = self.program {
            synth.process_midi_message(0, 0xC0, pg as i32, 0);
        }

        // Pre-render entire sequence into stereo buffer
        let mut left: Vec<f32> = Vec::new();
        let mut right: Vec<f32> = Vec::new();

        for event in sequence {
            match event {
                ScoreEvent::Note(note) => {
                    let num_samples =
                        (note.duration.as_secs_f64() * sample_rate as f64) as usize;
                    if num_samples == 0 {
                        continue;
                    }

                    synth.note_on(0, note.midi_note as i32, note.velocity as i32);

                    let mut l_buf = vec![0f32; num_samples];
                    let mut r_buf = vec![0f32; num_samples];
                    synth.render(&mut l_buf, &mut r_buf);

                    synth.note_off(0, note.midi_note as i32);

                    // Render a short release tail (50ms)
                    let release_samples = (0.05 * sample_rate as f64) as usize;
                    let mut l_rel = vec![0f32; release_samples];
                    let mut r_rel = vec![0f32; release_samples];
                    synth.render(&mut l_rel, &mut r_rel);

                    left.extend_from_slice(&l_buf);
                    left.extend_from_slice(&l_rel);
                    right.extend_from_slice(&r_buf);
                    right.extend_from_slice(&r_rel);
                }
                ScoreEvent::Rest(rest) => {
                    let num_samples =
                        (rest.duration.as_secs_f64() * sample_rate as f64) as usize;
                    if num_samples == 0 {
                        continue;
                    }
                    // Render silence (advances synthesizer state)
                    let mut l_buf = vec![0f32; num_samples];
                    let mut r_buf = vec![0f32; num_samples];
                    synth.render(&mut l_buf, &mut r_buf);

                    left.extend_from_slice(&l_buf);
                    right.extend_from_slice(&r_buf);
                }
            }
        }

        // Interleave L/R for rodio stereo
        let mut interleaved = Vec::with_capacity(left.len() * 2);
        for (l, r) in left.iter().zip(right.iter()) {
            interleaved.push(*l);
            interleaved.push(*r);
        }

        Box::new(rodio::buffer::SamplesBuffer::new(2, sample_rate, interleaved))
    }

    fn name(&self) -> &str {
        "soundfont"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_soundfont_synth_new() {
        let synth = SoundFontSynth::new(PathBuf::from("nonexistent.sf2"));
        assert_eq!(synth.name(), "soundfont");
    }
}
