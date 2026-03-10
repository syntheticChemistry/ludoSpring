// SPDX-License-Identifier: AGPL-3.0-or-later
//! Exp004: Folding adversarial — player vs AI protein folding.
//!
//! Models the Folding@Home / FoldIt concept: protein folding as a puzzle
//! game where human spatial intuition competes with algorithmic optimization.
//! Measures engagement, flow state, and difficulty adaptation.

use ludospring_barracuda::interaction::difficulty::{PerformanceWindow, suggest_adjustment};
use ludospring_barracuda::interaction::flow::{DifficultyCurve, FlowState, evaluate_flow};
use ludospring_barracuda::metrics::engagement::{EngagementSnapshot, compute_engagement};
use ludospring_barracuda::tolerances;

fn main() {
    println!("=== Exp004: Folding Adversarial (Player vs AI) ===\n");

    // Simulate a session where difficulty adapts to player performance
    let curve = DifficultyCurve::sigmoid(0.2, 0.9, 8.0);
    let mut window = PerformanceWindow::new(10);

    println!("Simulating 20-round folding session with DDA:\n");
    println!(
        "  {:>5} │ {:>8} │ {:>6} │ {:>10} │ {:>6}",
        "Round", "Progress", "Diff", "Flow", "Adjust"
    );
    println!("  ──────┼──────────┼────────┼────────────┼────────");

    let mut player_skill = 0.3_f64;
    for round in 0..20 {
        let progress = round as f64 / 19.0;
        let base_difficulty = curve.sample(progress);

        // Apply DDA adjustment
        let adjustment = suggest_adjustment(&window, tolerances::DDA_TARGET_SUCCESS_RATE);
        let difficulty = (base_difficulty + adjustment * 0.1).clamp(0.0, 1.0);

        // Evaluate flow state
        let flow = evaluate_flow(difficulty, player_skill, tolerances::FLOW_CHANNEL_WIDTH);

        // Simulate player outcome (skill grows, random success)
        let success = if difficulty < player_skill + 0.1 {
            0.8
        } else {
            0.3
        };
        window.record(success);

        println!(
            "  {:5} │ {:8.2} │ {:6.2} │ {:10?} │ {:+6.2}",
            round + 1,
            progress,
            difficulty,
            flow,
            adjustment
        );

        // Player learns (skill increases slowly)
        player_skill = (player_skill + 0.02).min(0.95);
    }

    // Compute engagement for a "good" session
    let snap = EngagementSnapshot {
        session_duration_s: 600.0,
        action_count: 180,
        exploration_breadth: 12,
        challenge_seeking: 8,
        retry_count: 15,
        deliberate_pauses: 20,
    };
    let metrics = compute_engagement(&snap);

    println!("\nSession engagement metrics:");
    println!("  Actions/min:      {:.1}", metrics.actions_per_minute);
    println!("  Exploration rate:  {:.2}", metrics.exploration_rate);
    println!("  Challenge appetite:{:.2}", metrics.challenge_appetite);
    println!("  Persistence:       {:.2}", metrics.persistence);
    println!("  Deliberation:      {:.2}", metrics.deliberation);
    println!("  Composite score:   {:.3}", metrics.composite);

    // Verify flow channel maintained for most rounds
    let mut flow_count = 0_u32;
    let mut skill = 0.3_f64;
    for round in 0..20 {
        let progress = round as f64 / 19.0;
        let difficulty = curve.sample(progress);
        if evaluate_flow(difficulty, skill, tolerances::FLOW_CHANNEL_WIDTH) == FlowState::Flow {
            flow_count += 1;
        }
        skill = (skill + 0.02).min(0.95);
    }
    println!(
        "\nFlow state maintained: {flow_count}/20 rounds ({:.0}%)",
        flow_count as f64 / 20.0 * 100.0
    );

    println!("\n=== Exp004 complete ===");
}
