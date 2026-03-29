use anyhow::{bail, Context, Result};
use clap::Parser;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

mod cli;

fn main() -> Result<()> {
    let args = cli::Cli::parse();

    // --test-audio: diagnostic mode that plays test tones through speakers
    if args.test_audio {
        return run_audio_test();
    }

    let score_path = args
        .score
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Score file required (or use --test-audio)"))?;

    // Load and parse score
    let score_text = std::fs::read_to_string(score_path)
        .with_context(|| format!("Failed to read {}", score_path.display()))?;
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

/// Audio diagnostic: plays a recognizable tone through each fidelity level
/// so you can verify your speakers/headphones actually produce sound.
fn run_audio_test() -> Result<()> {
    use haydn_performer::synth::SynthBackend;
    use haydn_performer::types::{NoteEvent, RestEvent, ScoreEvent};
    use std::time::Duration;

    eprintln!("=== Audio Diagnostic ===");
    eprintln!("You should hear 4 test tones, each with a different timbre.");
    eprintln!("If you hear nothing, check your system volume and audio output device.\n");

    let (_stream, stream_handle) =
        rodio::OutputStream::try_default().context("Failed to open audio output device")?;

    // C major triad (C4-E4-G4) at each fidelity level
    let test_sequence: Vec<ScoreEvent> = vec![
        ScoreEvent::Note(NoteEvent { midi_note: 60, duration: Duration::from_millis(400), velocity: 100 }),
        ScoreEvent::Note(NoteEvent { midi_note: 64, duration: Duration::from_millis(400), velocity: 100 }),
        ScoreEvent::Note(NoteEvent { midi_note: 67, duration: Duration::from_millis(400), velocity: 100 }),
        ScoreEvent::Rest(RestEvent { duration: Duration::from_millis(200) }),
    ];

    let labels = [
        "Fidelity 0 — pure sine",
        "Fidelity 1 — sine + ADSR envelope",
        "Fidelity 2 — blended waveforms + ADSR",
        "Fidelity 3 — expressive (vibrato + velocity)",
    ];

    for fidelity in 0..=3u8 {
        eprintln!("  Playing: {} ...", labels[fidelity as usize]);

        let backend = haydn_performer::synth::builtin::BuiltinSynth::new(fidelity);
        let source = backend.synthesize(&test_sequence, 44100);

        let sink = rodio::Sink::try_new(&stream_handle)
            .context("Failed to create audio sink")?;
        sink.append(source);
        sink.sleep_until_end();

        // Brief gap between fidelity levels
        std::thread::sleep(Duration::from_millis(300));
    }

    eprintln!("\n=== Results ===");
    eprintln!("  If you heard 4 distinct C-E-G triads: audio pipeline is working.");
    eprintln!("  If you heard nothing:");
    eprintln!("    1. Check system volume (not muted?)");
    eprintln!("    2. Check default audio output device in OS settings");
    eprintln!("    3. Try with headphones connected");
    eprintln!("  If some fidelities sounded identical: that's expected for 0 vs 1");
    eprintln!("    at short durations — ADSR differences are subtle on quick notes.");

    Ok(())
}
