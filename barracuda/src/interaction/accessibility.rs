// SPDX-License-Identifier: AGPL-3.0-or-later
//! Accessibility scoring — quantifying how playable a system is across abilities.
//!
//! Games are the frontier of accessibility because they demand simultaneous
//! visual, auditory, motor, and cognitive engagement. Scoring a game's
//! accessibility rigorously lets us optimize for universal play.
//!
//! # References
//! - IGDA Game Accessibility SIG guidelines
//! - Xbox Accessibility Guidelines (XAG)
//! - WCAG 2.1 (where applicable to interactive systems)

/// Accessibility dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AccessibilityDimension {
    /// Can a blind player play? (audio cues, descriptions, braille)
    Visual,
    /// Can a deaf player play? (visual cues, captions, haptics)
    Auditory,
    /// Can a player with limited mobility play? (input remapping, one-hand, sip-and-puff)
    Motor,
    /// Can a player with cognitive differences play? (difficulty, pacing, clarity)
    Cognitive,
}

/// Score for a single accessibility dimension (0.0 = inaccessible, 1.0 = fully accessible).
#[derive(Debug, Clone)]
pub struct DimensionScore {
    /// Which dimension.
    pub dimension: AccessibilityDimension,
    /// Score (0.0–1.0).
    pub score: f64,
    /// Specific issues found.
    pub issues: Vec<String>,
    /// Specific strengths found.
    pub strengths: Vec<String>,
}

/// Features present in a visual accessibility evaluation.
///
/// Refactored from individual bool parameters per Xbox Accessibility Guidelines
/// and IGDA recommendations.
#[derive(Debug, Clone, Copy, Default)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "each bool maps to a distinct accessibility feature per IGDA/XAG guidelines"
)]
pub struct VisualAccessibilityFeatures {
    /// Audio cues that convey visual events.
    pub audio_cues: bool,
    /// Natural language scene/object descriptions.
    pub descriptions: bool,
    /// Braille output support.
    pub braille: bool,
    /// Haptic feedback for spatial awareness.
    pub haptic: bool,
    /// Information encoded independently of color.
    pub color_independent: bool,
    /// User-scalable text size.
    pub scalable_text: bool,
}

/// Evaluate visual accessibility of a game interface.
///
/// Checks: alternative modalities, screen reader support, color independence,
/// text sizing, contrast.
#[must_use]
pub fn score_visual_accessibility(features: &VisualAccessibilityFeatures) -> DimensionScore {
    let mut score = 0.0;
    let mut issues = Vec::new();
    let mut strengths = Vec::new();

    let checks: &[(&str, &str, bool, f64)] = &[
        (
            "Audio cues for visual events",
            "No audio cues",
            features.audio_cues,
            0.2,
        ),
        (
            "Natural language descriptions",
            "No descriptions",
            features.descriptions,
            0.2,
        ),
        ("Braille output", "No braille", features.braille, 0.15),
        ("Haptic feedback", "No haptic", features.haptic, 0.15),
        (
            "Color-independent information",
            "Color-dependent",
            features.color_independent,
            0.15,
        ),
        (
            "Scalable text",
            "Fixed text size",
            features.scalable_text,
            0.15,
        ),
    ];

    for &(strength_msg, issue_msg, present, weight) in checks {
        if present {
            score += weight;
            strengths.push(strength_msg.into());
        } else {
            issues.push(issue_msg.into());
        }
    }

    DimensionScore {
        dimension: AccessibilityDimension::Visual,
        score,
        issues,
        strengths,
    }
}

/// Composite accessibility score across all dimensions.
#[derive(Debug, Clone)]
pub struct AccessibilityReport {
    /// Per-dimension scores.
    pub dimensions: Vec<DimensionScore>,
    /// Overall score (average of dimensions).
    pub overall: f64,
}

impl AccessibilityReport {
    /// Build a report from dimension scores.
    #[must_use]
    #[expect(
        clippy::cast_precision_loss,
        reason = "accessibility dimensions are few (≤10); len fits in f64"
    )]
    pub fn from_dimensions(dimensions: Vec<DimensionScore>) -> Self {
        let overall = if dimensions.is_empty() {
            0.0
        } else {
            dimensions.iter().map(|d| d.score).sum::<f64>() / dimensions.len() as f64
        };
        Self {
            dimensions,
            overall,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fully_accessible_scores_one() {
        let features = VisualAccessibilityFeatures {
            audio_cues: true,
            descriptions: true,
            braille: true,
            haptic: true,
            color_independent: true,
            scalable_text: true,
        };
        let score = score_visual_accessibility(&features);
        assert!((score.score - 1.0).abs() < 1e-10);
        assert!(score.issues.is_empty());
    }

    #[test]
    fn no_accessibility_scores_zero() {
        let score = score_visual_accessibility(&VisualAccessibilityFeatures::default());
        assert!((score.score - 0.0).abs() < 1e-10);
        assert_eq!(score.issues.len(), 6);
    }

    #[test]
    fn report_averages_dimensions() {
        let d1 = DimensionScore {
            dimension: AccessibilityDimension::Visual,
            score: 0.8,
            issues: vec![],
            strengths: vec![],
        };
        let d2 = DimensionScore {
            dimension: AccessibilityDimension::Auditory,
            score: 0.4,
            issues: vec![],
            strengths: vec![],
        };
        let report = AccessibilityReport::from_dimensions(vec![d1, d2]);
        assert!((report.overall - 0.6).abs() < 1e-10);
    }
}
