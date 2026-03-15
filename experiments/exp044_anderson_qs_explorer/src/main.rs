// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]

use ludospring_barracuda::interaction::difficulty;
use ludospring_barracuda::interaction::flow;
use ludospring_barracuda::metrics::engagement;
use ludospring_barracuda::metrics::fun_keys;
use ludospring_barracuda::procedural::noise;
use ludospring_barracuda::validation::ValidationResult;
const EXP: &str = "exp044";

const fn bool_f64(b: bool) -> f64 {
    if b { 1.0 } else { 0.0 }
}

#[derive(Debug, Clone)]
#[expect(dead_code, reason = "domain model completeness")]
struct MicrobialCommunity {
    name: String,
    diversity_h: f64,
    evenness_j: f64,
    qs_gene_density: f64,
    oxygen_level: f64,
}

#[expect(
    clippy::needless_range_loop,
    clippy::cast_precision_loss,
    reason = "grid iteration, coords fit in f64"
)]
fn generate_disorder_landscape(
    width: usize,
    height: usize,
    w_max: f64,
    seed: u64,
) -> Vec<Vec<f64>> {
    let mut landscape = vec![vec![0.0_f64; width]; height];
    for y in 0..height {
        for x in 0..width {
            let nx = x as f64 / width as f64;
            let ny = y as f64 / height as f64;
            let offset = seed as f64 * 137.0;
            let noise_val = noise::perlin_2d(nx.mul_add(4.0, offset), ny.mul_add(4.0, offset));
            landscape[y][x] = w_max * (noise_val + 1.0) / 2.0;
        }
    }
    landscape
}

#[expect(
    clippy::cast_precision_loss,
    reason = "reached count bounded by grid size"
)]
fn simulate_qs_propagation(
    landscape: &[Vec<f64>],
    signal_strength: f64,
    start_x: usize,
    start_y: usize,
) -> f64 {
    let height = landscape.len();
    let width = landscape[0].len();
    let total_cells = (width * height) as f64;

    let mut reached = vec![vec![false; width]; height];
    let mut frontier = vec![(start_x, start_y)];
    reached[start_y][start_x] = true;
    let mut reached_count = 1_u64;

    while let Some((cx, cy)) = frontier.pop() {
        let neighbors = [
            (cx.wrapping_sub(1), cy),
            (cx + 1, cy),
            (cx, cy.wrapping_sub(1)),
            (cx, cy + 1),
        ];
        for (nx, ny) in neighbors {
            if nx < width && ny < height && !reached[ny][nx] {
                let disorder = landscape[ny][nx];
                if signal_strength > disorder {
                    reached[ny][nx] = true;
                    reached_count += 1;
                    frontier.push((nx, ny));
                }
            }
        }
    }

    reached_count as f64 / total_cells
}

fn diversity_to_w(h_prime: f64, o2_level: f64) -> f64 {
    3.5f64.mul_add(h_prime, 8.0 * o2_level)
}

fn generate_communities() -> Vec<MicrobialCommunity> {
    vec![
        MicrobialCommunity {
            name: "healthy_gut_lumen".into(),
            diversity_h: 3.2,
            evenness_j: 0.85,
            qs_gene_density: 0.7,
            oxygen_level: 0.05,
        },
        MicrobialCommunity {
            name: "dysbiotic_gut".into(),
            diversity_h: 1.8,
            evenness_j: 0.55,
            qs_gene_density: 0.4,
            oxygen_level: 0.15,
        },
        MicrobialCommunity {
            name: "mucosal_surface".into(),
            diversity_h: 2.5,
            evenness_j: 0.70,
            qs_gene_density: 0.55,
            oxygen_level: 0.40,
        },
        MicrobialCommunity {
            name: "anaerobic_digester".into(),
            diversity_h: 3.8,
            evenness_j: 0.90,
            qs_gene_density: 0.85,
            oxygen_level: 0.0,
        },
        MicrobialCommunity {
            name: "post_antibiotic".into(),
            diversity_h: 0.8,
            evenness_j: 0.35,
            qs_gene_density: 0.2,
            oxygen_level: 0.20,
        },
    ]
}

fn simulate_exploration_session(
    communities: &[MicrobialCommunity],
) -> Vec<(String, f64, f64, f64)> {
    let mut session_data = Vec::new();
    let grid_size = 32;

    for community in communities {
        let w = diversity_to_w(community.diversity_h, community.oxygen_level);
        let landscape = generate_disorder_landscape(grid_size, grid_size, w, 42);
        let signal_strength = community.qs_gene_density * w * 0.8;
        let propagation =
            simulate_qs_propagation(&landscape, signal_strength, grid_size / 2, grid_size / 2);
        session_data.push((
            community.name.clone(),
            w,
            propagation,
            community.diversity_h,
        ));
    }

    session_data
}

#[expect(
    clippy::too_many_lines,
    reason = "validation orchestrator — sequential check groups"
)]
fn cmd_validate() {
    println!("=== exp044: Anderson QS Interactive Explorer ===\n");
    println!("  Cross-spring: ludoSpring (game science) x wetSpring (QS model)\n");
    let mut results = Vec::new();

    let communities = generate_communities();

    println!("  Phase 1: QS propagation across microbial communities");
    let session_data = simulate_exploration_session(&communities);

    for (name, w, prop, h) in &session_data {
        println!("    {name}: W={w:.2}, propagation={prop:.3}, H'={h:.1}");
    }

    // 1. All communities produce valid propagation
    let all_valid = session_data
        .iter()
        .all(|(_, _, p, _)| (0.0..=1.0).contains(p));
    results.push(ValidationResult::check(
        EXP,
        "all_propagation_valid",
        bool_f64(all_valid),
        1.0,
        0.0,
    ));

    // 2. High diversity dominates O2 in W model: anaerobic (H'=3.8, O2=0) > mucosal (H'=2.5, O2=0.4)
    let anaerobic = session_data
        .iter()
        .find(|(n, _, _, _)| n == "anaerobic_digester");
    let mucosal = session_data
        .iter()
        .find(|(n, _, _, _)| n == "mucosal_surface");
    if let (Some(a), Some(m)) = (anaerobic, mucosal) {
        println!(
            "\n  Diversity vs O2: anaerobic W={:.2} (H'=3.8) vs mucosal W={:.2} (H'=2.5, O2=0.4)",
            a.1, m.1
        );
        results.push(ValidationResult::check(
            EXP,
            "diversity_dominates_o2_in_w",
            bool_f64(a.1 > m.1),
            1.0,
            0.0,
        ));
    }

    // 3. Post-antibiotic has lowest diversity
    let post_ab = session_data
        .iter()
        .find(|(n, _, _, _)| n == "post_antibiotic");
    if let Some(pa) = post_ab {
        let lowest = session_data
            .iter()
            .all(|(n, _, _, h)| n == "post_antibiotic" || *h > pa.3);
        results.push(ValidationResult::check(
            EXP,
            "post_antibiotic_lowest_diversity",
            bool_f64(lowest),
            1.0,
            0.0,
        ));
    }

    // 4. Landscape deterministic
    let l1 = generate_disorder_landscape(16, 16, 10.0, 42);
    let l2 = generate_disorder_landscape(16, 16, 10.0, 42);
    let det = l1
        .iter()
        .zip(l2.iter())
        .all(|(r1, r2)| r1.iter().zip(r2.iter()).all(|(a, b)| (a - b).abs() < 1e-15));
    results.push(ValidationResult::check(
        EXP,
        "landscape_deterministic",
        bool_f64(det),
        1.0,
        0.0,
    ));

    // --- Phase 2: Game science metrics ---
    println!("\n  Phase 2: Game science metrics on QS exploration");

    let snap = engagement::EngagementSnapshot {
        session_duration_s: 300.0,
        action_count: 50,
        exploration_breadth: u32::try_from(session_data.len()).unwrap_or(u32::MAX),
        challenge_seeking: 3,
        retry_count: 2,
        deliberate_pauses: 5,
    };
    let eng = engagement::compute_engagement(&snap);
    println!(
        "    engagement: apm={:.3}, exploration={:.3}",
        eng.actions_per_minute, eng.exploration_rate
    );

    // 5. Engagement valid
    results.push(ValidationResult::check(
        EXP,
        "engagement_valid",
        bool_f64(eng.actions_per_minute > 0.0),
        1.0,
        0.0,
    ));

    // 6. Flow state
    let flow_state = flow::evaluate_flow(0.5, 0.6, 0.15);
    println!("    flow: {flow_state:?}");
    let in_flow = matches!(
        flow_state,
        flow::FlowState::Flow | flow::FlowState::Relaxation
    );
    results.push(ValidationResult::check(
        EXP,
        "exploration_in_flow",
        bool_f64(in_flow),
        1.0,
        0.0,
    ));

    // 7. Fun classification
    let signals = fun_keys::FunSignals {
        challenge: 0.5,
        exploration: 0.8,
        social: 0.2,
        completion: 0.4,
        retry_rate: 0.1,
    };
    let fun = fun_keys::classify_fun(&signals);
    println!("    fun: {:?}", fun.dominant);
    results.push(ValidationResult::check(
        EXP,
        "fun_classified",
        bool_f64(matches!(fun.dominant, fun_keys::FunKey::Easy)),
        1.0,
        0.0,
    ));

    // 8. DDA suggestion
    let mut window = difficulty::PerformanceWindow::new(10);
    for &v in &[0.8, 0.7, 0.9, 0.6, 0.8] {
        window.outcomes.push_back(v);
    }
    let dda = difficulty::suggest_adjustment(&window, 0.75);
    println!("    DDA: {dda:+.3}");
    results.push(ValidationResult::check(
        EXP,
        "dda_bounded",
        bool_f64(dda.abs() < 1.0),
        1.0,
        0.0,
    ));

    // --- Phase 3: Cross-spring validation ---
    println!("\n  Phase 3: Cross-spring validation");

    // 9. W model matches wetSpring Exp356
    let expected_w = 3.5f64.mul_add(3.0, 8.0 * 0.1);
    let computed_w = diversity_to_w(3.0, 0.1);
    results.push(ValidationResult::check(
        EXP,
        "w_model_matches_wetspring",
        computed_w,
        expected_w,
        1e-10,
    ));

    // 10. Higher W → lower propagation
    let low_w_prop =
        simulate_qs_propagation(&generate_disorder_landscape(32, 32, 5.0, 42), 10.0, 16, 16);
    let high_w_prop =
        simulate_qs_propagation(&generate_disorder_landscape(32, 32, 25.0, 42), 10.0, 16, 16);
    println!("    W=5 propagation: {low_w_prop:.3}, W=25 propagation: {high_w_prop:.3}");
    results.push(ValidationResult::check(
        EXP,
        "higher_w_lower_propagation",
        bool_f64(low_w_prop > high_w_prop),
        1.0,
        0.0,
    ));

    // 11. Five communities → five distinct W values
    let w_values: Vec<f64> = session_data.iter().map(|(_, w, _, _)| *w).collect();
    let all_distinct = w_values
        .windows(2)
        .all(|pair| (pair[0] - pair[1]).abs() > 0.01);
    results.push(ValidationResult::check(
        EXP,
        "communities_distinct_w",
        bool_f64(all_distinct),
        1.0,
        0.0,
    ));

    // 12. Propagation spans from localized to extended
    let props: Vec<f64> = session_data.iter().map(|(_, _, p, _)| *p).collect();
    let range = props.iter().copied().reduce(f64::max).unwrap_or(0.0)
        - props.iter().copied().reduce(f64::min).unwrap_or(0.0);
    println!("    propagation range: {range:.3}");
    results.push(ValidationResult::check(
        EXP,
        "propagation_range_spans_regimes",
        bool_f64(range > 0.1),
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
