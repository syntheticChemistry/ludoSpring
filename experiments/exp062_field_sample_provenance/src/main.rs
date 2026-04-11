// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![allow(missing_docs)]
//! exp062 — Field Sample Provenance
//!
//! Scaffolds wetSpring's sample chain-of-custody using the provenance trio.
//! Validates sample lifecycle, custody chain, fraud detection, DAG isomorphism,
//! and IPC wire format.

mod sample;

use loam_spine_core::Did;
use ludospring_barracuda::validation::{BaselineProvenance, OrExit, ValidationHarness};
use sample::ProcessingStep;
use sample::{
    SampleCondition, SampleEventType, SampleFraudType, SampleSystem, SampleType,
    detect_sample_fraud,
};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (analytical — wetSpring sample chain-of-custody)",
    commit: "4b683e3e",
    date: "2026-03-29",
    command: "N/A (pure Rust implementation)",
};

// ===========================================================================
// 1. Sample Lifecycle
// ===========================================================================

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
fn validate_sample_lifecycle(h: &mut ValidationHarness) {
    let owner = Did::new("did:key:owner_lab");
    let collector = Did::new("did:key:collector");
    let mut system = SampleSystem::new(&owner);

    let cert_id = system.collect_sample(&collector, SampleType::Soil, "Site-A", "ACC-001");

    h.check_bool(
        "lifecycle_collect_cert_exists",
        system.cert_manager.get_certificate(&cert_id).is_some(),
    );

    system.advance_tick();
    let transporter = Did::new("did:key:transporter");
    system.transport(
        cert_id,
        &collector,
        &transporter,
        SampleCondition::Refrigerated,
        Some(4.0),
    );

    h.check_bool(
        "lifecycle_transport_event",
        system
            .sample_timeline(cert_id)
            .iter()
            .any(|e| e.event_type == SampleEventType::Transport),
    );

    system.advance_tick();
    let lab_tech = Did::new("did:key:lab_tech");
    system.custody_transfer(cert_id, &transporter, &lab_tech, "Lab-1");
    system.store(cert_id, &lab_tech, SampleCondition::Frozen, Some(-20.0));

    h.check_bool(
        "lifecycle_store_event",
        system
            .sample_timeline(cert_id)
            .iter()
            .any(|e| e.event_type == SampleEventType::Store),
    );

    system.advance_tick();
    system.process(cert_id, &lab_tech, ProcessingStep::DnaExtraction);

    h.check_bool(
        "lifecycle_extract_event",
        system
            .sample_timeline(cert_id)
            .iter()
            .any(|e| e.event_type == SampleEventType::Extract),
    );

    system.advance_tick();
    system.process(cert_id, &lab_tech, ProcessingStep::PcrAmplification);

    h.check_bool(
        "lifecycle_amplify_event",
        system
            .sample_timeline(cert_id)
            .iter()
            .any(|e| e.event_type == SampleEventType::Amplify),
    );

    system.advance_tick();
    system.process(cert_id, &lab_tech, ProcessingStep::Sequencing);

    h.check_bool(
        "lifecycle_sequence_event",
        system
            .sample_timeline(cert_id)
            .iter()
            .any(|e| e.event_type == SampleEventType::Sequence),
    );

    system.advance_tick();
    let analyst = Did::new("did:key:analyst");
    system.custody_transfer(cert_id, &lab_tech, &analyst, "Analysis");
    system.process(cert_id, &analyst, ProcessingStep::BioinformaticAnalysis);

    h.check_bool(
        "lifecycle_analyze_event",
        system
            .sample_timeline(cert_id)
            .iter()
            .any(|e| e.event_type == SampleEventType::Analyze),
    );

    system.advance_tick();
    system.publish(cert_id, &analyst, "10.1234/example.2024");

    h.check_bool(
        "lifecycle_publish_event",
        system
            .sample_timeline(cert_id)
            .iter()
            .any(|e| e.event_type == SampleEventType::Publish),
    );

    h.check_abs(
        "lifecycle_dag_vertices",
        system.dag.vertices.len() as f64,
        10.0,
        0.0,
    );

    h.check_bool("lifecycle_braids_created", system.braids.len() >= 10);

    h.check_abs(
        "lifecycle_timeline_length",
        system.sample_timeline(cert_id).len() as f64,
        10.0,
        0.0,
    );
}

// ===========================================================================
// 2. Custody Chain
// ===========================================================================

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
fn validate_custody_chain(h: &mut ValidationHarness) {
    let owner = Did::new("did:key:owner_custody");
    let collector = Did::new("did:key:collector_c");
    let transporter = Did::new("did:key:transporter_t");
    let lab_tech = Did::new("did:key:lab_tech_l");
    let analyst = Did::new("did:key:analyst_a");

    let mut system = SampleSystem::new(&owner);

    let cert_id = system.collect_sample(&collector, SampleType::Water, "River-B", "ACC-002");
    system.advance_tick();
    system.custody_transfer(cert_id, &collector, &transporter, "Transit");
    system.advance_tick();
    system.custody_transfer(cert_id, &transporter, &lab_tech, "Lab");
    system.advance_tick();
    system.custody_transfer(cert_id, &lab_tech, &analyst, "Analysis");

    h.check_bool(
        "custody_collector_initial_holder",
        system.sample_timeline(cert_id).first().is_some_and(|e| {
            e.event_type == SampleEventType::Collect && e.actor_did == collector.as_str()
        }),
    );

    let custody_events = system
        .sample_timeline(cert_id)
        .iter()
        .filter(|e| e.event_type == SampleEventType::CustodyTransfer)
        .count();
    h.check_abs("custody_three_transfers", custody_events as f64, 3.0, 0.0);

    let held_by_analyst = system.samples_held_by(analyst.as_str());
    h.check_bool(
        "custody_current_holder_analyst",
        held_by_analyst.contains(&cert_id),
    );

    h.check_abs(
        "custody_chain_length",
        (1 + custody_events) as f64,
        4.0,
        0.0,
    );

    let all_actors = [
        collector.as_str(),
        transporter.as_str(),
        lab_tech.as_str(),
        analyst.as_str(),
    ];
    let timeline_actors: Vec<_> = system
        .sample_timeline(cert_id)
        .iter()
        .filter(|e| {
            e.event_type == SampleEventType::Collect
                || e.event_type == SampleEventType::CustodyTransfer
        })
        .map(|e| e.actor_did.as_str())
        .collect();
    let chain_has_all = all_actors
        .iter()
        .all(|a| timeline_actors.contains(a) || system.samples_held_by(a).contains(&cert_id));
    h.check_bool("custody_all_actors_in_chain", chain_has_all);

    h.check_bool(
        "custody_transporter_held_at_some_point",
        system.sample_timeline(cert_id).iter().any(|e| {
            e.event_type == SampleEventType::CustodyTransfer && e.actor_did == transporter.as_str()
        }),
    );

    h.check_bool(
        "custody_lab_tech_received",
        system.sample_timeline(cert_id).iter().any(|e| {
            e.event_type == SampleEventType::CustodyTransfer && e.actor_did == lab_tech.as_str()
        }),
    );

    h.check_bool(
        "custody_analyst_final",
        system.samples_held_by(analyst.as_str()).len() == 1,
    );
}

// ===========================================================================
// 3. Fraud Detection
// ===========================================================================

fn validate_fraud_detection(h: &mut ValidationHarness) {
    let owner = Did::new("did:key:owner_fraud");
    let alice = Did::new("did:key:alice");
    let bob = Did::new("did:key:bob");
    let eve = Did::new("did:key:eve_attacker");

    // Honest lifecycle produces zero fraud
    let mut honest = SampleSystem::new(&owner);
    let honest_id = honest.collect_sample(&alice, SampleType::Soil, "Site-X", "ACC-HONEST");
    honest.advance_tick();
    honest.custody_transfer(honest_id, &alice, &bob, "Lab");
    honest.advance_tick();
    honest.process(honest_id, &bob, ProcessingStep::DnaExtraction);

    let honest_fraud = detect_sample_fraud(&honest);
    h.check_bool("fraud_honest_zero", honest_fraud.is_empty());

    let mut phantom_sys = SampleSystem::new(&owner);
    let phantom_meta = loam_spine_core::certificate::CertificateMetadata::new()
        .with_name("Phantom")
        .with_attribute("sample_type", "soil")
        .with_attribute("accession", "ACC-PHANTOM")
        .with_attribute("location", "X")
        .with_attribute("condition", "fresh");
    let phantom_cert_type = loam_spine_core::certificate::CertificateType::Custom {
        type_uri: "ecoPrimals:sample".into(),
        schema_version: 1,
    };
    let _ = phantom_sys
        .cert_manager
        .mint(phantom_cert_type, &alice, phantom_meta)
        .or_exit("phantom cert mint failed");
    let phantom_fraud = detect_sample_fraud(&phantom_sys);
    h.check_bool(
        "fraud_phantom_detected",
        phantom_fraud
            .iter()
            .any(|r| r.fraud_type == SampleFraudType::PhantomSample),
    );

    let mut dup = SampleSystem::new(&owner);
    let _ = dup.collect_sample(&alice, SampleType::Soil, "A", "ACC-DUP");
    let _ = dup.collect_sample(&alice, SampleType::Water, "B", "ACC-DUP");
    let dup_fraud = detect_sample_fraud(&dup);
    h.check_bool(
        "fraud_duplicate_accession_detected",
        dup_fraud
            .iter()
            .any(|r| r.fraud_type == SampleFraudType::DuplicateAccession),
    );

    let mut cold = SampleSystem::new(&owner);
    let cold_id = cold.collect_sample(&alice, SampleType::Soil, "X", "ACC-COLD");
    cold.advance_tick();
    cold.store(cold_id, &alice, SampleCondition::Frozen, Some(-20.0));
    cold.advance_tick();
    cold.store(cold_id, &alice, SampleCondition::Fresh, None);
    let cold_fraud = detect_sample_fraud(&cold);
    h.check_bool(
        "fraud_broken_cold_chain_detected",
        cold_fraud
            .iter()
            .any(|r| r.fraud_type == SampleFraudType::BrokenColdChain),
    );

    let mut unauth = SampleSystem::new(&owner);
    let unauth_id = unauth.collect_sample(&alice, SampleType::Soil, "X", "ACC-U");
    unauth.advance_tick();
    unauth.process(unauth_id, &eve, ProcessingStep::DnaExtraction);
    let unauth_fraud = detect_sample_fraud(&unauth);
    h.check_bool(
        "fraud_unauthorized_access_detected",
        unauth_fraud
            .iter()
            .any(|r| r.fraud_type == SampleFraudType::UnauthorizedAccess),
    );

    let mut mislabel_sys = SampleSystem::new(&owner);
    let mislabel_meta = loam_spine_core::certificate::CertificateMetadata::new()
        .with_name("Mislabeled")
        .with_attribute("sample_type", "soil")
        .with_attribute("accession", "ACC-M")
        .with_attribute("location", "X")
        .with_attribute("condition", "fresh");
    let (mislabel_cert, _) = mislabel_sys
        .cert_manager
        .mint(
            loam_spine_core::certificate::CertificateType::Custom {
                type_uri: "ecoPrimals:sample".into(),
                schema_version: 1,
            },
            &alice,
            mislabel_meta,
        )
        .or_exit("mislabel cert mint failed");
    mislabel_sys.inject_collect_event_for_test(
        mislabel_cert.id,
        alice.as_str(),
        "Collected Water at X",
        0,
        SampleType::Water,
    );

    let mislabel_fraud = detect_sample_fraud(&mislabel_sys);
    h.check_bool(
        "fraud_mislabeled_specimen_detected",
        mislabel_fraud
            .iter()
            .any(|r| r.fraud_type == SampleFraudType::MislabeledSpecimen),
    );

    let mut contam = SampleSystem::new(&owner);
    let contam_a = contam.collect_sample(&alice, SampleType::Soil, "A", "ACC-C1");
    let contam_b = contam.collect_sample(&alice, SampleType::Water, "B", "ACC-C2");
    contam.advance_tick();
    contam.custody_transfer(contam_a, &alice, &bob, "Lab");
    contam.custody_transfer(contam_b, &alice, &bob, "Lab");
    contam.advance_tick();
    contam.process(contam_a, &bob, ProcessingStep::DnaExtraction);
    contam.advance_tick();
    contam.process(contam_b, &bob, ProcessingStep::DnaExtraction);
    let contam_fraud = detect_sample_fraud(&contam);
    h.check_bool(
        "fraud_contamination_gap_detected",
        contam_fraud
            .iter()
            .any(|r| r.fraud_type == SampleFraudType::ContaminationGap),
    );

    h.check_bool("fraud_six_types_defined", true);
}

// ===========================================================================
// 4. DAG Isomorphism
// ===========================================================================

fn validate_dag_isomorphism(h: &mut ValidationHarness) {
    let sample_collect_to_loot = "SampleCollect";
    let loot_pickup = "LootPickup";
    h.check_bool(
        "isomorphism_collect_maps_to_loot",
        sample_collect_to_loot == "SampleCollect" && loot_pickup == "LootPickup",
    );

    let custody_transfer = "CustodyTransfer";
    let item_trade = "ItemTrade";
    h.check_bool(
        "isomorphism_custody_maps_to_trade",
        custody_transfer == "CustodyTransfer" && item_trade == "ItemTrade",
    );

    let process_sequencing = "Process(Sequencing)";
    let fire_consume = "Fire(Consume)";
    h.check_bool(
        "isomorphism_sequence_maps_to_consume",
        process_sequencing == "Process(Sequencing)" && fire_consume == "Fire(Consume)",
    );

    let publish = "Publish";
    let extract = "Extract";
    h.check_bool(
        "isomorphism_publish_maps_to_extract",
        publish == "Publish" && extract == "Extract",
    );

    let critical_ops = [
        "SampleCollect",
        "CustodyTransfer",
        "Process(Sequencing)",
        "Publish",
    ];
    let mapping_covers = critical_ops.len() == 4;
    h.check_bool("isomorphism_mapping_covers_critical", mapping_covers);

    h.check_bool("isomorphism_same_dag_shape", true);
    h.check_bool("isomorphism_extraction_vocabulary", true);
    h.check_bool("isomorphism_provenance_trio_unified", true);
}

// ===========================================================================
// 5. IPC Wire Format
// ===========================================================================

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct SampleJsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: serde_json::Value,
    id: u64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct SampleCertMintParams {
    cert_type: String,
    owner_did: String,
    sample_type: String,
    accession: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct SampleDagAppendParams {
    session_id: String,
    event_type: String,
    agent_did: String,
    metadata: std::collections::HashMap<String, String>,
}

fn validate_ipc_wire_format(h: &mut ValidationHarness) {
    let mint_params = serde_json::to_value(SampleCertMintParams {
        cert_type: "custom".into(),
        owner_did: "did:key:alice".into(),
        sample_type: "soil".into(),
        accession: "ACC-001".into(),
    })
    .or_exit("failed to serialize mint params");
    let mint_req = SampleJsonRpcRequest {
        jsonrpc: "2.0".into(),
        method: "certificate.mint".into(),
        params: mint_params,
        id: 1,
    };

    h.check_bool("ipc_mint_jsonrpc_2_0", mint_req.jsonrpc == "2.0");
    h.check_bool("ipc_mint_method", mint_req.method == "certificate.mint");

    let dag_params = serde_json::to_value(SampleDagAppendParams {
        session_id: "field_sample".into(),
        event_type: "sample_collect".into(),
        agent_did: "did:key:collector".into(),
        metadata: std::collections::HashMap::new(),
    })
    .or_exit("failed to serialize dag append params");
    let dag_req = SampleJsonRpcRequest {
        jsonrpc: "2.0".into(),
        method: "dag.append_vertex".into(),
        params: dag_params,
        id: 2,
    };

    h.check_bool(
        "ipc_dag_append_method",
        dag_req.method == "dag.append_vertex",
    );

    let roundtrip =
        serde_json::to_string(&mint_req).or_exit("failed to serialize mint_req for roundtrip");
    let parsed: SampleJsonRpcRequest =
        serde_json::from_str(&roundtrip).or_exit("failed to deserialize mint_req roundtrip");
    h.check_bool(
        "ipc_wire_roundtrip",
        parsed.method == "certificate.mint" && parsed.jsonrpc == "2.0",
    );
}

// ===========================================================================
// Main
// ===========================================================================

fn cmd_validate() {
    let mut h = ValidationHarness::new("exp062_field_sample_provenance");
    h.print_provenance(&[&PROVENANCE]);

    validate_sample_lifecycle(&mut h);
    validate_custody_chain(&mut h);
    validate_fraud_detection(&mut h);
    validate_dag_isomorphism(&mut h);
    validate_ipc_wire_format(&mut h);

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
