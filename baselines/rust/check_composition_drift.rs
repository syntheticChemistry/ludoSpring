// SPDX-License-Identifier: AGPL-3.0-or-later
//! Composition target drift detector — verifies that `composition_targets.json`
//! matches current Rust library outputs (same snapshot as
//! [`ludospring_barracuda::composition_targets::snapshot`]).
//!
//! Analogous to `baselines/python/check_drift.py` for the composition golden file.
//!
//! Exit codes: `0` — no drift; `1` — drift or I/O error.

use std::fs;
use std::path::Path;
use std::process::ExitCode;

use serde_json::Value;

fn main() -> ExitCode {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../baselines/rust/composition_targets.json");
    let text = match fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!(
                "check_composition_drift: cannot read {}: {e}",
                path.display()
            );
            return ExitCode::from(1);
        }
    };
    let stored: Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(e) => {
            eprintln!(
                "check_composition_drift: invalid JSON in {}: {e}",
                path.display()
            );
            return ExitCode::from(1);
        }
    };

    match ludospring_barracuda::composition_targets::compare_stored_to_generated(&stored) {
        Ok(()) => {
            println!("No composition target drift detected.");
            ExitCode::SUCCESS
        }
        Err(msg) => {
            eprintln!("composition_targets drift:\n{msg}");
            ExitCode::from(1)
        }
    }
}
