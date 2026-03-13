// SPDX-License-Identifier: AGPL-3.0-or-later
//! Exp024: Doom-in-a-Terminal — playable first-person walker.
//!
//! Wires validated ludoSpring math into a playable terminal game:
//! - `procedural::bsp` for level generation (Fuchs, Kedem & Naylor 1980)
//! - `game::raycaster` for DDA ray marching (Carmack 1993)
//! - `metrics::tufte_gaming` for HUD quality analysis (Tufte 1983)
//! - ratatui for terminal rendering, crossterm for input
//!
//! The same raycaster works for molecular cave navigation. The same BSP
//! works for office layouts, museum floor plans, warehouse routing.
//! Terminal rendering = zero GPU dependency, runs on any SSH session.
//!
//! Controls: WASD move/strafe, Q/E or ←/→ rotate, Esc quit
#![forbid(unsafe_code)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]

use std::io::{self, Write as _};
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{cursor, execute};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use ludospring_barracuda::game::raycaster::{GridMap, RayHit, RayPlayer, cast_screen};
use ludospring_barracuda::metrics::tufte_gaming::{UiElement, analyze_game_ui};
use ludospring_barracuda::procedural::bsp::{Rect, generate_bsp};
use ludospring_barracuda::telemetry::events::{
    EventType, ExplorationPayload, InputRawPayload, PlayerMovePayload, SessionEndPayload,
    SessionStartPayload, TelemetryEvent,
};

const MAP_W: usize = 48;
const MAP_H: usize = 48;
const MAX_DEPTH: f64 = 20.0;
const TICK_HZ: u64 = 30;

#[expect(clippy::too_many_lines, reason = "game loop — setup, run, teardown")]
fn main() -> io::Result<()> {
    let seed: u64 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(42);

    let telemetry_path = format!("exp024_session_{seed}.ndjson");
    let sid = format!("doom-terminal-{seed}");

    let map = generate_level(seed);
    let (start_x, start_y) = find_spawn(&map);

    let mut player = RayPlayer {
        x: start_x,
        y: start_y,
        angle: 0.0,
        fov: std::f64::consts::FRAC_PI_3,
        speed: 3.0,
        turn_speed: 2.5,
    };

    tufte_analysis();

    let mut tfile = std::fs::File::create(&telemetry_path)?;
    let mut visited_cells: std::collections::HashSet<(usize, usize)> =
        std::collections::HashSet::new();

    emit_event(
        &mut tfile,
        0,
        &sid,
        EventType::SessionStart,
        &SessionStartPayload {
            game_name: "exp024_doom_terminal".into(),
            game_version: "0.1.0".into(),
            genre: "fps_walker".into(),
        },
    );

    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;
    let backend = CrosstermBackend::new(stdout);
    let mut term = Terminal::new(backend)?;

    let tick_dur = Duration::from_millis(1000 / TICK_HZ);
    let mut running = true;
    let mut frame_count: u64 = 0;
    let game_start = Instant::now();

    while running {
        let frame_start = Instant::now();
        let ts_ms = game_start.elapsed().as_millis() as u64;

        if event::poll(Duration::ZERO)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    emit_event(
                        &mut tfile,
                        ts_ms,
                        &sid,
                        EventType::InputRaw,
                        &InputRawPayload {
                            input_type: "key".into(),
                            key: format!("{key:?}"),
                            pressed: true,
                        },
                    );
                    running = handle_input(key.code, &mut player, &map, tick_dur);
                }
            }
        }

        emit_event(
            &mut tfile,
            ts_ms,
            &sid,
            EventType::PlayerMove,
            &PlayerMovePayload {
                x: player.x,
                y: player.y,
                angle: player.angle,
                speed: player.speed,
                ..Default::default()
            },
        );

        let cell = (player.x as usize, player.y as usize);
        if visited_cells.insert(cell) {
            emit_event(
                &mut tfile,
                ts_ms,
                &sid,
                EventType::ExplorationDiscover,
                &ExplorationPayload {
                    discovery_id: format!("cell_{}_{}", cell.0, cell.1),
                    category: "area".into(),
                    x: player.x,
                    y: player.y,
                },
            );
        }

        term.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(3)])
                .split(f.area());

            let view_area = chunks[0];
            let screen_w = view_area.width as usize;
            let screen_h = view_area.height as usize;

            let hits = cast_screen(&player, screen_w, &map, MAX_DEPTH);
            let viewport = render_viewport(&hits, screen_w, screen_h);

            let view = Paragraph::new(viewport)
                .block(Block::default().borders(Borders::NONE));
            f.render_widget(view, view_area);

            let elapsed = game_start.elapsed().as_secs();
            let hud_text = format!(
                " pos=({:.1},{:.1}) ang={:.0}° seed={seed} frame={frame_count} t={elapsed}s │ WASD:move Q/E:turn ESC:quit",
                player.x, player.y, player.angle.to_degrees()
            );
            let hud = Paragraph::new(Line::from(vec![
                Span::styled(hud_text, Style::default().fg(Color::Cyan)),
            ]))
            .block(Block::default().borders(Borders::TOP));
            f.render_widget(hud, chunks[1]);
        })?;

        frame_count += 1;
        let elapsed = frame_start.elapsed();
        if let Some(remaining) = tick_dur.checked_sub(elapsed) {
            std::thread::sleep(remaining);
        }
    }

    let duration_s = game_start.elapsed().as_secs_f64();
    let final_ts = game_start.elapsed().as_millis() as u64;
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

    println!("\nExp024: Doom-in-a-Terminal — session complete");
    println!("  Frames: {frame_count}");
    println!("  Duration: {duration_s:.1}s");
    println!("  Seed: {seed}");
    println!("  Map: {MAP_W}x{MAP_H} (BSP generated)");
    println!("  Cells explored: {}", visited_cells.len());
    println!("  Telemetry: {telemetry_path}");
    println!("  Analyze: cargo run -p ludospring-exp026 -- analyze {telemetry_path}");

    Ok(())
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

fn handle_input(key: KeyCode, player: &mut RayPlayer, map: &GridMap, dt: Duration) -> bool {
    let dt_s = dt.as_secs_f64();
    let (old_x, old_y) = (player.x, player.y);

    match key {
        KeyCode::Esc => return false,
        KeyCode::Char('w') | KeyCode::Up => player.move_forward(1.0, dt_s),
        KeyCode::Char('s') | KeyCode::Down => player.move_forward(-1.0, dt_s),
        KeyCode::Char('a') => player.strafe(-1.0, dt_s),
        KeyCode::Char('d') => player.strafe(1.0, dt_s),
        KeyCode::Char('q') | KeyCode::Left => player.rotate(-1.0, dt_s),
        KeyCode::Char('e') | KeyCode::Right => player.rotate(1.0, dt_s),
        _ => {}
    }

    // Collision: reject move if new position is inside a wall
    let cx = player.x as usize;
    let cy = player.y as usize;
    if cx < MAP_W && cy < MAP_H && map.get(cx, cy) {
        player.x = old_x;
        player.y = old_y;
    }

    true
}

fn render_viewport(hits: &[Option<RayHit>], width: usize, height: usize) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::with_capacity(height);

    for row in 0..height {
        let mut spans = Vec::with_capacity(width);
        for col in 0..width {
            let hit = hits.get(col).and_then(|h| h.as_ref());
            let ch = column_char(hit, row, height);
            let color = column_color(hit);
            spans.push(Span::styled(String::from(ch), Style::default().fg(color)));
        }
        lines.push(Line::from(spans));
    }

    lines
}

fn column_char(hit: Option<&RayHit>, row: usize, height: usize) -> char {
    let Some(hit) = hit else {
        return ' ';
    };

    let wall_height = (height as f64 / hit.distance.max(0.1)).min(height as f64);
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "bounded by height"
    )]
    let wall_h = wall_height as usize;
    let top = height.saturating_sub(wall_h) / 2;
    let bottom = top + wall_h;

    if row < top {
        ' '
    } else if row >= bottom {
        '.'
    } else if hit.distance < 3.0 {
        '█'
    } else if hit.distance < 6.0 {
        '▓'
    } else if hit.distance < 10.0 {
        '▒'
    } else {
        '░'
    }
}

fn column_color(hit: Option<&RayHit>) -> Color {
    let Some(hit) = hit else {
        return Color::DarkGray;
    };

    if hit.vertical_hit {
        if hit.distance < 5.0 {
            Color::White
        } else {
            Color::Gray
        }
    } else if hit.distance < 5.0 {
        Color::LightBlue
    } else {
        Color::Blue
    }
}

/// Generate a playable level from BSP + corridors.
fn generate_level(seed: u64) -> GridMap {
    let bounds = Rect::new(1.0, 1.0, (MAP_W - 2) as f64, (MAP_H - 2) as f64);
    let tree = generate_bsp(bounds, 8.0, seed);
    let leaves = tree.leaves();

    // Start with all walls
    let mut data = vec![true; MAP_W * MAP_H];

    // Carve rooms from BSP leaves (inset by 1 for walls)
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
                data[y * MAP_W + x] = false;
            }
        }
    }

    // Connect rooms with L-shaped corridors between adjacent leaf centers
    for i in 0..leaves.len().saturating_sub(1) {
        let (cx1, cy1) = leaves[i].center();
        let (cx2, cy2) = leaves[i + 1].center();
        carve_corridor(&mut data, cx1, cy1, cx2, cy2);
    }

    GridMap::new(MAP_W, MAP_H, data)
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "map coords"
)]
fn carve_corridor(data: &mut [bool], x1: f64, y1: f64, x2: f64, y2: f64) {
    let (mut cx, mut cy) = (x1 as usize, y1 as usize);
    let (tx, ty) = (x2 as usize, y2 as usize);

    // Horizontal leg
    while cx != tx {
        if cx < MAP_W && cy < MAP_H {
            data[cy * MAP_W + cx] = false;
        }
        if cx < tx {
            cx += 1;
        } else {
            cx -= 1;
        }
    }
    // Vertical leg
    while cy != ty {
        if cx < MAP_W && cy < MAP_H {
            data[cy * MAP_W + cx] = false;
        }
        if cy < ty {
            cy += 1;
        } else {
            cy -= 1;
        }
    }
    if cx < MAP_W && cy < MAP_H {
        data[cy * MAP_W + cx] = false;
    }
}

fn find_spawn(map: &GridMap) -> (f64, f64) {
    for y in 2..MAP_H - 2 {
        for x in 2..MAP_W - 2 {
            if !map.get(x, y) && !map.get(x + 1, y) && !map.get(x, y + 1) {
                return (x as f64 + 0.5, y as f64 + 0.5);
            }
        }
    }
    (MAP_W as f64 / 2.0, MAP_H as f64 / 2.0)
}

fn tufte_analysis() {
    let elements = vec![
        UiElement {
            name: "position_display".into(),
            bounds: [0.0, 0.95, 0.3, 0.05],
            data_values: 3,
            pixel_area: 200.0,
            data_ink_area: 180.0,
            critical: false,
        },
        UiElement {
            name: "controls_hint".into(),
            bounds: [0.3, 0.95, 0.4, 0.05],
            data_values: 1,
            pixel_area: 300.0,
            data_ink_area: 250.0,
            critical: false,
        },
        UiElement {
            name: "viewport".into(),
            bounds: [0.0, 0.0, 1.0, 0.95],
            data_values: 200,
            pixel_area: 50000.0,
            data_ink_area: 45000.0,
            critical: true,
        },
    ];

    let report = analyze_game_ui(&elements);
    eprintln!("Tufte HUD analysis:");
    eprintln!("  data-ink ratio: {:.3}", report.data_ink_ratio);
    eprintln!("  info density:   {:.3}", report.info_density);
    eprintln!("  screen coverage: {:.3}", report.screen_coverage);
    for note in &report.notes {
        eprintln!("  note: {note}");
    }
    eprintln!();
}
