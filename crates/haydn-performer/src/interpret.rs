use std::path::Path;

use haydn_tuning::TuningEngine;
use haydn_vm::HaydnVm;

pub struct InterpretState {
    pub engine: TuningEngine,
    pub vm: HaydnVm,
    pub tuning_name: String,
}

impl InterpretState {
    pub fn new(tuning_path: &Path) -> anyhow::Result<Self> {
        let engine = haydn_tuning::load_tuning_file(tuning_path).map_err(|errors| {
            let messages: Vec<String> = errors.iter().map(|e| format!("  - {e}")).collect();
            anyhow::anyhow!("Failed to load tuning file:\n{}", messages.join("\n"))
        })?;
        let tuning_name = engine.metadata().name.clone();
        let vm = HaydnVm::new();
        Ok(Self {
            engine,
            vm,
            tuning_name,
        })
    }

    pub fn process_note(&mut self, midi_note: u8) -> Vec<haydn_vm::StepResult> {
        match self.engine.map_note(midi_note) {
            Some(event) => self.vm.process_event(event),
            None => Vec::new(),
        }
    }
}
