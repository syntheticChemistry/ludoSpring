// SPDX-License-Identifier: AGPL-3.0-or-later
//! Validation harness — the ecoPrimals pattern for reproducible experiment
//! validation with provenance, structured checks, and deterministic exit codes.
//!
//! # hotSpring Pattern
//!
//! Every validation binary follows the same protocol:
//!
//! 1. Create a `ValidationHarness` with a descriptive name.
//! 2. Optionally attach `BaselineProvenance` records.
//! 3. Run checks via `check_abs`, `check_rel`, `check_bool`, `check_upper`,
//!    `check_lower`.
//! 4. Call `ValidationHarness::finish` — prints a summary and calls
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

/// Output sink for validation diagnostics.
///
/// Pluggable output allows the harness to write to stderr (the default for
/// validation binaries), a buffer (for embedding in tests), or any custom
/// destination.  The default [`StderrSink`] preserves backward compatibility
/// with the original hotSpring `eprintln!` pattern.
pub trait ValidationSink {
    /// Write a diagnostic line (no trailing newline expected — implementations add it).
    fn emit(&mut self, line: &str);
}

/// Default sink — writes to stderr.  Used by [`ValidationHarness::new`].
#[derive(Debug, Default)]
pub struct StderrSink;

impl ValidationSink for StderrSink {
    fn emit(&mut self, line: &str) {
        eprintln!("{line}");
    }
}

/// Buffer sink — collects lines for programmatic inspection in tests.
#[derive(Debug, Default)]
pub struct BufferSink {
    /// All lines emitted by the harness.
    pub lines: Vec<String>,
}

impl ValidationSink for BufferSink {
    fn emit(&mut self, line: &str) {
        self.lines.push(line.to_owned());
    }
}

/// Validation harness — accumulates checks and produces a summary with exit code.
///
/// Follows the hotSpring pattern: structured checks, provenance printing,
/// machine-readable summary line, and deterministic `process::exit`.
///
/// Output goes through a [`ValidationSink`], defaulting to stderr.  Use
/// [`ValidationHarness::with_sink`] to redirect output for embedding or testing.
pub struct ValidationHarness<S: ValidationSink = StderrSink> {
    name: String,
    checks: Vec<Check>,
    sink: S,
}

impl<S: core::fmt::Debug + ValidationSink> core::fmt::Debug for ValidationHarness<S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ValidationHarness")
            .field("name", &self.name)
            .field("checks", &self.checks)
            .field("sink", &self.sink)
            .finish()
    }
}

impl ValidationHarness<StderrSink> {
    /// Create a new harness with a descriptive experiment name.
    ///
    /// Output goes to stderr (the hotSpring default).
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            checks: Vec::new(),
            sink: StderrSink,
        }
    }
}

impl<S: ValidationSink> ValidationHarness<S> {
    /// Create a harness with a custom output sink.
    #[must_use]
    pub fn with_sink(name: &str, sink: S) -> Self {
        Self {
            name: name.into(),
            checks: Vec::new(),
            sink,
        }
    }

    /// Print provenance records for the baselines used in this validation.
    pub fn print_provenance(&mut self, baselines: &[&BaselineProvenance]) {
        self.sink.emit(&format!("═══ {} ═══", self.name));
        for (i, b) in baselines.iter().enumerate() {
            self.sink.emit(&format!("Baseline #{}", i + 1));
            self.sink.emit(&format!("{b}"));
        }
        self.sink.emit("");
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
            self.sink.emit(&format!("  {symbol} {label}"));
        } else {
            self.sink.emit(&format!("  {symbol} {label}: {detail}"));
        }
        self.checks.push(Check {
            label: label.into(),
            passed,
            detail: detail.into(),
        });
    }

    /// The accumulated check results.
    #[must_use]
    pub fn checks(&self) -> &[Check] {
        &self.checks
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

    /// Emit summary and return exit code (0 = all passed, 1 = any failure).
    ///
    /// Use this when you need the exit code without calling `process::exit`
    /// (e.g., in `#[test]` contexts).
    #[must_use]
    pub fn summary(&mut self) -> i32 {
        let passed = self.passed_count();
        let total = self.total_count();
        let ok = self.all_passed();
        let status = if ok { "PASS" } else { "FAIL" };
        self.sink.emit("");
        self.sink.emit(&format!(
            "═══ {name} validation: {passed}/{total} checks passed — {status} ═══",
            name = self.name,
        ));
        i32::from(!ok)
    }

    /// Absolute-or-relative tolerance check.
    ///
    /// Passes when *either* the absolute or relative test succeeds. Useful
    /// when values span many orders of magnitude (e.g., GPU f32 parity).
    pub fn check_abs_or_rel(
        &mut self,
        label: &str,
        observed: f64,
        expected: f64,
        abs_tol: f64,
        rel_tol: f64,
    ) {
        let delta = (observed - expected).abs();
        let denom = expected.abs();
        let abs_ok = delta <= abs_tol;
        let rel_ok = if denom < f64::EPSILON {
            delta <= rel_tol
        } else {
            (delta / denom) <= rel_tol
        };
        let passed = abs_ok || rel_ok;
        let detail = if passed {
            String::new()
        } else {
            format!(
                "observed={observed}, expected={expected}, delta={delta:.2e}, \
                 abs_tol={abs_tol:.2e}, rel_tol={rel_tol:.2e}"
            )
        };
        self.record(label, passed, &detail);
    }

    /// Emit summary and exit the process.
    ///
    /// Calls `std::process::exit(0)` if all checks passed, otherwise
    /// `std::process::exit(1)`. This is the standard hotSpring termination
    /// pattern for validation binaries.
    pub fn finish(&mut self) -> ! {
        std::process::exit(self.summary());
    }
}

// ── OrExit trait (groundSpring V112 / wetSpring V123 pattern) ───────

/// Zero-boilerplate exit for validation binaries.
///
/// Absorbed from groundSpring V112 and wetSpring V123. Replaces the
/// `let Ok(...) = expr else { eprintln!("FATAL: ..."); std::process::exit(1); }`
/// pattern with a single `.or_exit("context")` call.
///
/// Only intended for validation binaries (not library code).
pub trait OrExit<T> {
    /// Unwrap the value or print to stderr and exit with code 1.
    fn or_exit(self, context: &str) -> T;
}

impl<T, E: core::fmt::Display> OrExit<T> for Result<T, E> {
    fn or_exit(self, context: &str) -> T {
        match self {
            Ok(v) => v,
            Err(e) => {
                eprintln!("FATAL: {context}: {e}");
                std::process::exit(1);
            }
        }
    }
}

impl<T> OrExit<T> for Option<T> {
    fn or_exit(self, context: &str) -> T {
        self.unwrap_or_else(|| {
            eprintln!("FATAL: {context}: None");
            std::process::exit(1);
        })
    }
}

// ── Skip pattern (wetSpring V123) ───────────────────────────────────

/// Exit code for experiments that cannot run in the current environment.
///
/// Distinguishes "not applicable" (e.g., GPU not available) from "fail".
/// CI systems can treat exit code 2 as a skip rather than a failure.
pub const EXIT_SKIPPED: i32 = 2;

/// Print a skip reason to stderr and exit with [`EXIT_SKIPPED`].
///
/// Use this when an experiment requires hardware (GPU, NPU) or a primal
/// (toadStool, petalTongue) that is not available at runtime.
pub fn exit_skipped(reason: &str) -> ! {
    eprintln!("SKIP: {reason}");
    std::process::exit(EXIT_SKIPPED);
}

// ── Baseline JSON loader ────────────────────────────────────────────

/// Load a value from `combined_baselines.json` at a dot-separated JSON path.
///
/// Enables experiments to read Python baseline values at runtime instead of
/// transcribing them as hardcoded constants. The JSON file is produced by
/// `baselines/python/run_all_baselines.py`.
///
/// # Errors
///
/// Returns an error if the file cannot be read, parsed, or the path does
/// not resolve to a numeric value.
pub fn load_baseline_f64(json_path: &std::path::Path, key_path: &str) -> Result<f64, String> {
    let content = std::fs::read_to_string(json_path)
        .map_err(|e| format!("read {}: {e}", json_path.display()))?;
    let root: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("parse JSON: {e}"))?;

    let mut current = &root;
    for segment in key_path.split('.') {
        current = current
            .get(segment)
            .ok_or_else(|| format!("key not found: {segment} in path {key_path}"))?;
    }
    current
        .as_f64()
        .ok_or_else(|| format!("value at {key_path} is not numeric: {current}"))
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
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    // ── ValidationHarness tests ─────────────────────────────────────

    #[test]
    fn harness_all_pass() {
        let mut h = ValidationHarness::with_sink("test", BufferSink::default());
        h.check_abs("exact", 1.0, 1.0, 1e-10);
        h.check_bool("true", true);
        h.check_upper("below", 5.0, 10.0);
        h.check_lower("above", 10.0, 5.0);
        assert!(h.all_passed());
        assert_eq!(h.summary(), 0);
    }

    #[test]
    fn harness_detects_failure() {
        let mut h = ValidationHarness::with_sink("test", BufferSink::default());
        h.check_abs("exact", 1.0, 1.0, 1e-10);
        h.check_abs("off", 2.0, 1.0, 0.01);
        assert!(!h.all_passed());
        assert_eq!(h.passed_count(), 1);
        assert_eq!(h.total_count(), 2);
        assert_eq!(h.summary(), 1);
    }

    #[test]
    fn buffer_sink_captures_output() {
        let mut h = ValidationHarness::with_sink("cap-test", BufferSink::default());
        h.check_bool("ok", true);
        h.check_bool("fail", false);
        let code = h.summary();
        assert_eq!(code, 1);
        assert!(h.sink.lines.iter().any(|l| l.contains("✓ ok")));
        assert!(h.sink.lines.iter().any(|l| l.contains("✗ fail")));
        assert!(h.sink.lines.iter().any(|l| l.contains("FAIL")));
    }

    #[test]
    fn check_rel_near_zero_expected() {
        let mut h = ValidationHarness::with_sink("test", BufferSink::default());
        h.check_rel("near-zero", 1e-15, 0.0, 1e-10);
        assert!(h.all_passed());
    }

    #[test]
    fn check_rel_normal() {
        let mut h = ValidationHarness::with_sink("test", BufferSink::default());
        h.check_rel("1ppm", 1_000_001.0, 1_000_000.0, 1e-5);
        assert!(h.all_passed());
    }

    #[test]
    fn check_rel_fails() {
        let mut h = ValidationHarness::with_sink("test", BufferSink::default());
        h.check_rel("10%", 1.1, 1.0, 0.01);
        assert!(!h.all_passed());
    }

    #[test]
    fn check_upper_fails() {
        let mut h = ValidationHarness::with_sink("test", BufferSink::default());
        h.check_upper("over", 11.0, 10.0);
        assert!(!h.all_passed());
    }

    #[test]
    fn check_lower_fails() {
        let mut h = ValidationHarness::with_sink("test", BufferSink::default());
        h.check_lower("under", 4.0, 5.0);
        assert!(!h.all_passed());
    }

    #[test]
    fn check_bool_false_fails() {
        let mut h = ValidationHarness::with_sink("test", BufferSink::default());
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

    #[test]
    fn provenance_prints_through_sink() {
        let mut h = ValidationHarness::with_sink("prov-test", BufferSink::default());
        let p = BaselineProvenance {
            script: "run.py",
            commit: "abc123",
            date: "2026-03-15",
            command: "python3 run.py",
        };
        h.print_provenance(&[&p]);
        assert!(h.sink.lines.iter().any(|l| l.contains("prov-test")));
        assert!(h.sink.lines.iter().any(|l| l.contains("run.py")));
    }

    #[test]
    fn checks_accessor() {
        let mut h = ValidationHarness::with_sink("test", BufferSink::default());
        h.check_bool("a", true);
        h.check_bool("b", false);
        assert_eq!(h.checks().len(), 2);
        assert!(h.checks()[0].passed);
        assert!(!h.checks()[1].passed);
    }

    // ── OrExit tests ──────────────────────────────────────────────────

    #[test]
    fn or_exit_result_ok_returns_value() {
        let r: Result<i32, &str> = Ok(42);
        assert_eq!(r.or_exit("should not exit"), 42);
    }

    #[test]
    fn or_exit_option_some_returns_value() {
        let o: Option<i32> = Some(99);
        assert_eq!(o.or_exit("should not exit"), 99);
    }

    // ── check_abs_or_rel tests ─────────────────────────────────────

    #[test]
    fn abs_or_rel_passes_on_abs() {
        let mut h = ValidationHarness::with_sink("test", BufferSink::default());
        h.check_abs_or_rel("abs-wins", 1.000_001, 1.0, 1e-5, 1e-10);
        assert!(h.all_passed());
    }

    #[test]
    fn abs_or_rel_passes_on_rel() {
        let mut h = ValidationHarness::with_sink("test", BufferSink::default());
        h.check_abs_or_rel("rel-wins", 1_000_001.0, 1_000_000.0, 1e-10, 1e-5);
        assert!(h.all_passed());
    }

    #[test]
    fn abs_or_rel_fails_when_both_fail() {
        let mut h = ValidationHarness::with_sink("test", BufferSink::default());
        h.check_abs_or_rel("both-fail", 2.0, 1.0, 0.01, 0.01);
        assert!(!h.all_passed());
    }

    // ── exit_skipped ──────────────────────────────────────────────────

    #[test]
    fn exit_skipped_code_is_two() {
        assert_eq!(super::EXIT_SKIPPED, 2);
    }

    // ── load_baseline_f64 ───────────────────────────────────────────

    #[test]
    fn load_baseline_f64_reads_nested_numeric() {
        let path = std::env::temp_dir().join(format!(
            "ludospring_load_baseline_ok_{}.json",
            std::process::id()
        ));
        std::fs::write(&path, r#"{"outer":{"inner":2.5}}"#).unwrap();
        let v = load_baseline_f64(&path, "outer.inner").unwrap();
        assert!((v - 2.5).abs() < f64::EPSILON);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn load_baseline_f64_rejects_missing_path() {
        let path = std::env::temp_dir().join("ludospring_load_baseline_missing_xyz.json");
        let err = load_baseline_f64(&path, "a.b").unwrap_err();
        assert!(err.contains("read") || err.contains("No such file"));
    }

    #[test]
    fn load_baseline_f64_rejects_missing_key() {
        let path = std::env::temp_dir().join(format!(
            "ludospring_load_baseline_key_{}.json",
            std::process::id()
        ));
        std::fs::write(&path, r#"{"only":1}"#).unwrap();
        let err = load_baseline_f64(&path, "missing.leaf").unwrap_err();
        assert!(err.contains("key not found"));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn load_baseline_f64_rejects_non_numeric_leaf() {
        let path = std::env::temp_dir().join(format!(
            "ludospring_load_baseline_type_{}.json",
            std::process::id()
        ));
        std::fs::write(&path, r#"{"x":{"y":"not-a-number"}}"#).unwrap();
        let err = load_baseline_f64(&path, "x.y").unwrap_err();
        assert!(err.contains("not numeric"));
        let _ = std::fs::remove_file(&path);
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
