use anyhow::{bail, Context, Result};
use clap::Parser;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

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

    if !args.quiet {
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
            play_with_display(&backend, &sequence, synth_name, args.bpm, args.quiet)?;
        }
        cli::SynthType::Soundfont => {
            let sf_path = args
                .soundfont
                .ok_or_else(|| anyhow::anyhow!("--soundfont path required for soundfont backend"))?;
            let backend = haydn_performer::synth::soundfont::SoundFontSynth::new(sf_path);
            play_with_display(&backend, &sequence, synth_name, args.bpm, args.quiet)?;
        }
        cli::SynthType::Midi => {
            if let Some(ref midi_out) = args.midi_out {
                // Write MIDI file (no TUI, no audio)
                haydn_performer::midi::write_midi_file(&sequence, midi_out, args.bpm)
                    .map_err(|e| anyhow::anyhow!("{}", e))?;
                eprintln!("Wrote MIDI file to {}", midi_out.display());
            } else {
                // Send to MIDI port
                haydn_performer::midi::play_midi_port(
                    &sequence,
                    args.midi_port.as_deref(),
                )
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            }
        }
    }

    Ok(())
}

fn play_with_display(
    backend: &dyn haydn_performer::synth::SynthBackend,
    sequence: &haydn_performer::types::NoteSequence,
    backend_name: &str,
    bpm: u32,
    quiet: bool,
) -> Result<()> {
    let (_stream, stream_handle) =
        rodio::OutputStream::try_default().context("Failed to open audio output device")?;
    let sink = rodio::Sink::try_new(&stream_handle).context("Failed to create audio sink")?;

    let source = backend.synthesize(sequence, 44100);
    sink.append(source);

    if quiet || sink.empty() {
        if !sink.empty() {
            sink.sleep_until_end();
        }
        return Ok(());
    }

    // Start TUI display in a thread
    let stop_signal = Arc::new(AtomicBool::new(false));
    let display_stop = stop_signal.clone();
    let display_sequence = sequence.clone();
    let display_backend = backend_name.to_string();

    let display_handle = std::thread::spawn(move || {
        let _ = haydn_performer::display::run_display(
            &display_sequence,
            &display_backend,
            bpm,
            display_stop,
        );
    });

    // Wait for audio to finish
    sink.sleep_until_end();
    stop_signal.store(true, Ordering::Relaxed);

    // Wait for display thread to clean up
    let _ = display_handle.join();

    Ok(())
}
