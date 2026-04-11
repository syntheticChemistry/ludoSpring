# SPDX-License-Identifier: AGPL-3.0-or-later
# ludoSpring — reproducible quality gates
#
# Usage:
#   make check       — full quality gate (fmt + clippy + test + doc)
#   make test        — cargo test (barracuda + IPC integration)
#   make baselines   — re-run Python baselines and check for drift
#   make coverage    — line coverage via cargo-llvm-cov
#   make deny        — supply chain audit via cargo-deny
#   make all         — everything
#
# The CARGO_TARGET_DIR override ensures build scripts run on an exec-capable
# filesystem when the global env var points at a noexec mount.

.PHONY: all check fmt clippy test doc deny baselines drift coverage coverage-report clean

export CARGO_TARGET_DIR ?= $(HOME)/.cargo-build/ludoSpring/target
export CARGO_HOME ?= $(HOME)/.cargo

BARRACUDA_FEATURES := --features ipc
BARRACUDA_PKG := -p ludospring-barracuda

all: check baselines deny

check: fmt clippy test doc

fmt:
	cargo fmt --all -- --check

clippy:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
	cargo test $(BARRACUDA_FEATURES) $(BARRACUDA_PKG) --lib --tests
	cargo test -p ludospring-forge --lib --tests

doc:
	cargo doc $(BARRACUDA_FEATURES) $(BARRACUDA_PKG) --no-deps

deny:
	cargo deny check

baselines: drift
	@echo "✓ Python baselines: no drift"

drift:
	python3 baselines/python/check_drift.py

COV_ARGS := $(BARRACUDA_FEATURES) $(BARRACUDA_PKG) --lib --tests --ignore-filename-regex 'bin/'

coverage:
	cargo llvm-cov $(COV_ARGS) --html --fail-under-lines 90
	@echo "Coverage report: target/llvm-cov/html/index.html"

coverage-report:
	cargo llvm-cov $(COV_ARGS) --summary-only

clean:
	cargo clean
