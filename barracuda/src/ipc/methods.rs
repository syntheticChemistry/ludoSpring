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
    /// Dismiss a scene.
    pub const DISMISS: &str = "visualization.dismiss";
}

/// Interaction domain — petalTongue user input.
pub mod interaction {
    /// Subscribe to user interaction events (clicks, keys, gestures).
    pub const SUBSCRIBE: &str = "interaction.subscribe";
    /// Poll for pending user interaction events.
    pub const POLL: &str = "interaction.poll";
    /// Unsubscribe from interaction events.
    pub const UNSUBSCRIBE: &str = "interaction.unsubscribe";
}

/// Health domain — universal primal health checks.
pub mod health {
    /// Liveness probe (minimal latency, for monitoring).
    pub const LIVENESS: &str = "health.liveness";
    /// Basic health check (returns status + version).
    pub const CHECK: &str = "health.check";
    /// Readiness probe (full startup complete, ready for traffic).
    pub const READINESS: &str = "health.readiness";
    /// Short-form alias used by some primals.
    pub const SHORT: &str = "health";
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
    /// Alternate spelling used by some primals (BearDog/Songbird).
    pub const LIST_ALT: &str = "capabilities.list";
    /// Route a capability call to the inner dispatch pipeline.
    pub const CALL: &str = "capability.call";
    /// Deregister capabilities.
    pub const DEREGISTER: &str = "capability.deregister";
    /// Discover capability providers.
    pub const DISCOVER: &str = "capability.discover";
}

/// Activation domain — barraCuda HCI science.
pub mod activation {
    /// Fitts's Law index of difficulty.
    pub const FITTS: &str = "activation.fitts";
    /// Hick's Law decision time.
    pub const HICK: &str = "activation.hick";
    /// Steering Law path-following cost.
    pub const STEERING: &str = "activation.steering";
    /// Sigmoid activation function.
    pub const SIGMOID: &str = "activation.sigmoid";
}

/// Math / statistics domain — barraCuda compute.
pub mod math {
    /// Sigmoid function evaluation.
    pub const SIGMOID: &str = "math.sigmoid";
    /// Arithmetic mean of a dataset.
    pub const MEAN: &str = "math.mean";
    /// Variance of a dataset.
    pub const VARIANCE: &str = "math.variance";
}

/// Noise domain — procedural generation.
pub mod noise {
    /// 2D Perlin noise.
    pub const PERLIN_2D: &str = "noise.perlin2d";
    /// 3D Perlin noise.
    pub const PERLIN_3D: &str = "noise.perlin3d";
    /// Fractal Brownian motion.
    pub const FBM: &str = "noise.fbm";
}

/// Compute domain — toadStool GPU dispatch.
pub mod compute {
    /// Dispatch a GPU compute kernel.
    pub const DISPATCH: &str = "compute.dispatch";
    /// Submit a compute job.
    pub const SUBMIT: &str = "compute.submit";
    /// Retrieve compute results.
    pub const RESULT: &str = "compute.result";
    /// Query available compute capabilities.
    pub const CAPABILITIES: &str = "compute.capabilities";
}

/// Storage domain — NestGate content-addressed storage.
pub mod storage {
    /// Store a blob.
    pub const PUT: &str = "storage.put";
    /// Retrieve a blob by key.
    pub const GET: &str = "storage.get";
    /// List stored keys.
    pub const LIST: &str = "storage.list";
}

/// DAG domain — rhizoCrypt provenance DAG.
pub mod dag {
    /// Create a new DAG session.
    pub const SESSION_CREATE: &str = "dag.session.create";
    /// Append an event to the DAG.
    pub const EVENT_APPEND: &str = "dag.event.append";
    /// Compute the Merkle root of the DAG.
    pub const MERKLE_ROOT: &str = "dag.merkle.root";
    /// Get the frontier vertices.
    pub const FRONTIER_GET: &str = "dag.frontier.get";
}

/// Braid/attribution domain — sweetGrass provenance fabric.
pub mod braid {
    /// Create a new braid entry.
    pub const CREATE: &str = "braid.create";
    /// Query braid provenance.
    pub const QUERY: &str = "braid.query";
    /// Get the full provenance graph.
    pub const PROVENANCE_GRAPH: &str = "provenance.graph";
}

/// AI/inference domain — Squirrel.
pub mod ai {
    /// Query an AI model.
    pub const QUERY: &str = "ai.query";
    /// Request AI suggestions.
    pub const SUGGEST: &str = "ai.suggest";
    /// AI-powered analysis.
    pub const ANALYZE: &str = "ai.analyze";
}

/// Spine/ledger domain — loamSpine.
pub mod spine {
    /// Create a new ledger spine.
    pub const CREATE: &str = "spine.create";
    /// Seal a spine (make immutable).
    pub const SEAL: &str = "spine.seal";
    /// Append an entry to the spine.
    pub const ENTRY_APPEND: &str = "entry.append";
}

/// Tensor domain — barraCuda tensor operations.
pub mod tensor {
    /// Create a tensor from data + shape.
    pub const CREATE: &str = "tensor.create";
    /// Matrix multiplication of two tensors.
    pub const MATMUL: &str = "tensor.matmul";
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
            visualization::DISMISS,
            interaction::SUBSCRIBE,
            interaction::POLL,
            interaction::UNSUBSCRIBE,
            health::LIVENESS,
            health::CHECK,
            health::READINESS,
            lifecycle::STATUS,
            lifecycle::HEALTH,
            lifecycle::COMPOSITION,
            lifecycle::REGISTER,
            capability::LIST,
            capability::LIST_ALT,
            capability::CALL,
            capability::DEREGISTER,
            capability::DISCOVER,
            activation::FITTS,
            activation::HICK,
            activation::STEERING,
            activation::SIGMOID,
            math::SIGMOID,
            math::MEAN,
            math::VARIANCE,
            noise::PERLIN_2D,
            noise::PERLIN_3D,
            noise::FBM,
            compute::DISPATCH,
            compute::SUBMIT,
            compute::RESULT,
            compute::CAPABILITIES,
            storage::PUT,
            storage::GET,
            storage::LIST,
            dag::SESSION_CREATE,
            dag::EVENT_APPEND,
            dag::MERKLE_ROOT,
            dag::FRONTIER_GET,
            braid::CREATE,
            braid::QUERY,
            braid::PROVENANCE_GRAPH,
            ai::QUERY,
            ai::SUGGEST,
            ai::ANALYZE,
            spine::CREATE,
            spine::SEAL,
            spine::ENTRY_APPEND,
            tensor::CREATE,
            tensor::MATMUL,
        ];
        for method in all {
            assert!(method.contains('.'), "method {method:?} should be dotted");
            assert!(!method.starts_with('.'), "{method:?} starts with dot");
            assert!(!method.ends_with('.'), "{method:?} ends with dot");
        }
    }

    #[test]
    fn health_constants_match_capability_domains_fqn() {
        let fqns = crate::capability_domains::all_methods();
        assert!(
            fqns.contains(&health::LIVENESS),
            "capability_domains missing {}",
            health::LIVENESS
        );
        assert!(
            fqns.contains(&health::READINESS),
            "capability_domains missing {}",
            health::READINESS
        );
    }

    #[test]
    fn health_constants_match_niche_capabilities() {
        assert!(
            crate::niche::CAPABILITIES.contains(&health::LIVENESS),
            "niche CAPABILITIES missing {}",
            health::LIVENESS
        );
        assert!(
            crate::niche::CAPABILITIES.contains(&health::READINESS),
            "niche CAPABILITIES missing {}",
            health::READINESS
        );
    }
}
