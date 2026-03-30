use haydn_vm::{Event, HaydnVm, Operation, StepResult};
use ratatui::prelude::*;
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use std::io::stdout;

/// A single entry in the operation history panel.
pub struct HistoryEntry {
    pub note_name: String,
    pub velocity: u8,
    pub confidence: Option<f32>,
    pub operation: String,
    pub output_text: Option<String>,
    pub edge_case: Option<String>,
}

impl HistoryEntry {
    pub fn from_step(note: u8, velocity: u8, result: &StepResult) -> Self {
        let note_name = crate::note_name(note);

        let operation = match &result.operation {
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

        let output_text = result.output.as_ref().and_then(|bytes| {
            if bytes.is_empty() {
                None
            } else {
                Some(String::from_utf8_lossy(bytes).into_owned())
            }
        });

        let edge_case = result.edge_case.as_ref().map(|ec| format!("{:?}", ec));

        Self {
            note_name,
            velocity,
            confidence: None,
            operation,
            output_text,
            edge_case,
        }
    }
}

/// State model for the TUI dashboard.
pub struct TuiState {
    pub stack: Vec<i64>,
    pub history: Vec<HistoryEntry>,
    pub output: Vec<u8>,
    pub tuning_name: String,
    pub device_name: String,
    pub connected: bool,
    pub loop_state: String,
    pub input_mode: String,
    pub signal_level: Option<f32>,
}

const MAX_HISTORY: usize = 50;

impl TuiState {
    pub fn new(tuning_name: String, device_name: String, input_mode: String) -> Self {
        Self {
            stack: Vec::new(),
            history: Vec::new(),
            output: Vec::new(),
            tuning_name,
            device_name,
            connected: true,
            loop_state: "Normal".to_string(),
            input_mode,
            signal_level: None,
        }
    }

    pub fn update_from_step(&mut self, note: u8, velocity: u8, result: &StepResult) {
        let entry = HistoryEntry::from_step(note, velocity, result);
        self.history.push(entry);
        if self.history.len() > MAX_HISTORY {
            self.history.remove(0);
        }
        self.stack = result.stack_snapshot.clone();
        if let Some(ref out) = result.output {
            self.output.extend_from_slice(out);
        }
    }

    pub fn update_stack_and_output(&mut self, vm: &HaydnVm) {
        self.stack = vm.stack().to_vec();
        self.output = vm.output().to_vec();
    }
}

/// Render the three-panel dashboard into a ratatui frame.
pub fn render_dashboard(frame: &mut Frame, state: &TuiState) {
    let area = frame.area();

    // Outer block — matches haydn-performer's title pattern
    let outer = Block::default()
        .title(" haydn ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    frame.render_widget(outer, area);

    // Main layout: content + status bar
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Min(0),     // main content
            Constraint::Length(1),  // status bar
        ])
        .split(area);

    let content_area = vertical[0];
    let status_area = vertical[1];

    // Horizontal split: stack (left 25%) | right panels (75%)
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(75),
        ])
        .split(content_area);

    let stack_area = horizontal[0];
    let right_area = horizontal[1];

    // Right side: operations (70%) | output (30%)
    let right_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(70),
            Constraint::Percentage(30),
        ])
        .split(right_area);

    let ops_area = right_split[0];
    let output_area = right_split[1];

    // Stack panel
    render_stack(frame, state, stack_area);
    // Operations panel
    render_operations(frame, state, ops_area);
    // Output panel
    render_output(frame, state, output_area);
    // Status bar
    render_status(frame, state, status_area);
}

fn render_stack(frame: &mut Frame, state: &TuiState, area: Rect) {
    let block = Block::default()
        .title(" Stack ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let visible_height = inner.height as usize;
    if visible_height == 0 {
        return;
    }

    let total = state.stack.len();
    let mut lines: Vec<Line> = Vec::new();

    // Show top-of-stack first — iterate in reverse
    let show_count = total.min(visible_height);
    let overflow = total > visible_height;

    // If overflow, reserve one line for the indicator
    let display_count = if overflow { show_count.saturating_sub(1) } else { show_count };

    // Top of stack = last element of vec
    for i in 0..display_count {
        let idx = total - 1 - i;
        let val = state.stack[idx];
        let text = if val >= 32 && val <= 126 {
            format!("{:>6} '{}'", val, val as u8 as char)
        } else {
            format!("{:>6}", val)
        };
        lines.push(Line::from(Span::raw(text)));
    }

    if overflow {
        lines.push(
            Line::from(Span::styled(
                format!("── {} items ──", total),
                Style::default().fg(Color::DarkGray),
            ))
        );
    }

    let paragraph = Paragraph::new(lines).alignment(Alignment::Right);
    frame.render_widget(paragraph, inner);
}

fn render_operations(frame: &mut Frame, state: &TuiState, area: Rect) {
    let block = Block::default()
        .title(" Operations ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let visible_height = inner.height as usize;
    if visible_height == 0 {
        return;
    }

    // Show newest at bottom — take last N entries that fit
    let start = state.history.len().saturating_sub(visible_height);
    let visible = &state.history[start..];

    let lines: Vec<Line> = visible
        .iter()
        .map(|entry| {
            let note_info = if let Some(conf) = entry.confidence {
                format!("[{} ~{}%] → ", entry.note_name, (conf * 100.0) as u8)
            } else {
                format!("[{} v={}] → ", entry.note_name, entry.velocity)
            };
            let mut spans = vec![
                Span::raw(note_info),
                Span::styled(&entry.operation, Style::default().fg(Color::Cyan)),
            ];
            if let Some(ref text) = entry.output_text {
                spans.push(Span::raw(format!(" → '{}'", text)));
            }
            if let Some(ref ec) = entry.edge_case {
                spans.push(Span::styled(
                    format!(" ⚠ {}", ec),
                    Style::default().fg(Color::Yellow),
                ));
            }
            Line::from(spans)
        })
        .collect();

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn render_output(frame: &mut Frame, state: &TuiState, area: Rect) {
    let block = Block::default()
        .title(" Output ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let text = String::from_utf8_lossy(&state.output);
    let paragraph = Paragraph::new(text.into_owned());
    frame.render_widget(paragraph, inner);
}

fn render_status(frame: &mut Frame, state: &TuiState, area: Rect) {
    let device = if state.connected {
        Span::raw(format!("{}: {}", state.input_mode, state.device_name))
    } else {
        Span::styled("⚠ Disconnected", Style::default().fg(Color::Yellow))
    };

    let sep = Span::styled("  │  ", Style::default().fg(Color::DarkGray));

    let mut spans = vec![
        Span::raw(format!(" {}", state.tuning_name)),
        sep.clone(),
        device,
        sep.clone(),
        Span::raw(&state.loop_state),
    ];

    if let Some(level) = state.signal_level {
        let blocks = ["▁", "▂", "▃", "▅", "▇"];
        let idx = ((level.clamp(0.0, 1.0) * 4.0) as usize).min(4);
        let meter: String = blocks[..=idx].join("");
        spans.push(sep.clone());
        spans.push(Span::styled(meter, Style::default().fg(Color::Green)));
    }

    spans.push(sep);
    spans.push(Span::styled("q: quit", Style::default().fg(Color::DarkGray)));

    let line = Line::from(spans);

    let paragraph = Paragraph::new(line);
    frame.render_widget(paragraph, area);
}

/// Initialize the terminal for TUI rendering.
pub fn init_terminal() -> anyhow::Result<Terminal<CrosstermBackend<std::io::Stdout>>> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

/// Restore the terminal to its normal state.
pub fn restore_terminal() -> anyhow::Result<()> {
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
