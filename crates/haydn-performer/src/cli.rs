use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum SynthType {
    Builtin,
    Soundfont,
    Midi,
}

// Re-export for CLI access
pub use haydn_performer::synth::timbre::Instrument;

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

    /// Fidelity level (0=sine, 1=ADSR, 2=waveforms, 3=expressive, 4=realistic, 5=soundfont)
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

    /// List available audio input/output devices and exit
    #[arg(long)]
    pub list_devices: bool,

    /// Output audio device (substring match; use --list-devices to see options)
    #[arg(long)]
    pub output_device: Option<String>,

    /// Input audio device for loopback test (substring match)
    #[arg(long)]
    pub input_device: Option<String>,

    /// Playback volume (0.0 to 1.0, default 1.0)
    #[arg(long, default_value_t = 1.0)]
    pub volume: f32,

    /// Instrument timbre preset (piano, strings, flute, organ, clarinet, trumpet)
    #[arg(long, default_value = "piano", value_enum)]
    pub instrument: Instrument,

    /// Enable interpret mode: feed notes through tuning engine + VM alongside playback
    #[arg(long)]
    pub interpret: bool,

    /// Path to tuning file (required with --interpret)
    #[arg(long)]
    pub tuning: Option<PathBuf>,

    /// Pitch detection confidence threshold (0.0-1.0, lower = more sensitive)
    #[arg(long, default_value_t = 0.3)]
    pub confidence: f32,

    /// Noise gate in dB (more negative = more sensitive, e.g. -60)
    #[arg(long, default_value_t = -55.0)]
    pub noise_gate: f32,
}
