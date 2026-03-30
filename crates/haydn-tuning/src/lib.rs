pub mod types;
pub mod error;
pub mod validate;
pub mod engine;

pub use types::*;
pub use error::*;
pub use engine::TuningEngine;

const DEFAULT_PIANO_TOML: &str = include_str!("../tunings/piano.toml");
const DEFAULT_VOICE_TOML: &str = include_str!("../tunings/voice.toml");
const DEFAULT_GUITAR_TOML: &str = include_str!("../tunings/guitar.toml");

/// Load a tuning from a TOML string.
pub fn load_tuning(toml_content: &str) -> Result<TuningEngine, Vec<TuningError>> {
    let raw: RawTuningFile = toml::from_str(toml_content)
        .map_err(|e| vec![TuningError::Parse(e)])?;
    TuningEngine::compile(raw)
}

/// Load a tuning from a file path.
pub fn load_tuning_file(path: &std::path::Path) -> Result<TuningEngine, Vec<TuningError>> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| vec![TuningError::Io(e)])?;
    load_tuning(&content)
}

/// Load the built-in default piano tuning.
pub fn default_piano_tuning() -> TuningEngine {
    load_tuning(DEFAULT_PIANO_TOML)
        .expect("built-in piano tuning must be valid")
}

/// Load the built-in default voice tuning.
pub fn default_voice_tuning() -> TuningEngine {
    load_tuning(DEFAULT_VOICE_TOML)
        .expect("built-in voice tuning must be valid")
}

/// Load the built-in default guitar tuning.
pub fn default_guitar_tuning() -> TuningEngine {
    load_tuning(DEFAULT_GUITAR_TOML)
        .expect("built-in guitar tuning must be valid")
}
