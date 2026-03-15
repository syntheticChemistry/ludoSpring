// SPDX-License-Identifier: AGPL-3.0-or-later
//! Exp025: Roguelike Parameter Explorer — science as a game.
//!
//! Procedurally generated dungeon where engagement metrics guide generation,
//! DDA tunes difficulty, and each run explores a different region of the math.
//!
//! Uses validated ludoSpring models:
//! - `procedural::bsp` for room layout (Fuchs 1980)
//! - `procedural::noise` for terrain variation (Perlin 1985)
//! - `interaction::difficulty` for DDA (Hunicke 2005)
//! - `interaction::flow` for Flow state (Csikszentmihalyi 1990)
//! - `metrics::engagement` for session quality (Yannakakis 2018)
//! - `metrics::fun_keys` for fun classification (Lazzaro 2004)
//!
//! The same turn-based loop works for chess variants, music composition
//! (WFC for chord progressions), educational software, adaptive assessments.
//!
//! Controls: WASD/arrows move, . wait, i stats, Esc quit
#![forbid(unsafe_code)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    clippy::trivially_copy_pass_by_ref
)]

use std::io::{self, Write as _};
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{cursor, execute};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use ludospring_barracuda::interaction::difficulty::{PerformanceWindow, suggest_adjustment};
use ludospring_barracuda::interaction::flow::{FlowState, evaluate_flow};
use ludospring_barracuda::metrics::engagement::{EngagementSnapshot, compute_engagement};
use ludospring_barracuda::metrics::fun_keys::{FunSignals, classify_fun};
use ludospring_barracuda::procedural::bsp::{Rect, generate_bsp};
use ludospring_barracuda::procedural::noise::perlin_2d;
use ludospring_barracuda::telemetry::events::{
    ChallengePayload, EventType, ExplorationPayload, PlayerActionPayload, PlayerMovePayload,
    SessionEndPayload, SessionStartPayload, TelemetryEvent,
};
use ludospring_barracuda::tolerances;

const MAP_W: usize = 40;
const MAP_H: usize = 30;
const FOV_RADIUS: i32 = 8;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tile {
    Wall,
    Floor,
    Stairs,
    Item,
}

struct Player {
    x: usize,
    y: usize,
    hp: i32,
    max_hp: i32,
    items: u32,
    floor: u32,
}

struct Session {
    actions: u64,
    start: Instant,
    explored: Vec<bool>,
    challenges: u32,
    retries: u32,
    pauses: u32,
    perf: PerformanceWindow,
    difficulty_mod: f64,
}

fn main() -> io::Result<()> {
    run()
}

#[expect(clippy::too_many_lines, reason = "validation orchestrator")]
fn run() -> io::Result<()> {
    let seed: u64 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(42);

    let telemetry_path = format!("exp025_session_{seed}.ndjson");
    let sid = format!("roguelike-{seed}");
    let mut tfile = std::fs::File::create(&telemetry_path)?;

    let mut player = Player {
        x: 0,
        y: 0,
        hp: 20,
        max_hp: 20,
        items: 0,
        floor: 1,
    };
    let mut session = Session {
        actions: 0,
        start: Instant::now(),
        explored: vec![false; MAP_W * MAP_H],
        challenges: 0,
        retries: 0,
        pauses: 0,
        perf: PerformanceWindow::new(20),
        difficulty_mod: 0.0,
    };

    emit_event(
        &mut tfile,
        0,
        &sid,
        EventType::SessionStart,
        &SessionStartPayload {
            game_name: "exp025_roguelike_explorer".into(),
            game_version: "0.1.0".into(),
            genre: "roguelike".into(),
        },
    );

    let mut map = generate_floor(seed, player.floor, session.difficulty_mod);
    spawn_player(&map, &mut player);
    let mut show_stats = false;
    let mut messages: Vec<String> = vec![format!(
        "Floor {}: explore the dungeon. Find the stairs (>).",
        player.floor
    )];

    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;
    let backend = CrosstermBackend::new(stdout);
    let mut term = Terminal::new(backend)?;

    let mut running = true;

    while running {
        let ts_ms = session.start.elapsed().as_millis() as u64;

        term.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(1),
                    Constraint::Length(3),
                    Constraint::Length(3),
                ])
                .split(f.area());

            let view = render_map(&map, &player, &session.explored);
            let map_widget =
                Paragraph::new(view).block(Block::default().borders(Borders::ALL).title(format!(
                    " Floor {} | HP {}/{} | Items {} ",
                    player.floor, player.hp, player.max_hp, player.items
                )));
            f.render_widget(map_widget, chunks[0]);

            let flow = evaluate_flow(
                session.difficulty_mod.mul_add(0.3, 0.5),
                session.perf.estimated_skill().max(0.1),
                tolerances::FLOW_CHANNEL_WIDTH,
            );
            let hud = format!(
                " Flow: {} | DDA: {:+.2} | Skill: {:.2} | Actions: {} ",
                flow.as_str(),
                session.difficulty_mod,
                session.perf.estimated_skill(),
                session.actions,
            );
            let hud_widget = Paragraph::new(Line::from(vec![Span::styled(
                hud,
                Style::default().fg(flow_color(&flow)),
            )]))
            .block(Block::default().borders(Borders::TOP));
            f.render_widget(hud_widget, chunks[1]);

            let msg = if show_stats {
                format_stats(&player, &session)
            } else {
                messages.last().cloned().unwrap_or_default()
            };
            let msg_widget = Paragraph::new(Line::from(vec![Span::styled(
                format!(" {msg}"),
                Style::default().fg(Color::Yellow),
            )]));
            f.render_widget(msg_widget, chunks[2]);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    show_stats = false;
                    match key.code {
                        KeyCode::Esc => running = false,
                        KeyCode::Char('i') => show_stats = true,
                        KeyCode::Char('.') => {
                            session.actions += 1;
                            session.pauses += 1;
                            messages.push("You wait...".into());
                            emit_event(
                                &mut tfile,
                                ts_ms,
                                &sid,
                                EventType::PlayerAction,
                                &PlayerActionPayload {
                                    action: "wait".into(),
                                    success: true,
                                    ..Default::default()
                                },
                            );
                        }
                        KeyCode::Char('w') | KeyCode::Up => {
                            try_move(
                                &map,
                                &mut player,
                                &mut session,
                                &mut messages,
                                &mut tfile,
                                &sid,
                                ts_ms,
                                0,
                                -1,
                            );
                        }
                        KeyCode::Char('s') | KeyCode::Down => {
                            try_move(
                                &map,
                                &mut player,
                                &mut session,
                                &mut messages,
                                &mut tfile,
                                &sid,
                                ts_ms,
                                0,
                                1,
                            );
                        }
                        KeyCode::Char('a') | KeyCode::Left => {
                            try_move(
                                &map,
                                &mut player,
                                &mut session,
                                &mut messages,
                                &mut tfile,
                                &sid,
                                ts_ms,
                                -1,
                                0,
                            );
                        }
                        KeyCode::Char('d') | KeyCode::Right => {
                            try_move(
                                &map,
                                &mut player,
                                &mut session,
                                &mut messages,
                                &mut tfile,
                                &sid,
                                ts_ms,
                                1,
                                0,
                            );
                        }
                        _ => {}
                    }

                    if map[player.y * MAP_W + player.x] == Tile::Stairs {
                        let adj =
                            suggest_adjustment(&session.perf, tolerances::DDA_TARGET_SUCCESS_RATE);
                        session.difficulty_mod += adj * 0.3;
                        session.difficulty_mod = session.difficulty_mod.clamp(-1.0, 1.0);

                        emit_event(
                            &mut tfile,
                            ts_ms,
                            &sid,
                            EventType::ChallengeComplete,
                            &ChallengePayload {
                                challenge_id: format!("floor_{}", player.floor),
                                difficulty: session.difficulty_mod.mul_add(0.3, 0.5),
                                challenge_type: "floor_clear".into(),
                            },
                        );

                        player.floor += 1;
                        player.hp = (player.hp + 5).min(player.max_hp);

                        emit_event(
                            &mut tfile,
                            ts_ms,
                            &sid,
                            EventType::ExplorationDiscover,
                            &ExplorationPayload {
                                discovery_id: format!("floor_{}", player.floor),
                                category: "floor".into(),
                                x: player.x as f64,
                                y: player.y as f64,
                            },
                        );

                        session.explored = vec![false; MAP_W * MAP_H];
                        map = generate_floor(
                            seed.wrapping_add(u64::from(player.floor)),
                            player.floor,
                            session.difficulty_mod,
                        );
                        spawn_player(&map, &mut player);

                        messages.push(format!(
                            "Floor {}! DDA adjustment: {:+.2} (difficulty_mod={:.2})",
                            player.floor, adj, session.difficulty_mod
                        ));
                    }
                }
            }
        }
    }

    let duration_s = session.start.elapsed().as_secs_f64();
    let final_ts = session.start.elapsed().as_millis() as u64;
    emit_event(
        &mut tfile,
        final_ts,
        &sid,
        EventType::SessionEnd,
        &SessionEndPayload {
            duration_s,
            reason: "quit".into(),
        },
    );

    terminal::disable_raw_mode()?;
    execute!(term.backend_mut(), LeaveAlternateScreen, cursor::Show)?;

    print_session_summary(&player, &session);
    println!("  Telemetry: {telemetry_path}");
    println!("  Analyze: cargo run -p ludospring-exp026 -- analyze {telemetry_path}");

    Ok(())
}

#[expect(
    clippy::too_many_arguments,
    clippy::similar_names,
    reason = "domain nomenclature"
)]
fn try_move(
    map: &[Tile],
    player: &mut Player,
    session: &mut Session,
    messages: &mut Vec<String>,
    tfile: &mut std::fs::File,
    sid: &str,
    ts_ms: u64,
    dx: i32,
    dy: i32,
) {
    session.actions += 1;

    let nx = player.x as i32 + dx;
    let ny = player.y as i32 + dy;

    if nx < 0 || ny < 0 || nx >= MAP_W as i32 || ny >= MAP_H as i32 {
        return;
    }

    let (nx, ny) = (nx as usize, ny as usize);
    let tile = map[ny * MAP_W + nx];

    if tile == Tile::Wall {
        messages.push("Bump! Wall blocks your path.".into());
        session.perf.record(0.0);
        emit_event(
            tfile,
            ts_ms,
            sid,
            EventType::ChallengeFail,
            &ChallengePayload {
                challenge_id: format!("wall_{nx}_{ny}"),
                difficulty: 0.1,
                challenge_type: "navigation".into(),
            },
        );
        return;
    }

    player.x = nx;
    player.y = ny;
    session.explored[ny * MAP_W + nx] = true;
    session.perf.record(1.0);

    emit_event(
        tfile,
        ts_ms,
        sid,
        EventType::PlayerMove,
        &PlayerMovePayload {
            x: nx as f64,
            y: ny as f64,
            ..Default::default()
        },
    );

    match tile {
        Tile::Item => {
            player.items += 1;
            session.challenges += 1;
            messages.push(format!("Found an item! ({} total)", player.items));
            emit_event(
                tfile,
                ts_ms,
                sid,
                EventType::PlayerAction,
                &PlayerActionPayload {
                    action: "collect".into(),
                    success: true,
                    target: "item".into(),
                },
            );
            emit_event(
                tfile,
                ts_ms,
                sid,
                EventType::ChallengeEncounter,
                &ChallengePayload {
                    challenge_id: format!("item_{nx}_{ny}"),
                    difficulty: 0.3,
                    challenge_type: "collection".into(),
                },
            );
            emit_event(
                tfile,
                ts_ms,
                sid,
                EventType::ChallengeComplete,
                &ChallengePayload {
                    challenge_id: format!("item_{nx}_{ny}"),
                    difficulty: 0.3,
                    challenge_type: "collection".into(),
                },
            );
        }
        Tile::Stairs => {
            messages.push("You found the stairs! Descending...".into());
        }
        _ => {}
    }
}

fn emit_event<P: serde::Serialize>(
    file: &mut std::fs::File,
    ts: u64,
    session_id: &str,
    event_type: EventType,
    payload: &P,
) {
    let evt = TelemetryEvent {
        timestamp_ms: ts,
        session_id: session_id.into(),
        event_type,
        payload: serde_json::to_value(payload).unwrap_or_default(),
    };
    if let Ok(json) = serde_json::to_string(&evt) {
        let _ = writeln!(file, "{json}");
    }
}

fn generate_floor(seed: u64, floor: u32, difficulty_mod: f64) -> Vec<Tile> {
    let bounds = Rect::new(1.0, 1.0, (MAP_W - 2) as f64, (MAP_H - 2) as f64);
    let min_size = difficulty_mod.mul_add(-2.0, 10.0).max(5.0);
    let tree = generate_bsp(bounds, min_size, seed);
    let leaves = tree.leaves();

    let mut tiles = vec![Tile::Wall; MAP_W * MAP_H];

    for leaf in &leaves {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "map coords"
        )]
        let (x0, y0) = ((leaf.x + 1.0) as usize, (leaf.y + 1.0) as usize);
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "map coords"
        )]
        let (x1, y1) = (
            ((leaf.x + leaf.w - 1.0) as usize).min(MAP_W - 1),
            ((leaf.y + leaf.h - 1.0) as usize).min(MAP_H - 1),
        );
        for y in y0..y1 {
            for x in x0..x1 {
                tiles[y * MAP_W + x] = Tile::Floor;
            }
        }
    }

    // Corridors
    for i in 0..leaves.len().saturating_sub(1) {
        let (cx1, cy1) = leaves[i].center();
        let (cx2, cy2) = leaves[i + 1].center();
        carve_corridor(&mut tiles, cx1, cy1, cx2, cy2);
    }

    // Place items using Perlin noise density
    let item_threshold = difficulty_mod.mul_add(-0.1, 0.3);
    for y in 1..MAP_H - 1 {
        for x in 1..MAP_W - 1 {
            if tiles[y * MAP_W + x] == Tile::Floor {
                let noise_val = perlin_2d(
                    (x as f64).mul_add(0.3, seed as f64 * 0.01),
                    (y as f64).mul_add(0.3, f64::from(floor) * 7.0),
                );
                if noise_val > item_threshold {
                    tiles[y * MAP_W + x] = Tile::Item;
                }
            }
        }
    }

    // Place stairs in the last room
    if let Some(last) = leaves.last() {
        let (cx, cy) = last.center();
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "map coords"
        )]
        let (sx, sy) = (cx as usize, cy as usize);
        if sx < MAP_W && sy < MAP_H {
            tiles[sy * MAP_W + sx] = Tile::Stairs;
        }
    }

    tiles
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "map coords"
)]
fn carve_corridor(tiles: &mut [Tile], x1: f64, y1: f64, x2: f64, y2: f64) {
    let (mut cx, mut cy) = (x1 as usize, y1 as usize);
    let (tx, ty) = (x2 as usize, y2 as usize);

    while cx != tx {
        if cx < MAP_W && cy < MAP_H && tiles[cy * MAP_W + cx] == Tile::Wall {
            tiles[cy * MAP_W + cx] = Tile::Floor;
        }
        if cx < tx {
            cx += 1;
        } else {
            cx -= 1;
        }
    }
    while cy != ty {
        if cx < MAP_W && cy < MAP_H && tiles[cy * MAP_W + cx] == Tile::Wall {
            tiles[cy * MAP_W + cx] = Tile::Floor;
        }
        if cy < ty {
            cy += 1;
        } else {
            cy -= 1;
        }
    }
}

fn spawn_player(map: &[Tile], player: &mut Player) {
    for y in 2..MAP_H - 2 {
        for x in 2..MAP_W - 2 {
            if map[y * MAP_W + x] != Tile::Wall {
                player.x = x;
                player.y = y;
                return;
            }
        }
    }
}

fn render_map(map: &[Tile], player: &Player, explored: &[bool]) -> Vec<Line<'static>> {
    let mut lines = Vec::with_capacity(MAP_H);

    for y in 0..MAP_H {
        let mut spans = Vec::with_capacity(MAP_W);
        for x in 0..MAP_W {
            let dx = x as i32 - player.x as i32;
            let dy = y as i32 - player.y as i32;
            let dist_sq = dx * dx + dy * dy;
            let in_fov = dist_sq <= FOV_RADIUS * FOV_RADIUS;
            let was_explored = explored[y * MAP_W + x];

            if x == player.x && y == player.y {
                spans.push(Span::styled(
                    "@",
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ));
            } else if in_fov {
                let (ch, color) = tile_glyph(map[y * MAP_W + x], true);
                spans.push(Span::styled(String::from(ch), Style::default().fg(color)));
            } else if was_explored {
                let (ch, _) = tile_glyph(map[y * MAP_W + x], false);
                spans.push(Span::styled(
                    String::from(ch),
                    Style::default().fg(Color::DarkGray),
                ));
            } else {
                spans.push(Span::raw(" "));
            }
        }
        lines.push(Line::from(spans));
    }

    lines
}

const fn tile_glyph(tile: Tile, lit: bool) -> (char, Color) {
    match tile {
        Tile::Wall => ('#', if lit { Color::Gray } else { Color::DarkGray }),
        Tile::Floor => ('.', if lit { Color::White } else { Color::DarkGray }),
        Tile::Stairs => ('>', if lit { Color::Magenta } else { Color::DarkGray }),
        Tile::Item => ('*', if lit { Color::Yellow } else { Color::DarkGray }),
    }
}

const fn flow_color(flow: &FlowState) -> Color {
    match flow {
        FlowState::Flow => Color::Green,
        FlowState::Relaxation => Color::Cyan,
        FlowState::Arousal => Color::Yellow,
        FlowState::Boredom => Color::Blue,
        FlowState::Anxiety => Color::Red,
    }
}

fn format_stats(player: &Player, session: &Session) -> String {
    let explored_pct =
        session.explored.iter().filter(|&&e| e).count() * 100 / session.explored.len().max(1);
    format!(
        "Floor {} | HP {}/{} | Items {} | Explored {}% | Actions {} | Skill {:.2}",
        player.floor,
        player.hp,
        player.max_hp,
        player.items,
        explored_pct,
        session.actions,
        session.perf.estimated_skill(),
    )
}

fn print_session_summary(player: &Player, session: &Session) {
    let duration = session.start.elapsed().as_secs_f64();
    let explored_count = session.explored.iter().filter(|&&e| e).count();

    println!("\n════════════════════════════════════════════════════════════");
    println!("  Exp025: Roguelike Explorer — Session Summary");
    println!("════════════════════════════════════════════════════════════\n");
    println!("  Floors completed: {}", player.floor.saturating_sub(1));
    println!("  Items collected:  {}", player.items);
    println!("  Actions taken:    {}", session.actions);
    println!("  Duration:         {duration:.1}s");
    println!("  Final difficulty:  {:+.2}", session.difficulty_mod);
    println!("  Estimated skill:   {:.2}", session.perf.estimated_skill());
    println!();

    let snap = EngagementSnapshot {
        session_duration_s: duration,
        action_count: session.actions,
        #[expect(clippy::cast_possible_truncation, reason = "explored count fits u32")]
        exploration_breadth: explored_count as u32,
        challenge_seeking: session.challenges,
        retry_count: session.retries,
        deliberate_pauses: session.pauses,
    };
    let eng = compute_engagement(&snap);

    println!("  Engagement metrics (Yannakakis 2018):");
    println!("    APM:             {:.1}", eng.actions_per_minute);
    println!("    Exploration:     {:.2}", eng.exploration_rate);
    println!("    Challenge:       {:.2}", eng.challenge_appetite);
    println!("    Persistence:     {:.2}", eng.persistence);
    println!("    Deliberation:    {:.2}", eng.deliberation);
    println!("    Composite:       {:.3}", eng.composite);
    println!();

    let fun = classify_fun(&FunSignals {
        challenge: session.perf.estimated_skill(),
        exploration: eng.exploration_rate,
        social: 0.0,
        completion: f64::from(player.items) / 20.0,
        retry_rate: f64::from(session.retries) / session.actions.max(1) as f64,
    });
    println!("  Fun classification (Lazzaro 2004):");
    println!("    Dominant: {} Fun", fun.dominant.as_str());
    println!(
        "    Hard={:.2} Easy={:.2} People={:.2} Serious={:.2}",
        fun.scores.hard, fun.scores.easy, fun.scores.people, fun.scores.serious
    );

    let flow = evaluate_flow(
        session.difficulty_mod.mul_add(0.3, 0.5),
        session.perf.estimated_skill().max(0.1),
        tolerances::FLOW_CHANNEL_WIDTH,
    );
    println!("  Final flow state: {}", flow.as_str());

    let adj = suggest_adjustment(&session.perf, tolerances::DDA_TARGET_SUCCESS_RATE);
    println!("  DDA recommendation: {adj:+.2}");
    println!();
}
