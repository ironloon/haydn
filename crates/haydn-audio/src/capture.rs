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

    // Spawn analysis thread
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
            } else {
                thread::sleep(Duration::from_millis(1));
            }
        }
    });

    Ok((rx, stream))
}
