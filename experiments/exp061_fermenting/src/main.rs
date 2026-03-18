// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! exp061 — Fermenting System
//!
//! Memory-bound digital objects with full provenance: NFT without crypto.
//!
//! A "ferment" is a digital object whose value accumulates through use,
//! like a culture that transforms raw materials into something richer.
//! The provenance DAG is the culture. The loamSpine certificate is the
//! vessel. The history cannot be forged — you cannot un-ferment.
//!
//! This experiment validates:
//!   1. Cosmetic schema: rarity, skin, color, material, wear — round-trip
//!   2. Certificate lifecycle: mint → inspect → trade → loan → return → consume
//!   3. Trading protocol: offer, accept, reject, cancel, atomic swap
//!   4. Object memory: event timeline per object, cross-object queries
//!   5. Trio integration: rhizoCrypt DAG, loamSpine certs, sweetGrass braids
//!   6. Ownership: only owners can trade/loan, only borrowers can return
//!   7. Full scenario: two players, multiple objects, trades, loans, achievements
//!   8. Composable deployment: IPC wire format for inter-primal orchestration

mod ferment;
#[allow(dead_code)]
// wire format types for IPC contract — constructed by remote callers, not locally
mod protocol;
mod validate_objects;
mod validate_systems;

use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (analytical — provenance trio integration)",
    commit: "N/A",
    date: "N/A",
    command: "N/A (pure Rust implementation)",
};

fn cmd_validate() {
    let mut h = ValidationHarness::new("exp061_fermenting");
    h.print_provenance(&[&PROVENANCE]);

    validate_objects::validate_cosmetic_schema(&mut h);
    validate_objects::validate_certificate_lifecycle(&mut h);
    validate_systems::validate_trading_protocol(&mut h);
    validate_objects::validate_object_memory(&mut h);
    validate_systems::validate_trio_integration(&mut h);
    validate_objects::validate_ownership_enforcement(&mut h);
    validate_systems::validate_full_scenario(&mut h);
    validate_systems::validate_composable_deployment(&mut h);

    h.finish();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("validate") | None => cmd_validate(),
        Some(other) => {
            eprintln!("Unknown command: {other}");
            std::process::exit(1);
        }
    }
}
