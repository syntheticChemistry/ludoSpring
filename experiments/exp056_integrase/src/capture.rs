// SPDX-License-Identifier: AGPL-3.0-or-later
//
//! exp056 — Integrase: Pokemon-like capture/bonding from population dynamics.
//!
//! Recreates capture, type matchups, evolution, and bonding mechanics using
//! open math: Wright-Fisher fixation, quorum sensing thresholds, competitive
//! exclusion (Gause 1934), and Markov chains. All models predate proprietary
//! implementations.

/// Entity elemental type — analogous to Pokemon types.
///
/// Niche partitioning from competitive exclusion (Gause 1934) creates
/// intransitive type advantages: no single type dominates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityType {
    Fire,
    Water,
    Earth,
    Air,
    Void,
}

impl EntityType {
    /// All entity types in canonical order (for matrix indexing).
    pub const ALL: [Self; 5] = [Self::Fire, Self::Water, Self::Earth, Self::Air, Self::Void];

    /// Index for 5x5 effectiveness matrix (0..5).
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Fire => 0,
            Self::Water => 1,
            Self::Earth => 2,
            Self::Air => 3,
            Self::Void => 4,
        }
    }
}

/// Type effectiveness matrix derived from competitive exclusion (Gause 1934).
///
/// Niche partitioning creates rock-paper-scissors-like intransitive cycles:
/// Fire > Earth > Water > Air > Void > Fire. No type dominates all others.
///
/// # Reference
/// Gause, G.F. (1934). *The Struggle for Existence*. Williams & Wilkins.
#[derive(Debug, Clone)]
pub struct TypeMatchup {
    /// 5x5 matrix: `effectiveness[attacker][defender]`.
    /// 2.0 = super effective, 1.0 = neutral, 0.5 = not very effective.
    matrix: [[f64; 5]; 5],
}

impl TypeMatchup {
    /// Build the canonical intransitive cycle: Fire > Earth > Water > Air > Void > Fire.
    #[must_use]
    pub const fn new() -> Self {
        // Fire>Earth, Earth>Water, Water>Air, Air>Void, Void>Fire; Fire vs Water=0.5 (resist)
        let matrix = [
            [1.0, 0.5, 2.0, 1.0, 0.5], // Fire (vs Water=0.5: water resists fire)
            [1.0, 1.0, 0.5, 2.0, 1.0], // Water
            [0.5, 2.0, 1.0, 1.0, 1.0], // Earth
            [1.0, 0.5, 1.0, 1.0, 2.0], // Air
            [2.0, 1.0, 1.0, 0.5, 1.0], // Void
        ];
        Self { matrix }
    }

    /// Effectiveness of attacker vs defender (2.0, 1.0, or 0.5).
    ///
    /// # Reference
    /// Gause (1934): competitive exclusion → niche partitioning → type advantages.
    #[must_use]
    pub const fn effectiveness(&self, attacker: EntityType, defender: EntityType) -> f64 {
        self.matrix[attacker.index()][defender.index()]
    }
}

impl Default for TypeMatchup {
    fn default() -> Self {
        Self::new()
    }
}

/// A wild entity that can be encountered and captured.
#[derive(Debug, Clone)]
#[expect(
    dead_code,
    reason = "Domain model fields used in game logic, not all read in validation"
)]
pub struct WildEntity {
    /// Unique identifier.
    pub id: u32,
    /// Display name.
    pub name: String,
    /// Elemental type.
    pub entity_type: EntityType,
    /// Base combat power.
    pub base_power: f64,
    /// Capture difficulty in [0.0, 1.0]; higher = harder to capture.
    pub capture_difficulty: f64,
    /// Growth rate for experience scaling.
    pub growth_rate: f64,
}

impl WildEntity {
    /// Create a new wild entity.
    #[must_use]
    pub fn new(
        id: u32,
        name: impl Into<String>,
        entity_type: EntityType,
        base_power: f64,
        capture_difficulty: f64,
        growth_rate: f64,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            entity_type,
            base_power,
            capture_difficulty: capture_difficulty.clamp(0.0, 1.0),
            growth_rate,
        }
    }
}

/// A captured entity with progression state.
#[derive(Debug, Clone)]
#[expect(
    dead_code,
    reason = "entity field is domain model; validation reads level/experience"
)]
pub struct CapturedEntity {
    /// The underlying wild entity template.
    pub entity: WildEntity,
    /// Accumulated experience points.
    pub experience: u64,
    /// Current level (1-based).
    pub level: u32,
    /// Bond strength from repeated positive encounters [0.0, 1.0].
    pub bond_strength: f64,
    /// Number of encounters survived.
    pub encounters_survived: u32,
}

impl CapturedEntity {
    /// Create from a wild entity (fresh capture).
    #[must_use]
    #[expect(clippy::missing_const_for_fn, reason = "mutates self")]
    pub fn from_wild(entity: WildEntity) -> Self {
        Self {
            entity,
            experience: 0,
            level: 1,
            bond_strength: 0.5,
            encounters_survived: 0,
        }
    }

    /// Experience required to reach the next level (simplified: level * 100).
    #[must_use]
    pub fn exp_for_next_level(&self) -> u64 {
        u64::from(self.level) * 100
    }

    /// Gain experience; level increases when threshold is crossed.
    pub fn gain_experience(&mut self, amount: u64) {
        self.experience += amount;
        while self.experience >= self.exp_for_next_level() {
            self.level += 1;
        }
    }
}

/// Capture attempt parameters (for documentation; probability computed by `capture_probability`).
#[derive(Debug, Clone, Copy)]
#[expect(
    dead_code,
    reason = "Domain model struct for capture attempt parameters"
)]
pub struct CaptureAttempt {
    /// Ball strength (higher = better capture chance).
    pub ball_strength: f64,
    /// Effective population size for Wright-Fisher (default 10).
    pub effective_n: f64,
}

impl CaptureAttempt {
    /// Create a capture attempt.
    #[must_use]
    #[expect(dead_code, reason = "Domain model constructor for capture attempt")]
    pub const fn new(ball_strength: f64, effective_n: f64) -> Self {
        Self {
            ball_strength,
            effective_n,
        }
    }
}

/// Wright-Fisher fixation probability for capture success.
///
/// p = (1 - e^(-2*s)) / (1 - e^(-2*N*s))
/// where s = selection coefficient (`ball_strength` vs `capture_difficulty`),
/// N = effective population size.
///
/// # Reference
/// Wright, S. (1931). Evolution in Mendelian populations. *Genetics* 16:97–159.
#[must_use]
pub fn capture_probability(ball_strength: f64, entity: &WildEntity) -> f64 {
    capture_probability_with_n(ball_strength, entity, 10.0)
}

/// Capture probability with explicit effective population size N.
#[must_use]
pub fn capture_probability_with_n(
    ball_strength: f64,
    entity: &WildEntity,
    effective_n: f64,
) -> f64 {
    // Selection coefficient: favorable when ball is strong and entity is easy.
    // s = ball_strength * (1 - capture_difficulty), clamped to avoid extremes.
    let s = ball_strength * (1.0 - entity.capture_difficulty);
    let s = s.clamp(-10.0, 10.0);

    if s.abs() < 1e-10 {
        return 1.0 / (effective_n + 1.0);
    }

    let n = effective_n.max(1.0);
    let exp_neg_2s = (-2.0 * s).exp();
    let exp_neg_2n_s = (-2.0 * n * s).exp();

    let num = 1.0 - exp_neg_2s;
    let den = 1.0 - exp_neg_2n_s;

    if den.abs() < 1e-10 {
        return if s > 0.0 { 1.0 } else { 0.0 };
    }

    let p = num / den;
    p.clamp(0.0, 1.0)
}

/// Quorum sensing threshold model for bond-based capture.
///
/// Entity accumulates "bond signal" over encounters. When signal exceeds
/// threshold, irreversible state change occurs (wild → captured).
///
/// # Reference
/// Waters, C.M. & Bassler, B.L. (2005). Quorum sensing: cell-to-cell
/// communication in bacteria. *Annu. Rev. Cell Dev. Biol.* 21:319–346.
#[derive(Debug, Clone)]
pub struct QsThreshold {
    /// Accumulated bond signal.
    pub signal: f64,
    /// Threshold for irreversible transition (wild → captured).
    pub threshold: f64,
    /// Whether the transition has occurred (irreversible).
    pub transitioned: bool,
}

impl QsThreshold {
    /// Create with initial signal and threshold.
    #[must_use]
    pub fn new(initial_signal: f64, threshold: f64) -> Self {
        let transitioned = initial_signal >= threshold;
        Self {
            signal: initial_signal,
            threshold,
            transitioned,
        }
    }

    /// Add bond signal from an encounter.
    pub fn add_signal(&mut self, amount: f64) {
        if self.transitioned {
            return;
        }
        self.signal += amount;
        if self.signal >= self.threshold {
            self.transitioned = true;
        }
    }

    /// Whether the entity is still wild (below threshold, not transitioned).
    #[must_use]
    pub const fn is_wild(&self) -> bool {
        !self.transitioned
    }

    /// Whether the entity has transitioned to captured state.
    #[must_use]
    pub const fn is_captured(&self) -> bool {
        self.transitioned
    }
}

/// Evolution chain as a Markov process with absorbing final form.
///
/// Experience crosses thresholds → state transitions. Final form is absorbing.
///
/// # Reference
/// Markov, A.A. (1906). Extension of the law of large numbers to dependent
/// quantities. *Izvestiya Fiziko-matematicheskogo obschestva*.
#[derive(Debug, Clone)]
pub struct EvolutionChain {
    /// Current form index (0 = base, 1 = stage 1, etc.).
    pub current_form: usize,
    /// Experience thresholds for evolution: [100, 300, 600] → 4 forms.
    pub exp_thresholds: Vec<u64>,
    /// Whether current form is final (absorbing state).
    pub is_absorbing: bool,
}

impl EvolutionChain {
    /// Create with default thresholds [100, 300, 600] (4 forms).
    #[must_use]
    pub fn new() -> Self {
        Self {
            current_form: 0,
            exp_thresholds: vec![100, 300, 600],
            is_absorbing: false,
        }
    }

    /// Create with custom thresholds.
    #[must_use]
    #[expect(clippy::missing_const_for_fn, reason = "mutates self")]
    pub fn with_thresholds(exp_thresholds: Vec<u64>) -> Self {
        let is_absorbing = exp_thresholds.is_empty();
        Self {
            current_form: 0,
            exp_thresholds,
            is_absorbing,
        }
    }

    /// Advance form when experience crosses next threshold.
    pub fn evolve_if_ready(&mut self, total_experience: u64) {
        if self.is_absorbing {
            return;
        }
        let next_threshold = self
            .exp_thresholds
            .get(self.current_form)
            .copied()
            .unwrap_or(u64::MAX);
        if total_experience >= next_threshold {
            self.current_form += 1;
            if self.current_form >= self.exp_thresholds.len() {
                self.is_absorbing = true;
            }
        }
    }

    /// Whether the chain is in its final (absorbing) form.
    #[must_use]
    pub const fn is_final_form(&self) -> bool {
        self.is_absorbing
    }
}

impl Default for EvolutionChain {
    fn default() -> Self {
        Self::new()
    }
}
