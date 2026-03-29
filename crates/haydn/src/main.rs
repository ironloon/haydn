use std::io::Write;
use std::sync::mpsc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use midir::MidiInput;

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
}

const NOTE_NAMES: [&str; 12] = ["C", "C#", "D", "Eb", "E", "F", "F#", "G", "Ab", "A", "Bb", "B"];

fn note_name(midi: u8) -> String {
    let name = NOTE_NAMES[(midi % 12) as usize];
    let octave = (midi as i16 / 12) - 1;
    format!("{}{}", name, octave)
}

#[derive(Debug)]
enum MidiMsg {
    NoteOn { note: u8, velocity: u8 },
    NoteOff { note: u8 },
}

fn midi_callback(_timestamp: u64, message: &[u8], tx: &mut mpsc::Sender<MidiMsg>) {
    if let Ok(event) = midly::live::LiveEvent::parse(message) {
        if let midly::live::LiveEvent::Midi { message: msg, .. } = event {
            match msg {
                midly::MidiMessage::NoteOn { key, vel } => {
                    if vel.as_int() == 0 {
                        let _ = tx.send(MidiMsg::NoteOff { note: key.as_int() });
                    } else {
                        let _ = tx.send(MidiMsg::NoteOn {
                            note: key.as_int(),
                            velocity: vel.as_int(),
                        });
                    }
                }
                midly::MidiMessage::NoteOff { key, .. } => {
                    let _ = tx.send(MidiMsg::NoteOff { note: key.as_int() });
                }
                _ => {}
            }
        }
    }
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

fn print_event_log(note: u8, velocity: u8, result: &haydn_vm::StepResult) {
    use haydn_vm::{Event, Operation};

    let mapping = match &result.operation {
        Operation::Pushed(v) => format!("Push({})", v),
        Operation::Executed(op) => format!("Op({:?})", op),
        Operation::LoopEntered => "LoopEntered".to_string(),
        Operation::LoopSkipped => "LoopSkipped".to_string(),
        Operation::LoopExited => "LoopExited".to_string(),
        Operation::LoopReplaying => "LoopReplaying".to_string(),
        Operation::ReplayStep(evt) => match evt {
            Event::Push(v) => format!("Replay(Push({}))", v),
            Event::Op(op) => format!("Replay(Op({:?}))", op),
        },
        Operation::EndOfBufferReplay => "EndOfBufferReplay".to_string(),
        Operation::EndOfBufferExit => "EndOfBufferExit".to_string(),
        Operation::Noop => "Noop".to_string(),
    };

    let stack = format!("{:?}", result.stack_snapshot);
    let mut line = format!("[{} v={}] → {}  |  Stack: {}", note_name(note), velocity, mapping, stack);

    if let Some(ref output) = result.output {
        if !output.is_empty() {
            let text = String::from_utf8_lossy(output);
            line.push_str(&format!("  |  Out: \"{}\"", text));
        }
    }

    if let Some(ref ec) = result.edge_case {
        line.push_str(&format!("  |  ⚠ {:?}", ec));
    }

    println!("{}", line);
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

    // Select MIDI device
    let midi_in = MidiInput::new("haydn")?;
    let (port, connected_name) = select_midi_port(&midi_in, cli.midi_device.as_deref())?;

    // Set up channel and connect
    let (tx, rx) = mpsc::channel::<MidiMsg>();
    let reconnect_tx = tx.clone();
    let mut conn = midi_in.connect(&port, "haydn-input", midi_callback, tx)?;

    // Ctrl+C handler
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::Relaxed);
    })?;

    // Main event loop
    let mut vm = haydn_vm::HaydnVm::new();
    println!("Listening for MIDI input... (Ctrl+C to quit)\n");

    while running.load(Ordering::Relaxed) {
        match rx.recv_timeout(Duration::from_millis(500)) {
            Ok(MidiMsg::NoteOn { note, velocity }) => {
                match engine.map_note(note) {
                    Some(event) => {
                        let results = vm.process_event(event);
                        for result in &results {
                            print_event_log(note, velocity, result);
                        }
                    }
                    None => {
                        println!("[{} v={}] → (unmapped)", note_name(note), velocity);
                    }
                }
            }
            Ok(MidiMsg::NoteOff { note }) => {
                println!("[{} released]", note_name(note));
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if let Ok(probe) = MidiInput::new("haydn-probe") {
                    let ports = probe.ports();
                    let still_connected = ports.iter().any(|p| {
                        probe.port_name(p).ok().as_deref() == Some(connected_name.as_str())
                    });
                    if !still_connected {
                        drop(conn);
                        eprintln!("\n⚠ MIDI device disconnected. Waiting for device... (Ctrl+C to quit)");
                        conn = wait_for_reconnect(
                            &connected_name,
                            reconnect_tx.clone(),
                            &running,
                        )?;
                        eprintln!("✓ MIDI device reconnected. Resuming session.\n");
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

    vm.close();
    println!("\n─── Session Summary ───");
    println!("Final stack ({} items): {:?}", vm.stack().len(), vm.stack());
    if !vm.output().is_empty() {
        match std::str::from_utf8(vm.output()) {
            Ok(s) => println!("Program output: {}", s),
            Err(_) => println!("Program output (bytes): {:?}", vm.output()),
        }
    }
    println!("───────────────────────");

    Ok(())
}
