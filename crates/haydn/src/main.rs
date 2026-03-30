use std::io::Write;
use std::sync::mpsc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use clap::Parser;
use crossterm::event::{self as ct_event, Event as CtEvent, KeyCode, KeyEventKind};
use crossterm::ExecutableCommand;
use midir::MidiInput;

use haydn::{MidiMsg, display, midi_callback, note_name, process_note, process_note_structured, format_session_summary};

#[derive(Parser, Debug)]
#[command(name = "haydn", about = "Haydn — an esoteric programming language performed by music")]
struct Cli {
    /// List available MIDI input ports and exit
    #[arg(long)]
    list_midi: bool,

    /// Connect to MIDI device matching this name (substring match)
    #[arg(long)]
    midi_device: Option<String>,

    /// Path to a tuning file (.toml). Defaults to built-in piano tuning.
    #[arg(long)]
    tuning: Option<std::path::PathBuf>,

    /// Quiet mode — use scrolling text log instead of TUI dashboard
    #[arg(long, short)]
    quiet: bool,

    /// Demo mode — simulate MIDI input without hardware
    #[arg(long)]
    demo: bool,
}

fn select_midi_port(
    midi_in: &MidiInput,
    device_filter: Option<&str>,
) -> Result<(midir::MidiInputPort, String)> {
    let ports = midi_in.ports();
    if ports.is_empty() {
        anyhow::bail!(
            "No MIDI input devices found.\n\n\
             Troubleshooting:\n  \
             - Connect a MIDI device via USB\n  \
             - Check your OS MIDI/audio settings\n  \
             - Run 'haydn --list-midi' to verify detection"
        );
    }

    if let Some(filter) = device_filter {
        let filter_lower = filter.to_lowercase();
        for port in &ports {
            if let Ok(name) = midi_in.port_name(port) {
                if name.to_lowercase().contains(&filter_lower) {
                    println!("Connected to: {}", name);
                    return Ok((port.clone(), name));
                }
            }
        }
        let available: Vec<String> = ports
            .iter()
            .filter_map(|p| midi_in.port_name(p).ok())
            .collect();
        anyhow::bail!(
            "No MIDI device matching '{}' found.\n\nAvailable devices:\n{}",
            filter,
            available
                .iter()
                .enumerate()
                .map(|(i, n)| format!("  {}: {}", i + 1, n))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }

    if ports.len() == 1 {
        let name = midi_in.port_name(&ports[0])?;
        println!("Connected to: {}", name);
        return Ok((ports[0].clone(), name));
    }

    // Multiple devices, interactive selection
    println!("Multiple MIDI devices found:");
    let mut names = Vec::new();
    for (i, port) in ports.iter().enumerate() {
        let name = midi_in.port_name(port)?;
        println!("  {}: {}", i + 1, name);
        names.push(name);
    }

    loop {
        print!("Select MIDI device (1-{}): ", ports.len());
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if let Ok(idx) = input.trim().parse::<usize>() {
            if idx >= 1 && idx <= ports.len() {
                let name = names[idx - 1].clone();
                println!("Connected to: {}", name);
                return Ok((ports[idx - 1].clone(), name));
            }
        }
        println!("Invalid selection. Please enter a number between 1 and {}.", ports.len());
    }
}

fn wait_for_reconnect(
    device_name: &str,
    tx: mpsc::Sender<MidiMsg>,
    running: &Arc<AtomicBool>,
) -> Result<midir::MidiInputConnection<mpsc::Sender<MidiMsg>>> {
    loop {
        std::thread::sleep(Duration::from_secs(1));
        if !running.load(Ordering::Relaxed) {
            anyhow::bail!("Session interrupted while waiting for MIDI device");
        }
        if let Ok(probe) = MidiInput::new("haydn-probe") {
            let ports = probe.ports();
            let found = ports.iter().find(|p| {
                probe.port_name(p).ok().as_deref() == Some(device_name)
            });
            if found.is_some() {
                // probe was consumed by ports() borrow — create fresh MidiInput to connect
                let midi_in = MidiInput::new("haydn-reconnect")?;
                let ports = midi_in.ports();
                if let Some(port) = ports.iter().find(|p| {
                    midi_in.port_name(p).ok().as_deref() == Some(device_name)
                }) {
                    let conn = midi_in.connect(port, "haydn-input", midi_callback, tx)?;
                    return Ok(conn);
                }
            }
        }
    }
}

/// Simulate a MIDI performance: pushes values, does arithmetic, prints output.
/// Sends NoteOn events through the channel with human-like timing.
fn run_demo_sequence(tx: mpsc::Sender<MidiMsg>, running: &Arc<AtomicBool>) {
    // Piano tuning: notes 36-59 push values (note - 60), so all negative (-24..-1).
    // Operations on white keys 60+. To get positive ASCII, multiply two negatives.
    //
    // Note map: 52=E3→-8, 51=Eb3→-9, 53=F3→-7, 45=A2→-15, 57=A3→-3, 49=C#3→-11
    //           55=G3→-5
    // Ops:      60=Add, 62=Sub, 64=Dup, 67=Mul, 81=PrintChar, 83=PrintNum
    let sequence: Vec<(u8, u64)> = vec![
        // Push -8, Push -9, Mul → 72 ('H'), PrintChar
        (52, 400), (51, 400), (67, 500), (81, 600),
        // Push -7, Push -15, Mul → 105 ('i'), PrintChar
        (53, 400), (45, 400), (67, 500), (81, 600),
        // Push -3, Push -11, Mul → 33 ('!'), PrintChar
        (57, 400), (49, 400), (67, 500), (81, 600),
        // Push -5, Push -5, Mul → 25, PrintNum
        (55, 400), (55, 300), (67, 400), (83, 600),
        // Push -5, Dup, Mul → 25, leave on stack to show panel
        (55, 400), (64, 300), (67, 500),
        // Push -3, Push -2 — more visible stack items
        (57, 400), (58, 500),
    ];

    // Small initial delay so TUI renders empty state first
    std::thread::sleep(Duration::from_millis(800));

    for (note, delay_ms) in sequence {
        if !running.load(Ordering::Relaxed) {
            return;
        }
        let _ = tx.send(MidiMsg::NoteOn { note, velocity: 80 });
        std::thread::sleep(Duration::from_millis(delay_ms));
    }

    // Keep running so user can inspect the final state (quit with 'q')
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // --list-midi: enumerate and exit
    if cli.list_midi {
        let midi_in = MidiInput::new("haydn-list")?;
        let ports = midi_in.ports();
        if ports.is_empty() {
            println!("No MIDI input devices found.");
        } else {
            println!("Available MIDI input devices:");
            for (i, port) in ports.iter().enumerate() {
                if let Ok(name) = midi_in.port_name(port) {
                    println!("  {}: {}", i + 1, name);
                }
            }
        }
        return Ok(());
    }

    // Load tuning engine
    let mut engine = if let Some(ref path) = cli.tuning {
        haydn_tuning::load_tuning_file(path).map_err(|errors| {
            let msgs: Vec<String> = errors.iter().map(|e| format!("  - {}", e)).collect();
            anyhow::anyhow!("Tuning file errors:\n{}", msgs.join("\n"))
        })?
    } else {
        haydn_tuning::default_piano_tuning()
    };
    println!("Tuning: {} (root={})", engine.metadata().name, note_name(engine.root_note()));

    // Set up channel
    let (tx, rx) = mpsc::channel::<MidiMsg>();

    // Ctrl+C handler
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::Relaxed);
    })?;

    // MIDI source: real device or demo simulator
    let connected_name;
    let mut _conn: Option<midir::MidiInputConnection<mpsc::Sender<MidiMsg>>> = None;
    let reconnect_tx = tx.clone();

    if cli.demo {
        connected_name = "Demo (simulated)".to_string();
        let demo_tx = tx.clone();
        let demo_running = running.clone();
        std::thread::spawn(move || {
            run_demo_sequence(demo_tx, &demo_running);
        });
    } else {
        let midi_in = MidiInput::new("haydn")?;
        let (port, name) = select_midi_port(&midi_in, cli.midi_device.as_deref())?;
        connected_name = name;
        _conn = Some(midi_in.connect(&port, "haydn-input", midi_callback, tx.clone())?);
    }

    // Main event loop
    let mut vm = haydn_vm::HaydnVm::new();

    if cli.quiet {
        // Quiet mode — existing scrolling text log (Phase 4 behavior)
        println!("Listening for MIDI input... (Ctrl+C to quit)\n");

        while running.load(Ordering::Relaxed) {
            match rx.recv_timeout(Duration::from_millis(500)) {
                Ok(MidiMsg::NoteOn { note, velocity }) => {
                    for line in process_note(note, velocity, &mut engine, &mut vm) {
                        println!("{}", line);
                    }
                }
                Ok(MidiMsg::NoteOff { note }) => {
                    println!("[{} released]", note_name(note));
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if !cli.demo {
                        if let Ok(probe) = MidiInput::new("haydn-probe") {
                            let ports = probe.ports();
                            let still_connected = ports.iter().any(|p| {
                                probe.port_name(p).ok().as_deref() == Some(connected_name.as_str())
                            });
                            if !still_connected {
                                _conn = None;
                                eprintln!("\n⚠ MIDI device disconnected. Waiting for device... (Ctrl+C to quit)");
                                _conn = Some(wait_for_reconnect(
                                    &connected_name,
                                    reconnect_tx.clone(),
                                    &running,
                                )?);
                                eprintln!("✓ MIDI device reconnected. Resuming session.\n");
                            }
                        }
                    }
                    continue;
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    eprintln!("Channel disconnected unexpectedly. Ending session.");
                    break;
                }
            }
        }
    } else {
        // TUI mode — real-time dashboard (default)

        // Ensure terminal is restored on panic
        let default_panic = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let _ = crossterm::terminal::disable_raw_mode();
            let _ = std::io::stdout().execute(crossterm::terminal::LeaveAlternateScreen);
            default_panic(info);
        }));

        let mut terminal = display::init_terminal()?;
        let mut tui_state = display::TuiState::new(
            engine.metadata().name.clone(),
            connected_name.clone(),
            "MIDI".to_string(),
        );

        let frame_budget = Duration::from_millis(33); // ~30fps
        let mut last_render = Instant::now();
        let mut last_disconnect_check = Instant::now();
        let mut dirty = true;

        // Render initial empty state
        terminal.draw(|frame| display::render_dashboard(frame, &tui_state))?;

        while running.load(Ordering::Relaxed) {
            // 1. Poll keyboard events (non-blocking)
            if ct_event::poll(Duration::from_millis(1))? {
                if let CtEvent::Key(key) = ct_event::read()? {
                    if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                        break;
                    }
                }
            }

            // 2. Drain all pending MIDI events (non-blocking)
            loop {
                match rx.try_recv() {
                    Ok(MidiMsg::NoteOn { note, velocity }) => {
                        if let Some(results) = process_note_structured(note, &mut engine, &mut vm) {
                            for result in &results {
                                tui_state.update_from_step(note, velocity, result);
                            }
                        } else {
                            tui_state.history.push(display::HistoryEntry {
                                note_name: note_name(note),
                                velocity,
                                confidence: None,
                                operation: "(unmapped)".to_string(),
                                output_text: None,
                                edge_case: None,
                            });
                        }
                        tui_state.loop_state = vm.loop_state_name().to_string();
                        tui_state.update_stack_and_output(&vm);
                        dirty = true;
                    }
                    Ok(MidiMsg::NoteOff { .. }) => {}
                    Err(std::sync::mpsc::TryRecvError::Empty) => break,
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => break,
                }
            }

            // 3. Check MIDI disconnect periodically (every 500ms, skip in demo mode)
            if !cli.demo && last_disconnect_check.elapsed() > Duration::from_millis(500) {
                last_disconnect_check = Instant::now();
                if let Ok(probe) = MidiInput::new("haydn-probe") {
                    let ports = probe.ports();
                    let still_connected = ports.iter().any(|p| {
                        probe.port_name(p).ok().as_deref() == Some(connected_name.as_str())
                    });
                    if !still_connected {
                        _conn = None;
                        tui_state.connected = false;
                        terminal.draw(|frame| display::render_dashboard(frame, &tui_state))?;
                        _conn = Some(wait_for_reconnect(
                            &connected_name,
                            reconnect_tx.clone(),
                            &running,
                        )?);
                        tui_state.connected = true;
                        dirty = true;
                    }
                }
            }

            // 4. Render if dirty or frame budget elapsed
            if dirty || last_render.elapsed() >= frame_budget {
                terminal.draw(|frame| display::render_dashboard(frame, &tui_state))?;
                last_render = Instant::now();
                dirty = false;
            }
        }

        display::restore_terminal()?;
    }

    vm.close();
    println!("\n{}", format_session_summary(&vm));

    Ok(())
}
