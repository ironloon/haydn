use crate::types::{NoteSequence, ScoreEvent};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Gauge, Paragraph};
use std::io::stdout;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

const NOTE_NAMES: [&str; 12] = [
    "C", "C#", "D", "Eb", "E", "F", "F#", "G", "Ab", "A", "Bb", "B",
];

/// Convert a MIDI note number to a human-readable name (e.g., 60 → "C4").
pub fn midi_to_name(midi: u8) -> String {
    let note = NOTE_NAMES[(midi % 12) as usize];
    let octave = (midi as i16 / 12) - 1;
    format!("{}{}", note, octave)
}

/// Compute the total duration of a note sequence.
fn total_duration(sequence: &NoteSequence) -> Duration {
    sequence
        .iter()
        .map(|e| match e {
            ScoreEvent::Note(n) => n.duration,
            ScoreEvent::Rest(r) => r.duration,
        })
        .sum()
}

/// Determine which event index is active at the given elapsed time.
fn current_index(sequence: &NoteSequence, elapsed: Duration) -> usize {
    let mut acc = Duration::ZERO;
    for (i, event) in sequence.iter().enumerate() {
        let dur = match event {
            ScoreEvent::Note(n) => n.duration,
            ScoreEvent::Rest(r) => r.duration,
        };
        acc += dur;
        if acc > elapsed {
            return i;
        }
    }
    sequence.len().saturating_sub(1)
}

/// Build the scrolling note display string centered on the current index.
fn note_display(sequence: &NoteSequence, current: usize, window: usize) -> String {
    let half = window / 2;
    let start = current.saturating_sub(half);
    let end = (start + window).min(sequence.len());

    let mut parts = Vec::new();
    for i in start..end {
        let name = match &sequence[i] {
            ScoreEvent::Note(n) => midi_to_name(n.midi_note),
            ScoreEvent::Rest(_) => "~".to_string(),
        };
        if i == current {
            parts.push(format!("[{}]", name));
        } else {
            parts.push(name);
        }
    }

    parts.join("  →  ")
}

/// Count measures (approximate: 4 beats = 1 measure at the given BPM).
fn measure_info(sequence: &NoteSequence, current: usize, bpm: u32) -> (usize, usize) {
    let beat_duration = Duration::from_secs_f64(60.0 / bpm as f64);
    let measure_duration = beat_duration * 4;

    let mut elapsed = Duration::ZERO;
    let mut current_measure = 1;
    for (i, event) in sequence.iter().enumerate() {
        if i >= current {
            break;
        }
        let dur = match event {
            ScoreEvent::Note(n) => n.duration,
            ScoreEvent::Rest(r) => r.duration,
        };
        elapsed += dur;
        current_measure = (elapsed.as_secs_f64() / measure_duration.as_secs_f64()).floor() as usize + 1;
    }

    let total_dur = total_duration(sequence);
    let total_measures = (total_dur.as_secs_f64() / measure_duration.as_secs_f64()).ceil() as usize;
    (current_measure, total_measures.max(1))
}

/// Run the TUI playback display on the current thread.
///
/// Blocks until playback ends (stop_signal is set) or user presses 'q'.
pub fn run_display(
    sequence: &NoteSequence,
    backend_name: &str,
    bpm: u32,
    stop_signal: Arc<AtomicBool>,
) -> anyhow::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let total_dur = total_duration(sequence);
    let start = Instant::now();

    loop {
        if stop_signal.load(Ordering::Relaxed) {
            break;
        }

        // Poll for input (non-blocking, 33ms ≈ 30fps)
        if event::poll(Duration::from_millis(33))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    stop_signal.store(true, Ordering::Relaxed);
                    break;
                }
            }
        }

        let elapsed = start.elapsed();
        if elapsed >= total_dur {
            break;
        }

        let idx = current_index(sequence, elapsed);
        let progress = if total_dur.as_secs_f64() > 0.0 {
            (elapsed.as_secs_f64() / total_dur.as_secs_f64()).min(1.0)
        } else {
            1.0
        };

        let note_line = note_display(sequence, idx, 9);
        let (cur_measure, total_measures) = measure_info(sequence, idx, bpm);
        let percent = (progress * 100.0) as u16;

        terminal.draw(|frame| {
            let area = frame.area();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(3), // Note display
                    Constraint::Length(3), // Progress bar
                    Constraint::Length(2), // Measure / note info
                    Constraint::Length(2), // Backend info
                    Constraint::Min(0),    // Spacer
                ])
                .split(area);

            // Title block
            let block = Block::default()
                .title(" haydn-performer ")
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Rounded);
            frame.render_widget(block, area);

            // Note display
            let note_para = Paragraph::new(format!("  ♪  {}", note_line))
                .alignment(Alignment::Center);
            frame.render_widget(note_para, chunks[0]);

            // Progress bar
            let gauge = Gauge::default()
                .gauge_style(Style::default().fg(Color::Cyan))
                .percent(percent)
                .label(format!("{}%", percent));
            frame.render_widget(gauge, chunks[1]);

            // Measure / note counter
            let info = Paragraph::new(format!(
                "  Measure {} / {}    Note {} / {}",
                cur_measure,
                total_measures,
                idx + 1,
                sequence.len()
            ));
            frame.render_widget(info, chunks[2]);

            // Backend info
            let backend_info = Paragraph::new(format!(
                "  Backend: {}    BPM: {}",
                backend_name, bpm
            ));
            frame.render_widget(backend_info, chunks[3]);
        })?;
    }

    // Restore terminal
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{NoteEvent, RestEvent};

    #[test]
    fn test_midi_to_name() {
        assert_eq!(midi_to_name(60), "C4");
        assert_eq!(midi_to_name(69), "A4");
        assert_eq!(midi_to_name(61), "C#4");
        assert_eq!(midi_to_name(48), "C3");
    }

    #[test]
    fn test_total_duration() {
        let seq = vec![
            ScoreEvent::Note(NoteEvent {
                midi_note: 60,
                duration: Duration::from_millis(500),
                velocity: 80,
            }),
            ScoreEvent::Rest(RestEvent {
                duration: Duration::from_millis(250),
            }),
        ];
        assert_eq!(total_duration(&seq), Duration::from_millis(750));
    }

    #[test]
    fn test_current_index() {
        let seq = vec![
            ScoreEvent::Note(NoteEvent {
                midi_note: 60,
                duration: Duration::from_millis(500),
                velocity: 80,
            }),
            ScoreEvent::Note(NoteEvent {
                midi_note: 64,
                duration: Duration::from_millis(500),
                velocity: 80,
            }),
        ];
        assert_eq!(current_index(&seq, Duration::from_millis(0)), 0);
        assert_eq!(current_index(&seq, Duration::from_millis(400)), 0);
        assert_eq!(current_index(&seq, Duration::from_millis(600)), 1);
    }

    #[test]
    fn test_note_display() {
        let seq = vec![
            ScoreEvent::Note(NoteEvent {
                midi_note: 60,
                duration: Duration::from_millis(500),
                velocity: 80,
            }),
            ScoreEvent::Note(NoteEvent {
                midi_note: 62,
                duration: Duration::from_millis(500),
                velocity: 80,
            }),
            ScoreEvent::Note(NoteEvent {
                midi_note: 64,
                duration: Duration::from_millis(500),
                velocity: 80,
            }),
        ];
        let display = note_display(&seq, 1, 5);
        assert!(display.contains("[D4]"));
        assert!(display.contains("C4"));
        assert!(display.contains("E4"));
    }
}
