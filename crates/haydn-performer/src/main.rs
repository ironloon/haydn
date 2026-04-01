use anyhow::{bail, Context, Result};
use clap::Parser;
use cpal::traits::{DeviceTrait, HostTrait};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

mod cli;

fn main() -> Result<()> {
    let args = cli::Cli::parse();

    // --list-devices: enumerate audio devices and exit
    if args.list_devices {
        return list_audio_devices();
    }

    // Validate volume range
    if args.volume < 0.0 || args.volume > 1.0 {
        bail!("--volume must be between 0.0 and 1.0 (got {})", args.volume);
    }

    // Validate fidelity range
    if args.fidelity > 5 {
        bail!("Fidelity must be 0-5 (got {})", args.fidelity);
    }

    // Validate fidelity 5 requires --soundfont
    if args.fidelity == 5 && args.soundfont.is_none() {
        bail!("--fidelity 5 requires --soundfont <path>. Download a GM SoundFont \
               (e.g., FluidR3_GM.sf2 or TimGM6mb.sf2) and provide the path.");
    }

    // Validate --interpret requires --tuning
    if args.interpret && args.tuning.is_none() {
        bail!("--interpret requires --tuning <path>. Specify which tuning file maps notes to VM opcodes.\n\
               Example: haydn-performer score.ly --interpret --tuning piano.toml");
    }

    // --test-audio: diagnostic mode that plays test tones through speakers
    if args.test_audio {
        return run_audio_test(args.output_device.as_deref(), args.volume);
    }

    // --test-loopback: play tone + record via mic to verify end-to-end audio
    if args.test_loopback {
        return run_loopback_test(
            args.output_device.as_deref(),
            args.input_device.as_deref(),
            args.volume,
        );
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
            3 => {
                let inst = format!("{:?}", args.instrument).to_lowercase();
                // Leak is fine: synth_name lives for the program's duration
                Box::leak(format!("built-in ({inst})").into_boxed_str()) as &str
            }
            4 => {
                let inst = format!("{:?}", args.instrument).to_lowercase();
                Box::leak(format!("built-in realistic ({inst})").into_boxed_str()) as &str
            }
            5 => "soundfont (SF2)",
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
            let backend = if args.fidelity == 5 {
                haydn_performer::synth::builtin::BuiltinSynth::with_soundfont(
                    args.instrument,
                    args.soundfont.clone().unwrap(), // validated above
                )
            } else {
                haydn_performer::synth::builtin::BuiltinSynth::with_instrument(
                    args.fidelity,
                    args.instrument,
                )
            };
            if args.interpret {
                play_with_interpret(
                    &backend,
                    &sequence,
                    synth_name,
                    args.bpm,
                    args.quiet,
                    args.output_device.as_deref(),
                    args.volume,
                    args.tuning.as_ref().unwrap(),
                )?;
            } else {
                play_with_display(
                    &backend,
                    &sequence,
                    synth_name,
                    args.bpm,
                    args.quiet,
                    args.output_device.as_deref(),
                    args.volume,
                )?;
            }
        }
        cli::SynthType::Soundfont => {
            let sf_path = args
                .soundfont
                .ok_or_else(|| anyhow::anyhow!("--soundfont path required for soundfont backend"))?;
            let backend = haydn_performer::synth::soundfont::SoundFontSynth::new(sf_path);
            if args.interpret {
                play_with_interpret(
                    &backend,
                    &sequence,
                    synth_name,
                    args.bpm,
                    args.quiet,
                    args.output_device.as_deref(),
                    args.volume,
                    args.tuning.as_ref().unwrap(),
                )?;
            } else {
                play_with_display(
                    &backend,
                    &sequence,
                    synth_name,
                    args.bpm,
                    args.quiet,
                    args.output_device.as_deref(),
                    args.volume,
                )?;
            }
        }
        cli::SynthType::Midi => {
            if args.interpret {
                bail!("--interpret is not supported with --synth midi (no audio synthesis for dual playback)");
            }
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
    output_device_name: Option<&str>,
    volume: f32,
) -> Result<()> {
    let (_stream, stream_handle) = open_output(output_device_name)?;
    let sink = rodio::Sink::try_new(&stream_handle).context("Failed to create audio sink")?;
    sink.set_volume(volume);

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

fn play_with_interpret(
    backend: &dyn haydn_performer::synth::SynthBackend,
    sequence: &haydn_performer::types::NoteSequence,
    backend_name: &str,
    bpm: u32,
    quiet: bool,
    output_device_name: Option<&str>,
    volume: f32,
    tuning_path: &std::path::Path,
) -> Result<()> {
    use haydn_performer::types::ScoreEvent;

    let interpret = haydn_performer::interpret::InterpretState::new(tuning_path)?;

    let (_stream, stream_handle) = open_output(output_device_name)?;
    let sink = rodio::Sink::try_new(&stream_handle).context("Failed to create audio sink")?;
    sink.set_volume(volume);

    let source = backend.synthesize(sequence, 44100);
    sink.append(source);

    if quiet {
        // Process all notes through VM and print to stderr
        let mut interpret = interpret;
        for event in sequence {
            if let ScoreEvent::Note(ref n) = event {
                let results = interpret.process_note(n.midi_note);
                for result in &results {
                    eprintln!("{}", haydn::format_event_log(n.midi_note, n.velocity, result));
                }
            }
        }
        sink.sleep_until_end();
        eprintln!("{}", haydn::format_session_summary(&interpret.vm));
        return Ok(());
    }

    // TUI mode: wrap interpret in Arc<Mutex<>> for display thread
    let interpret = Arc::new(Mutex::new(interpret));
    let stop_signal = Arc::new(AtomicBool::new(false));
    let display_stop = stop_signal.clone();
    let display_sequence = sequence.clone();
    let display_backend = backend_name.to_string();
    let interpret_clone = interpret.clone();

    let display_handle = std::thread::spawn(move || {
        let _ = haydn_performer::display::run_interpret_display(
            &display_sequence,
            &display_backend,
            bpm,
            display_stop,
            interpret_clone,
        );
    });

    sink.sleep_until_end();
    stop_signal.store(true, Ordering::Relaxed);
    let _ = display_handle.join();

    // Print session summary after TUI closes
    if let Ok(interp) = interpret.lock() {
        eprintln!("{}", haydn::format_session_summary(&interp.vm));
    }

    Ok(())
}

/// List all available audio input and output devices.
fn list_audio_devices() -> Result<()> {
    let host = cpal::default_host();

    eprintln!("=== Audio Devices ===\n");

    // Output devices
    let default_out = host.default_output_device();
    let default_out_name = default_out
        .as_ref()
        .and_then(|d| d.name().ok())
        .unwrap_or_default();

    eprintln!("Output devices:");
    match host.output_devices() {
        Ok(devices) => {
            let mut count = 0;
            for device in devices {
                let name = device.name().unwrap_or_else(|_| "unknown".into());
                let marker = if name == default_out_name { " (default)" } else { "" };
                eprintln!("  - {}{}", name, marker);
                count += 1;
            }
            if count == 0 {
                eprintln!("  (none found)");
            }
        }
        Err(e) => eprintln!("  Error enumerating: {}", e),
    }

    eprintln!();

    // Input devices
    let default_in = host.default_input_device();
    let default_in_name = default_in
        .as_ref()
        .and_then(|d| d.name().ok())
        .unwrap_or_default();

    eprintln!("Input devices:");
    match host.input_devices() {
        Ok(devices) => {
            let mut count = 0;
            for device in devices {
                let name = device.name().unwrap_or_else(|_| "unknown".into());
                let marker = if name == default_in_name { " (default)" } else { "" };
                eprintln!("  - {}{}", name, marker);
                count += 1;
            }
            if count == 0 {
                eprintln!("  (none found)");
            }
        }
        Err(e) => eprintln!("  Error enumerating: {}", e),
    }

    eprintln!("\nUse --output-device \"<name>\" or --input-device \"<name>\" to select.");
    eprintln!("Partial name matches work (e.g. --output-device \"Headphones\").");

    Ok(())
}

/// Open an audio output stream, optionally targeting a named device.
fn open_output(device_name: Option<&str>) -> Result<(rodio::OutputStream, rodio::OutputStreamHandle)> {
    match device_name {
        Some(name) => {
            let host = cpal::default_host();
            let device = host
                .output_devices()
                .context("Failed to enumerate output devices")?
                .find(|d| {
                    d.name()
                        .map(|n| n.to_lowercase().contains(&name.to_lowercase()))
                        .unwrap_or(false)
                })
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "No output device matching \"{}\". Use --list-devices to see available devices.",
                        name
                    )
                })?;
            let actual_name = device.name().unwrap_or_else(|_| "unknown".into());
            eprintln!("  Using output device: {}", actual_name);
            rodio::OutputStream::try_from_device(&device)
                .context(format!("Failed to open output device \"{}\"", actual_name))
        }
        None => rodio::OutputStream::try_default().context("Failed to open default audio output"),
    }
}

/// Find an input device by name substring, or fall back to default.
fn find_input_device(device_name: Option<&str>) -> Result<cpal::Device> {
    let host = cpal::default_host();
    match device_name {
        Some(name) => {
            let device = host
                .input_devices()
                .context("Failed to enumerate input devices")?
                .find(|d| {
                    d.name()
                        .map(|n| n.to_lowercase().contains(&name.to_lowercase()))
                        .unwrap_or(false)
                })
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "No input device matching \"{}\". Use --list-devices to see available devices.",
                        name
                    )
                })?;
            let actual_name = device.name().unwrap_or_else(|_| "unknown".into());
            eprintln!("  Using input device: {}", actual_name);
            Ok(device)
        }
        None => host
            .default_input_device()
            .ok_or_else(|| anyhow::anyhow!("No microphone / input device found")),
    }
}

/// Audio diagnostic: plays a recognizable tone through each fidelity level
/// so you can verify your speakers/headphones actually produce sound.
fn run_audio_test(output_device: Option<&str>, volume: f32) -> Result<()> {
    use haydn_performer::synth::SynthBackend;
    use haydn_performer::types::{NoteEvent, RestEvent, ScoreEvent};
    use std::time::Duration;

    eprintln!("=== Audio Diagnostic ===");
    eprintln!("You should hear 4 test tones, each with a different timbre.");
    eprintln!("If you hear nothing, check your system volume and audio output device.\n");

    let (_stream, stream_handle) = open_output(output_device)?;

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
        sink.set_volume(volume);
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

/// Loopback test: plays a 440Hz tone through speakers while recording from the
/// default microphone, then checks whether the mic captured the expected frequency.
///
/// Requirements: speakers + microphone active (not headphones-only).
fn run_loopback_test(
    output_device: Option<&str>,
    input_device_name: Option<&str>,
    volume: f32,
) -> Result<()> {
    use cpal::traits::{DeviceTrait, StreamTrait};
    use std::sync::Mutex;
    use std::time::Duration;

    const TEST_FREQ: f32 = 440.0; // A4
    const SAMPLE_RATE: u32 = 44100;
    const PLAY_SECS: f64 = 2.0;

    eprintln!("=== Loopback Test ===");
    eprintln!("This plays a 440Hz tone through speakers and records from your mic.");
    eprintln!("Make sure speakers + mic are active (not just headphones).\n");

    // --- Set up mic recording ---
    let input_device = find_input_device(input_device_name)?;

    let input_name = input_device.name().unwrap_or_else(|_| "unknown".into());
    eprintln!("  Mic: {}", input_name);

    let input_config = input_device
        .default_input_config()
        .context("Failed to get mic config")?;
    let mic_sample_rate = input_config.sample_rate().0;
    let mic_channels = input_config.channels() as usize;

    eprintln!(
        "  Mic config: {}Hz, {} ch, {:?}",
        mic_sample_rate, mic_channels, input_config.sample_format()
    );

    let recorded: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
    let recorded_clone = recorded.clone();

    let input_stream = match input_config.sample_format() {
        cpal::SampleFormat::F32 => {
            let rec = recorded_clone;
            input_device.build_input_stream(
                &input_config.into(),
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    // Take just first channel
                    let mut buf = rec.lock().unwrap();
                    for chunk in data.chunks(mic_channels) {
                        buf.push(chunk[0]);
                    }
                },
                |err| eprintln!("  Mic error: {}", err),
                None,
            )?
        }
        cpal::SampleFormat::I16 => {
            let rec = recorded_clone;
            input_device.build_input_stream(
                &input_config.into(),
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    let mut buf = rec.lock().unwrap();
                    for chunk in data.chunks(mic_channels) {
                        buf.push(chunk[0] as f32 / i16::MAX as f32);
                    }
                },
                |err| eprintln!("  Mic error: {}", err),
                None,
            )?
        }
        fmt => bail!("Unsupported mic sample format: {:?}", fmt),
    };

    // --- Start recording ---
    input_stream.play().context("Failed to start mic recording")?;
    eprintln!("  Recording started...");

    // Small delay to let mic settle
    std::thread::sleep(Duration::from_millis(200));

    // --- Play 440Hz tone ---
    eprintln!("  Playing 440Hz test tone for {:.0}s...", PLAY_SECS);
    let (_stream, stream_handle) = open_output(output_device)?;
    let sink = rodio::Sink::try_new(&stream_handle)?;
    sink.set_volume(volume);

    let tone = haydn_performer::synth::sine::SineSource::new(
        TEST_FREQ,
        Duration::from_secs_f64(PLAY_SECS),
        SAMPLE_RATE,
        0.8,
    );
    sink.append(tone);
    sink.sleep_until_end();

    // Record a bit of tail
    std::thread::sleep(Duration::from_millis(300));
    drop(input_stream);

    // --- Analyze recorded audio ---
    let samples = recorded.lock().unwrap();
    let num_samples = samples.len();
    eprintln!("\n  Recorded {} samples ({:.2}s at {}Hz)",
        num_samples,
        num_samples as f64 / mic_sample_rate as f64,
        mic_sample_rate
    );

    if num_samples < 1000 {
        eprintln!("\n  FAIL: Too few samples recorded. Mic may not be working.");
        eprintln!("  Check: System Settings > Sound > Input device");
        return Ok(());
    }

    // Check signal level (RMS)
    let rms: f32 = (samples.iter().map(|s| s * s).sum::<f32>() / num_samples as f32).sqrt();
    eprintln!("  RMS level: {:.4} ({:.1} dB)", rms, 20.0 * rms.log10());

    if rms < 0.001 {
        eprintln!("\n  FAIL: Mic recorded near-silence (RMS < 0.001).");
        eprintln!("  The tone played but the mic didn't pick it up.");
        eprintln!("  Causes: mic muted, speakers too quiet, or using headphones.");
        return Ok(());
    }

    // Zero-crossing frequency estimation (simple but effective for pure tones)
    let dominant_freq = estimate_frequency(&samples, mic_sample_rate);
    eprintln!("  Estimated dominant frequency: {:.1}Hz (expected: {:.1}Hz)", dominant_freq, TEST_FREQ);

    let tolerance = 30.0; // Hz — generous because room acoustics vary
    if (dominant_freq - TEST_FREQ).abs() < tolerance {
        eprintln!("\n  PASS: Mic captured the 440Hz tone successfully!");
        eprintln!("  Audio output -> speakers -> mic -> capture pipeline works end-to-end.");
    } else if dominant_freq > 50.0 {
        eprintln!("\n  PARTIAL: Mic captured audio at {:.0}Hz (expected 440Hz).", dominant_freq);
        eprintln!("  The mic is working but picked up a different dominant frequency.");
        eprintln!("  This could be ambient noise or room resonance. Try in a quieter space.");
    } else {
        eprintln!("\n  FAIL: Could not detect a clear tone from the mic recording.");
        eprintln!("  The mic recorded some signal but no clear pitch.");
    }

    Ok(())
}

/// Simple zero-crossing frequency estimation.
/// Counts positive-to-negative zero crossings and derives frequency.
fn estimate_frequency(samples: &[f32], sample_rate: u32) -> f32 {
    // Skip the first 10% (mic settling) and last 10% (tail noise)
    let start = samples.len() / 10;
    let end = samples.len() * 9 / 10;
    if end <= start + 100 {
        return 0.0;
    }
    let region = &samples[start..end];

    let mut crossings = 0u32;
    for pair in region.windows(2) {
        if pair[0] >= 0.0 && pair[1] < 0.0 {
            crossings += 1;
        }
    }

    let duration_secs = region.len() as f32 / sample_rate as f32;
    if duration_secs > 0.0 {
        crossings as f32 / duration_secs
    } else {
        0.0
    }
}
