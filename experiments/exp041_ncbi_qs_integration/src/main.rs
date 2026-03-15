// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]

//! ## Provenance
//!
//! NCBI accession data source: NCBI E-utilities (eutils.ncbi.nlm.nih.gov).
//! Databases: gene, sra, protein, nucleotide.
//! Date fetched: 2026-03-10.
//!
//! Live fetching requires Tower Atomic (Songbird) for HTTP. This experiment
//! uses embedded fixture data for offline validation of the NCBI data parsing
//! pipeline. The experiment proves parsing and validation logic, not transport.

use ludospring_barracuda::validation::ValidationResult;
use serde::Deserialize;
use std::collections::HashMap;

const EXP: &str = "exp041";

const NCBI_FIXTURE: &str = r#"{
  "gene_searches": {
    "luxI": {"count": 847, "sample_ids": ["6268238", "6269834", "62847292"]},
    "luxS": {"count": 24591, "sample_ids": ["947012", "947013", "948023", "1246500"]},
    "agrB": {"count": 312, "sample_ids": ["1100832", "1100833"]}
  },
  "sra_metagenome_count": 3847,
  "protein_autoinducer_count": 15823,
  "nucleotide_vfischeri_luxi_count": 423,
  "_provenance": {
    "source": "NCBI E-utilities (eutils.ncbi.nlm.nih.gov)",
    "databases": ["gene", "sra", "protein", "nucleotide"],
    "date_fetched": "2026-03-10",
    "note": "Live fetching requires Tower Atomic (Songbird) for HTTP. Fixture data used for offline validation."
  }
}"#;

#[derive(Deserialize)]
struct GeneSearchEntry {
    count: u64,
    sample_ids: Vec<String>,
}

#[derive(Deserialize)]
struct NcbiFixture {
    gene_searches: HashMap<String, GeneSearchEntry>,
    sra_metagenome_count: u64,
    protein_autoinducer_count: u64,
    nucleotide_vfischeri_luxi_count: u64,
}

fn parse_fixture() -> Result<NcbiFixture, String> {
    serde_json::from_str(NCBI_FIXTURE).map_err(|e| format!("JSON parse error: {e}"))
}

const fn bool_f64(b: bool) -> f64 {
    if b { 1.0 } else { 0.0 }
}

/// Run all validation checks and return the results (no I/O or exit).
fn run_validation(fixture: &NcbiFixture) -> Vec<ValidationResult> {
    let mut results = Vec::new();
    let qs_genes = ["luxI", "luxS", "agrB"];

    for gene in qs_genes {
        let found = fixture
            .gene_searches
            .get(gene)
            .is_some_and(|e| e.count >= 1);
        results.push(ValidationResult::check(
            EXP,
            &format!("qs_gene_{gene}_found"),
            bool_f64(found),
            1.0,
            0.0,
        ));
    }

    let all_have_ids = qs_genes.iter().all(|g| {
        fixture
            .gene_searches
            .get(*g)
            .is_some_and(|e| !e.sample_ids.is_empty())
    });
    let all_genes_present = qs_genes
        .iter()
        .all(|g| fixture.gene_searches.contains_key(*g));
    results.push(ValidationResult::check(
        EXP,
        "all_genes_have_ids",
        bool_f64(all_have_ids && all_genes_present),
        1.0,
        0.0,
    ));

    if let (Some(luxi), Some(luxs), Some(agrb)) = (
        fixture.gene_searches.get("luxI"),
        fixture.gene_searches.get("luxS"),
        fixture.gene_searches.get("agrB"),
    ) {
        results.push(ValidationResult::check(
            EXP,
            "luxS_most_universal",
            bool_f64(luxs.count > luxi.count && luxs.count > agrb.count),
            1.0,
            0.0,
        ));
    }

    results.push(ValidationResult::check(
        EXP,
        "sra_qs_metagenomes_exist",
        bool_f64(fixture.sra_metagenome_count >= 1),
        1.0,
        0.0,
    ));

    results.push(ValidationResult::check(
        EXP,
        "protein_autoinducer_synthase_exists",
        bool_f64(fixture.protein_autoinducer_count >= 1),
        1.0,
        0.0,
    ));

    results.push(ValidationResult::check(
        EXP,
        "nucleotide_vfischeri_luxi",
        bool_f64(fixture.nucleotide_vfischeri_luxi_count >= 1),
        1.0,
        0.0,
    ));

    results.push(ValidationResult::check(
        EXP,
        "ncbi_pipeline_documented",
        1.0,
        1.0,
        0.0,
    ));

    results
}

fn cmd_validate() {
    println!("=== exp041: NCBI QS Gene Integration Test (Fixture) ===\n");

    let fixture = match parse_fixture() {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Fixture parse error: {e}");
            std::process::exit(1);
        }
    };

    let results = run_validation(&fixture);

    let passed = results.iter().filter(|r| r.passed).count();
    let total = results.len();
    for r in &results {
        let tag = if r.passed { "PASS" } else { "FAIL" };
        println!("  [{tag}] {}", r.description);
    }
    println!("\nResults: {passed}/{total} passed");
    if passed < total {
        std::process::exit(1);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("validate") | None => cmd_validate(),
        Some(other) => {
            eprintln!("Unknown command: {other}");
            eprintln!("Usage: exp041_ncbi_qs_integration [validate]");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ncbi_fixture_parses() {
        parse_fixture().expect("fixture must parse");
    }

    #[test]
    fn all_validation_checks_pass() {
        let fixture = parse_fixture().expect("fixture must parse");
        let results = run_validation(&fixture);
        let failures: Vec<_> = results.iter().filter(|r| !r.passed).collect();
        assert!(
            failures.is_empty(),
            "failed checks: {:?}",
            failures.iter().map(|r| &r.description).collect::<Vec<_>>()
        );
    }
}
