// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]

use ludospring_barracuda::interaction::difficulty;
use ludospring_barracuda::interaction::flow;
use ludospring_barracuda::metrics::engagement;
use ludospring_barracuda::metrics::fun_keys;
use ludospring_barracuda::procedural::noise;
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (analytical — wetSpring Anderson QS model)",
    commit: "4b683e3e",
    date: "2026-03-15",
    command: "N/A (cross-spring ludoSpring × wetSpring)",
};

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

fn cmd_validate() {
    let mut h = ValidationHarness::new("exp044_anderson_qs_explorer");
    h.print_provenance(&[&PROVENANCE]);

    let communities = generate_communities();
    let session_data = simulate_exploration_session(&communities);

    // 1. All communities produce valid propagation
    let all_valid = session_data
        .iter()
        .all(|(_, _, p, _)| (0.0..=1.0).contains(p));
    h.check_bool("all_propagation_valid", all_valid);

    // 2. High diversity dominates O2 in W model: anaerobic (H'=3.8, O2=0) > mucosal (H'=2.5, O2=0.4)
    let anaerobic = session_data
        .iter()
        .find(|(n, _, _, _)| n == "anaerobic_digester");
    let mucosal = session_data
        .iter()
        .find(|(n, _, _, _)| n == "mucosal_surface");
    if let (Some(a), Some(m)) = (anaerobic, mucosal) {
        h.check_bool("diversity_dominates_o2_in_w", a.1 > m.1);
    }

    // 3. Post-antibiotic has lowest diversity
    let post_ab = session_data
        .iter()
        .find(|(n, _, _, _)| n == "post_antibiotic");
    if let Some(pa) = post_ab {
        let lowest = session_data
            .iter()
            .all(|(n, _, _, div)| n == "post_antibiotic" || *div > pa.3);
        h.check_bool("post_antibiotic_lowest_diversity", lowest);
    }

    // 4. Landscape deterministic
    let l1 = generate_disorder_landscape(16, 16, 10.0, 42);
    let l2 = generate_disorder_landscape(16, 16, 10.0, 42);
    let det = l1
        .iter()
        .zip(l2.iter())
        .all(|(r1, r2)| r1.iter().zip(r2.iter()).all(|(x, y)| (x - y).abs() < 1e-15));
    h.check_bool("landscape_deterministic", det);

    // --- Phase 2: Game science metrics ---
    let snap = engagement::EngagementSnapshot {
        session_duration_s: 300.0,
        action_count: 50,
        exploration_breadth: u32::try_from(session_data.len()).unwrap_or(u32::MAX),
        challenge_seeking: 3,
        retry_count: 2,
        deliberate_pauses: 5,
    };
    let eng = engagement::compute_engagement(&snap);

    // 5. Engagement valid
    h.check_bool("engagement_valid", eng.actions_per_minute > 0.0);

    // 6. Flow state
    let flow_state = flow::evaluate_flow(0.5, 0.6, 0.15);
    let in_flow = matches!(
        flow_state,
        flow::FlowState::Flow | flow::FlowState::Relaxation
    );
    h.check_bool("exploration_in_flow", in_flow);

    // 7. Fun classification
    let signals = fun_keys::FunSignals {
        challenge: 0.5,
        exploration: 0.8,
        social: 0.2,
        completion: 0.4,
        retry_rate: 0.1,
    };
    let fun = fun_keys::classify_fun(&signals);
    h.check_bool(
        "fun_classified",
        matches!(fun.dominant, fun_keys::FunKey::Easy),
    );

    // 8. DDA suggestion
    let mut window = difficulty::PerformanceWindow::new(10);
    for &v in &[0.8, 0.7, 0.9, 0.6, 0.8] {
        window.outcomes.push_back(v);
    }
    let dda = difficulty::suggest_adjustment(&window, 0.75);
    h.check_bool("dda_bounded", dda.abs() < 1.0);

    // --- Phase 3: Cross-spring validation ---

    // 9. W model matches wetSpring Exp356
    let expected_w = 3.5f64.mul_add(3.0, 8.0 * 0.1);
    let computed_w = diversity_to_w(3.0, 0.1);
    h.check_abs("w_model_matches_wetspring", computed_w, expected_w, 1e-10);

    // 10. Higher W → lower propagation
    let low_w_prop =
        simulate_qs_propagation(&generate_disorder_landscape(32, 32, 5.0, 42), 10.0, 16, 16);
    let high_w_prop =
        simulate_qs_propagation(&generate_disorder_landscape(32, 32, 25.0, 42), 10.0, 16, 16);
    h.check_bool("higher_w_lower_propagation", low_w_prop > high_w_prop);

    // 11. Five communities → five distinct W values
    let w_values: Vec<f64> = session_data.iter().map(|(_, w, _, _)| *w).collect();
    let all_distinct = w_values
        .windows(2)
        .all(|pair| (pair[0] - pair[1]).abs() > 0.01);
    h.check_bool("communities_distinct_w", all_distinct);

    // 12. Propagation spans from localized to extended
    let props: Vec<f64> = session_data.iter().map(|(_, _, p, _)| *p).collect();
    let range = props.iter().copied().reduce(f64::max).unwrap_or(0.0)
        - props.iter().copied().reduce(f64::min).unwrap_or(0.0);
    h.check_bool("propagation_range_spans_regimes", range > 0.1);

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
