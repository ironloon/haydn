use std::time::Duration;

/// A single note event with MIDI note number, duration, and velocity.
/// This is the interchange format — MIDI note numbers (0-127, C4=60).
#[derive(Debug, Clone, PartialEq)]
pub struct NoteEvent {
    pub midi_note: u8,
    pub duration: Duration,
    pub velocity: u8,
}

/// A rest (silence) for a duration.
#[derive(Debug, Clone, PartialEq)]
pub struct RestEvent {
    pub duration: Duration,
}

/// An element in a parsed score.
#[derive(Debug, Clone, PartialEq)]
pub enum ScoreEvent {
    Note(NoteEvent),
    Rest(RestEvent),
}

/// A complete parsed score — ordered sequence of events.
pub type NoteSequence = Vec<ScoreEvent>;
