use crate::synth::effects;
use crate::synth::expressive::ExpressiveSource;
use crate::synth::sine::{midi_to_freq, SilenceSource, SineSource};
use crate::synth::timbre::Instrument;
use crate::synth::waveform::WaveformSource;
use crate::synth::SynthBackend;
use crate::types::{NoteSequence, ScoreEvent};

use super::adsr::AdsrEnvelope;

/// Built-in synthesizer with configurable fidelity.
/// - Fidelity 0: Pure sine waves
/// - Fidelity 1: Sine + ADSR envelope (default)
/// - Fidelity 2: Blended waveforms + ADSR (richer timbre)
/// - Fidelity 3: Expressive (additive synthesis with instrument profiles)
pub struct BuiltinSynth {
    pub fidelity: u8,
    pub instrument: Instrument,
}

impl BuiltinSynth {
    pub fn new(fidelity: u8) -> Self {
        Self {
            fidelity: fidelity.min(3),
            instrument: Instrument::default(),
        }
    }

    pub fn with_instrument(fidelity: u8, instrument: Instrument) -> Self {
        Self {
            fidelity: fidelity.min(3),
            instrument,
        }
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

        let mut buffer: Vec<f32> = Vec::new();

        for event in sequence {
            match event {
                ScoreEvent::Note(note) => {
                    let freq = midi_to_freq(note.midi_note);
                    let velocity_scale = note.velocity as f32 / 127.0;

                    match fidelity {
                        0 => {
                            // Raw sine
                            let sine = SineSource::new(freq, note.duration, sr, velocity_scale);
                            buffer.extend(sine);
                        }
                        1 => {
                            // Sine + ADSR
                            let mut sine = SineSource::new(freq, note.duration, sr, velocity_scale);
                            let mut env = AdsrEnvelope::piano(note.duration, sr);
                            while let Some(sample) = sine.next() {
                                buffer.push(sample * env.amplitude());
                            }
                        }
                        2 => {
                            // Waveform blend + ADSR
                            let mut waveform =
                                WaveformSource::new(freq, note.duration, sr, velocity_scale);
                            let mut env = AdsrEnvelope::piano(note.duration, sr);
                            while let Some(sample) = waveform.next() {
                                buffer.push(sample * env.amplitude());
                            }
                        }
                        3 => {
                            // Expressive: additive synthesis + spectral evolution
                            // + attack transients + instrument envelope
                            let mut expressive = ExpressiveSource::with_instrument(
                                freq,
                                note.duration,
                                sr,
                                1.0,
                                note.velocity,
                                self.instrument,
                            );
                            let profile = self.instrument.profile();
                            let mut env = AdsrEnvelope::from_profile(&profile, note.duration, sr);
                            while let Some(sample) = expressive.next() {
                                buffer.push(sample * env.amplitude());
                            }
                        }
                        _ => unreachable!(),
                    }
                }
                ScoreEvent::Rest(rest) => {
                    let silence = SilenceSource::new(rest.duration, sr);
                    buffer.extend(silence);
                }
            }
        }

        // Fidelity 3: apply post-processing chain (reverb → saturation → stereo)
        if fidelity == 3 {
            // Soft saturation to warm peaks and prevent clipping
            effects::soft_saturate(&mut buffer, 1.3);

            // Algorithmic reverb for room acoustics
            let mut reverb = effects::Reverb::hall(sr);
            reverb.process(&mut buffer);

            // Convert to stereo with Haas-effect width
            let stereo = effects::mono_to_stereo(&buffer, sr);
            return Box::new(rodio::buffer::SamplesBuffer::new(2, sample_rate, stereo));
        }

        Box::new(rodio::buffer::SamplesBuffer::new(1, sample_rate, buffer))
    }

    fn name(&self) -> &str {
        match self.fidelity {
            0 => "built-in (sine)",
            1 => "built-in (sine + ADSR)",
            2 => "built-in (waveform + ADSR)",
            3 => "built-in (expressive)",
            _ => "built-in",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{NoteEvent, RestEvent};
    use std::time::Duration;

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
