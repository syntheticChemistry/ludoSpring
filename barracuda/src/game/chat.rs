// SPDX-License-Identifier: AGPL-3.0-or-later
//! BM-005: Chat / messaging — multiplayer communication channel model.
//!
//! Game chat is message passing with constraints: latency budgets, rate
//! limiting, content filtering, and fan-out to lobby members. This module
//! models the message pipeline and provides throughput metrics.
//!
//! # BM-005 Benchmark Target
//!
//! Chat throughput: N messages/second through the pipeline (receive →
//! validate → filter → fan-out). The pipeline must stay within the
//! game tick budget (16.6ms at 60Hz) without blocking the game loop.
//!
//! # Design
//!
//! Messages flow through a staged pipeline:
//! 1. **Receive** — accept from sender, assign sequence number
//! 2. **Validate** — check length, encoding, rate limit
//! 3. **Filter** — content moderation (placeholder for ML-based filters)
//! 4. **Fan-out** — deliver to all recipients in the channel
//!
//! The pipeline is modeled as pure functions on immutable messages,
//! suitable for benchmarking without I/O.

use std::collections::VecDeque;

/// A chat message in the pipeline.
#[derive(Debug, Clone)]
pub struct ChatMessage {
    /// Sender identifier.
    pub sender_id: u64,
    /// Channel/lobby this message belongs to.
    pub channel_id: u64,
    /// Message body (UTF-8 text).
    pub body: String,
    /// Monotonic sequence number within the channel.
    pub seq: u64,
    /// Timestamp (monotonic nanoseconds from session start).
    pub timestamp_ns: u64,
}

/// Validation result for a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationResult {
    /// Message is valid and can proceed.
    Valid,
    /// Message exceeds maximum length.
    TooLong,
    /// Message is empty.
    Empty,
    /// Sender exceeded rate limit.
    RateLimited,
}

/// Chat channel configuration.
#[derive(Debug, Clone)]
pub struct ChannelConfig {
    /// Maximum message body length in bytes.
    pub max_message_len: usize,
    /// Maximum messages per sender per second.
    pub rate_limit_per_sec: u32,
    /// Number of recipients in this channel.
    pub recipient_count: u32,
}

impl Default for ChannelConfig {
    fn default() -> Self {
        Self {
            max_message_len: 500,
            rate_limit_per_sec: 5,
            recipient_count: 4,
        }
    }
}

/// Rate limiter using a sliding window (token bucket).
#[derive(Debug)]
pub struct RateLimiter {
    window_ns: u64,
    max_tokens: u32,
    timestamps: VecDeque<u64>,
}

impl RateLimiter {
    /// Create a rate limiter allowing `max_per_sec` messages per second.
    #[must_use]
    pub const fn new(max_per_sec: u32) -> Self {
        Self {
            window_ns: 1_000_000_000,
            max_tokens: max_per_sec,
            timestamps: VecDeque::new(),
        }
    }

    /// Check if a message at the given timestamp is allowed.
    pub fn check(&mut self, timestamp_ns: u64) -> bool {
        let window_start = timestamp_ns.saturating_sub(self.window_ns);
        while self.timestamps.front().is_some_and(|&t| t < window_start) {
            self.timestamps.pop_front();
        }

        if self.timestamps.len() < self.max_tokens as usize {
            self.timestamps.push_back(timestamp_ns);
            true
        } else {
            false
        }
    }

    /// Reset the limiter state.
    pub fn reset(&mut self) {
        self.timestamps.clear();
    }
}

/// Validate a message against channel constraints.
#[must_use]
pub const fn validate_message(msg: &ChatMessage, config: &ChannelConfig) -> ValidationResult {
    if msg.body.is_empty() {
        return ValidationResult::Empty;
    }
    if msg.body.len() > config.max_message_len {
        return ValidationResult::TooLong;
    }
    ValidationResult::Valid
}

/// Fan-out cost estimate: how many deliveries a message generates.
#[must_use]
pub const fn fanout_cost(config: &ChannelConfig) -> u32 {
    config.recipient_count.saturating_sub(1)
}

/// Pipeline throughput metrics for a batch of messages.
#[derive(Debug, Clone)]
pub struct PipelineMetrics {
    /// Total messages submitted.
    pub submitted: u64,
    /// Messages that passed validation.
    pub validated: u64,
    /// Messages rejected by validation.
    pub rejected: u64,
    /// Messages rate-limited.
    pub rate_limited: u64,
    /// Total fan-out deliveries.
    pub deliveries: u64,
}

/// Process a batch of messages through the pipeline, returning metrics.
pub fn process_batch(
    messages: &[ChatMessage],
    config: &ChannelConfig,
    limiter: &mut RateLimiter,
) -> PipelineMetrics {
    let mut metrics = PipelineMetrics {
        submitted: messages.len() as u64,
        validated: 0,
        rejected: 0,
        rate_limited: 0,
        deliveries: 0,
    };

    let fanout = u64::from(fanout_cost(config));

    for msg in messages {
        match validate_message(msg, config) {
            ValidationResult::Valid => {
                if limiter.check(msg.timestamp_ns) {
                    metrics.validated += 1;
                    metrics.deliveries += fanout;
                } else {
                    metrics.rate_limited += 1;
                }
            }
            _ => {
                metrics.rejected += 1;
            }
        }
    }

    metrics
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions use unwrap for clarity")]
mod tests {
    use super::*;

    fn test_message(sender: u64, body: &str, seq: u64, ts_ns: u64) -> ChatMessage {
        ChatMessage {
            sender_id: sender,
            channel_id: 1,
            body: body.to_owned(),
            seq,
            timestamp_ns: ts_ns,
        }
    }

    #[test]
    fn valid_message_passes() {
        let config = ChannelConfig::default();
        let msg = test_message(1, "hello world", 1, 0);
        assert_eq!(validate_message(&msg, &config), ValidationResult::Valid);
    }

    #[test]
    fn empty_message_rejected() {
        let config = ChannelConfig::default();
        let msg = test_message(1, "", 1, 0);
        assert_eq!(validate_message(&msg, &config), ValidationResult::Empty);
    }

    #[test]
    fn too_long_message_rejected() {
        let config = ChannelConfig {
            max_message_len: 10,
            ..Default::default()
        };
        let msg = test_message(1, "this message is way too long for the limit", 1, 0);
        assert_eq!(validate_message(&msg, &config), ValidationResult::TooLong);
    }

    #[test]
    fn rate_limiter_allows_within_limit() {
        let mut limiter = RateLimiter::new(3);
        assert!(limiter.check(100));
        assert!(limiter.check(200));
        assert!(limiter.check(300));
    }

    #[test]
    fn rate_limiter_blocks_over_limit() {
        let mut limiter = RateLimiter::new(2);
        assert!(limiter.check(100));
        assert!(limiter.check(200));
        assert!(!limiter.check(300));
    }

    #[test]
    fn rate_limiter_window_slides() {
        let mut limiter = RateLimiter::new(2);
        assert!(limiter.check(0));
        assert!(limiter.check(500_000_000));
        assert!(!limiter.check(600_000_000));
        assert!(limiter.check(1_100_000_000));
    }

    #[test]
    fn rate_limiter_reset_clears_state() {
        let mut limiter = RateLimiter::new(1);
        assert!(limiter.check(0));
        assert!(!limiter.check(1));
        limiter.reset();
        assert!(limiter.check(2));
    }

    #[test]
    fn fanout_cost_excludes_sender() {
        let config = ChannelConfig {
            recipient_count: 4,
            ..Default::default()
        };
        assert_eq!(fanout_cost(&config), 3);
    }

    #[test]
    fn fanout_cost_solo_is_zero() {
        let config = ChannelConfig {
            recipient_count: 1,
            ..Default::default()
        };
        assert_eq!(fanout_cost(&config), 0);
    }

    #[test]
    fn process_batch_counts_correctly() {
        let config = ChannelConfig::default();
        let mut limiter = RateLimiter::new(10);
        let messages: Vec<ChatMessage> = (0..5)
            .map(|i| test_message(1, "hello", i, i * 100_000_000))
            .collect();

        let metrics = process_batch(&messages, &config, &mut limiter);
        assert_eq!(metrics.submitted, 5);
        assert_eq!(metrics.validated, 5);
        assert_eq!(metrics.rejected, 0);
        assert_eq!(metrics.rate_limited, 0);
        assert_eq!(metrics.deliveries, 5 * u64::from(fanout_cost(&config)));
    }

    #[test]
    fn process_batch_rate_limits() {
        let config = ChannelConfig {
            rate_limit_per_sec: 2,
            ..Default::default()
        };
        let mut limiter = RateLimiter::new(2);
        let messages: Vec<ChatMessage> = (0..5)
            .map(|i| test_message(1, "hello", i, i * 100))
            .collect();

        let metrics = process_batch(&messages, &config, &mut limiter);
        assert_eq!(metrics.validated, 2);
        assert_eq!(metrics.rate_limited, 3);
    }

    #[test]
    fn process_batch_rejects_invalid() {
        let config = ChannelConfig::default();
        let mut limiter = RateLimiter::new(10);
        let messages = vec![
            test_message(1, "valid", 1, 0),
            test_message(1, "", 2, 100),
            test_message(1, "also valid", 3, 200),
        ];

        let metrics = process_batch(&messages, &config, &mut limiter);
        assert_eq!(metrics.validated, 2);
        assert_eq!(metrics.rejected, 1);
    }
}
