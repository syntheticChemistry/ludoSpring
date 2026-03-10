// SPDX-License-Identifier: AGPL-3.0-or-later
//! Validation harness and test utilities.
//!
//! Follows the ecoPrimals validation pattern: each experiment is a
//! reproducible test against published results or known-good baselines.

/// Validation result for an experiment.
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
