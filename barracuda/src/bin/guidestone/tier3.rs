use super::constants::CROSS_ATOMIC_PAYLOAD_B64;
use primalspring::composition::{CompositionContext, call_or_skip};
use primalspring::validation::ValidationResult;

// ════════════════════════════════════════════════════════════════════════
// Tier 3: FULL NUCLEUS (cross-atomic validation)
// ════════════════════════════════════════════════════════════════════════

/// BearDog crypto: hash a base64-encoded payload, verify non-empty hash.
pub fn validate_security(ctx: &mut CompositionContext, v: &mut ValidationResult) {
    let result = call_or_skip(
        ctx,
        v,
        "nucleus:crypto_hash",
        "security",
        "crypto.hash",
        serde_json::json!({"algorithm": "blake3", "data": CROSS_ATOMIC_PAYLOAD_B64}),
    );

    if let Some(ref res) = result {
        let hash = res
            .get("hash")
            .or_else(|| res.get("result"))
            .and_then(serde_json::Value::as_str);

        if let Some(h) = hash {
            v.check_bool(
                "nucleus:crypto_hash_length",
                !h.is_empty() && h.len() >= 32,
                &format!("BLAKE3 hash length={} chars", h.len()),
            );
        } else {
            v.check_bool(
                "nucleus:crypto_hash_length",
                false,
                &format!("no hash string in response: {res}"),
            );
        }
    }
}

/// NestGate storage: store a value, retrieve it, verify roundtrip.
pub fn validate_storage(ctx: &mut CompositionContext, v: &mut ValidationResult) {
    let store_key = "ludospring-guidestone-test";
    let store_value = "game-science-roundtrip-v1";

    let stored = call_or_skip(
        ctx,
        v,
        "nucleus:storage_store",
        "storage",
        "storage.store",
        serde_json::json!({
            "key": store_key,
            "value": store_value,
            "family_id": "ludospring-validation"
        }),
    );

    if stored.is_none() {
        v.check_skip(
            "nucleus:storage_retrieve",
            "storage.store skipped — cannot test retrieve",
        );
        v.check_skip(
            "nucleus:storage_roundtrip",
            "storage.store skipped — cannot test roundtrip",
        );
        return;
    }

    let retrieved = call_or_skip(
        ctx,
        v,
        "nucleus:storage_retrieve",
        "storage",
        "storage.retrieve",
        serde_json::json!({
            "key": store_key,
            "family_id": "ludospring-validation"
        }),
    );

    if let Some(ref res) = retrieved {
        let val = res
            .get("value")
            .or_else(|| res.get("result"))
            .and_then(serde_json::Value::as_str);

        v.check_bool(
            "nucleus:storage_roundtrip",
            val == Some(store_value),
            &format!(
                "stored={store_value:?}, retrieved={:?}",
                val.unwrap_or("<none>")
            ),
        );
    } else {
        v.check_skip("nucleus:storage_roundtrip", "storage.retrieve skipped");
    }
}

/// Cross-atomic pipeline: hash(BearDog) → store(NestGate) → retrieve → verify.
pub fn validate_cross_atomic(ctx: &mut CompositionContext, v: &mut ValidationResult) {
    // Step 1: Hash the payload via BearDog (base64-encoded)
    let hash_result = call_or_skip(
        ctx,
        v,
        "nucleus:pipeline_hash",
        "security",
        "crypto.hash",
        serde_json::json!({"algorithm": "blake3", "data": CROSS_ATOMIC_PAYLOAD_B64}),
    );

    let hash_hex = hash_result.as_ref().and_then(|res| {
        res.get("hash")
            .or_else(|| res.get("result"))
            .and_then(serde_json::Value::as_str)
            .map(String::from)
    });

    let Some(ref hex) = hash_hex else {
        v.check_skip(
            "nucleus:pipeline_store",
            "crypto.hash unavailable — cannot continue pipeline",
        );
        v.check_skip("nucleus:pipeline_verify", "pipeline aborted");
        return;
    };

    // Step 2: Store the hash via NestGate
    let pipeline_key = "ludospring-pipeline-hash";
    let stored = call_or_skip(
        ctx,
        v,
        "nucleus:pipeline_store",
        "storage",
        "storage.store",
        serde_json::json!({
            "key": pipeline_key,
            "value": hex,
            "family_id": "ludospring-validation"
        }),
    );

    if stored.is_none() {
        v.check_skip("nucleus:pipeline_verify", "storage.store unavailable");
        return;
    }

    // Step 3: Retrieve and verify
    let retrieved = call_or_skip(
        ctx,
        v,
        "nucleus:pipeline_retrieve",
        "storage",
        "storage.retrieve",
        serde_json::json!({
            "key": pipeline_key,
            "family_id": "ludospring-validation"
        }),
    );

    if let Some(ref res) = retrieved {
        let val = res
            .get("value")
            .or_else(|| res.get("result"))
            .and_then(serde_json::Value::as_str);

        v.check_bool(
            "nucleus:pipeline_verify",
            val == Some(hex.as_str()),
            &format!(
                "hash(BearDog)→store(NestGate)→retrieve: match={}",
                val == Some(hex.as_str())
            ),
        );
    } else {
        v.check_skip("nucleus:pipeline_verify", "storage.retrieve unavailable");
    }
}
