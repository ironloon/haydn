pub mod gate;
pub mod onset;
pub mod pitch;
pub mod state_machine;
pub mod types;

pub use gate::NoiseGate;
pub use onset::{OnsetDetector, OnsetResult};
pub use pitch::{freq_to_midi, PitchDetector, PitchEstimate};
pub use state_machine::{NoteState, NoteStateMachine};
pub use types::{AudioConfig, AudioError, AudioMsg};
