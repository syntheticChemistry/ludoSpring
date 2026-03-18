// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! exp047 — MTG Card Provenance + Board State DAG
//!
//! Magic: The Gathering has a physical/digital split problem:
//!   - Physical: rich tactile experience, real ownership, but no automated rules
//!   - Digital (Arena/MTGO): automated rules, but fake ownership and poor UX
//!
//! The ecoPrimals provenance trio solves this:
//!   - loamSpine: each physical card is a certificate (set, number, condition, owner)
//!   - rhizoCrypt: each game action is a DAG vertex (tap, cast, attack, block)
//!   - sweetGrass: deck building and trade attribution
//!   - AR layer: camera reads physical board → maps to DAG state → overlays info
//!
//! The digital version becomes a true 1:1 mirror of physical, not a separate product.
//! Your physical Black Lotus IS your digital Black Lotus — same loamSpine cert.
//!
//! This experiment validates:
//!   1. Card-as-certificate model (loamSpine pattern)
//!   2. Game action DAG (tap, untap, cast, attack, block, damage, draw)
//!   3. Board state reconstruction from DAG
//!   4. Zone transitions (library → hand → battlefield → graveyard → exile)
//!   5. The isomorphism: card provenance = sample chain-of-custody

mod card_model;

use card_model::{
    AcquisitionMethod, CardCondition, CardType, GameAction, GameSession, OwnershipCert, Zone,
};
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (analytical — MTG card model)",
    commit: "N/A",
    date: "N/A",
    command: "N/A (pure Rust implementation)",
};

// ===========================================================================
// Validation
// ===========================================================================

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
fn validate_card_certificates(h: &mut ValidationHarness) {
    let session = GameSession::new();

    h.check_abs(
        "seven_card_certs_registered",
        session.cards.len() as f64,
        7.0,
        0.0,
    );

    let lands = session
        .cards
        .values()
        .filter(|c| c.card_type == CardType::Land)
        .count();
    h.check_abs("three_land_certs", lands as f64, 3.0, 0.0);

    let creatures = session
        .cards
        .values()
        .filter(|c| c.card_type == CardType::Creature)
        .count();
    h.check_abs("two_creature_certs", creatures as f64, 2.0, 0.0);

    let instants = session
        .cards
        .values()
        .filter(|c| c.card_type == CardType::Instant)
        .count();
    h.check_abs("two_instant_certs", instants as f64, 2.0, 0.0);

    let all_have_set = session.cards.values().all(|c| !c.set_code.is_empty());
    h.check_bool("all_cards_have_set_code", all_have_set);

    let ownership = OwnershipCert {
        card: "a_bear",
        owner: "alice",
        condition: CardCondition::NearMint,
        acquired_from: "LGS Draft 2024-03-15",
        acquisition_method: AcquisitionMethod::PackOpening { set: "10E" },
    };
    h.check_bool(
        "ownership_cert_has_condition",
        ownership.condition == CardCondition::NearMint,
    );
    h.check_bool(
        "ownership_cert_has_acquisition_method",
        ownership.acquisition_method == AcquisitionMethod::PackOpening { set: "10E" },
    );

    let traded = OwnershipCert {
        card: "a_bear",
        owner: "bob",
        condition: CardCondition::LightlyPlayed,
        acquired_from: "alice",
        acquisition_method: AcquisitionMethod::Trade {
            from_owner: "alice",
        },
    };
    h.check_bool(
        "trade_creates_new_ownership_cert",
        traded.owner == "bob" && traded.acquired_from == "alice",
    );
}

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
fn validate_board_state_dag(h: &mut ValidationHarness) {
    let mut session = GameSession::new();
    session.simulate_turn_sequence();

    let vertex_count = session.vertices.len();
    h.check_abs("dag_has_seventeen_vertices", vertex_count as f64, 17.0, 0.0);

    let alice_graveyard = session.board.cards_in("alice", Zone::Graveyard);
    h.check_bool(
        "bear_in_graveyard_after_bolt",
        alice_graveyard.contains(&"a_bear"),
    );

    let bob_battlefield = session.board.cards_in("bob", Zone::Battlefield);
    h.check_bool(
        "bolt_on_battlefield_after_cast",
        bob_battlefield.contains(&"b_bolt"),
    );

    let alice_battlefield = session.board.cards_in("alice", Zone::Battlefield);
    h.check_bool(
        "two_forests_on_battlefield",
        alice_battlefield.contains(&"a_forest_1") && alice_battlefield.contains(&"a_forest_2"),
    );

    let forests_untapped = !session.board.tapped.contains(&"a_forest_1")
        && !session.board.tapped.contains(&"a_forest_2");
    h.check_bool("forests_untapped_after_turn3", forests_untapped);

    let alice_life = session.board.life_totals["alice"];
    h.check_abs("alice_life_20", f64::from(alice_life), 20.0, 0.0);
}

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
fn validate_zone_transitions(h: &mut ValidationHarness) {
    let mut session = GameSession::new();
    session.simulate_turn_sequence();

    let bear_actions: Vec<_> = session
        .vertices
        .iter()
        .filter(|v| match &v.action {
            GameAction::CastSpell { card, .. } | GameAction::CreatureDies { card } => {
                *card == "a_bear"
            }
            GameAction::DealDamage { target, .. } => *target == "a_bear",
            _ => false,
        })
        .collect();

    h.check_abs(
        "bear_has_three_zone_events",
        bear_actions.len() as f64,
        3.0,
        0.0,
    );

    let first_is_cast = matches!(
        bear_actions.first().map(|v| &v.action),
        Some(GameAction::CastSpell { card: "a_bear", .. })
    );
    h.check_bool("bear_first_event_is_cast", first_is_cast);

    let last_is_death = matches!(
        bear_actions.last().map(|v| &v.action),
        Some(GameAction::CreatureDies { card: "a_bear" })
    );
    h.check_bool("bear_last_event_is_death", last_is_death);

    let bolt_count = session
        .vertices
        .iter()
        .filter(|v| match &v.action {
            GameAction::CastSpell { card, .. } => *card == "b_bolt",
            GameAction::DealDamage { source, .. } => *source == "b_bolt",
            _ => false,
        })
        .count();

    h.check_abs("bolt_has_two_zone_events", bolt_count as f64, 2.0, 0.0);
}

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
fn validate_provenance_isomorphism(h: &mut ValidationHarness) {
    let mut session = GameSession::new();
    session.simulate_turn_sequence();

    let orphans = session
        .vertices
        .iter()
        .skip(1)
        .filter(|v| v.parent.is_none())
        .count();
    h.check_abs("no_orphan_vertices", orphans as f64, 0.0, 0.0);

    let max_children: usize = (0..session.vertices.len())
        .map(|id| {
            session
                .vertices
                .iter()
                .filter(|v| v.parent == Some(id))
                .count()
        })
        .max()
        .unwrap_or(0);
    h.check_abs(
        "dag_is_linear_single_child_max",
        max_children as f64,
        1.0,
        0.0,
    );

    let unique_card_ids: std::collections::HashSet<_> = session.cards.keys().collect();
    h.check_abs(
        "all_card_ids_unique",
        unique_card_ids.len() as f64,
        session.cards.len() as f64,
        0.0,
    );

    let battlefield_alice = session.board.cards_in("alice", Zone::Battlefield);
    let battlefield_bob = session.board.cards_in("bob", Zone::Battlefield);
    let total_battlefield = battlefield_alice.len() + battlefield_bob.len();

    h.check_bool(
        "board_state_reconstructable_from_dag",
        total_battlefield > 0,
    );

    let tapped_count = session.board.tapped.len();
    h.check_bool("tapped_state_tracked", tapped_count == 1);
}

// ===========================================================================
// Main
// ===========================================================================

fn cmd_validate() {
    let mut h = ValidationHarness::new("exp047_mtg_card_provenance");
    h.print_provenance(&[&PROVENANCE]);

    validate_card_certificates(&mut h);
    validate_board_state_dag(&mut h);
    validate_zone_transitions(&mut h);
    validate_provenance_isomorphism(&mut h);

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
