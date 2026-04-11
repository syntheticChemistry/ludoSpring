// SPDX-License-Identifier: AGPL-3.0-or-later
//! Tufte validation dashboard — what does good game UI look like?

use std::fs;
use std::path::Path;

use ludospring_barracuda::metrics::engagement::{EngagementSnapshot, compute_engagement};
use ludospring_barracuda::metrics::tufte_gaming::{UiElement, analyze_game_ui};
use ludospring_barracuda::visualization::PetalTonguePushClient;

use super::hud_fixtures::{
    fps_hud_elements, puzzle_hud_elements, rpg_hud_elements, rts_hud_elements, sandbox_hud_elements,
};

use serde_json::{Value, json};

type Scenario = (&'static str, Value);

pub fn cmd_tufte_dashboard() -> Result<(), String> {
    eprintln!("╔═══════════════════════════════════════════════════════════╗");
    eprintln!("║  ludoSpring Tufte Validation Dashboard                   ║");
    eprintln!("║  What does good game UI look like? (Tufte 1983)          ║");
    eprintln!("╚═══════════════════════════════════════════════════════════╝");
    eprintln!();

    eprintln!("Building Tufte analysis scenarios...");
    let scenarios: Vec<Scenario> = vec![
        ("genre_comparison", build_genre_comparison()),
        ("minimap_multiples", build_minimap_small_multiples()),
        ("cognitive_load_sweep", build_cognitive_load_sweep()),
    ];
    eprintln!("  {} scenarios built.\n", scenarios.len());

    let base = std::env::var("LUDOSPRING_OUTPUT_DIR").unwrap_or_else(|_| "sandbox".into());
    let out_dir = Path::new(&base).join("tufte");
    fs::create_dir_all(&out_dir)
        .map_err(|e| format!("cannot create {}: {e}", out_dir.display()))?;

    let mut written = 0u32;
    for (name, payload) in &scenarios {
        let path = out_dir.join(format!("{name}.json"));
        match serde_json::to_string_pretty(payload) {
            Ok(json_str) => match fs::write(&path, &json_str) {
                Ok(()) => written += 1,
                Err(e) => eprintln!("  ERROR: write {}: {e}", path.display()),
            },
            Err(e) => eprintln!("  ERROR: serialize {name}: {e}"),
        }
    }
    eprintln!(
        "Wrote {written}/{} Tufte analysis files to {}",
        scenarios.len(),
        out_dir.display()
    );

    if let Ok(client) = PetalTonguePushClient::discover() {
        eprintln!("\npetalTongue discovered — pushing Tufte analysis via IPC...");
        let mut pushed = 0u32;
        for (name, payload) in &scenarios {
            if let Err(e) = client.push_render(name, name, payload) {
                eprintln!("  push {name}: {e}");
            } else {
                pushed += 1;
            }
        }
        eprintln!("Pushed {pushed}/{} scenarios.", scenarios.len());
    } else {
        eprintln!("\npetalTongue not running — Tufte analyses saved as JSON.");
        eprintln!("To view: petaltongue ui --scenario {}/", out_dir.display());
    }

    eprintln!("\nDone. Tufte validation complete.");
    Ok(())
}

// ── Analysis 1: Side-by-side Genre Comparison ────────────────────────

fn build_genre_comparison() -> Value {
    let genres: &[(&str, Vec<UiElement>)] = &[
        ("fps", fps_hud_elements()),
        ("rts", rts_hud_elements()),
        ("rpg", rpg_hud_elements()),
        ("puzzle", puzzle_hud_elements()),
        ("sandbox", sandbox_hud_elements()),
    ];

    let mut genre_names = Vec::new();
    let mut data_ink_ratios = Vec::new();
    let mut info_densities = Vec::new();
    let mut screen_coverages = Vec::new();
    let mut per_genre = Vec::new();

    for (name, elements) in genres {
        let report = analyze_game_ui(elements);
        genre_names.push(*name);
        data_ink_ratios.push(report.data_ink_ratio);
        info_densities.push(report.info_density);
        screen_coverages.push(report.screen_coverage);

        let element_details: Vec<Value> = report
            .elements
            .iter()
            .map(|ea| {
                json!({
                    "name": ea.name,
                    "data_ink_ratio": ea.data_ink_ratio,
                    "simplifiable": ea.simplifiable,
                    "recommendations": ea.recommendations,
                })
            })
            .collect();

        per_genre.push(json!({
            "genre": name,
            "data_ink_ratio": report.data_ink_ratio,
            "info_density": report.info_density,
            "screen_coverage": report.screen_coverage,
            "notes": report.notes,
            "elements": element_details,
        }));
    }

    json!({
        "channel": "UiAnalysis",
        "id": "genre-comparison-5way",
        "label": "Tufte Genre Comparison: 5 Genres Side-by-Side (Tufte 1983)",
        "genres": genre_names,
        "data_ink_ratios": data_ink_ratios,
        "info_densities": info_densities,
        "screen_coverages": screen_coverages,
        "per_genre": per_genre,
    })
}

// ── Analysis 2: Minimap Small Multiples ──────────────────────────────

fn build_minimap_small_multiples() -> Value {
    let minimaps: &[(&str, UiElement, &str)] = &[
        (
            "doom_automap",
            UiElement {
                name: "doom automap".into(),
                bounds: [0.0, 0.0, 1.0, 1.0],
                data_values: 200,
                pixel_area: 64000.0,
                data_ink_area: 50000.0,
                critical: false,
            },
            "Full-screen overlay, line-only — near-pure data ink.",
        ),
        (
            "rts_minimap",
            UiElement {
                name: "rts minimap".into(),
                bounds: [0.78, 0.02, 0.2, 0.2],
                data_values: 150,
                pixel_area: 5000.0,
                data_ink_area: 3500.0,
                critical: true,
            },
            "Persistent corner panel — color-coded terrain + unit dots.",
        ),
        (
            "rpg_world_map",
            UiElement {
                name: "rpg world map".into(),
                bounds: [0.1, 0.1, 0.8, 0.8],
                data_values: 80,
                pixel_area: 60000.0,
                data_ink_area: 20000.0,
                critical: false,
            },
            "Decorative parchment overlay — heavy border chrome.",
        ),
        (
            "souls_minimap",
            UiElement {
                name: "souls no-minimap".into(),
                bounds: [0.0, 0.0, 0.0, 0.0],
                data_values: 0,
                pixel_area: 0.0,
                data_ink_area: 0.0,
                critical: false,
            },
            "Deliberate absence — navigation through spatial memory.",
        ),
    ];

    let mut entries = Vec::new();
    for (id, element, description) in minimaps {
        let report = analyze_game_ui(std::slice::from_ref(element));
        entries.push(json!({
            "id": id,
            "description": description,
            "data_ink_ratio": report.data_ink_ratio,
            "info_density": report.info_density,
            "screen_coverage": report.screen_coverage,
            "data_values": element.data_values,
            "notes": report.notes,
        }));
    }

    json!({
        "channel": "UiAnalysis",
        "id": "minimap-small-multiples",
        "label": "Minimap Design: Small Multiples Analysis (Tufte 1983)",
        "minimaps": entries,
        "tufte_principle": "Small multiples: same design structure, different data.",
    })
}

// ── Analysis 3: Cognitive Load Sweep ─────────────────────────────────

/// Sweep HUD complexity from minimal (2 elements) to maximal (20 elements)
/// and measure how engagement responds to information overload.
fn build_cognitive_load_sweep() -> Value {
    let mut sweep = Vec::new();

    for n_elements in (2..=20).step_by(2) {
        let elements = generate_scaled_hud(n_elements);
        let report = analyze_game_ui(&elements);

        let session = EngagementSnapshot {
            session_duration_s: 1800.0,
            action_count: engagement_from_complexity(n_elements),
            exploration_breadth: exploration_from_complexity(n_elements),
            challenge_seeking: 10,
            retry_count: retries_from_complexity(n_elements),
            deliberate_pauses: pauses_from_complexity(n_elements),
        };
        let engagement = compute_engagement(&session);

        sweep.push(json!({
            "hud_elements": n_elements,
            "data_ink_ratio": report.data_ink_ratio,
            "info_density": report.info_density,
            "screen_coverage": report.screen_coverage,
            "engagement": engagement.composite,
            "actions_per_minute": engagement.actions_per_minute,
            "deliberation": engagement.deliberation,
            "notes": report.notes,
        }));
    }

    json!({
        "channel": "UiAnalysis",
        "id": "cognitive-load-sweep",
        "label": "Cognitive Load Sweep: HUD Complexity vs Engagement",
        "sweep": sweep,
        "hypothesis": "Engagement peaks at moderate complexity; excessive HUD elements cause cognitive overload.",
    })
}

/// Scale a HUD by adding elements with progressively worse data-ink ratios.
fn generate_scaled_hud(n: u32) -> Vec<UiElement> {
    let base = fps_hud_elements();
    let mut elements: Vec<UiElement> = base.into_iter().take(n.min(4) as usize).collect();

    let extra_pool = [
        ("kill_feed", [0.75, 0.15, 0.25, 0.2], 5, 800.0, 400.0),
        ("compass", [0.35, 0.0, 0.3, 0.03], 8, 400.0, 300.0),
        ("teammate_hp", [0.0, 0.3, 0.1, 0.15], 3, 500.0, 200.0),
        ("weapon_wheel", [0.35, 0.35, 0.3, 0.3], 8, 3000.0, 900.0),
        ("damage_indicator", [0.4, 0.4, 0.2, 0.2], 4, 1500.0, 300.0),
        ("objectives", [0.75, 0.4, 0.25, 0.1], 3, 600.0, 200.0),
        ("score_ticker", [0.3, 0.0, 0.4, 0.02], 2, 200.0, 100.0),
        (
            "ability_cooldowns",
            [0.1, 0.85, 0.15, 0.08],
            4,
            500.0,
            350.0,
        ),
        ("stance_indicator", [0.0, 0.75, 0.05, 0.05], 1, 100.0, 30.0),
        ("chat_box", [0.0, 0.5, 0.2, 0.15], 3, 900.0, 400.0),
        ("scoreboard", [0.1, 0.1, 0.8, 0.8], 50, 20000.0, 8000.0),
        ("spectator_bar", [0.0, 0.0, 1.0, 0.03], 4, 800.0, 200.0),
        ("loadout_display", [0.85, 0.7, 0.15, 0.25], 6, 1200.0, 400.0),
        ("ping_indicator", [0.95, 0.0, 0.05, 0.02], 1, 50.0, 40.0),
        ("xp_bar", [0.1, 0.98, 0.8, 0.02], 1, 400.0, 300.0),
        ("battle_pass", [0.0, 0.0, 0.15, 0.05], 2, 300.0, 80.0),
    ];

    let extra_count = (n.saturating_sub(4) as usize).min(extra_pool.len());
    for &(name, bounds, dv, pa, dia) in &extra_pool[..extra_count] {
        elements.push(UiElement {
            name: name.into(),
            bounds,
            data_values: dv,
            pixel_area: pa,
            data_ink_area: dia,
            critical: false,
        });
    }

    elements
}

/// Model: actions decrease once cognitive overload hits (~12 elements).
fn engagement_from_complexity(n: u32) -> u64 {
    let peak = 12;
    let base = 500u64;
    if n <= peak {
        base + u64::from(n) * 80
    } else {
        let over = n - peak;
        (base + u64::from(peak) * 80).saturating_sub(u64::from(over) * 60)
    }
}

/// Model: exploration drops with HUD clutter.
const fn exploration_from_complexity(n: u32) -> u32 {
    20u32.saturating_sub(n / 2)
}

/// Model: retries increase with confusing UIs.
const fn retries_from_complexity(n: u32) -> u32 {
    n * 2
}

/// Model: deliberate pauses increase as players parse more UI.
const fn pauses_from_complexity(n: u32) -> u32 {
    n * 3
}
