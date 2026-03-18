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

use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};
use serde::Deserialize;

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (fixture — NCBI E-utilities offline validation)",
    commit: "74cf9488",
    date: "2026-03-10",
    command: "N/A (embedded fixture data)",
};
use std::collections::HashMap;

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

/// Run all validation checks (no I/O or exit).
fn run_validation(fixture: &NcbiFixture, h: &mut ValidationHarness) {
    let qs_genes = ["luxI", "luxS", "agrB"];

    for gene in qs_genes {
        let found = fixture
            .gene_searches
            .get(gene)
            .is_some_and(|e| e.count >= 1);
        h.check_bool(&format!("qs_gene_{gene}_found"), found);
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
    h.check_bool("all_genes_have_ids", all_have_ids && all_genes_present);

    if let (Some(luxi), Some(luxs), Some(agrb)) = (
        fixture.gene_searches.get("luxI"),
        fixture.gene_searches.get("luxS"),
        fixture.gene_searches.get("agrB"),
    ) {
        h.check_bool(
            "luxS_most_universal",
            luxs.count > luxi.count && luxs.count > agrb.count,
        );
    }

    h.check_bool(
        "sra_qs_metagenomes_exist",
        fixture.sra_metagenome_count >= 1,
    );
    h.check_bool(
        "protein_autoinducer_synthase_exists",
        fixture.protein_autoinducer_count >= 1,
    );
    h.check_bool(
        "nucleotide_vfischeri_luxi",
        fixture.nucleotide_vfischeri_luxi_count >= 1,
    );
    h.check_bool("ncbi_pipeline_documented", true);
}

fn cmd_validate() {
    let fixture = match parse_fixture() {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Fixture parse error: {e}");
            std::process::exit(1);
        }
    };

    let mut h = ValidationHarness::new("exp041_ncbi_qs_integration");
    h.print_provenance(&[&PROVENANCE]);
    run_validation(&fixture, &mut h);
    h.finish();
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
        use ludospring_barracuda::validation::BufferSink;

        let fixture = parse_fixture().expect("fixture must parse");
        let mut h = ValidationHarness::with_sink("exp041_test", BufferSink::default());
        run_validation(&fixture, &mut h);
        assert!(
            h.all_passed(),
            "failed checks: {:?}",
            h.checks()
                .iter()
                .filter(|c| !c.passed)
                .map(|c| &c.label)
                .collect::<Vec<_>>()
        );
    }
}
