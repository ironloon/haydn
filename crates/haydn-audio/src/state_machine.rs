use crate::gate::NoiseGate;
use crate::onset::OnsetDetector;
use crate::pitch::PitchEstimate;
use crate::types::{AudioConfig, AudioMsg};

/// Internal note state.
#[derive(Debug, Clone)]
pub enum NoteState {
    Silence,
    Onset,
    Active { note: u8, confidence: f32 },
}

/// Converts frame-by-frame pitch/onset/gate results into discrete NoteOn/NoteOff events.
pub struct NoteStateMachine {
    state: NoteState,
    gate: NoiseGate,
    onset: OnsetDetector,
    confidence_threshold: f32,
    min_note_ms: u32,
    frames_since_last_note: u32,
    onset_frames: u8,
    sample_rate: u32,
    hop_size: usize,
}

impl NoteStateMachine {
    pub fn new(config: &AudioConfig) -> Self {
        Self {
            state: NoteState::Silence,
            gate: NoiseGate::new(config.noise_gate_db),
            onset: OnsetDetector::new(config.onset_threshold_db),
            confidence_threshold: config.confidence_threshold,
            min_note_ms: config.min_note_ms,
            frames_since_last_note: u32::MAX, // allow first note immediately
            onset_frames: 0,
            sample_rate: config.sample_rate,
            hop_size: config.hop_size,
        }
    }

    /// How many milliseconds each hop represents.
    fn ms_per_frame(&self) -> f32 {
        self.hop_size as f32 / self.sample_rate as f32 * 1000.0
    }

    /// Check if enough time has passed since last note for debounce.
    fn debounce_allows(&self) -> bool {
        let elapsed_ms = self.frames_since_last_note as f32 * self.ms_per_frame();
        elapsed_ms >= self.min_note_ms as f32
    }

    /// Process one audio frame and return any emitted events.
    ///
    /// `samples` is the current hop of audio (for RMS/onset analysis).
    /// `pitch` is the pitch detection result for the current analysis window.
    pub fn process_frame(
        &mut self,
        samples: &[f32],
        pitch: Option<&PitchEstimate>,
    ) -> Vec<AudioMsg> {
        let gate_active = self.gate.is_active(samples);
        let onset_result = self.onset.process_frame(samples);
        self.frames_since_last_note = self.frames_since_last_note.saturating_add(1);

        let mut events = Vec::new();

        match self.state.clone() {
            NoteState::Silence => {
                if gate_active && onset_result.is_onset {
                    self.state = NoteState::Onset;
                    self.onset_frames = 0;
                }
            }

            NoteState::Onset => {
                self.onset_frames += 1;

                if let Some(est) = pitch {
                    if est.confidence >= self.confidence_threshold && self.debounce_allows() {
                        self.state = NoteState::Active {
                            note: est.midi_note,
                            confidence: est.confidence,
                        };
                        self.frames_since_last_note = 0;
                        events.push(AudioMsg::NoteOn {
                            note: est.midi_note,
                            confidence: est.confidence,
                        });
                    }
                }

                // Timeout: if no pitch confirmed within ~5 frames, go back to silence
                if matches!(self.state, NoteState::Onset) && self.onset_frames > 5 {
                    self.state = NoteState::Silence;
                }
            }

            NoteState::Active { note, .. } => {
                if !gate_active {
                    // Signal dropped to silence
                    self.state = NoteState::Silence;
                    events.push(AudioMsg::NoteOff);
                } else if onset_result.is_onset {
                    // Possible new note
                    if let Some(est) = pitch {
                        if est.midi_note != note
                            && est.confidence >= self.confidence_threshold
                            && self.debounce_allows()
                        {
                            // Note change: NoteOff old, NoteOn new
                            events.push(AudioMsg::NoteOff);
                            events.push(AudioMsg::NoteOn {
                                note: est.midi_note,
                                confidence: est.confidence,
                            });
                            self.state = NoteState::Active {
                                note: est.midi_note,
                                confidence: est.confidence,
                            };
                            self.frames_since_last_note = 0;
                        }
                    }
                }
                // Otherwise: same note sustained, pitch dropout but gate active — stay Active
            }
        }

        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> AudioConfig {
        AudioConfig {
            min_note_ms: 80,
            ..AudioConfig::default()
        }
    }

    fn silence_frame() -> Vec<f32> {
        vec![0.0f32; 512]
    }

    fn loud_sine_frame(freq: f32, offset: usize) -> Vec<f32> {
        (0..512)
            .map(|i| {
                let t = (i + offset) as f32 / 44100.0;
                (2.0 * std::f32::consts::PI * freq * t).sin() * 0.5
            })
            .collect()
    }

    fn pitch_est(note: u8, confidence: f32) -> PitchEstimate {
        PitchEstimate {
            frequency_hz: 440.0,
            midi_note: note,
            cents_offset: 0.0,
            confidence,
        }
    }

    #[test]
    fn silence_no_events() {
        let mut sm = NoteStateMachine::new(&default_config());
        let frame = silence_frame();
        for _ in 0..10 {
            let events = sm.process_frame(&frame, None);
            assert!(events.is_empty());
        }
    }

    #[test]
    fn onset_to_note_on() {
        let mut sm = NoteStateMachine::new(&default_config());

        // Establish silence baseline
        for _ in 0..5 {
            sm.process_frame(&silence_frame(), None);
        }

        // Loud frame triggers onset, then pitch confirmation triggers NoteOn
        let loud = loud_sine_frame(440.0, 0);
        let est = pitch_est(69, 0.9);

        // First loud frame: onset detected → ONSET state
        let events = sm.process_frame(&loud, Some(&est));
        // The onset and pitch can happen in the same frame,
        // transitioning SILENCE → ONSET → ACTIVE
        // OR onset fires, then next frame confirms pitch
        // Either way, within a few frames we should get NoteOn

        let mut all_events = events;
        for i in 1..5 {
            let frame = loud_sine_frame(440.0, i * 512);
            let events = sm.process_frame(&frame, Some(&est));
            all_events.extend(events);
        }

        let note_ons: Vec<_> = all_events
            .iter()
            .filter(|e| matches!(e, AudioMsg::NoteOn { .. }))
            .collect();
        assert_eq!(note_ons.len(), 1, "should get exactly one NoteOn");
        match note_ons[0] {
            AudioMsg::NoteOn { note, .. } => assert_eq!(*note, 69),
            _ => unreachable!(),
        }
    }

    #[test]
    fn sustained_no_duplicate() {
        let mut sm = NoteStateMachine::new(&default_config());

        // Silence baseline
        for _ in 0..5 {
            sm.process_frame(&silence_frame(), None);
        }

        // Get to Active state
        let est = pitch_est(69, 0.9);
        let mut all_events = Vec::new();
        for i in 0..20 {
            let frame = loud_sine_frame(440.0, i * 512);
            let events = sm.process_frame(&frame, Some(&est));
            all_events.extend(events);
        }

        let note_ons: Vec<_> = all_events
            .iter()
            .filter(|e| matches!(e, AudioMsg::NoteOn { .. }))
            .collect();
        assert_eq!(
            note_ons.len(),
            1,
            "sustained note should produce exactly one NoteOn"
        );
    }

    #[test]
    fn note_off_on_silence() {
        let mut sm = NoteStateMachine::new(&default_config());

        // Silence baseline then note on
        for _ in 0..5 {
            sm.process_frame(&silence_frame(), None);
        }
        let est = pitch_est(69, 0.9);
        let mut all_events = Vec::new();
        for i in 0..10 {
            let frame = loud_sine_frame(440.0, i * 512);
            all_events.extend(sm.process_frame(&frame, Some(&est)));
        }

        // Now silence → should get NoteOff
        for _ in 0..5 {
            all_events.extend(sm.process_frame(&silence_frame(), None));
        }

        let note_offs: Vec<_> = all_events
            .iter()
            .filter(|e| matches!(e, AudioMsg::NoteOff))
            .collect();
        assert!(
            !note_offs.is_empty(),
            "should get NoteOff when signal drops to silence"
        );
    }

    #[test]
    fn note_change_produces_off_then_on() {
        let mut sm = NoteStateMachine::new(&AudioConfig {
            min_note_ms: 10, // short debounce for test
            ..AudioConfig::default()
        });

        // Silence baseline
        for _ in 0..5 {
            sm.process_frame(&silence_frame(), None);
        }

        // First note
        let est_a = pitch_est(69, 0.9);
        let mut all_events = Vec::new();
        for i in 0..10 {
            let frame = loud_sine_frame(440.0, i * 512);
            all_events.extend(sm.process_frame(&frame, Some(&est_a)));
        }

        // Brief silence gap to reset onset
        for _ in 0..5 {
            all_events.extend(sm.process_frame(&silence_frame(), None));
        }

        // Second note (different pitch)
        let est_c = pitch_est(72, 0.9);
        for i in 0..10 {
            let frame = loud_sine_frame(523.25, i * 512);
            all_events.extend(sm.process_frame(&frame, Some(&est_c)));
        }

        let note_ons: Vec<_> = all_events
            .iter()
            .filter(|e| matches!(e, AudioMsg::NoteOn { .. }))
            .collect();
        assert!(
            note_ons.len() >= 2,
            "should get at least 2 NoteOn events for two different notes"
        );
    }
}
