// SPDX-License-Identifier: AGPL-3.0-or-later
//! Validation harness — the ecoPrimals pattern for reproducible experiment
//! validation with provenance, structured checks, and deterministic exit codes.
//!
//! # hotSpring Pattern
//!
//! Every validation binary follows the same protocol:
//!
//! 1. Create a [`ValidationHarness`] with a descriptive name.
//! 2. Optionally attach [`BaselineProvenance`] records.
//! 3. Run checks via `check_abs`, `check_rel`, `check_bool`, `check_upper`,
//!    `check_lower`.
//! 4. Call [`ValidationHarness::finish`] — prints a summary and calls
//!    `std::process::exit(0)` on success, `std::process::exit(1)` on any failure.
//!
//! # Example
//!
//! ```rust,no_run
//! use ludospring_barracuda::validation::{ValidationHarness, BaselineProvenance};
//!
//! let provenance = BaselineProvenance {
//!     script: "baselines/python/run_all_baselines.py",
//!     commit: "74cf9488",
//!     date: "2026-03-15",
//!     command: "python3 baselines/python/run_all_baselines.py",
//! };
//!
//! let mut h = ValidationHarness::new("Fitts Law Parity");
//! h.print_provenance(&[&provenance]);
//! h.check_abs("Fitts MT D100 W10", 708.847, 708.847, 1e-10);
//! h.check_rel("Large value", 1e12 + 1.0, 1e12, 1e-6);
//! h.check_bool("Finite check", 3.14_f64.is_finite());
//! h.finish();
//! ```

/// Provenance record tying expected values to their source computation.
///
/// Every hardcoded expected value in a validation binary should trace back
/// to a `BaselineProvenance` record identifying the script, commit, date,
/// and exact command that produced it.
#[derive(Debug, Clone)]
pub struct BaselineProvenance {
    /// Path to the script that generated the baseline values.
    pub script: &'static str,
    /// Git commit hash (short or full) of the repo when the baseline was generated.
    pub commit: &'static str,
    /// Date the baseline was generated (ISO 8601 or human-readable).
    pub date: &'static str,
    /// Exact command used to regenerate the baseline.
    pub command: &'static str,
}

impl core::fmt::Display for BaselineProvenance {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "  script:  {}\n  commit:  {}\n  date:    {}\n  command: {}",
            self.script, self.commit, self.date, self.command,
        )
    }
}

/// Individual check result with provenance-aware labeling.
#[derive(Debug, Clone)]
pub struct Check {
    /// Human-readable label.
    pub label: String,
    /// Whether this check passed.
    pub passed: bool,
    /// Detail message on failure.
    pub detail: String,
}

/// Validation harness — accumulates checks and produces a summary with exit code.
///
/// Follows the hotSpring pattern: structured checks, provenance printing,
/// machine-readable summary line, and deterministic `process::exit`.
#[derive(Debug)]
pub struct ValidationHarness {
    name: String,
    checks: Vec<Check>,
}

impl ValidationHarness {
    /// Create a new harness with a descriptive experiment name.
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            checks: Vec::new(),
        }
    }

    /// Print provenance records for the baselines used in this validation.
    pub fn print_provenance(&self, baselines: &[&BaselineProvenance]) {
        eprintln!("═══ {} ═══", self.name);
        for (i, b) in baselines.iter().enumerate() {
            eprintln!("Baseline #{}", i + 1);
            eprintln!("{b}");
        }
        eprintln!();
    }

    /// Absolute tolerance check: `|observed - expected| <= tolerance`.
    pub fn check_abs(&mut self, label: &str, observed: f64, expected: f64, tolerance: f64) {
        let delta = (observed - expected).abs();
        let passed = delta <= tolerance;
        let detail = if passed {
            String::new()
        } else {
            format!(
                "observed={observed}, expected={expected}, delta={delta:.2e}, tolerance={tolerance:.2e}"
            )
        };
        self.record(label, passed, &detail);
    }

    /// Relative tolerance check: `|observed - expected| / |expected| <= tolerance`.
    ///
    /// Falls back to absolute comparison against `tolerance` when `expected` is near zero.
    pub fn check_rel(&mut self, label: &str, observed: f64, expected: f64, tolerance: f64) {
        let delta = (observed - expected).abs();
        let denom = expected.abs();
        let passed = if denom < f64::EPSILON {
            delta <= tolerance
        } else {
            (delta / denom) <= tolerance
        };
        let detail = if passed {
            String::new()
        } else {
            let rel = if denom < f64::EPSILON {
                f64::INFINITY
            } else {
                delta / denom
            };
            format!(
                "observed={observed}, expected={expected}, rel_err={rel:.2e}, tolerance={tolerance:.2e}"
            )
        };
        self.record(label, passed, &detail);
    }

    /// Boolean assertion check.
    pub fn check_bool(&mut self, label: &str, condition: bool) {
        let detail = if condition {
            String::new()
        } else {
            "condition was false".into()
        };
        self.record(label, condition, &detail);
    }

    /// Upper-bound check: `observed <= threshold`.
    pub fn check_upper(&mut self, label: &str, observed: f64, threshold: f64) {
        let passed = observed <= threshold;
        let detail = if passed {
            String::new()
        } else {
            format!("observed={observed}, threshold={threshold}")
        };
        self.record(label, passed, &detail);
    }

    /// Lower-bound check: `observed >= threshold`.
    pub fn check_lower(&mut self, label: &str, observed: f64, threshold: f64) {
        let passed = observed >= threshold;
        let detail = if passed {
            String::new()
        } else {
            format!("observed={observed}, threshold={threshold}")
        };
        self.record(label, passed, &detail);
    }

    fn record(&mut self, label: &str, passed: bool, detail: &str) {
        let symbol = if passed { "✓" } else { "✗" };
        if passed {
            eprintln!("  {symbol} {label}");
        } else {
            eprintln!("  {symbol} {label}: {detail}");
        }
        self.checks.push(Check {
            label: label.into(),
            passed,
            detail: detail.into(),
        });
    }

    /// Number of checks that passed.
    #[must_use]
    pub fn passed_count(&self) -> usize {
        self.checks.iter().filter(|c| c.passed).count()
    }

    /// Total number of checks.
    #[must_use]
    pub const fn total_count(&self) -> usize {
        self.checks.len()
    }

    /// Whether all checks passed.
    #[must_use]
    pub fn all_passed(&self) -> bool {
        self.checks.iter().all(|c| c.passed)
    }

    /// Print summary and return exit code (0 = all passed, 1 = any failure).
    ///
    /// Use this when you need the exit code without calling `process::exit`
    /// (e.g., in `#[test]` contexts).
    #[must_use]
    pub fn summary(&self) -> i32 {
        let passed = self.passed_count();
        let total = self.total_count();
        let ok = self.all_passed();
        let status = if ok { "PASS" } else { "FAIL" };
        eprintln!();
        eprintln!(
            "═══ {name} validation: {passed}/{total} checks passed — {status} ═══",
            name = self.name,
        );
        i32::from(!ok)
    }

    /// Print summary and exit the process.
    ///
    /// Calls `std::process::exit(0)` if all checks passed, otherwise
    /// `std::process::exit(1)`. This is the standard hotSpring termination
    /// pattern for validation binaries.
    pub fn finish(&self) -> ! {
        std::process::exit(self.summary());
    }
}

// ── Legacy API (backward-compatible) ────────────────────────────────

/// Validation result for a single experiment check (legacy API).
///
/// Prefer [`ValidationHarness`] for new code. This struct is retained for
/// backward compatibility with existing experiments.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Experiment identifier.
    pub experiment: String,
    /// Whether the validation passed.
    pub passed: bool,
    /// Measured value.
    pub measured: f64,
    /// Expected value.
    pub expected: f64,
    /// Tolerance (absolute).
    pub tolerance: f64,
    /// Human-readable description.
    pub description: String,
}

impl ValidationResult {
    /// Create a result by checking measured against expected within tolerance.
    #[must_use]
    pub fn check(
        experiment: &str,
        description: &str,
        measured: f64,
        expected: f64,
        tolerance: f64,
    ) -> Self {
        Self {
            experiment: experiment.into(),
            description: description.into(),
            measured,
            expected,
            tolerance,
            passed: (measured - expected).abs() <= tolerance,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── ValidationHarness tests ─────────────────────────────────────

    #[test]
    fn harness_all_pass() {
        let mut h = ValidationHarness::new("test");
        h.check_abs("exact", 1.0, 1.0, 1e-10);
        h.check_bool("true", true);
        h.check_upper("below", 5.0, 10.0);
        h.check_lower("above", 10.0, 5.0);
        assert!(h.all_passed());
        assert_eq!(h.summary(), 0);
    }

    #[test]
    fn harness_detects_failure() {
        let mut h = ValidationHarness::new("test");
        h.check_abs("exact", 1.0, 1.0, 1e-10);
        h.check_abs("off", 2.0, 1.0, 0.01);
        assert!(!h.all_passed());
        assert_eq!(h.passed_count(), 1);
        assert_eq!(h.total_count(), 2);
        assert_eq!(h.summary(), 1);
    }

    #[test]
    fn check_rel_near_zero_expected() {
        let mut h = ValidationHarness::new("test");
        h.check_rel("near-zero", 1e-15, 0.0, 1e-10);
        assert!(h.all_passed());
    }

    #[test]
    fn check_rel_normal() {
        let mut h = ValidationHarness::new("test");
        h.check_rel("1ppm", 1_000_001.0, 1_000_000.0, 1e-5);
        assert!(h.all_passed());
    }

    #[test]
    fn check_rel_fails() {
        let mut h = ValidationHarness::new("test");
        h.check_rel("10%", 1.1, 1.0, 0.01);
        assert!(!h.all_passed());
    }

    #[test]
    fn check_upper_fails() {
        let mut h = ValidationHarness::new("test");
        h.check_upper("over", 11.0, 10.0);
        assert!(!h.all_passed());
    }

    #[test]
    fn check_lower_fails() {
        let mut h = ValidationHarness::new("test");
        h.check_lower("under", 4.0, 5.0);
        assert!(!h.all_passed());
    }

    #[test]
    fn check_bool_false_fails() {
        let mut h = ValidationHarness::new("test");
        h.check_bool("nope", false);
        assert!(!h.all_passed());
    }

    #[test]
    fn provenance_display() {
        let p = BaselineProvenance {
            script: "run.py",
            commit: "abc123",
            date: "2026-03-15",
            command: "python3 run.py",
        };
        let s = format!("{p}");
        assert!(s.contains("run.py"));
        assert!(s.contains("abc123"));
    }

    // ── Legacy ValidationResult tests ───────────────────────────────

    #[test]
    fn passing_validation() {
        let r = ValidationResult::check("test", "example", 1.001, 1.0, 0.01);
        assert!(r.passed);
    }

    #[test]
    fn failing_validation() {
        let r = ValidationResult::check("test", "example", 2.0, 1.0, 0.01);
        assert!(!r.passed);
    }
}
