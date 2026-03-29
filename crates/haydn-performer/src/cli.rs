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
    /// Path to a LilyPond (.ly) score file (not needed with --test-audio)
    pub score: Option<PathBuf>,

    /// Run audio diagnostic: plays a test tone through each fidelity level
    #[arg(long)]
    pub test_audio: bool,

    /// Loopback test: plays a tone through speakers and records via mic to verify
    #[arg(long)]
    pub test_loopback: bool,

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

    /// Write MIDI output to this file instead of playing audio (--synth midi)
    #[arg(long)]
    pub midi_out: Option<PathBuf>,

    /// Send MIDI to this named port (--synth midi, auto-detect if not set)
    #[arg(long)]
    pub midi_port: Option<String>,

    /// Quiet mode — play audio without TUI display
    #[arg(long, short)]
    pub quiet: bool,
}
