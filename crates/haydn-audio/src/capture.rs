use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::SampleFormat;
use ringbuf::{traits::*, HeapRb};

use crate::pitch::mcleod::McLeodDetector;
use crate::pitch::yin::YinDetector;
use crate::pitch::PitchDetector;
use crate::state_machine::NoteStateMachine;
use crate::types::{AudioConfig, AudioError, AudioMsg};

/// List all available audio input device names.
pub fn list_audio_input_devices() -> Result<Vec<String>, AudioError> {
    let host = cpal::default_host();
    let devices = host
        .input_devices()
        .map_err(|e| AudioError::DeviceNotFound(format!("failed to enumerate input devices: {e}")))?;
    let names: Vec<String> = devices
        .filter_map(|d| d.name().ok())
        .collect();
    Ok(names)
}

/// Find an audio input device by name (case-insensitive substring match), or the default.
pub fn find_audio_input_device(device_name: Option<&str>) -> Result<cpal::Device, AudioError> {
    let host = cpal::default_host();
    match device_name {
        Some(name) => {
            let device = host
                .input_devices()
                .map_err(|e| {
                    AudioError::DeviceNotFound(format!("failed to enumerate input devices: {e}"))
                })?
                .find(|d| {
                    d.name()
                        .map(|n| n.to_lowercase().contains(&name.to_lowercase()))
                        .unwrap_or(false)
                })
                .ok_or_else(|| {
                    let available = list_audio_input_devices()
                        .unwrap_or_default()
                        .join(", ");
                    AudioError::DeviceNotFound(format!(
                        "no input device matching \"{name}\". Available: [{available}]"
                    ))
                })?;
            Ok(device)
        }
        None => host
            .default_input_device()
            .ok_or_else(|| AudioError::DeviceNotFound("no default input device found".into())),
    }
}

/// Start the audio capture pipeline.
///
/// Returns a receiver for `AudioMsg` events and the cpal `Stream` (must be kept alive).
/// The pipeline: cpal callback → ringbuf SPSC → analysis thread → pitch + onset + gate + state machine → mpsc channel.
pub fn start_audio_capture(
    device: cpal::Device,
    config: AudioConfig,
) -> Result<(mpsc::Receiver<AudioMsg>, cpal::Stream), AudioError> {
    let supported_config = device
        .default_input_config()
        .map_err(|e| AudioError::ConfigError(format!("failed to get input config: {e}")))?;

    let actual_sample_rate = supported_config.sample_rate().0;
    let channels = supported_config.channels() as usize;
    let sample_format = supported_config.sample_format();

    // Ring buffer: 16384 samples ≈ 371ms at 44100Hz
    let rb = HeapRb::<f32>::new(16384);
    let (mut prod, mut cons) = rb.split();

    let (tx, rx) = mpsc::channel::<AudioMsg>();

    let stream_config: cpal::StreamConfig = supported_config.into();

    // Build the input stream based on sample format
    let stream = match sample_format {
        SampleFormat::F32 => device.build_input_stream(
            &stream_config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                // Extract mono (first channel) and push to ring buffer
                // Zero allocations — only ring buffer push operations
                for chunk in data.chunks(channels) {
                    let _ = prod.try_push(chunk[0]);
                }
            },
            |err| eprintln!("Audio stream error: {err}"),
            None,
        ),
        SampleFormat::I16 => device.build_input_stream(
            &stream_config,
            move |data: &[i16], _: &cpal::InputCallbackInfo| {
                for chunk in data.chunks(channels) {
                    let sample = chunk[0] as f32 / i16::MAX as f32;
                    let _ = prod.try_push(sample);
                }
            },
            |err| eprintln!("Audio stream error: {err}"),
            None,
        ),
        other => {
            return Err(AudioError::ConfigError(format!(
                "unsupported sample format: {other:?}"
            )));
        }
    }
    .map_err(|e| AudioError::StreamError(format!("failed to build input stream: {e}")))?;

    stream
        .play()
        .map_err(|e| AudioError::StreamError(format!("failed to start stream: {e}")))?;

    // Spawn analysis thread (shared with start_loopback_capture below)
    let window_size = config.window_size;
    let hop_size = config.hop_size;
    let algorithm = config.algorithm.clone();
    let confidence_threshold = config.confidence_threshold;

    thread::spawn(move || {
        // Create pitch detector based on config
        let mut detector: Box<dyn PitchDetector> = if algorithm == "yin" {
            Box::new(YinDetector::new(window_size, confidence_threshold))
        } else {
            Box::new(McLeodDetector::new(window_size, confidence_threshold))
        };

        let mut state_machine = NoteStateMachine::new(&config);
        let mut analysis_buffer = vec![0.0f32; window_size];

        loop {
            let available = cons.occupied_len();
            if available >= hop_size {
                // Shift buffer left by hop_size, fill new samples from ring buffer
                analysis_buffer.copy_within(hop_size.., 0);
                let fill_start = window_size - hop_size;
                cons.pop_slice(&mut analysis_buffer[fill_start..]);

                // Run pitch detection on full window
                let pitch = detector.detect(&analysis_buffer, actual_sample_rate);

                // Run state machine on the new hop portion (for RMS/onset analysis)
                let events =
                    state_machine.process_frame(&analysis_buffer[fill_start..], pitch.as_ref());

                // Send events to consumer
                for event in events {
                    if tx.send(event).is_err() {
                        return; // Receiver dropped, shut down
                    }
                }

                // Send signal level (RMS of the hop portion) for TUI meter
                let hop_buf = &analysis_buffer[fill_start..];
                let rms = (hop_buf.iter().map(|s| s * s).sum::<f32>() / hop_buf.len() as f32).sqrt();
                if tx.send(AudioMsg::SignalLevel(rms)).is_err() {
                    return;
                }
            } else {
                thread::sleep(Duration::from_millis(1));
            }
        }
    });

    Ok((rx, stream))
}

/// Start the audio analysis pipeline without a hardware input device (loopback mode).
///
/// Returns a receiver for `AudioMsg` events and a ring-buffer producer.
/// The caller (e.g. `LoopbackTap`) feeds the exact same mono f32 PCM samples going
/// to the speakers into this producer; the analysis thread pitch-detects them.
///
/// Design: no silence-gate buffer-clear.  Consecutive piano ADSR notes produce a
/// brief near-zero region at each note boundary — zeroing the window and waiting for
/// it to refill (settle) causes repeated re-silence loops when settle hops land on
/// another boundary.  Instead we let the 4096-sample window fill organically: the
/// old-note tail is near-zero and contributes negligibly to the autocorrelation.
/// Silence detection still emits NoteOff (needed for rests), but does NOT clear the
/// analysis buffer or introduce any settle delay.
pub fn start_loopback_capture(
    config: AudioConfig,
) -> Result<(mpsc::Receiver<AudioMsg>, ringbuf::HeapProd<f32>), AudioError> {
    let actual_sample_rate = config.sample_rate;
    let window_size = config.window_size;
    let hop_size = config.hop_size;
    // Require high confidence — synthetic audio is clean.  0.85 filters out the
    // brief low-confidence frames during ADSR attack transients.
    let confidence_threshold = config.confidence_threshold.max(0.85);
    // -20 dB gate is used only to emit NoteOff for rests; it does NOT clear the
    // analysis buffer (clearing causes settle-loop problems at note boundaries).
    let noise_gate_db = config.noise_gate_db.max(-20.0);

    // Require this many consecutive hops of the same MIDI note before firing NoteOn.
    // 8 hops × 11.6 ms/hop ≈ 93 ms.  The 4096-sample window gives low notes ~23 cycles,
    // which suppresses steady-state McLeod sub-octave errors.  8 stable hops outlasts
    // the 3–5-hop transient where McLeod detects a sub-harmonic (e.g. G3 = G4/2 or
    // A4 = A5/2) during the window-fill period when an old note's tail is still present.
    const STABLE_HOPS: u8 = 8;

    let rb = HeapRb::<f32>::new(16384);
    let (prod, mut cons) = rb.split();

    let (tx, rx) = mpsc::channel::<AudioMsg>();

    thread::spawn(move || {
        let mut detector = McLeodDetector::new(window_size, confidence_threshold);
        let mut analysis_buffer = vec![0.0f32; window_size];

        let mut active_note: Option<u8> = None; // last NoteOn emitted
        let mut candidate: Option<u8> = None;   // note accumulating stability
        let mut candidate_count: u8 = 0;

        loop {
            if cons.occupied_len() < hop_size {
                // No data yet — wait a short time for the ring to fill.
                // The cpal hardware callback drives production at the audio-device
                // rate, so this naturally throttles the analysis to real-time.
                thread::sleep(Duration::from_millis(1));
                continue;
            }

            // Slide window
            analysis_buffer.copy_within(hop_size.., 0);
            let fill_start = window_size - hop_size;
            cons.pop_slice(&mut analysis_buffer[fill_start..]);

            let hop_buf = &analysis_buffer[fill_start..];
            let rms_db = crate::gate::NoiseGate::rms_db(hop_buf);
            let rms = (hop_buf.iter().map(|s| s * s).sum::<f32>() / hop_buf.len() as f32)
                .sqrt();

            if rms_db < noise_gate_db {
                // --- SILENCE (rest or end of score) ---
                // Emit NoteOff if needed; reset candidate accumulation.
                // Do NOT clear the analysis buffer — consecutive piano notes have
                // a brief near-zero region between them and clearing would trigger
                // a re-silence loop as each settle hop lands on the next boundary.
                if active_note.is_some() {
                    active_note = None;
                    if tx.send(AudioMsg::NoteOff).is_err() {
                        return;
                    }
                }
                candidate = None;
                candidate_count = 0;
            } else {
                // --- DETECT ---
                if let Some(est) = detector.detect(&analysis_buffer, actual_sample_rate) {
                    if est.confidence >= confidence_threshold {
                        if candidate == Some(est.midi_note) {
                            candidate_count = candidate_count.saturating_add(1);
                        } else {
                            candidate = Some(est.midi_note);
                            candidate_count = 1;
                        }

                        if candidate_count >= STABLE_HOPS && active_note != candidate {
                            if active_note.is_some() {
                                if tx.send(AudioMsg::NoteOff).is_err() {
                                    return;
                                }
                            }
                            if tx.send(AudioMsg::NoteOn {
                                note: est.midi_note,
                                confidence: est.confidence,
                            })
                            .is_err()
                            {
                                return;
                            }
                            active_note = candidate;
                        }
                    }
                }
            }

            if tx.send(AudioMsg::SignalLevel(rms)).is_err() {
                return;
            }
        }
    });

    Ok((rx, prod))
}
