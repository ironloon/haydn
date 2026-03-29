use crate::synth::sine::{midi_to_freq, SilenceSource, SineSource};
use crate::synth::SynthBackend;
use crate::types::{NoteSequence, ScoreEvent};
use std::time::Duration;

use super::adsr::AdsrEnvelope;

/// Built-in synthesizer with configurable fidelity.
/// - Fidelity 0: Pure sine waves
/// - Fidelity 1: Sine + ADSR envelope (default)
pub struct BuiltinSynth {
    pub fidelity: u8,
}

impl BuiltinSynth {
    pub fn new(fidelity: u8) -> Self {
        Self { fidelity }
    }
}

/// A source that applies an ADSR envelope to a sine wave.
struct AdsrSineSource {
    sine: SineSource,
    envelope: AdsrEnvelope,
}

impl Iterator for AdsrSineSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let sample = self.sine.next()?;
        let amp = self.envelope.amplitude();
        Some(sample * amp)
    }
}

impl rodio::Source for AdsrSineSource {
    fn current_frame_len(&self) -> Option<usize> {
        self.sine.current_frame_len()
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        self.sine.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        self.sine.total_duration()
    }
}

impl SynthBackend for BuiltinSynth {
    fn synthesize(
        &self,
        sequence: &NoteSequence,
        sample_rate: u32,
    ) -> Box<dyn rodio::Source<Item = f32> + Send> {
        let fidelity = self.fidelity;
        let sr = sample_rate;

        // Pre-render all events into a Vec<f32> buffer
        let mut buffer: Vec<f32> = Vec::new();

        for event in sequence {
            match event {
                ScoreEvent::Note(note) => {
                    let freq = midi_to_freq(note.midi_note);
                    let velocity_scale = note.velocity as f32 / 127.0;

                    if fidelity >= 1 {
                        // ADSR-shaped sine
                        let mut sine =
                            SineSource::new(freq, note.duration, sr, velocity_scale);
                        let mut env = AdsrEnvelope::piano(note.duration, sr);
                        while let Some(sample) = sine.next() {
                            buffer.push(sample * env.amplitude());
                        }
                    } else {
                        // Raw sine
                        let sine = SineSource::new(freq, note.duration, sr, velocity_scale);
                        buffer.extend(sine);
                    }
                }
                ScoreEvent::Rest(rest) => {
                    let silence = SilenceSource::new(rest.duration, sr);
                    buffer.extend(silence);
                }
            }
        }

        Box::new(rodio::buffer::SamplesBuffer::new(1, sample_rate, buffer))
    }

    fn name(&self) -> &str {
        match self.fidelity {
            0 => "built-in (sine)",
            1 => "built-in (sine + ADSR)",
            _ => "built-in",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{NoteEvent, RestEvent};

    #[test]
    fn test_builtin_synth_produces_audio() {
        let synth = BuiltinSynth::new(0);
        let sequence = vec![ScoreEvent::Note(NoteEvent {
            midi_note: 69, // A4
            duration: Duration::from_millis(100),
            velocity: 80,
        })];
        let source = synth.synthesize(&sequence, 44100);
        let total = source.total_duration();
        assert!(total.is_some());
    }

    #[test]
    fn test_builtin_synth_adsr_mode() {
        let synth = BuiltinSynth::new(1);
        let sequence = vec![
            ScoreEvent::Note(NoteEvent {
                midi_note: 60,
                duration: Duration::from_millis(200),
                velocity: 80,
            }),
            ScoreEvent::Rest(RestEvent {
                duration: Duration::from_millis(100),
            }),
            ScoreEvent::Note(NoteEvent {
                midi_note: 64,
                duration: Duration::from_millis(200),
                velocity: 80,
            }),
        ];
        let source = synth.synthesize(&sequence, 44100);
        assert!(source.total_duration().is_some());
    }

    #[test]
    fn test_builtin_synth_name() {
        assert_eq!(BuiltinSynth::new(0).name(), "built-in (sine)");
        assert_eq!(BuiltinSynth::new(1).name(), "built-in (sine + ADSR)");
    }
}
