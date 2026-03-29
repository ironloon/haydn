use crate::types::{NoteSequence, ScoreEvent};
use std::path::Path;

/// Write a note sequence as a Standard MIDI File (.mid).
///
/// Uses Format 0 (single track) with 480 ticks per quarter note.
pub fn write_midi_file(sequence: &NoteSequence, path: &Path, bpm: u32) -> Result<(), String> {
    use midly::num::{u15, u24, u28, u4, u7};
    use midly::{Format, Header, MetaMessage, MidiMessage, Smf, Timing, Track, TrackEvent, TrackEventKind};

    let ticks_per_quarter: u16 = 480;
    let microseconds_per_beat: u32 = 60_000_000 / bpm;

    let mut track: Track = Vec::new();

    // Tempo meta event at delta 0
    track.push(TrackEvent {
        delta: u28::new(0),
        kind: TrackEventKind::Meta(MetaMessage::Tempo(u24::new(microseconds_per_beat))),
    });

    for event in sequence {
        match event {
            ScoreEvent::Note(note) => {
                let duration_secs = note.duration.as_secs_f64();
                let ticks =
                    (duration_secs * ticks_per_quarter as f64 * bpm as f64 / 60.0) as u32;
                let ticks = ticks.max(1);

                // Note On
                track.push(TrackEvent {
                    delta: u28::new(0),
                    kind: TrackEventKind::Midi {
                        channel: u4::new(0),
                        message: MidiMessage::NoteOn {
                            key: u7::new(note.midi_note.min(127)),
                            vel: u7::new(note.velocity.min(127)),
                        },
                    },
                });

                // Note Off after duration
                track.push(TrackEvent {
                    delta: u28::new(ticks),
                    kind: TrackEventKind::Midi {
                        channel: u4::new(0),
                        message: MidiMessage::NoteOff {
                            key: u7::new(note.midi_note.min(127)),
                            vel: u7::new(0),
                        },
                    },
                });
            }
            ScoreEvent::Rest(rest) => {
                let duration_secs = rest.duration.as_secs_f64();
                let ticks =
                    (duration_secs * ticks_per_quarter as f64 * bpm as f64 / 60.0) as u32;
                if ticks > 0 {
                    // Add a rest as a meta text event (or just advance delta on next event)
                    // We'll push the delta onto the next event.
                    // midly handles this: if there's already a delta, we need to accumulate.
                    // Simplest: add a dummy meta event with the rest's delta
                    track.push(TrackEvent {
                        delta: u28::new(ticks),
                        kind: TrackEventKind::Meta(MetaMessage::Text(b"")),
                    });
                }
            }
        }
    }

    // End of track
    track.push(TrackEvent {
        delta: u28::new(0),
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });

    let header = Header::new(
        Format::SingleTrack,
        Timing::Metrical(u15::new(ticks_per_quarter)),
    );
    let mut smf = Smf::new(header);
    smf.tracks.push(track);

    smf.save(path).map_err(|e| format!("Failed to write MIDI file: {}", e))?;
    Ok(())
}

/// Send a note sequence to a MIDI output port in real time.
pub fn play_midi_port(sequence: &NoteSequence, port_name: Option<&str>) -> Result<(), String> {
    let midi_out =
        midir::MidiOutput::new("haydn-performer").map_err(|e| format!("MIDI init error: {}", e))?;

    let ports = midi_out.ports();

    let conn = if let Some(name) = port_name {
        // Find port by name
        let port = ports
            .iter()
            .find(|p| {
                midi_out
                    .port_name(p)
                    .map(|n| n.contains(name))
                    .unwrap_or(false)
            })
            .ok_or_else(|| {
                let available: Vec<String> = ports
                    .iter()
                    .filter_map(|p| midi_out.port_name(p).ok())
                    .collect();
                format!(
                    "MIDI port '{}' not found. Available: {}",
                    name,
                    if available.is_empty() {
                        "(none)".to_string()
                    } else {
                        available.join(", ")
                    }
                )
            })?;
        midi_out
            .connect(port, "haydn-performer")
            .map_err(|e| format!("MIDI connect error: {}", e))?
    } else if ports.is_empty() {
        return Err(
            "No MIDI output ports found. On Windows, install a virtual MIDI loopback \
             driver (e.g., loopMIDI) or connect a MIDI device."
                .to_string(),
        );
    } else {
        // Use the first available port
        midi_out
            .connect(&ports[0], "haydn-performer")
            .map_err(|e| format!("MIDI connect error: {}", e))?
    };

    let mut conn = conn;

    for event in sequence {
        match event {
            ScoreEvent::Note(note) => {
                // Note On: status 0x90, note, velocity
                conn.send(&[0x90, note.midi_note.min(127), note.velocity.min(127)])
                    .map_err(|e| format!("MIDI send error: {}", e))?;

                std::thread::sleep(note.duration);

                // Note Off: status 0x80, note, velocity 0
                conn.send(&[0x80, note.midi_note.min(127), 0])
                    .map_err(|e| format!("MIDI send error: {}", e))?;
            }
            ScoreEvent::Rest(rest) => {
                std::thread::sleep(rest.duration);
            }
        }
    }

    // Close connection
    conn.close();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{NoteEvent, RestEvent};
    use std::time::Duration;

    #[test]
    fn test_write_midi_file() {
        let sequence = vec![
            ScoreEvent::Note(NoteEvent {
                midi_note: 60,
                duration: Duration::from_millis(500),
                velocity: 80,
            }),
            ScoreEvent::Rest(RestEvent {
                duration: Duration::from_millis(250),
            }),
            ScoreEvent::Note(NoteEvent {
                midi_note: 64,
                duration: Duration::from_millis(500),
                velocity: 100,
            }),
        ];

        let dir = std::env::temp_dir();
        let path = dir.join("haydn_test_output.mid");

        write_midi_file(&sequence, &path, 120).expect("Failed to write MIDI file");

        // Read it back and verify
        let data = std::fs::read(&path).expect("Failed to read MIDI file");
        let smf = midly::Smf::parse(&data).expect("Failed to parse MIDI file");

        assert_eq!(smf.tracks.len(), 1);

        // Count NoteOn events
        let note_on_count = smf.tracks[0]
            .iter()
            .filter(|e| matches!(e.kind, midly::TrackEventKind::Midi { message: midly::MidiMessage::NoteOn { .. }, .. }))
            .count();
        assert_eq!(note_on_count, 2);

        // Clean up
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_write_midi_file_empty_sequence() {
        let sequence = vec![];
        let dir = std::env::temp_dir();
        let path = dir.join("haydn_test_empty.mid");

        write_midi_file(&sequence, &path, 120).expect("Failed to write MIDI file");

        let data = std::fs::read(&path).expect("Failed to read MIDI file");
        let smf = midly::Smf::parse(&data).expect("Failed to parse MIDI file");
        assert_eq!(smf.tracks.len(), 1);

        let _ = std::fs::remove_file(&path);
    }
}
