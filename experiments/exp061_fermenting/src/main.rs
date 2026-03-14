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
#[allow(
    dead_code,
    reason = "wire format completeness — types used at runtime over IPC"
)]
mod protocol;
mod validate_objects;
mod validate_systems;

use ludospring_barracuda::validation::ValidationResult;

const EXP: &str = "exp061_fermenting";

const fn bool_f64(b: bool) -> f64 {
    if b { 1.0 } else { 0.0 }
}

fn cmd_validate() {
    println!("=== exp061: Fermenting System ===\n");

    let mut all_results = Vec::new();

    let sections: Vec<(&str, Vec<ValidationResult>)> = vec![
        (
            "Cosmetic Schema",
            validate_objects::validate_cosmetic_schema(),
        ),
        (
            "Certificate Lifecycle",
            validate_objects::validate_certificate_lifecycle(),
        ),
        (
            "Trading Protocol",
            validate_systems::validate_trading_protocol(),
        ),
        ("Object Memory", validate_objects::validate_object_memory()),
        (
            "Trio Integration",
            validate_systems::validate_trio_integration(),
        ),
        (
            "Ownership Enforcement",
            validate_objects::validate_ownership_enforcement(),
        ),
        ("Full Scenario", validate_systems::validate_full_scenario()),
        (
            "Composable Deployment — IPC Wire Format",
            validate_systems::validate_composable_deployment(),
        ),
    ];

    for (name, results) in sections {
        println!("--- {name} ---");
        for v in &results {
            println!(
                "  [{}] {}",
                if v.passed { "PASS" } else { "FAIL" },
                v.description
            );
        }
        all_results.extend(results);
        println!();
    }

    let passed = all_results.iter().filter(|r| r.passed).count();
    let total = all_results.len();
    println!("=== SUMMARY: {passed}/{total} checks passed ===");

    if passed != total {
        println!("\nFAILED:");
        for r in all_results.iter().filter(|r| !r.passed) {
            println!(
                "  {} — measured={}, expected={}",
                r.description, r.measured, r.expected
            );
        }
        std::process::exit(1);
    }
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
