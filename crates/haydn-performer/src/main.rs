use anyhow::{bail, Context, Result};
use clap::Parser;

mod cli;

fn main() -> Result<()> {
    let args = cli::Cli::parse();

    // Load and parse score
    let score_text = std::fs::read_to_string(&args.score)
        .with_context(|| format!("Failed to read {}", args.score.display()))?;
    let sequence = haydn_performer::parser::parse(&score_text, args.bpm)
        .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;

    if sequence.is_empty() {
        bail!("Score is empty — nothing to play");
    }

    if !args.quiet {
        let synth_name = match args.synth {
            cli::SynthType::Builtin => match args.fidelity {
                0 => "built-in (sine)",
                1 => "built-in (sine + ADSR)",
                2 => "built-in (waveform + ADSR)",
                3 => "built-in (expressive)",
                _ => "built-in",
            },
            cli::SynthType::Soundfont => "soundfont (SF2)",
            cli::SynthType::Midi => "midi",
        };
        eprintln!(
            "Playing {} events at {} BPM with {}",
            sequence.len(),
            args.bpm,
            synth_name
        );
    }

    match args.synth {
        cli::SynthType::Builtin => {
            let backend = haydn_performer::synth::builtin::BuiltinSynth::new(args.fidelity);
            play_audio(&backend, &sequence)?;
        }
        cli::SynthType::Soundfont => {
            let sf_path = args
                .soundfont
                .ok_or_else(|| anyhow::anyhow!("--soundfont path required for soundfont backend"))?;
            let backend = haydn_performer::synth::soundfont::SoundFontSynth::new(sf_path);
            play_audio(&backend, &sequence)?;
        }
        cli::SynthType::Midi => {
            bail!("MIDI output not yet implemented (coming in Plan 04)");
        }
    }

    Ok(())
}

fn play_audio(
    backend: &dyn haydn_performer::synth::SynthBackend,
    sequence: &haydn_performer::types::NoteSequence,
) -> Result<()> {
    let (_stream, stream_handle) =
        rodio::OutputStream::try_default().context("Failed to open audio output device")?;
    let sink =
        rodio::Sink::try_new(&stream_handle).context("Failed to create audio sink")?;

    let source = backend.synthesize(sequence, 44100);
    sink.append(source);

    if !sink.empty() {
        sink.sleep_until_end();
    }

    Ok(())
}
