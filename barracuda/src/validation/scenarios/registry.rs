// SPDX-License-Identifier: AGPL-3.0-or-later
//! Scenario registry — tracks, tiers, and metadata per absorbed experiment.

use std::fmt;

/// Validation tier: determines whether a scenario needs live primals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    /// Pure Rust validation — no IPC, safe for CI.
    Rust,
    /// Requires deployed primals from plasmidBin.
    Live,
    /// Runs in both modes (Rust subset + Live full).
    Both,
}

impl fmt::Display for Tier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rust => write!(f, "rust"),
            Self::Live => write!(f, "live"),
            Self::Both => write!(f, "both"),
        }
    }
}

impl Tier {
    /// Parse from CLI string (case-insensitive).
    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "rust" | "1" | "tier1" => Some(Self::Rust),
            "live" | "2" | "tier2" => Some(Self::Live),
            "both" | "all" => Some(Self::Both),
            _ => None,
        }
    }
}

/// Validation track: domain category for filtering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Track {
    InteractionScience,
    ProceduralGeneration,
    EngagementMetrics,
    CompositionParity,
    PerformanceBudget,
    CrossAtomic,
}

impl fmt::Display for Track {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InteractionScience => write!(f, "interaction"),
            Self::ProceduralGeneration => write!(f, "procedural"),
            Self::EngagementMetrics => write!(f, "engagement"),
            Self::CompositionParity => write!(f, "composition"),
            Self::PerformanceBudget => write!(f, "performance"),
            Self::CrossAtomic => write!(f, "cross-atomic"),
        }
    }
}

/// Static metadata for a scenario — provenance, classification, identity.
#[derive(Debug, Clone)]
pub struct ScenarioMeta {
    pub id: &'static str,
    pub track: Track,
    pub tier: Tier,
    pub provenance_crate: &'static str,
    pub provenance_date: &'static str,
    pub description: &'static str,
}

/// A runnable scenario with metadata and execution function.
pub struct Scenario {
    pub meta: ScenarioMeta,
    pub run: fn(&mut crate::validation::ValidationHarness),
}

/// Registry of all absorbed scenarios.
pub struct ScenarioRegistry {
    scenarios: Vec<Scenario>,
}

impl ScenarioRegistry {
    pub fn new() -> Self {
        Self {
            scenarios: Vec::new(),
        }
    }

    pub fn register(&mut self, scenario: Scenario) {
        self.scenarios.push(scenario);
    }

    pub fn all(&self) -> &[Scenario] {
        &self.scenarios
    }

    pub fn filter_by_tier(&self, tier: Tier) -> Vec<&Scenario> {
        self.scenarios
            .iter()
            .filter(|s| s.meta.tier == tier || s.meta.tier == Tier::Both || tier == Tier::Both)
            .collect()
    }

    pub fn filter_by_track(&self, track: Track) -> Vec<&Scenario> {
        self.scenarios
            .iter()
            .filter(|s| s.meta.track == track)
            .collect()
    }

    pub fn find(&self, id: &str) -> Option<&Scenario> {
        self.scenarios.iter().find(|s| s.meta.id == id)
    }

    pub fn len(&self) -> usize {
        self.scenarios.len()
    }

    pub fn is_empty(&self) -> bool {
        self.scenarios.is_empty()
    }
}

impl Default for ScenarioRegistry {
    fn default() -> Self {
        Self::new()
    }
}
