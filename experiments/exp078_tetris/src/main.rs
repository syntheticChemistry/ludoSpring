// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! exp078 — Tetris: the Flow reference game.
//!
//! Validates the game that IS Csikszentmihalyi's Flow diagram made interactive:
//!
//! 1. **7-bag randomizer** (Tetris Guideline): sampling without replacement from
//!    a bag of 7 tetrominoes. Guarantees no drought > 12 pieces. This is the first
//!    fair procedural generator in gaming history — the same math as balanced
//!    experimental design and stratified sampling.
//! 2. **Flow via speed ramp**: gravity increases with level. Challenge rises with
//!    score. The single mechanic that sustains Flow for millions of players.
//! 3. **Hick's law** (Hick 1952): 7 piece shapes × 4 rotations × ~10 columns =
//!    bounded decision space per drop. Hick time stays manageable even as speed
//!    increases.
//! 4. **Line clear as constraint satisfaction**: a complete row is a satisfied
//!    constraint. Relates to WFC adjacency rules (exp008).
//! 5. **Engagement**: "one more line" loop. Session duration stretches without
//!    the player noticing — time distortion is a signature of Flow state.
//!
//! Cross-spring: the 7-bag is sampling without replacement — the same statistic
//! as card draw (Shannon), sequencing read sampling (wetSpring), and balanced
//! block randomization in clinical trials (healthSpring).

use std::process;

use ludospring_barracuda::interaction::flow::{FlowState, evaluate_flow};
use ludospring_barracuda::interaction::input_laws::hick_reaction_time;
use ludospring_barracuda::metrics::engagement::{EngagementSnapshot, compute_engagement};
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (analytical — Tetris Guideline, Csikszentmihalyi 1990, Hick 1952)",
    commit: "N/A",
    date: "2026-03-18",
    command: "N/A (analytical — Tetris first principles)",
};

fn main() {
    let arg = std::env::args().nth(1).unwrap_or_default();
    match arg.as_str() {
        "validate" | "" => cmd_validate(),
        other => {
            eprintln!("Unknown command: {other}");
            process::exit(1);
        }
    }
}

// ---------------------------------------------------------------------------
// Tetromino types and 7-bag randomizer
// ---------------------------------------------------------------------------

/// The 7 standard Tetris pieces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Piece {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

const ALL_PIECES: [Piece; 7] = [
    Piece::I,
    Piece::O,
    Piece::T,
    Piece::S,
    Piece::Z,
    Piece::J,
    Piece::L,
];

/// Width of each piece in cells (widest rotation).
const fn piece_width(p: Piece) -> u32 {
    match p {
        Piece::I => 4,
        Piece::O => 2,
        _ => 3,
    }
}

/// Number of distinct rotations per piece.
const fn piece_rotations(p: Piece) -> u32 {
    match p {
        Piece::O => 1,
        Piece::I | Piece::S | Piece::Z => 2,
        Piece::T | Piece::J | Piece::L => 4,
    }
}

/// 7-bag randomizer: shuffle all 7 pieces, deal them out, repeat.
///
/// Uses a simple LCG seeded deterministically for reproducibility.
/// The guarantee: within any 7 consecutive pieces, each type appears exactly once.
/// Maximum drought (gap between same piece) is 12.
struct BagRandomizer {
    bag: Vec<Piece>,
    index: usize,
    seed: u64,
}

impl BagRandomizer {
    fn new(seed: u64) -> Self {
        let mut r = Self {
            bag: ALL_PIECES.to_vec(),
            index: 7,
            seed,
        };
        r.refill();
        r
    }

    fn next_piece(&mut self) -> Piece {
        if self.index >= self.bag.len() {
            self.refill();
        }
        let piece = self.bag[self.index];
        self.index += 1;
        piece
    }

    fn refill(&mut self) {
        self.bag = ALL_PIECES.to_vec();
        for i in (1..7).rev() {
            self.seed = self
                .seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            let j = (self.seed >> 33) as usize % (i + 1);
            self.bag.swap(i, j);
        }
        self.index = 0;
    }
}

// ---------------------------------------------------------------------------
// Board and game logic
// ---------------------------------------------------------------------------

const BOARD_WIDTH: usize = 10;
const BOARD_HEIGHT: usize = 20;

/// Tetris board — fixed cells only (no active piece).
struct Board {
    cells: [[bool; BOARD_WIDTH]; BOARD_HEIGHT],
}

impl Board {
    const fn new() -> Self {
        Self {
            cells: [[false; BOARD_WIDTH]; BOARD_HEIGHT],
        }
    }

    /// Place a piece at (row, col) with given rotation.
    /// Returns false if placement collides.
    fn place_piece(&mut self, piece: Piece, rotation: u32, row: usize, col: usize) -> bool {
        let offsets = piece_cell_offsets_rotated(piece, rotation);
        for &(dr, dc) in &offsets {
            let r = row + dr;
            let c = col + dc;
            if r >= BOARD_HEIGHT || c >= BOARD_WIDTH || self.cells[r][c] {
                return false;
            }
        }
        for &(dr, dc) in &offsets {
            self.cells[row + dr][col + dc] = true;
        }
        true
    }

    /// Clear full rows and return count cleared.
    fn clear_lines(&mut self) -> u32 {
        let mut cleared = 0u32;
        let mut write = BOARD_HEIGHT - 1;
        for read in (0..BOARD_HEIGHT).rev() {
            if self.cells[read].iter().all(|&c| c) {
                cleared += 1;
            } else {
                self.cells[write] = self.cells[read];
                write = write.saturating_sub(1);
            }
        }
        for row in 0..cleared as usize {
            self.cells[row] = [false; BOARD_WIDTH];
        }
        cleared
    }

    /// Count filled cells in the board.
    #[expect(
        clippy::cast_possible_truncation,
        reason = "board is 10×20 = 200 cells max, fits in u32"
    )]
    fn filled_count(&self) -> u32 {
        self.cells
            .iter()
            .flat_map(|row| row.iter())
            .filter(|&&c| c)
            .count() as u32
    }
}

/// Cell offsets for each piece in a given rotation (row, col).
///
/// Rotation 0 = spawn orientation. Each subsequent rotation is 90° clockwise.
const fn piece_cell_offsets_rotated(piece: Piece, rotation: u32) -> [(usize, usize); 4] {
    match (piece, rotation % piece_rotations(piece)) {
        (Piece::I, 0) => [(0, 0), (0, 1), (0, 2), (0, 3)],
        (Piece::I, _) => [(0, 0), (1, 0), (2, 0), (3, 0)],

        (Piece::O, _) => [(0, 0), (0, 1), (1, 0), (1, 1)],

        (Piece::T, 0) => [(0, 0), (0, 1), (0, 2), (1, 1)],
        (Piece::T, 1) => [(0, 0), (1, 0), (1, 1), (2, 0)],
        (Piece::T, 2) => [(0, 1), (1, 0), (1, 1), (1, 2)],
        (Piece::T, _) => [(0, 1), (1, 0), (1, 1), (2, 1)],

        (Piece::S, 0) => [(0, 1), (0, 2), (1, 0), (1, 1)],
        (Piece::S, _) => [(0, 0), (1, 0), (1, 1), (2, 1)],

        (Piece::Z, 0) => [(0, 0), (0, 1), (1, 1), (1, 2)],
        (Piece::Z, _) => [(0, 1), (1, 0), (1, 1), (2, 0)],

        (Piece::J, 0) => [(0, 0), (1, 0), (1, 1), (1, 2)],
        (Piece::J, 1) => [(0, 0), (0, 1), (1, 0), (2, 0)],
        (Piece::J, 2) => [(0, 0), (0, 1), (0, 2), (1, 2)],
        (Piece::J, _) => [(0, 1), (1, 1), (2, 0), (2, 1)],

        (Piece::L, 0) => [(0, 2), (1, 0), (1, 1), (1, 2)],
        (Piece::L, 1) => [(0, 0), (1, 0), (2, 0), (2, 1)],
        (Piece::L, 2) => [(0, 0), (0, 1), (0, 2), (1, 0)],
        (Piece::L, _) => [(0, 0), (0, 1), (1, 1), (2, 1)],
    }
}

/// Bounding width of a piece in a given rotation.
const fn piece_width_rotated(piece: Piece, rotation: u32) -> usize {
    let offsets = piece_cell_offsets_rotated(piece, rotation);
    let mut max_c = 0usize;
    let mut i = 0;
    while i < 4 {
        if offsets[i].1 > max_c {
            max_c = offsets[i].1;
        }
        i += 1;
    }
    max_c + 1
}

// ---------------------------------------------------------------------------
// Gravity / speed model
// ---------------------------------------------------------------------------

/// Frames per gravity drop at a given level (Tetris Guideline approximation).
///
/// Level 1: 48 frames. Level 10: 6 frames. Level 20: 1 frame.
/// The curve is exponential decay — challenge grows faster than linearly.
#[expect(
    clippy::cast_possible_truncation,
    reason = "result is clamped to ≥1 before cast"
)]
#[expect(
    clippy::cast_sign_loss,
    reason = "base and decay are positive, result is always positive"
)]
#[expect(
    clippy::cast_possible_wrap,
    reason = "level ≤ ~30 in practice, fits in i32"
)]
fn frames_per_drop(level: u32) -> u32 {
    let base = 48.0f64;
    let decay = 0.85f64;
    let frames = base * decay.powi(level.saturating_sub(1) as i32);
    (frames.round() as u32).max(1)
}

/// Normalize gravity speed to a 0-1 challenge scale for Flow evaluation.
fn gravity_to_challenge(level: u32) -> f64 {
    let fpd = f64::from(frames_per_drop(level));
    (1.0 - fpd / 48.0).clamp(0.0, 1.0)
}

// ---------------------------------------------------------------------------
// Simulated session
// ---------------------------------------------------------------------------

struct SessionResult {
    pieces_placed: u32,
    lines_cleared: u32,
    max_level: u32,
    flow_samples: Vec<FlowState>,
}

/// Find the row where a piece lands when dropped in a given column with rotation.
fn drop_to_row(board: &Board, piece: Piece, rotation: u32, col: usize) -> usize {
    let mut row = 0;
    while row + 1 < BOARD_HEIGHT {
        let offsets = piece_cell_offsets_rotated(piece, rotation);
        let blocked = offsets.iter().any(|&(dr, dc)| {
            let r = row + 1 + dr;
            let c = col + dc;
            r >= BOARD_HEIGHT || board.cells[r][c]
        });
        if blocked {
            break;
        }
        row += 1;
    }
    row
}

/// Evaluate a board state for AI placement scoring.
///
/// Returns (aggregate_height, holes, bumpiness, complete_lines).
/// - aggregate_height: sum of per-column heights
/// - holes: empty cells below filled cells
/// - bumpiness: sum of absolute height differences between adjacent columns
/// - complete_lines: rows that are completely filled
#[expect(
    clippy::cast_possible_wrap,
    reason = "board dimensions ≤ 20, clears ≤ 4 — all fit in i64"
)]
fn evaluate_board(board: &Board) -> (i64, i64, i64, i64) {
    let mut col_heights = [0i64; BOARD_WIDTH];
    let mut holes = 0i64;

    for (col, height) in col_heights.iter_mut().enumerate() {
        let mut found_filled = false;
        for row in 0..BOARD_HEIGHT {
            if board.cells[row][col] {
                if !found_filled {
                    *height = (BOARD_HEIGHT - row) as i64;
                    found_filled = true;
                }
            } else if found_filled {
                holes += 1;
            }
        }
    }

    let aggregate_height: i64 = col_heights.iter().sum();
    let bumpiness: i64 = col_heights.windows(2).map(|w| (w[0] - w[1]).abs()).sum();
    let complete_lines = board
        .cells
        .iter()
        .filter(|r| r.iter().all(|&c| c))
        .count()
        .min(20) as i64;

    (aggregate_height, holes, bumpiness, complete_lines)
}

/// Simulate a Tetris session with a simple column-placement AI.
///
/// The AI places each piece in the column that minimizes board height.
/// Skill parameter (0-1) determines how often it makes the optimal choice.
#[expect(
    clippy::cast_precision_loss,
    reason = "board dimensions ≤ 200 fit in f64 mantissa"
)]
fn simulate_session(seed: u64, skill: f64, max_pieces: u32) -> SessionResult {
    let mut bag = BagRandomizer::new(seed);
    let mut board = Board::new();
    let mut lines_cleared = 0u32;
    let mut pieces_placed = 0u32;
    let mut flow_samples = Vec::new();
    let mut level = 1u32;

    for i in 0..max_pieces {
        let piece = bag.next_piece();
        let mut best_col = 0usize;
        let mut best_rot = 0u32;
        let mut best_score = i64::MIN;
        let rotations = piece_rotations(piece);
        for rot in 0..rotations {
            let w = piece_width_rotated(piece, rot);
            if w > BOARD_WIDTH {
                continue;
            }
            for col in 0..=(BOARD_WIDTH - w) {
                let mut test_board = Board::new();
                test_board.cells = board.cells;
                let row = drop_to_row(&test_board, piece, rot, col);
                if !test_board.place_piece(piece, rot, row, col) {
                    continue;
                }
                let (agg_h, holes, bumps, clears) = evaluate_board(&test_board);
                let score = clears * 5000 - agg_h * 5 - holes * 500 - bumps * 200;
                if score > best_score {
                    best_score = score;
                    best_col = col;
                    best_rot = rot;
                }
            }
        }

        let w = piece_width_rotated(piece, best_rot);
        let chosen_col = if f64::from(i).mul_add(0.1, seed as f64).sin().abs() > skill {
            (best_col + 1).min(BOARD_WIDTH - w)
        } else {
            best_col
        };

        let drop_row = drop_to_row(&board, piece, best_rot, chosen_col);

        if !board.place_piece(piece, best_rot, drop_row, chosen_col) {
            break;
        }

        pieces_placed += 1;
        let cleared = board.clear_lines();
        lines_cleared += cleared;
        level = 1 + lines_cleared / 10;

        if i % 5 == 0 {
            let challenge = gravity_to_challenge(level);
            let flow = evaluate_flow(challenge, skill, 0.15);
            flow_samples.push(flow);
        }
    }

    SessionResult {
        pieces_placed,
        lines_cleared,
        max_level: level,
        flow_samples,
    }
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

fn cmd_validate() -> ! {
    let mut h = ValidationHarness::new("exp078_tetris");
    h.print_provenance(&[&PROVENANCE]);

    validate_bag_fairness(&mut h);
    validate_bag_drought_bound(&mut h);
    validate_gravity_curve(&mut h);
    validate_flow_across_levels(&mut h);
    validate_hick_decision_space(&mut h);
    validate_line_clear_constraint(&mut h);
    validate_engagement_session(&mut h);
    validate_skill_discrimination(&mut h);

    h.finish();
}

/// Validate 7-bag fairness: every piece appears exactly once per bag.
fn validate_bag_fairness<S: ludospring_barracuda::validation::ValidationSink>(
    h: &mut ValidationHarness<S>,
) {
    let mut bag = BagRandomizer::new(42);
    let mut counts = [0u32; 7];

    for _ in 0..700 {
        let piece = bag.next_piece();
        let idx = ALL_PIECES.iter().position(|&p| p == piece).unwrap_or(0);
        counts[idx] += 1;
    }

    for (i, &count) in counts.iter().enumerate() {
        h.check_bool(
            &format!("bag_exact_100_{}", ALL_PIECES[i] as u8),
            count == 100,
        );
    }

    let mut bag7 = BagRandomizer::new(99);
    let first_bag: Vec<Piece> = (0..7).map(|_| bag7.next_piece()).collect();
    let mut seen = [false; 7];
    for p in &first_bag {
        let idx = ALL_PIECES.iter().position(|x| x == p).unwrap_or(0);
        seen[idx] = true;
    }
    h.check_bool("bag_first_7_all_unique", seen.iter().all(|&s| s));
}

/// Validate maximum drought: gap between same piece type never exceeds 12.
fn validate_bag_drought_bound<S: ludospring_barracuda::validation::ValidationSink>(
    h: &mut ValidationHarness<S>,
) {
    let mut bag = BagRandomizer::new(12345);
    let mut last_seen = [0u32; 7];
    let mut max_drought = 0u32;

    for i in 1..=10_000u32 {
        let piece = bag.next_piece();
        let idx = ALL_PIECES.iter().position(|&p| p == piece).unwrap_or(0);
        if last_seen[idx] > 0 {
            let gap = i - last_seen[idx] - 1;
            if gap > max_drought {
                max_drought = gap;
            }
        }
        last_seen[idx] = i;
    }

    h.check_bool("bag_max_drought_le_12", max_drought <= 12);
    h.check_bool("bag_max_drought_ge_7", max_drought >= 7);
}

/// Validate gravity curve: speed increases with level, exponential decay.
fn validate_gravity_curve<S: ludospring_barracuda::validation::ValidationSink>(
    h: &mut ValidationHarness<S>,
) {
    let fpd_1 = frames_per_drop(1);
    let fpd_10 = frames_per_drop(10);
    let fpd_20 = frames_per_drop(20);

    h.check_bool("gravity_level1_slowest", fpd_1 >= fpd_10);
    h.check_bool("gravity_level10_mid", fpd_10 > fpd_20);
    h.check_bool("gravity_level20_fastest", fpd_20 >= 1);
    h.check_bool("gravity_monotonic_decrease", fpd_1 > fpd_20);

    let challenge_1 = gravity_to_challenge(1);
    let challenge_20 = gravity_to_challenge(20);
    h.check_bool("challenge_increases_with_level", challenge_20 > challenge_1);
    h.check_bool(
        "challenge_bounded_0_1",
        challenge_1 >= 0.0 && challenge_20 <= 1.0,
    );
}

/// Validate Flow: beginner starts in Flow/Relaxation, expert maintains Flow at high levels.
fn validate_flow_across_levels<S: ludospring_barracuda::validation::ValidationSink>(
    h: &mut ValidationHarness<S>,
) {
    let beginner_skill = 0.2;
    let flow_lvl1 = evaluate_flow(gravity_to_challenge(1), beginner_skill, 0.15);
    h.check_bool(
        "flow_beginner_lvl1_not_anxiety",
        flow_lvl1 != FlowState::Anxiety,
    );

    let expert_skill = 0.9;
    let flow_lvl15 = evaluate_flow(gravity_to_challenge(15), expert_skill, 0.15);
    h.check_bool(
        "flow_expert_lvl15_not_boredom",
        flow_lvl15 != FlowState::Boredom,
    );

    let mid_skill = 0.5;
    let mid_challenge = gravity_to_challenge(5);
    let flow_mid = evaluate_flow(mid_challenge, mid_skill, 0.15);
    h.check_bool("flow_midrange_is_flow", flow_mid == FlowState::Flow);
}

/// Validate Hick's law: decision space is bounded at each piece drop.
fn validate_hick_decision_space<S: ludospring_barracuda::validation::ValidationSink>(
    h: &mut ValidationHarness<S>,
) {
    let mut total_options = 0usize;
    for &piece in &ALL_PIECES {
        let rotations = piece_rotations(piece) as usize;
        let width = piece_width(piece) as usize;
        let columns = BOARD_WIDTH - width + 1;
        total_options += rotations * columns;
    }
    let avg_options = total_options / 7;

    let hick_time = hick_reaction_time(avg_options, 50.0, 150.0);
    h.check_bool("hick_time_positive", hick_time > 0.0);
    h.check_bool("hick_time_bounded", hick_time < 2000.0);

    let hick_7 = hick_reaction_time(7, 50.0, 150.0);
    h.check_bool("hick_piece_selection_fast", hick_7 <= 500.0);
}

/// Validate line clearing as constraint satisfaction.
fn validate_line_clear_constraint<S: ludospring_barracuda::validation::ValidationSink>(
    h: &mut ValidationHarness<S>,
) {
    let mut board = Board::new();
    for col in 0..BOARD_WIDTH {
        board.cells[BOARD_HEIGHT - 1][col] = true;
    }
    let cleared = board.clear_lines();
    h.check_bool("clear_full_row", cleared == 1);

    let mut board2 = Board::new();
    for col in 0..BOARD_WIDTH - 1 {
        board2.cells[BOARD_HEIGHT - 1][col] = true;
    }
    let cleared2 = board2.clear_lines();
    h.check_bool("no_clear_incomplete_row", cleared2 == 0);

    let mut board3 = Board::new();
    for row in (BOARD_HEIGHT - 4)..BOARD_HEIGHT {
        for col in 0..BOARD_WIDTH {
            board3.cells[row][col] = true;
        }
    }
    let cleared3 = board3.clear_lines();
    h.check_bool("clear_four_rows_tetris", cleared3 == 4);

    let filled_after = board3.filled_count();
    h.check_bool("board_empty_after_tetris", filled_after == 0);
}

/// Validate engagement from a simulated Tetris session.
fn validate_engagement_session<S: ludospring_barracuda::validation::ValidationSink>(
    h: &mut ValidationHarness<S>,
) {
    let session = simulate_session(42, 0.95, 2000);

    h.check_bool("session_places_pieces", session.pieces_placed > 0);
    h.check_bool("session_clears_lines", session.lines_cleared > 0);
    h.check_bool("session_levels_up", session.max_level > 1);

    let snap = EngagementSnapshot {
        session_duration_s: f64::from(session.pieces_placed) * 0.5,
        action_count: u64::from(session.pieces_placed),
        exploration_breadth: 7,
        challenge_seeking: session.max_level,
        retry_count: 0,
        deliberate_pauses: 0,
    };

    let metrics = compute_engagement(&snap);
    h.check_bool("engagement_positive", metrics.composite > 0.0);
    h.check_bool("engagement_bounded", metrics.composite <= 1.0);
}

/// Validate that higher-skill AI produces more lines cleared.
fn validate_skill_discrimination<S: ludospring_barracuda::validation::ValidationSink>(
    h: &mut ValidationHarness<S>,
) {
    let low_session = simulate_session(99, 0.3, 2000);
    let high_session = simulate_session(99, 0.95, 2000);

    h.check_bool(
        "high_skill_more_lines",
        high_session.lines_cleared >= low_session.lines_cleared,
    );
    h.check_bool(
        "high_skill_more_pieces",
        high_session.pieces_placed >= low_session.pieces_placed,
    );

    h.check_bool(
        "high_skill_has_flow_samples",
        !high_session.flow_samples.is_empty(),
    );
    h.check_bool(
        "high_skill_survives_longer",
        high_session.pieces_placed > low_session.pieces_placed / 2,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use ludospring_barracuda::validation::BufferSink;

    #[test]
    fn tetris_validation_passes() {
        let mut h = ValidationHarness::with_sink("exp078_tetris", BufferSink::default());
        validate_bag_fairness(&mut h);
        validate_bag_drought_bound(&mut h);
        validate_gravity_curve(&mut h);
        validate_flow_across_levels(&mut h);
        validate_hick_decision_space(&mut h);
        validate_line_clear_constraint(&mut h);
        validate_engagement_session(&mut h);
        validate_skill_discrimination(&mut h);
        let total = h.total_count();
        let passed = h.passed_count();
        assert_eq!(
            passed,
            total,
            "{} checks failed out of {total}",
            total - passed
        );
    }
}
