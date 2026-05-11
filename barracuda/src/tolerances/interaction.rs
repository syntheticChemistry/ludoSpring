// SPDX-License-Identifier: AGPL-3.0-or-later
//! Interaction model tolerances — Fitts, Hick, Steering, Flow, DDA.

/// Fitts's law intercept (mouse, desktop) in milliseconds.
///
/// Source: MacKenzie, I.S. (1992). "Fitts' law as a research and design tool
/// in human-computer interaction." Human-Computer Interaction, 7(1), pp. 91–139.
pub const FITTS_A_MOUSE_MS: f64 = 50.0;

/// Fitts's law slope (mouse, desktop) in milliseconds per bit.
///
/// Source: MacKenzie (1992), Table 2, average across experiments.
pub const FITTS_B_MOUSE_MS: f64 = 150.0;

/// Hick's law base reaction time in milliseconds.
///
/// Source: Hick, W.E. (1952). "On the rate of gain of information."
/// Quarterly Journal of Experimental Psychology, 4(1), pp. 11–26.
pub const HICK_A_MS: f64 = 200.0;

/// Hick's law processing time per bit in milliseconds.
///
/// Source: Hyman, R. (1953). "Stimulus information as a determinant of
/// reaction time." Journal of Experimental Psychology, 45(3), pp. 188–196.
pub const HICK_B_MS: f64 = 150.0;

/// Steering law intercept in milliseconds.
///
/// Source: Accot, J. & Zhai, S. (1997). "Beyond Fitts' law: models for
/// trajectory-based HCI tasks." CHI '97, pp. 295–302. Empirical fit
/// for mouse-based tunnel steering; 10ms accounts for initial latency.
pub const STEERING_A_MS: f64 = 10.0;

/// Steering law index coefficient in milliseconds per D/W unit.
///
/// Source: Accot & Zhai (1997), Table 1, mouse condition.
pub const STEERING_B_MS: f64 = 5.0;

/// Flow channel half-width (normalized challenge–skill space).
///
/// Source: Chen, J. (2007). "Flow in Games." M.S. Thesis, USC.
/// Figure 3.2 — the "flow zone" band is approximately ±0.15 around the
/// challenge = skill diagonal.
pub const FLOW_CHANNEL_WIDTH: f64 = 0.15;

/// Target success rate for dynamic difficulty adjustment.
///
/// Source: Hunicke, R. (2005). "The case for dynamic difficulty adjustment
/// in games." ACM SIGCHI '05. Section 4 recommends 0.6–0.75; 0.7 is the
/// midpoint.
pub const DDA_TARGET_SUCCESS_RATE: f64 = 0.7;

#[cfg(test)]
#[allow(
    clippy::assertions_on_constants,
    reason = "validating constant relationships is the purpose of these tests"
)]
mod tests {
    use super::*;

    #[test]
    fn fitts_coefficients_positive() {
        assert!(FITTS_A_MOUSE_MS >= 0.0);
        assert!(FITTS_B_MOUSE_MS > 0.0);
    }

    #[test]
    fn hick_coefficients_positive() {
        assert!(HICK_A_MS > 0.0);
        assert!(HICK_B_MS > 0.0);
    }

    #[test]
    fn steering_coefficients_positive() {
        assert!(STEERING_A_MS >= 0.0);
        assert!(STEERING_B_MS > 0.0);
    }

    #[test]
    fn flow_channel_width_in_unit_interval() {
        assert!(FLOW_CHANNEL_WIDTH > 0.0 && FLOW_CHANNEL_WIDTH < 0.5);
    }

    #[test]
    fn dda_target_success_rate_valid() {
        assert!(DDA_TARGET_SUCCESS_RATE > 0.0 && DDA_TARGET_SUCCESS_RATE < 1.0);
    }

    #[test]
    fn fitts_mt_for_known_id() {
        let d = 300.0_f64;
        let w = 50.0_f64;
        let id = (2.0 * d / w).log2();
        let mt = FITTS_B_MOUSE_MS.mul_add(id, FITTS_A_MOUSE_MS);
        assert!(mt > FITTS_A_MOUSE_MS, "MT must exceed intercept");
        assert!(mt < 1000.0, "MT for moderate ID should be < 1s");
    }
}
