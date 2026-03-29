#[derive(Debug, thiserror::Error)]
pub enum TuningError {
    #[error("TOML parse error: {0}")]
    Parse(#[from] toml::de::Error),

    #[error("in [{section}]: {message}")]
    Validation { section: String, message: String },

    #[error("unknown opcode '{found}'{}", suggestion.as_ref().map(|s| format!(" — did you mean '{s}'?")).unwrap_or_default())]
    UnknownOpcode {
        found: String,
        suggestion: Option<String>,
    },

    #[error("overlapping rules: {rule_a} and {rule_b} both match note {note}")]
    Overlap {
        rule_a: String,
        rule_b: String,
        note: u8,
    },

    #[error("note {note} out of MIDI range (0-127)")]
    NoteOutOfRange { note: u8 },

    #[error("{0}")]
    Io(#[from] std::io::Error),
}

pub struct ValidationResult {
    pub errors: Vec<TuningError>,
    pub warnings: Vec<String>,
}
