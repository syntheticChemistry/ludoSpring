// SPDX-License-Identifier: AGPL-3.0-or-later
//! JSON-RPC method name constants for external primal capabilities.
//!
//! Mirrors the relevant subsets of `primalspring::ipc::methods` so
//! ludoSpring's IPC layer can reference canonical method names without
//! depending on the `primalspring` crate at runtime (which is gated
//! behind the `guidestone` feature).
//!
//! Constants prevent typo-induced silent routing failures and provide
//! a single inventory of external methods ludoSpring recognizes.

/// Visualization domain — petalTongue live rendering.
pub mod visualization {
    /// Push a render payload to the viz layer.
    pub const RENDER: &str = "visualization.render";
    /// Push a composed scene (game state, dashboard, etc.).
    pub const RENDER_SCENE: &str = "visualization.render.scene";
    /// Append a streaming update (time-series, live metrics).
    pub const RENDER_STREAM: &str = "visualization.render.stream";
    /// Push a science dashboard layout.
    pub const RENDER_DASHBOARD: &str = "visualization.render.dashboard";
    /// Push a grammar-of-graphics specification.
    pub const RENDER_GRAMMAR: &str = "visualization.render.grammar";
    /// Export a session replay (SVG timeline, audio archive).
    pub const EXPORT: &str = "visualization.export";
    /// Tufte pre-flight validation on a UI composition.
    pub const VALIDATE: &str = "visualization.validate";
}

/// Interaction domain — petalTongue user input.
pub mod interaction {
    /// Subscribe to user interaction events (clicks, keys, gestures).
    pub const SUBSCRIBE: &str = "interaction.subscribe";
    /// Poll for pending user interaction events.
    pub const POLL: &str = "interaction.poll";
}

/// Health domain — universal primal health checks.
pub mod health {
    /// Liveness probe (minimal latency, for monitoring).
    pub const LIVENESS: &str = "health.liveness";
    /// Basic health check (returns status + version).
    pub const CHECK: &str = "health.check";
}

/// Lifecycle domain — primal lifecycle management.
pub mod lifecycle {
    /// Current lifecycle status (starting, ready, degraded, stopping).
    pub const STATUS: &str = "lifecycle.status";
    /// Lifecycle health (alias for health.check).
    pub const HEALTH: &str = "lifecycle.health";
    /// Composition report.
    pub const COMPOSITION: &str = "lifecycle.composition";
    /// Register a primal with the lifecycle manager.
    pub const REGISTER: &str = "lifecycle.register";
}

/// Capability domain — capability-based routing.
pub mod capability {
    /// List all capabilities this primal exposes.
    pub const LIST: &str = "capability.list";
    /// Route a capability call to the inner dispatch pipeline.
    pub const CALL: &str = "capability.call";
    /// Deregister capabilities.
    pub const DEREGISTER: &str = "capability.deregister";
    /// Discover capability providers.
    pub const DISCOVER: &str = "capability.discover";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_constants_are_dotted() {
        let all = [
            visualization::RENDER,
            visualization::RENDER_SCENE,
            visualization::RENDER_STREAM,
            visualization::RENDER_DASHBOARD,
            visualization::RENDER_GRAMMAR,
            visualization::EXPORT,
            visualization::VALIDATE,
            interaction::SUBSCRIBE,
            interaction::POLL,
            health::LIVENESS,
            health::CHECK,
            lifecycle::STATUS,
            lifecycle::HEALTH,
            lifecycle::COMPOSITION,
            lifecycle::REGISTER,
            capability::LIST,
            capability::CALL,
            capability::DEREGISTER,
            capability::DISCOVER,
        ];
        for method in all {
            assert!(
                method.contains('.'),
                "method {method:?} should be dotted"
            );
            assert!(!method.starts_with('.'), "{method:?} starts with dot");
            assert!(!method.ends_with('.'), "{method:?} ends with dot");
        }
    }
}
