// SPDX-License-Identifier: AGPL-3.0-or-later
//! Meta-runner: executes each `validate_*` binary as a subprocess and exits 0 only
//! if all complete successfully.
#![forbid(unsafe_code)]

use std::path::Path;
use std::process::{Command, Stdio};

struct Validator {
    name: &'static str,
    /// When true, exit code 2 is treated as skip (not failure), per ecosystem convention.
    skip_on_exit_2: bool,
}

fn main() -> Result<(), String> {
    let exe_dir = std::env::current_exe()
        .map_err(|e| format!("current_exe: {e}"))?
        .parent()
        .ok_or_else(|| "current_exe has no parent".to_string())?
        .to_path_buf();

    let validators = [
        Validator {
            name: "validate_interaction",
            skip_on_exit_2: false,
        },
        Validator {
            name: "validate_procedural",
            skip_on_exit_2: false,
        },
        Validator {
            name: "validate_engagement",
            skip_on_exit_2: false,
        },
        Validator {
            name: "validate_composition",
            skip_on_exit_2: true,
        },
        Validator {
            name: "validate_primal_proof",
            skip_on_exit_2: true,
        },
    ];

    let mut any_fail = false;
    for Validator {
        name,
        skip_on_exit_2,
    } in validators
    {
        let path = exe_dir.join(name);
        if !Path::new(&path).exists() {
            if name == "validate_composition" || name == "validate_primal_proof" {
                eprintln!("  ○ {name} (binary not built — requires `ipc` feature; skipping)");
            } else {
                eprintln!("  ✗ {name} (binary missing at {})", path.display());
                any_fail = true;
            }
            continue;
        }

        let out = Command::new(&path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| format!("spawn {}: {e}", path.display()))?;
        let code = out.status.code().unwrap_or(-1);
        if code == 0 {
            eprintln!("  ✓ {name} (exit 0)");
        } else if code == 2 && skip_on_exit_2 {
            eprintln!("  ○ {name} (exit 2 — skipped, not counted as failure)");
            if !out.stderr.is_empty() {
                eprintln!("{}", String::from_utf8_lossy(&out.stderr));
            }
        } else {
            eprintln!("  ✗ {name} (exit {code})");
            if !out.stderr.is_empty() {
                eprintln!("{}", String::from_utf8_lossy(&out.stderr));
            }
            if !out.stdout.is_empty() {
                eprintln!("{}", String::from_utf8_lossy(&out.stdout));
            }
            any_fail = true;
        }
    }

    eprintln!();
    if any_fail {
        eprintln!("validate_all: FAIL (one or more validators failed)");
        std::process::exit(1);
    }
    eprintln!("validate_all: PASS (all validators exited 0, or skipped with exit 2 where allowed)");
    Ok(())
}
