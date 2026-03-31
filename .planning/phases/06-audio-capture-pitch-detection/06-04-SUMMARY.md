# Plan 06-04 Summary: Audio Capture Pipeline

## What was built
- `crates/haydn-audio/src/capture.rs` — Complete audio capture pipeline
- `crates/haydn-audio/tests/integration.rs` — 6 full-pipeline integration tests

## Key decisions
- Three-thread architecture: cpal callback → ringbuf SPSC (16384 capacity) → analysis thread → mpsc AudioMsg
- Zero-allocation audio callback — only ring buffer push operations
- Sliding window analysis: shift left by hop_size, fill new samples, run pitch + state machine
- SampleFormat::F32 and I16 supported; other formats return ConfigError
- McLeod NSDF as default algorithm in integration tests (config-switchable to YIN)

## Public API
- `start_audio_capture(device, config) -> Result<(Receiver<AudioMsg>, Stream), AudioError>` — main entry point
- `find_audio_input_device(name: Option<&str>) -> Result<Device, AudioError>` — case-insensitive substring match
- `list_audio_input_devices() -> Result<Vec<String>, AudioError>` — enumerate available inputs

## Test results
- 47 total tests passing (31 unit + 6 integration + 6 onset + 4 pitch accuracy)
- Integration tests cover: silence, single note, two sequential notes, sustained note, low frequency E2, noise below gate
- Workspace compiles cleanly (`cargo check --workspace`)

## Files changed
- Created: `crates/haydn-audio/src/capture.rs` (148 lines)
- Modified: `crates/haydn-audio/src/lib.rs` (added capture module + re-exports)
- Created: `crates/haydn-audio/tests/integration.rs` (155 lines)

## Commit
`121de31` — feat(haydn-audio): audio capture pipeline with cpal→ringbuf→analysis thread
