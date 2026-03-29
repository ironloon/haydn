use std::collections::BTreeMap;

use serde::Deserialize;

use crate::error::TuningError;

/// Top-level TOML tuning file structure.
#[derive(Debug, Deserialize)]
pub struct RawTuningFile {
    pub metadata: Metadata,
    pub values: Option<ValuesSection>,
    pub operations: Option<BTreeMap<String, String>>,
    #[serde(default)]
    pub rules: Vec<RawRule>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Metadata {
    pub name: String,
    pub root_note: u8,
    pub instrument: String,
    pub author: Option<String>,
    #[serde(default)]
    pub partial_coverage: bool,
}

#[derive(Debug, Deserialize)]
pub struct ValuesSection {
    pub range: [u8; 2],
}

#[derive(Debug, Deserialize)]
pub struct RawRule {
    pub trigger: RawTrigger,
    pub emit: RawEmit,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RawTrigger {
    Note { note: u8 },
    Range { from: u8, to: u8 },
    Interval { semitones: i8 },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RawEmit {
    Push { value: i64 },
    PushOffset,
    Op { opcode: String },
}

/// All 19 valid opcode names.
const OPCODE_NAMES: &[(&str, haydn_vm::Opcode)] = &[
    ("dup", haydn_vm::Opcode::Dup),
    ("swap", haydn_vm::Opcode::Swap),
    ("drop", haydn_vm::Opcode::Drop),
    ("rotate", haydn_vm::Opcode::Rotate),
    ("add", haydn_vm::Opcode::Add),
    ("sub", haydn_vm::Opcode::Sub),
    ("mul", haydn_vm::Opcode::Mul),
    ("div", haydn_vm::Opcode::Div),
    ("mod", haydn_vm::Opcode::Mod),
    ("eq", haydn_vm::Opcode::Eq),
    ("gt", haydn_vm::Opcode::Gt),
    ("lt", haydn_vm::Opcode::Lt),
    ("loop_start", haydn_vm::Opcode::LoopStart),
    ("loop_end", haydn_vm::Opcode::LoopEnd),
    ("print_num", haydn_vm::Opcode::PrintNum),
    ("print_char", haydn_vm::Opcode::PrintChar),
    ("read", haydn_vm::Opcode::Read),
    ("store", haydn_vm::Opcode::Store),
    ("load", haydn_vm::Opcode::Load),
];

/// Parse an opcode name string into an `Opcode` variant.
/// On mismatch, suggests the closest name if one is close enough.
pub fn parse_opcode(name: &str) -> Result<haydn_vm::Opcode, TuningError> {
    let name = name.trim().to_lowercase();
    for &(n, op) in OPCODE_NAMES {
        if n == name {
            return Ok(op);
        }
    }

    // Try to suggest a close match
    let suggestion = suggest_opcode(&name);
    Err(TuningError::UnknownOpcode {
        found: name,
        suggestion,
    })
}

/// Suggest a matching opcode name using substring containment and edit distance.
fn suggest_opcode(input: &str) -> Option<String> {
    // Substring containment: "dupe" contains "dup"
    for &(name, _) in OPCODE_NAMES {
        if input.contains(name) || name.contains(input) {
            return Some(name.to_string());
        }
    }

    // Edit distance for short names
    let mut best: Option<(&str, usize)> = None;
    for &(name, _) in OPCODE_NAMES {
        let dist = edit_distance(input, name);
        if dist <= 2 {
            if best.is_none() || dist < best.unwrap().1 {
                best = Some((name, dist));
            }
        }
    }
    best.map(|(name, _)| name.to_string())
}

/// Simple Levenshtein edit distance.
fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let (m, n) = (a.len(), b.len());
    let mut dp = vec![vec![0usize; n + 1]; m + 1];
    for i in 0..=m {
        dp[i][0] = i;
    }
    for j in 0..=n {
        dp[0][j] = j;
    }
    for i in 1..=m {
        for j in 1..=n {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }
    dp[m][n]
}
