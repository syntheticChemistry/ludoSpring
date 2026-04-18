// SPDX-License-Identifier: AGPL-3.0-or-later
//! Generate composition validation targets from direct Rust library calls.
//!
//! These targets serve the same role for primal composition that Python
//! baselines serve for Rust code: golden reference values that composition
//! experiments validate against via IPC.
//!
//! # Evolution path
//!
//! ```text
//! Python baseline → validates → Rust library code
//! Rust library code → validates → IPC composition (this file generates targets)
//! IPC composition → validates → NUCLEUS deployment (biomeOS graph)
//! ```
//!
//! # Usage
//!
//! ```sh
//! cargo run --example generate_composition_targets --features ipc \
//!     > baselines/rust/composition_targets.json
//! ```
//!
//! The output JSON records the exact result each science method should
//! return for a given set of inputs. Composition experiments call the
//! same methods via IPC and compare to these targets within tolerance.

fn main() -> Result<(), serde_json::Error> {
    let json = ludospring_barracuda::composition_targets::snapshot();
    println!("{}", serde_json::to_string_pretty(&json)?);
    Ok(())
}
