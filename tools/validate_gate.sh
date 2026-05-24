#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# validate_gate.sh — Validate ludoSpring domain science against live NUCLEUS
#
# Exercises the proto-nucleate validation_capabilities from
# primalSpring/graphs/downstream/downstream_manifest.toml against live primals
# deployed on the current gate.
#
# Prerequisites:
#   - NUCLEUS launched (nucleus_launcher.sh --family-id <id> --composition full)
#   - At minimum: beardog, toadstool, nestgate reachable
#
# Usage:
#   ./tools/validate_gate.sh                  # auto-detect family
#   ./tools/validate_gate.sh --family-id irongate
#   ./tools/validate_gate.sh --json           # machine-readable output

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

FAMILY_ID="${FAMILY_ID:-irongate}"
JSON_OUTPUT=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        --family-id) FAMILY_ID="$2"; shift 2 ;;
        --json)      JSON_OUTPUT=true; shift ;;
        --help)      echo "Usage: $0 [--family-id ID] [--json]"; exit 0 ;;
        *)           echo "Unknown: $1"; exit 1 ;;
    esac
done

RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}/biomeos"
TMP_BIOMEOS="/tmp/biomeos/biomeos"

if ! $JSON_OUTPUT; then
    echo ""
    echo "══════════════════════════════════════════════════════════════"
    echo "  ludoSpring Gate Validation — ironGate"
    echo "══════════════════════════════════════════════════════════════"
    echo ""
    echo "  Family:   $FAMILY_ID"
    echo "  Runtime:  $RUNTIME_DIR"
    echo "  Fallback: $TMP_BIOMEOS"
    echo ""
fi

# Delegate IPC probing to Python (avoids shell quoting issues with JSON)
IPC_RESULTS=$(python3 "$SCRIPT_DIR/validate_gate_ipc.py" "$FAMILY_ID" "$RUNTIME_DIR" "$TMP_BIOMEOS" 2>&1)
IPC_EXIT=$?

echo "$IPC_RESULTS"
echo ""

# Rust domain science
if ! $JSON_OUTPUT; then
    echo "── Rust Domain Science (cargo test) ─────────────────────────"
    echo ""
fi

cd "$PROJECT_ROOT"
TEST_OUTPUT=$(cargo test --workspace --features ipc,local 2>&1 | grep "^test result:")
TOTAL_PASS=$(echo "$TEST_OUTPUT" | awk '{sum += $4} END {print sum}')

if ! $JSON_OUTPUT; then
    printf "  %-50s PASS (%s tests)\n" "cargo test --workspace" "$TOTAL_PASS"
fi

echo ""
echo "══════════════════════════════════════════════════════════════"
echo "  Gate validation complete."
echo "══════════════════════════════════════════════════════════════"
