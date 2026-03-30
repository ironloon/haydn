pub mod capture;
pub mod gate;
pub mod onset;
pub mod pitch;
pub mod state_machine;
pub mod types;

pub use capture::{find_audio_input_device, list_audio_input_devices, start_audio_capture};
pub use gate::NoiseGate;
pub use onset::{OnsetDetector, OnsetResult};
pub use pitch::{freq_to_midi, PitchDetector, PitchEstimate};
pub use state_machine::{NoteState, NoteStateMachine};
pub use types::{AudioConfig, AudioError, AudioMsg};
