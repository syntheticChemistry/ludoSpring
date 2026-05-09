// SPDX-License-Identifier: AGPL-3.0-or-later
//! Scenario: Interaction Laws — Fitts, Hick, Flow.
//! Absorbed from exp005_interaction_models + exp011_dda_interaction.

use super::registry::{Scenario, ScenarioMeta, Tier, Track};
use crate::interaction::flow::flow_channel_metrics;
use crate::interaction::input_laws::{fitts_movement_time, hick_reaction_time};
use crate::tolerances::ANALYTICAL_TOL;
use crate::validation::{BaselineProvenance, ValidationHarness};

pub const SCENARIO: Scenario = Scenario {
    meta: ScenarioMeta {
        id: "interaction_laws",
        track: Track::InteractionScience,
        tier: Tier::Rust,
        provenance_crate: "exp005_interaction_models",
        provenance_date: "2026-04-11",
        description: "Validate Fitts, Hick, and Flow models against Python golden values",
    },
    run: run_interaction_laws,
};

fn run_interaction_laws(h: &mut ValidationHarness) {
    let prov = BaselineProvenance {
        script: "baselines/python/interaction_laws.py",
        commit: "231928a",
        date: "2026-04-17",
        command: "python3 baselines/python/interaction_laws.py",
    };
    h.print_provenance(&[&prov]);

    // Fitts' Law: MT = a + b × log₂(2D/W + 1) [Shannon formulation]
    let fitts_mt = fitts_movement_time(100.0, 10.0, 50.0, 150.0);
    h.check_abs(
        "Fitts MT D=100 W=10",
        fitts_mt,
        708.847_613_416_814,
        ANALYTICAL_TOL,
    );

    // Hick's Law: RT = a + b × log₂(N + 1)
    let hick_rt = hick_reaction_time(7, 200.0, 150.0);
    h.check_abs("Hick RT N=7", hick_rt, 650.0, ANALYTICAL_TOL);

    // Flow state: optimal when challenge ≈ skill (within channel)
    let (flow_intensity, in_flow) = flow_channel_metrics(0.7, 0.7, 0.3);
    h.check_bool("Flow optimal at challenge≈skill", in_flow);
    h.check_bool("Flow intensity positive", flow_intensity > 0.0);
}
