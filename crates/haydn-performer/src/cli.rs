use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum SynthType {
    Builtin,
    Soundfont,
    Midi,
}

#[derive(Parser, Debug)]
#[command(
    name = "haydn-performer",
    about = "Play LilyPond notation through synthesized audio"
)]
pub struct Cli {
    /// Path to a LilyPond (.ly) score file
    pub score: PathBuf,

    /// Synthesis backend
    #[arg(long, default_value = "builtin", value_enum)]
    pub synth: SynthType,

    /// Fidelity level (0=sine, 1=ADSR, 2=waveforms, 3=expressive)
    #[arg(long, default_value_t = 1)]
    pub fidelity: u8,

    /// Tempo in beats per minute
    #[arg(long, default_value_t = 120)]
    pub bpm: u32,

    /// Path to SoundFont (.sf2) file (required for --synth soundfont)
    #[arg(long)]
    pub soundfont: Option<PathBuf>,

    /// Quiet mode — play audio without TUI display
    #[arg(long, short)]
    pub quiet: bool,
}
