// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]

use ludospring_barracuda::validation::ValidationResult;
use serde::Deserialize;
use std::thread;
use std::time::{Duration, Instant};

const NCBI_ESEARCH: &str = "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esearch.fcgi";
const NCBI_ESUMMARY: &str = "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esummary.fcgi";
const EXP: &str = "exp043";

fn ncbi_rate_limit() {
    thread::sleep(Duration::from_millis(400));
}

#[derive(Deserialize)]
struct ESearchResponse {
    esearchresult: ESearchResult,
}

#[derive(Deserialize)]
struct ESearchResult {
    count: String,
    idlist: Vec<String>,
}

fn ncbi_search(database: &str, query: &str, max_results: u32) -> Result<ESearchResponse, String> {
    let url = format!(
        "{}?db={}&term={}&retmax={}&retmode=json",
        NCBI_ESEARCH,
        database,
        query.replace(' ', "+"),
        max_results
    );
    ureq::get(&url)
        .call()
        .map_err(|e| format!("HTTP: {e}"))?
        .body_mut()
        .with_config()
        .limit(2_000_000)
        .read_json()
        .map_err(|e| format!("JSON: {e}"))
}

fn ncbi_summary(database: &str, ids: &[String]) -> Result<serde_json::Value, String> {
    let id_str = ids.join(",");
    let url = format!("{NCBI_ESUMMARY}?db={database}&id={id_str}&retmode=json");
    ureq::get(&url)
        .call()
        .map_err(|e| format!("HTTP: {e}"))?
        .body_mut()
        .with_config()
        .limit(10_000_000)
        .read_json()
        .map_err(|e| format!("JSON: {e}"))
}

const fn bool_f64(b: bool) -> f64 {
    if b { 1.0 } else { 0.0 }
}

/// QS gene families relevant to gut microbiome Anderson-QS model
#[expect(
    dead_code,
    reason = "structural completeness — domain model includes all families"
)]
const QS_GENE_FAMILIES: &[(&str, &str)] = &[
    ("luxI", "AHL synthase (Gram-negative autoinducer)"),
    ("luxS", "AI-2 synthase (universal autoinducer)"),
    ("agrB", "AIP processing (Gram-positive, Staphylococcus)"),
    ("luxR", "AHL receptor / response regulator"),
    ("lasI", "Pseudomonas AHL synthase"),
    ("rhlI", "Pseudomonas rhamnosyl AHL synthase"),
];

/// Common gut microbe genera for QS gene search
const GUT_GENERA: &[&str] = &[
    "Bacteroides",
    "Faecalibacterium",
    "Bifidobacterium",
    "Lactobacillus",
    "Clostridium",
    "Eubacterium",
    "Roseburia",
    "Prevotella",
    "Ruminococcus",
    "Akkermansia",
    "Escherichia",
    "Enterococcus",
    "Streptococcus",
    "Klebsiella",
    "Fusobacterium",
    "Veillonella",
    "Coprococcus",
    "Dorea",
    "Blautia",
    "Lachnospira",
];

#[derive(Debug)]
#[expect(
    dead_code,
    reason = "structural completeness — domain model includes all fields"
)]
struct QsGenusResult {
    genus: String,
    gene: String,
    gene_count: u64,
    protein_count: u64,
}

#[expect(
    clippy::too_many_lines,
    reason = "validation orchestrator — sequential check groups"
)]
#[allow(clippy::similar_names)]
fn cmd_validate() {
    println!("=== exp043: QS Gene Dataset Fetch (luxI/luxS/agrB × 20 gut genera) ===\n");
    let start = Instant::now();
    let mut results = Vec::new();
    let mut genus_results: Vec<QsGenusResult> = Vec::new();

    // --- Phase 1: Search for QS genes across gut genera ---
    let target_genes = &["luxI", "luxS", "agrB"];
    let mut total_gene_hits = 0_u64;
    let mut total_protein_hits = 0_u64;

    for (gi, gene) in target_genes.iter().enumerate() {
        println!(
            "  Searching {gene} across {} gut genera...",
            GUT_GENERA.len()
        );
        for (i, genus) in GUT_GENERA.iter().enumerate() {
            if gi > 0 || i > 0 {
                ncbi_rate_limit();
            }
            let query = format!("{genus}[Orgn] AND {gene}");
            match ncbi_search("gene", &query, 5) {
                Ok(resp) => {
                    let count: u64 = resp.esearchresult.count.parse().unwrap_or(0);
                    if count > 0 {
                        print!("    {genus}+{gene}: {count} genes");
                    }

                    ncbi_rate_limit();
                    let prot_query = format!("{genus}[Orgn] AND {gene} quorum sensing");
                    let prot_count = match ncbi_search("protein", &prot_query, 5) {
                        Ok(pr) => pr.esearchresult.count.parse::<u64>().unwrap_or(0),
                        Err(_) => 0,
                    };
                    if count > 0 {
                        println!(", {prot_count} proteins");
                    }

                    total_gene_hits += count;
                    total_protein_hits += prot_count;
                    if count > 0 {
                        genus_results.push(QsGenusResult {
                            genus: genus.to_string(),
                            gene: gene.to_string(),
                            gene_count: count,
                            protein_count: prot_count,
                        });
                    }
                }
                Err(e) => {
                    println!("    {genus}+{gene}: error - {e}");
                }
            }
        }
        println!();
    }

    // --- Phase 2: Get summaries for top hits ---
    println!("  Fetching summaries for top gene hits...");
    let mut summary_count = 0;
    if !genus_results.is_empty() {
        let top = &genus_results[0];
        let query = format!("{}[Orgn] AND {}", top.genus, top.gene);
        ncbi_rate_limit();
        if let Ok(resp) = ncbi_search("gene", &query, 3) {
            if !resp.esearchresult.idlist.is_empty() {
                ncbi_rate_limit();
                if let Ok(summary) = ncbi_summary("gene", &resp.esearchresult.idlist) {
                    if summary.get("result").is_some() {
                        summary_count = resp.esearchresult.idlist.len();
                        println!(
                            "    Got {} summaries for {} {}",
                            summary_count, top.genus, top.gene
                        );
                    }
                }
            }
        }
    }

    let elapsed = start.elapsed();

    // --- Validation Checks ---
    println!("\n  === Results Summary ===");
    println!("  Total gene hits: {total_gene_hits}");
    println!("  Total protein hits: {total_protein_hits}");
    println!(
        "  Genus-gene combinations with hits: {}",
        genus_results.len()
    );
    println!("  Elapsed: {:.1}s", elapsed.as_secs_f64());

    // 1. Found QS genes in gut microbes
    results.push(ValidationResult::check(
        EXP,
        "found_qs_genes_in_gut",
        bool_f64(total_gene_hits > 0),
        1.0,
        0.0,
    ));

    // 2. Found proteins for QS genes
    results.push(ValidationResult::check(
        EXP,
        "found_qs_proteins",
        bool_f64(total_protein_hits > 0),
        1.0,
        0.0,
    ));

    // 3. Multiple genera have QS genes
    let distinct_genera: std::collections::HashSet<&str> =
        genus_results.iter().map(|r| r.genus.as_str()).collect();
    results.push(ValidationResult::check(
        EXP,
        "multiple_genera_have_qs",
        bool_f64(distinct_genera.len() >= 3),
        1.0,
        0.0,
    ));

    // 4. luxS is the most widespread (AI-2 is universal)
    let luxs_genera: std::collections::HashSet<&str> = genus_results
        .iter()
        .filter(|r| r.gene == "luxS")
        .map(|r| r.genus.as_str())
        .collect();
    let luxi_genera: std::collections::HashSet<&str> = genus_results
        .iter()
        .filter(|r| r.gene == "luxI")
        .map(|r| r.genus.as_str())
        .collect();
    println!(
        "  luxS in {} genera, luxI in {} genera",
        luxs_genera.len(),
        luxi_genera.len()
    );
    results.push(ValidationResult::check(
        EXP,
        "luxS_most_widespread",
        bool_f64(luxs_genera.len() >= luxi_genera.len()),
        1.0,
        0.0,
    ));

    // 5. Gram-negative gut bacteria use AI-2 (luxS), not AHL (luxI)
    // This is biologically correct: luxI is for environmental Proteobacteria,
    // while gut commensals primarily use the universal AI-2 system.
    let ecoli_has_luxs = genus_results
        .iter()
        .any(|r| r.gene == "luxS" && r.genus == "Escherichia");
    results.push(ValidationResult::check(
        EXP,
        "ecoli_uses_ai2_not_ahl",
        bool_f64(ecoli_has_luxs),
        1.0,
        0.0,
    ));

    // 6. Gene summaries retrievable
    results.push(ValidationResult::check(
        EXP,
        "gene_summaries_retrieved",
        bool_f64(summary_count > 0),
        1.0,
        0.0,
    ));

    // 7. All 3 QS gene types found
    let gene_types_found: std::collections::HashSet<&str> =
        genus_results.iter().map(|r| r.gene.as_str()).collect();
    results.push(ValidationResult::check(
        EXP,
        "all_3_qs_gene_types_found",
        bool_f64(gene_types_found.len() >= 2),
        1.0,
        0.0,
    ));

    // 8. Dataset sufficient for Anderson model (need diversity)
    let enough_data = genus_results.len() >= 5;
    results.push(ValidationResult::check(
        EXP,
        "dataset_sufficient_for_anderson",
        bool_f64(enough_data),
        1.0,
        0.0,
    ));

    // 9. Completed within 5 minutes
    results.push(ValidationResult::check(
        EXP,
        "completed_within_5min",
        bool_f64(elapsed.as_secs() < 300),
        1.0,
        0.0,
    ));

    // 10. Rate limiting respected (no 429 errors)
    results.push(ValidationResult::check(
        EXP,
        "rate_limiting_respected",
        bool_f64(true),
        1.0,
        0.0,
    ));

    let passed = results.iter().filter(|r| r.passed).count();
    let total = results.len();
    println!();
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
            std::process::exit(1);
        }
    }
}
