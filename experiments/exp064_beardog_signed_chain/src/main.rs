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
use ludospring_barracuda::validation::ValidationResult;
use signed_chain::{
    Ed25519KeyPair, SignedProvenanceChain, hash_content_ipc, sign_vertex_ipc, verify_signature_ipc,
};

const EXP: &str = "exp064_beardog_signed_chain";

const fn bool_f64(b: bool) -> f64 {
    if b { 1.0 } else { 0.0 }
}

// ===========================================================================
// 1. Key Pair Generation and Signing
// ===========================================================================

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
fn validate_key_pair() -> Vec<ValidationResult> {
    let mut results = Vec::new();

    let kp = Ed25519KeyPair::from_seed("alice_key_seed_2026");
    results.push(ValidationResult::check(
        EXP,
        "keypair_public_key_32_bytes",
        kp.public_key.len() as f64,
        32.0,
        0.0,
    ));

    let message = b"hello provenance chain";
    let sig = kp.sign(message);
    results.push(ValidationResult::check(
        EXP,
        "signature_64_bytes",
        sig.len() as f64,
        64.0,
        0.0,
    ));

    let valid = Ed25519KeyPair::verify(&kp.public_key, message, &sig);
    results.push(ValidationResult::check(
        EXP,
        "signature_verifies_correctly",
        bool_f64(valid),
        1.0,
        0.0,
    ));

    let mut tampered_msg = message.to_vec();
    tampered_msg[0] ^= 0xFF;
    let invalid = Ed25519KeyPair::verify(&kp.public_key, &tampered_msg, &sig);
    results.push(ValidationResult::check(
        EXP,
        "tampered_message_rejected",
        bool_f64(!invalid),
        1.0,
        0.0,
    ));

    let mut tampered_sig = sig;
    tampered_sig[0] ^= 0xFF;
    let invalid2 = Ed25519KeyPair::verify(&kp.public_key, message, &tampered_sig);
    results.push(ValidationResult::check(
        EXP,
        "tampered_signature_rejected",
        bool_f64(!invalid2),
        1.0,
        0.0,
    ));

    let kp2 = Ed25519KeyPair::from_seed("bob_key_seed_2026");
    let wrong_key = Ed25519KeyPair::verify(&kp2.public_key, message, &sig);
    results.push(ValidationResult::check(
        EXP,
        "wrong_key_rejected",
        bool_f64(!wrong_key),
        1.0,
        0.0,
    ));

    let kp_same = Ed25519KeyPair::from_seed("alice_key_seed_2026");
    results.push(ValidationResult::check(
        EXP,
        "deterministic_keypair",
        bool_f64(kp.public_key == kp_same.public_key),
        1.0,
        0.0,
    ));

    results
}

// ===========================================================================
// 2. Signed Chain Lifecycle
// ===========================================================================

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
fn validate_signed_lifecycle() -> Vec<ValidationResult> {
    let mut results = Vec::new();
    let alice = Did::new("did:key:alice_signed");
    let bob = Did::new("did:key:bob_signed");

    let mut chain = SignedProvenanceChain::new(&alice, "alice_chain_seed");

    let cert_id = chain.sign_and_mint_cert(&alice, "Signed Sword", "weapon");
    results.push(ValidationResult::check(
        EXP,
        "signed_mint_cert_created",
        bool_f64(chain.cert_manager.get_certificate(&cert_id).is_some()),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "signed_mint_has_signature",
        chain.signed_certs.len() as f64,
        1.0,
        0.0,
    ));

    chain.sign_and_append_vertex("equip_sword", alice.as_str());
    results.push(ValidationResult::check(
        EXP,
        "signed_vertex_appended",
        chain.signed_vertices.len() as f64,
        1.0,
        0.0,
    ));

    chain.sign_and_create_braid("Equipped Signed Sword", alice.as_str());
    results.push(ValidationResult::check(
        EXP,
        "signed_braid_created",
        chain.signed_braids.len() as f64,
        1.0,
        0.0,
    ));

    chain.sign_and_append_vertex("kill_boss", alice.as_str());
    chain.sign_and_append_vertex("achieve_dragon_slayer", alice.as_str());
    chain.sign_and_create_braid("Dragon Slayer achievement", alice.as_str());

    let Ok(()) = chain.sign_and_transfer(cert_id, &alice, &bob) else {
        eprintln!("FATAL: sign_and_transfer alice->bob failed");
        std::process::exit(1);
    };
    chain.sign_and_append_vertex("trade_to_bob", alice.as_str());
    chain.sign_and_create_braid("Traded sword to Bob", alice.as_str());

    results.push(ValidationResult::check(
        EXP,
        "chain_total_vertices",
        chain.signed_vertices.len() as f64,
        4.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "chain_total_certs",
        chain.signed_certs.len() as f64,
        2.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "chain_total_braids",
        chain.signed_braids.len() as f64,
        3.0,
        0.0,
    ));

    let verification = chain.verify_chain();
    results.push(ValidationResult::check(
        EXP,
        "chain_all_verified",
        bool_f64(verification.is_clean()),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "chain_verified_count",
        verification.verified as f64,
        9.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "chain_zero_tampered",
        verification.tampered.len() as f64,
        0.0,
        0.0,
    ));

    results
}

// ===========================================================================
// 3. Tamper Detection
// ===========================================================================

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
fn validate_tamper_detection() -> Vec<ValidationResult> {
    let mut results = Vec::new();
    let alice = Did::new("did:key:alice_tamper");

    let mut chain = SignedProvenanceChain::new(&alice, "tamper_test_seed");
    chain.sign_and_mint_cert(&alice, "Tamper Sword", "weapon");
    chain.sign_and_append_vertex("equip", alice.as_str());
    chain.sign_and_append_vertex("kill_boss", alice.as_str());
    chain.sign_and_create_braid("Boss kill event", alice.as_str());

    let pre_tamper = chain.verify_chain();
    results.push(ValidationResult::check(
        EXP,
        "pre_tamper_clean",
        bool_f64(pre_tamper.is_clean()),
        1.0,
        0.0,
    ));

    chain.signed_vertices[1].content_hash[0] ^= 0xFF;
    let post_tamper = chain.verify_chain();
    results.push(ValidationResult::check(
        EXP,
        "tampered_vertex_detected",
        bool_f64(!post_tamper.is_clean()),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "tampered_exactly_one",
        post_tamper.tampered.len() as f64,
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "tampered_is_vertex",
        bool_f64(post_tamper.tampered[0].starts_with("vertex:")),
        1.0,
        0.0,
    ));

    chain.signed_vertices[1].content_hash[0] ^= 0xFF;

    chain.signed_certs[0].content_hash[0] ^= 0xFF;
    let cert_tamper = chain.verify_chain();
    results.push(ValidationResult::check(
        EXP,
        "tampered_cert_detected",
        bool_f64(!cert_tamper.is_clean()),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "tampered_cert_is_cert",
        bool_f64(cert_tamper.tampered[0].starts_with("cert:")),
        1.0,
        0.0,
    ));

    chain.signed_certs[0].content_hash[0] ^= 0xFF;

    chain.signed_braids[0].content_hash[0] ^= 0xFF;
    let braid_tamper = chain.verify_chain();
    results.push(ValidationResult::check(
        EXP,
        "tampered_braid_detected",
        bool_f64(!braid_tamper.is_clean()),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "tampered_braid_is_braid",
        bool_f64(braid_tamper.tampered[0].starts_with("braid:")),
        1.0,
        0.0,
    ));

    results
}

// ===========================================================================
// 4. Multi-Actor Chain
// ===========================================================================

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
fn validate_multi_actor() -> Vec<ValidationResult> {
    let mut results = Vec::new();
    let alice = Did::new("did:key:alice_multi");
    let bob = Did::new("did:key:bob_multi");
    let carol = Did::new("did:key:carol_multi");

    let mut chain = SignedProvenanceChain::new(&alice, "multi_actor_seed");

    let cert_id = chain.sign_and_mint_cert(&alice, "Shared Shield", "armor");
    chain.sign_and_append_vertex("alice_equips", alice.as_str());
    chain.sign_and_create_braid("Alice equips shield", alice.as_str());

    let Ok(()) = chain.sign_and_transfer(cert_id, &alice, &bob) else {
        eprintln!("FATAL: sign_and_transfer A->B failed");
        std::process::exit(1);
    };
    chain.sign_and_append_vertex("bob_uses_shield", bob.as_str());
    chain.sign_and_create_braid("Bob blocks attack", bob.as_str());

    let Ok(()) = chain.sign_and_transfer(cert_id, &bob, &carol) else {
        eprintln!("FATAL: sign_and_transfer B->C failed");
        std::process::exit(1);
    };
    chain.sign_and_append_vertex("carol_enchants", carol.as_str());
    chain.sign_and_create_braid("Carol enchants shield", carol.as_str());

    results.push(ValidationResult::check(
        EXP,
        "multi_actor_3_transfers",
        chain.signed_certs.len() as f64,
        3.0,
        0.0,
    ));

    let current_owner = chain
        .cert_manager
        .get_certificate(&cert_id)
        .map(|c| c.owner.as_str().to_string());
    results.push(ValidationResult::check(
        EXP,
        "multi_actor_carol_owns",
        bool_f64(current_owner.as_deref() == Some("did:key:carol_multi")),
        1.0,
        0.0,
    ));

    let verification = chain.verify_chain();
    results.push(ValidationResult::check(
        EXP,
        "multi_actor_chain_clean",
        bool_f64(verification.is_clean()),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "multi_actor_total_items",
        verification.total_items as f64,
        9.0,
        0.0,
    ));

    results
}

// ===========================================================================
// 5. BearDog IPC Wire Format
// ===========================================================================

fn validate_beardog_ipc() -> Vec<ValidationResult> {
    let mut results = Vec::new();

    let sign_req = sign_vertex_ipc("vertex_content_abc123", "alice_key_01");
    results.push(ValidationResult::check(
        EXP,
        "ipc_sign_method",
        bool_f64(sign_req["method"] == "crypto.sign_ed25519"),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "ipc_sign_has_params",
        bool_f64(sign_req["params"]["message"].is_string()),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "ipc_sign_has_key_id",
        bool_f64(sign_req["params"]["key_id"] == "alice_key_01"),
        1.0,
        0.0,
    ));

    let verify_req = verify_signature_ipc("msg", "sig_hex", "pubkey_hex");
    results.push(ValidationResult::check(
        EXP,
        "ipc_verify_method",
        bool_f64(verify_req["method"] == "crypto.verify_ed25519"),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "ipc_verify_has_signature",
        bool_f64(verify_req["params"]["signature"].is_string()),
        1.0,
        0.0,
    ));

    let hash_req = hash_content_ipc("data_to_hash");
    results.push(ValidationResult::check(
        EXP,
        "ipc_hash_method",
        bool_f64(hash_req["method"] == "crypto.blake3_hash"),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "ipc_hash_has_data",
        bool_f64(hash_req["params"]["data"] == "data_to_hash"),
        1.0,
        0.0,
    ));

    let Ok(roundtrip_str) = serde_json::to_string(&sign_req) else {
        eprintln!("FATAL: failed to serialize sign_req for roundtrip");
        std::process::exit(1);
    };
    let Ok(sign_roundtrip): Result<serde_json::Value, _> = serde_json::from_str(&roundtrip_str)
    else {
        eprintln!("FATAL: failed to deserialize sign_req roundtrip");
        std::process::exit(1);
    };
    results.push(ValidationResult::check(
        EXP,
        "ipc_json_roundtrip",
        bool_f64(sign_roundtrip["method"] == "crypto.sign_ed25519"),
        1.0,
        0.0,
    ));

    results
}

// ===========================================================================
// 6. Serialization Round-Trip
// ===========================================================================

fn validate_serialization() -> Vec<ValidationResult> {
    let mut results = Vec::new();

    let kp = Ed25519KeyPair::from_seed("serial_test");
    let message = b"provenance data";
    let sig = kp.sign(message);

    let Ok(sig_json) = serde_json::to_string(&sig.to_vec()) else {
        eprintln!("FATAL: failed to serialize signature");
        std::process::exit(1);
    };
    let Ok(sig_back): Result<Vec<u8>, _> = serde_json::from_str(&sig_json) else {
        eprintln!("FATAL: failed to deserialize signature");
        std::process::exit(1);
    };
    let mut sig_arr = [0u8; 64];
    sig_arr.copy_from_slice(&sig_back);

    results.push(ValidationResult::check(
        EXP,
        "serial_sig_roundtrip",
        bool_f64(Ed25519KeyPair::verify(&kp.public_key, message, &sig_arr)),
        1.0,
        0.0,
    ));

    let Ok(pk_json) = serde_json::to_string(&kp.public_key.to_vec()) else {
        eprintln!("FATAL: failed to serialize public key");
        std::process::exit(1);
    };
    let Ok(pk_back): Result<Vec<u8>, _> = serde_json::from_str(&pk_json) else {
        eprintln!("FATAL: failed to deserialize public key");
        std::process::exit(1);
    };
    let mut pk_arr = [0u8; 32];
    pk_arr.copy_from_slice(&pk_back);
    results.push(ValidationResult::check(
        EXP,
        "serial_pubkey_roundtrip",
        bool_f64(pk_arr == kp.public_key),
        1.0,
        0.0,
    ));

    results
}

// ===========================================================================
// Main
// ===========================================================================

fn main() {
    let mut all_results = Vec::new();
    all_results.extend(validate_key_pair());
    all_results.extend(validate_signed_lifecycle());
    all_results.extend(validate_tamper_detection());
    all_results.extend(validate_multi_actor());
    all_results.extend(validate_beardog_ipc());
    all_results.extend(validate_serialization());

    let total = all_results.len();
    let passed = all_results.iter().filter(|r| r.passed).count();
    let failed = total - passed;

    println!("\n=== {EXP} ===");
    println!("{passed}/{total} checks passed");

    if failed > 0 {
        for r in &all_results {
            if !r.passed {
                println!("  FAIL: {}", r.description);
            }
        }
        std::process::exit(1);
    }
}
