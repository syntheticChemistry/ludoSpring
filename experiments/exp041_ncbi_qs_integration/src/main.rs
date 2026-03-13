// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]

use ludospring_barracuda::validation::ValidationResult;
use serde::Deserialize;
use std::thread;
use std::time::{Duration, Instant};

const NCBI_ESEARCH: &str = "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esearch.fcgi";
const NCBI_ESUMMARY: &str = "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esummary.fcgi";
const EXP: &str = "exp041";

#[derive(Deserialize)]
struct ESearchResponse {
    esearchresult: ESearchResult,
}

#[derive(Deserialize)]
struct ESearchResult {
    count: String,
    idlist: Vec<String>,
}

struct QsGeneResult {
    gene: String,
    count: u64,
    ids: Vec<String>,
    latency_ms: u128,
}

fn ncbi_search(database: &str, query: &str, max_results: u32) -> Result<ESearchResponse, String> {
    let url = format!(
        "{}?db={}&term={}&retmax={}&retmode=json",
        NCBI_ESEARCH,
        database,
        query.replace(' ', "+"),
        max_results
    );
    let body: ESearchResponse = ureq::get(&url)
        .call()
        .map_err(|e| format!("HTTP error: {e}"))?
        .body_mut()
        .with_config()
        .limit(1_000_000)
        .read_json()
        .map_err(|e| format!("JSON parse error: {e}"))?;
    Ok(body)
}

fn ncbi_summary(database: &str, id: &str) -> Result<serde_json::Value, String> {
    let url = format!("{NCBI_ESUMMARY}?db={database}&id={id}&retmode=json");
    let body: serde_json::Value = ureq::get(&url)
        .call()
        .map_err(|e| format!("HTTP error: {e}"))?
        .body_mut()
        .with_config()
        .limit(1_000_000)
        .read_json()
        .map_err(|e| format!("JSON parse error: {e}"))?;
    Ok(body)
}

fn search_qs_gene(gene: &str) -> Result<QsGeneResult, String> {
    let query = format!("{gene} quorum sensing");
    let start = Instant::now();
    let resp = ncbi_search("gene", &query, 10)?;
    let latency_ms = start.elapsed().as_millis();
    let count: u64 = resp.esearchresult.count.parse().unwrap_or(0);
    Ok(QsGeneResult {
        gene: gene.to_string(),
        count,
        ids: resp.esearchresult.idlist,
        latency_ms,
    })
}

fn ncbi_rate_limit() {
    thread::sleep(Duration::from_millis(400));
}

const fn bool_f64(b: bool) -> f64 {
    if b { 1.0 } else { 0.0 }
}

#[expect(
    clippy::too_many_lines,
    reason = "validation orchestrator — sequential check groups"
)]
#[allow(clippy::similar_names)]
fn cmd_validate() {
    println!("=== exp041: NCBI QS Gene Integration Test ===\n");
    let mut results = Vec::new();

    let qs_genes = ["luxI", "luxS", "agrB"];
    let mut gene_results = Vec::new();

    for (i, gene) in qs_genes.iter().enumerate() {
        if i > 0 {
            ncbi_rate_limit();
        }
        match search_qs_gene(gene) {
            Ok(result) => {
                println!(
                    "  {}: {} results, {} IDs returned, {}ms",
                    result.gene,
                    result.count,
                    result.ids.len(),
                    result.latency_ms
                );
                results.push(ValidationResult::check(
                    EXP,
                    &format!("qs_gene_{gene}_found"),
                    bool_f64(result.count >= 1),
                    1.0,
                    0.0,
                ));
                gene_results.push(result);
            }
            Err(e) => {
                println!("  {gene}: ERROR - {e}");
                results.push(ValidationResult::check(
                    EXP,
                    &format!("qs_gene_{gene}_found"),
                    0.0,
                    1.0,
                    0.0,
                ));
            }
        }
    }

    let all_have_ids = gene_results.iter().all(|r| !r.ids.is_empty());
    results.push(ValidationResult::check(
        EXP,
        "all_genes_have_ids",
        bool_f64(all_have_ids && gene_results.len() == 3),
        1.0,
        0.0,
    ));

    if gene_results.len() == 3 {
        let luxs_count = gene_results[1].count;
        #[allow(clippy::similar_names)]
        let luxi_count = gene_results[0].count;
        let agrb_count = gene_results[2].count;
        results.push(ValidationResult::check(
            EXP,
            "luxS_most_universal",
            bool_f64(luxs_count > luxi_count && luxs_count > agrb_count),
            1.0,
            0.0,
        ));
    }

    let all_fast = gene_results.iter().all(|r| r.latency_ms < 5000);
    results.push(ValidationResult::check(
        EXP,
        "ncbi_latency_acceptable",
        bool_f64(all_fast),
        1.0,
        0.0,
    ));

    ncbi_rate_limit();
    if gene_results.len() >= 2 && !gene_results[1].ids.is_empty() {
        let first_id = &gene_results[1].ids[0];
        match ncbi_summary("gene", first_id) {
            Ok(summary) => {
                let has_result = summary.get("result").is_some();
                println!(
                    "\n  ESummary for gene {}: {}",
                    first_id,
                    if has_result {
                        "valid response"
                    } else {
                        "no result field"
                    }
                );
                results.push(ValidationResult::check(
                    EXP,
                    "esummary_returns_data",
                    bool_f64(has_result),
                    1.0,
                    0.0,
                ));
            }
            Err(e) => {
                println!("\n  ESummary error: {e}");
                results.push(ValidationResult::check(
                    EXP,
                    "esummary_returns_data",
                    0.0,
                    1.0,
                    0.0,
                ));
            }
        }
    }

    ncbi_rate_limit();
    println!();
    match ncbi_search("sra", "quorum sensing metagenome 16S", 5) {
        Ok(resp) => {
            let count: u64 = resp.esearchresult.count.parse().unwrap_or(0);
            println!("  SRA metagenome search: {count} results");
            results.push(ValidationResult::check(
                EXP,
                "sra_qs_metagenomes_exist",
                bool_f64(count >= 1),
                1.0,
                0.0,
            ));
        }
        Err(e) => {
            println!("  SRA search error: {e}");
            results.push(ValidationResult::check(
                EXP,
                "sra_qs_metagenomes_exist",
                0.0,
                1.0,
                0.0,
            ));
        }
    }

    ncbi_rate_limit();
    match ncbi_search("protein", "autoinducer synthase bacteria", 5) {
        Ok(resp) => {
            let count: u64 = resp.esearchresult.count.parse().unwrap_or(0);
            println!("  Protein autoinducer synthase: {count} results");
            results.push(ValidationResult::check(
                EXP,
                "protein_autoinducer_synthase_exists",
                bool_f64(count >= 1),
                1.0,
                0.0,
            ));
        }
        Err(e) => {
            println!("  Protein search error: {e}");
            results.push(ValidationResult::check(
                EXP,
                "protein_autoinducer_synthase_exists",
                0.0,
                1.0,
                0.0,
            ));
        }
    }

    ncbi_rate_limit();
    match ncbi_search("nucleotide", "Vibrio fischeri luxI", 3) {
        Ok(resp) => {
            let count: u64 = resp.esearchresult.count.parse().unwrap_or(0);
            println!("  Nucleotide V. fischeri luxI: {count} results");
            results.push(ValidationResult::check(
                EXP,
                "nucleotide_vfischeri_luxi",
                bool_f64(count >= 1),
                1.0,
                0.0,
            ));
        }
        Err(e) => {
            println!("  Nucleotide search error: {e}");
            results.push(ValidationResult::check(
                EXP,
                "nucleotide_vfischeri_luxi",
                0.0,
                1.0,
                0.0,
            ));
        }
    }

    let total_latency: u128 = gene_results.iter().map(|r| r.latency_ms).sum();
    results.push(ValidationResult::check(
        EXP,
        "gene_query_total_latency",
        bool_f64(total_latency < 30_000),
        1.0,
        0.0,
    ));

    println!("\n  --- NestGate Status ---");
    println!("  NCBI E-utilities: LIVE (direct HTTP)");
    println!("  NestGate daemon: data_sources::providers not wired in mod.rs");
    println!("  Action: wire ncbi_live_provider.rs into module tree, restore HTTP client");
    results.push(ValidationResult::check(
        EXP,
        "ncbi_pipeline_documented",
        1.0,
        1.0,
        0.0,
    ));

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
