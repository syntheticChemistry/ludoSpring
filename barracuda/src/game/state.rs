// SPDX-License-Identifier: AGPL-3.0-or-later
//! Game state machines and tick models.
//!
//! A game is a state machine with a clock. This module provides the formal
//! primitives: states, transitions, tick budgets, and deterministic replay.

/// Phase of a game session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionPhase {
    /// Loading assets, generating world.
    Loading,
    /// Active gameplay.
    Playing,
    /// Paused (time frozen, input restricted).
    Paused,
    /// Game over / results screen.
    Ended,
}

/// A single game tick's worth of state updates.
#[derive(Debug, Clone)]
pub struct TickBudget {
    /// Fixed timestep in seconds.
    pub dt: f64,
    /// Maximum physics sub-steps per frame.
    pub max_substeps: u32,
    /// Whether to record inputs for deterministic replay.
    pub record_inputs: bool,
}

impl Default for TickBudget {
    fn default() -> Self {
        Self {
            dt: crate::tolerances::DEFAULT_DT_S,
            max_substeps: 4,
            record_inputs: false,
        }
    }
}

/// Recorded input for deterministic replay.
#[derive(Debug, Clone)]
pub struct InputRecord {
    /// Tick number when the input occurred.
    pub tick: u64,
    /// Serialized input event.
    pub payload: Vec<u8>,
}

/// Replay buffer for deterministic reproduction.
#[derive(Debug, Default, Clone)]
pub struct ReplayBuffer {
    /// Ordered sequence of recorded inputs.
    pub inputs: Vec<InputRecord>,
    /// Current playback position.
    pub cursor: usize,
}

impl ReplayBuffer {
    /// Record an input at the given tick.
    pub fn record(&mut self, tick: u64, payload: Vec<u8>) {
        self.inputs.push(InputRecord { tick, payload });
    }

    /// Get the next input if it matches the current tick.
    pub fn next_for_tick(&mut self, tick: u64) -> Option<&InputRecord> {
        if self.cursor < self.inputs.len() && self.inputs[self.cursor].tick == tick {
            let input = &self.inputs[self.cursor];
            self.cursor += 1;
            Some(input)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tolerances::ANALYTICAL_TOL;

    #[test]
    fn replay_buffer_records_and_plays_back() {
        let mut buf = ReplayBuffer::default();
        buf.record(0, vec![1, 2, 3]);
        buf.record(1, vec![4, 5, 6]);
        buf.record(3, vec![7, 8, 9]);

        assert!(buf.next_for_tick(0).is_some());
        assert!(buf.next_for_tick(1).is_some());
        assert!(buf.next_for_tick(2).is_none());
        assert!(buf.next_for_tick(3).is_some());
    }

    #[test]
    fn default_tick_budget_is_60hz() {
        let budget = TickBudget::default();
        assert!((budget.dt - 1.0 / 60.0).abs() < ANALYTICAL_TOL);
    }
}
