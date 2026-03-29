// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! exp064 — BearDog-Signed Provenance Chain
//!
//! Wires `BearDog` `crypto.sign_ed25519` and `crypto.verify_ed25519` into
//! provenance trio operations, proving cryptographic integrity end-to-end.
//!
//! Every DAG vertex, certificate operation, and braid is signed. The chain
//! can be verified by walking all items and checking signatures. Tampering
//! is detected at the exact point of modification.

mod signed_chain;

use loam_spine_core::Did;
use ludospring_barracuda::validation::{BaselineProvenance, OrExit, ValidationHarness};
use signed_chain::{
    Ed25519KeyPair, SignedProvenanceChain, hash_content_ipc, sign_vertex_ipc, verify_signature_ipc,
};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (analytical — BearDog Ed25519 + provenance trio)",
    commit: "4b683e3e",
    date: "2026-03-29",
    command: "N/A (pure Rust implementation)",
};

// ===========================================================================
// 1. Key Pair Generation and Signing
// ===========================================================================

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
fn validate_key_pair(h: &mut ValidationHarness) {
    let kp = Ed25519KeyPair::from_seed("alice_key_seed_2026");
    h.check_abs(
        "keypair_public_key_32_bytes",
        kp.public_key.len() as f64,
        32.0,
        0.0,
    );

    let message = b"hello provenance chain";
    let sig = kp.sign(message);
    h.check_abs("signature_64_bytes", sig.len() as f64, 64.0, 0.0);

    let valid = Ed25519KeyPair::verify(&kp.public_key, message, &sig);
    h.check_bool("signature_verifies_correctly", valid);

    let mut tampered_msg = message.to_vec();
    tampered_msg[0] ^= 0xFF;
    let invalid = Ed25519KeyPair::verify(&kp.public_key, &tampered_msg, &sig);
    h.check_bool("tampered_message_rejected", !invalid);

    let mut tampered_sig = sig;
    tampered_sig[0] ^= 0xFF;
    let invalid2 = Ed25519KeyPair::verify(&kp.public_key, message, &tampered_sig);
    h.check_bool("tampered_signature_rejected", !invalid2);

    let kp2 = Ed25519KeyPair::from_seed("bob_key_seed_2026");
    let wrong_key = Ed25519KeyPair::verify(&kp2.public_key, message, &sig);
    h.check_bool("wrong_key_rejected", !wrong_key);

    let kp_same = Ed25519KeyPair::from_seed("alice_key_seed_2026");
    h.check_bool("deterministic_keypair", kp.public_key == kp_same.public_key);
}

// ===========================================================================
// 2. Signed Chain Lifecycle
// ===========================================================================

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
fn validate_signed_lifecycle(h: &mut ValidationHarness) {
    let alice = Did::new("did:key:alice_signed");
    let bob = Did::new("did:key:bob_signed");

    let mut chain = SignedProvenanceChain::new(&alice, "alice_chain_seed");

    let cert_id = chain.sign_and_mint_cert(&alice, "Signed Sword", "weapon");
    h.check_bool(
        "signed_mint_cert_created",
        chain.cert_manager.get_certificate(&cert_id).is_some(),
    );
    h.check_abs(
        "signed_mint_has_signature",
        chain.signed_certs.len() as f64,
        1.0,
        0.0,
    );

    chain.sign_and_append_vertex("equip_sword", alice.as_str());
    h.check_abs(
        "signed_vertex_appended",
        chain.signed_vertices.len() as f64,
        1.0,
        0.0,
    );

    chain.sign_and_create_braid("Equipped Signed Sword", alice.as_str());
    h.check_abs(
        "signed_braid_created",
        chain.signed_braids.len() as f64,
        1.0,
        0.0,
    );

    chain.sign_and_append_vertex("kill_boss", alice.as_str());
    chain.sign_and_append_vertex("achieve_dragon_slayer", alice.as_str());
    chain.sign_and_create_braid("Dragon Slayer achievement", alice.as_str());

    chain
        .sign_and_transfer(cert_id, &alice, &bob)
        .or_exit("sign_and_transfer alice->bob failed");
    chain.sign_and_append_vertex("trade_to_bob", alice.as_str());
    chain.sign_and_create_braid("Traded sword to Bob", alice.as_str());

    h.check_abs(
        "chain_total_vertices",
        chain.signed_vertices.len() as f64,
        4.0,
        0.0,
    );
    h.check_abs(
        "chain_total_certs",
        chain.signed_certs.len() as f64,
        2.0,
        0.0,
    );
    h.check_abs(
        "chain_total_braids",
        chain.signed_braids.len() as f64,
        3.0,
        0.0,
    );

    let verification = chain.verify_chain();
    h.check_bool("chain_all_verified", verification.is_clean());
    h.check_abs(
        "chain_verified_count",
        verification.verified as f64,
        9.0,
        0.0,
    );
    h.check_abs(
        "chain_zero_tampered",
        verification.tampered.len() as f64,
        0.0,
        0.0,
    );
}

// ===========================================================================
// 3. Tamper Detection
// ===========================================================================

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
fn validate_tamper_detection(h: &mut ValidationHarness) {
    let alice = Did::new("did:key:alice_tamper");

    let mut chain = SignedProvenanceChain::new(&alice, "tamper_test_seed");
    chain.sign_and_mint_cert(&alice, "Tamper Sword", "weapon");
    chain.sign_and_append_vertex("equip", alice.as_str());
    chain.sign_and_append_vertex("kill_boss", alice.as_str());
    chain.sign_and_create_braid("Boss kill event", alice.as_str());

    let pre_tamper = chain.verify_chain();
    h.check_bool("pre_tamper_clean", pre_tamper.is_clean());

    chain.signed_vertices[1].content_hash[0] ^= 0xFF;
    let post_tamper = chain.verify_chain();
    h.check_bool("tampered_vertex_detected", !post_tamper.is_clean());
    h.check_abs(
        "tampered_exactly_one",
        post_tamper.tampered.len() as f64,
        1.0,
        0.0,
    );
    h.check_bool(
        "tampered_is_vertex",
        post_tamper.tampered[0].starts_with("vertex:"),
    );

    chain.signed_vertices[1].content_hash[0] ^= 0xFF;

    chain.signed_certs[0].content_hash[0] ^= 0xFF;
    let cert_tamper = chain.verify_chain();
    h.check_bool("tampered_cert_detected", !cert_tamper.is_clean());
    h.check_bool(
        "tampered_cert_is_cert",
        cert_tamper.tampered[0].starts_with("cert:"),
    );

    chain.signed_certs[0].content_hash[0] ^= 0xFF;

    chain.signed_braids[0].content_hash[0] ^= 0xFF;
    let braid_tamper = chain.verify_chain();
    h.check_bool("tampered_braid_detected", !braid_tamper.is_clean());
    h.check_bool(
        "tampered_braid_is_braid",
        braid_tamper.tampered[0].starts_with("braid:"),
    );
}

// ===========================================================================
// 4. Multi-Actor Chain
// ===========================================================================

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
fn validate_multi_actor(h: &mut ValidationHarness) {
    let alice = Did::new("did:key:alice_multi");
    let bob = Did::new("did:key:bob_multi");
    let carol = Did::new("did:key:carol_multi");

    let mut chain = SignedProvenanceChain::new(&alice, "multi_actor_seed");

    let cert_id = chain.sign_and_mint_cert(&alice, "Shared Shield", "armor");
    chain.sign_and_append_vertex("alice_equips", alice.as_str());
    chain.sign_and_create_braid("Alice equips shield", alice.as_str());

    chain
        .sign_and_transfer(cert_id, &alice, &bob)
        .or_exit("sign_and_transfer A->B failed");
    chain.sign_and_append_vertex("bob_uses_shield", bob.as_str());
    chain.sign_and_create_braid("Bob blocks attack", bob.as_str());

    chain
        .sign_and_transfer(cert_id, &bob, &carol)
        .or_exit("sign_and_transfer B->C failed");
    chain.sign_and_append_vertex("carol_enchants", carol.as_str());
    chain.sign_and_create_braid("Carol enchants shield", carol.as_str());

    h.check_abs(
        "multi_actor_3_transfers",
        chain.signed_certs.len() as f64,
        3.0,
        0.0,
    );

    let current_owner = chain
        .cert_manager
        .get_certificate(&cert_id)
        .map(|c| c.owner.as_str().to_string());
    h.check_bool(
        "multi_actor_carol_owns",
        current_owner.as_deref() == Some("did:key:carol_multi"),
    );

    let verification = chain.verify_chain();
    h.check_bool("multi_actor_chain_clean", verification.is_clean());
    h.check_abs(
        "multi_actor_total_items",
        verification.total_items as f64,
        9.0,
        0.0,
    );
}

// ===========================================================================
// 5. BearDog IPC Wire Format
// ===========================================================================

fn validate_beardog_ipc(h: &mut ValidationHarness) {
    let sign_req = sign_vertex_ipc("vertex_content_abc123", "alice_key_01");
    h.check_bool(
        "ipc_sign_method",
        sign_req["method"] == "crypto.sign_ed25519",
    );
    h.check_bool(
        "ipc_sign_has_params",
        sign_req["params"]["message"].is_string(),
    );
    h.check_bool(
        "ipc_sign_has_key_id",
        sign_req["params"]["key_id"] == "alice_key_01",
    );

    let verify_req = verify_signature_ipc("msg", "sig_hex", "pubkey_hex");
    h.check_bool(
        "ipc_verify_method",
        verify_req["method"] == "crypto.verify_ed25519",
    );
    h.check_bool(
        "ipc_verify_has_signature",
        verify_req["params"]["signature"].is_string(),
    );

    let hash_req = hash_content_ipc("data_to_hash");
    h.check_bool(
        "ipc_hash_method",
        hash_req["method"] == "crypto.blake3_hash",
    );
    h.check_bool(
        "ipc_hash_has_data",
        hash_req["params"]["data"] == "data_to_hash",
    );

    let roundtrip_str =
        serde_json::to_string(&sign_req).or_exit("failed to serialize sign_req for roundtrip");
    let sign_roundtrip: serde_json::Value =
        serde_json::from_str(&roundtrip_str).or_exit("failed to deserialize sign_req roundtrip");
    h.check_bool(
        "ipc_json_roundtrip",
        sign_roundtrip["method"] == "crypto.sign_ed25519",
    );
}

// ===========================================================================
// 6. Serialization Round-Trip
// ===========================================================================

fn validate_serialization(h: &mut ValidationHarness) {
    let kp = Ed25519KeyPair::from_seed("serial_test");
    let message = b"provenance data";
    let sig = kp.sign(message);

    let sig_json = serde_json::to_string(&sig.to_vec()).or_exit("failed to serialize signature");
    let sig_back: Vec<u8> = serde_json::from_str(&sig_json).or_exit("failed to deserialize signature");
    let mut sig_arr = [0u8; 64];
    sig_arr.copy_from_slice(&sig_back);

    h.check_bool(
        "serial_sig_roundtrip",
        Ed25519KeyPair::verify(&kp.public_key, message, &sig_arr),
    );

    let pk_json =
        serde_json::to_string(&kp.public_key.to_vec()).or_exit("failed to serialize public key");
    let pk_back: Vec<u8> = serde_json::from_str(&pk_json).or_exit("failed to deserialize public key");
    let mut pk_arr = [0u8; 32];
    pk_arr.copy_from_slice(&pk_back);
    h.check_bool("serial_pubkey_roundtrip", pk_arr == kp.public_key);
}

// ===========================================================================
// Main
// ===========================================================================

fn main() {
    let mut h = ValidationHarness::new("exp064_beardog_signed_chain");
    h.print_provenance(&[&PROVENANCE]);

    validate_key_pair(&mut h);
    validate_signed_lifecycle(&mut h);
    validate_tamper_detection(&mut h);
    validate_multi_actor(&mut h);
    validate_beardog_ipc(&mut h);
    validate_serialization(&mut h);

    h.finish();
}
