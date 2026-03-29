pub mod adsr;
pub mod builtin;
pub mod expressive;
pub mod sine;
pub mod soundfont;
pub mod timbre;
pub mod waveform;

use crate::types::NoteSequence;

/// Trait for synthesis backends. Each layer implements this.
pub trait SynthBackend {
    /// Synthesize a note sequence into a playable rodio source.
    fn synthesize(
        &self,
        sequence: &NoteSequence,
        sample_rate: u32,
    ) -> Box<dyn rodio::Source<Item = f32> + Send>;

    /// Human-readable backend name for display.
    fn name(&self) -> &str;
}
