// SPDX-License-Identifier: AGPL-3.0-or-later
//! Shared HUD element fixtures for game UI dashboards.
//!
//! Canonical `UiElement` sets for common game genres, used by both the
//! main game science dashboard and the Tufte validation dashboard.

use ludospring_barracuda::metrics::tufte_gaming::UiElement;

/// Doom-style FPS HUD: health, ammo, crosshair, minimap.
#[must_use]
pub fn fps_hud_elements() -> Vec<UiElement> {
    vec![
        UiElement {
            name: "health".into(),
            bounds: [0.05, 0.9, 0.15, 0.05],
            data_values: 1,
            pixel_area: 200.0,
            data_ink_area: 150.0,
            critical: true,
        },
        UiElement {
            name: "ammo".into(),
            bounds: [0.85, 0.9, 0.1, 0.05],
            data_values: 1,
            pixel_area: 150.0,
            data_ink_area: 100.0,
            critical: true,
        },
        UiElement {
            name: "crosshair".into(),
            bounds: [0.48, 0.48, 0.04, 0.04],
            data_values: 1,
            pixel_area: 20.0,
            data_ink_area: 18.0,
            critical: true,
        },
        UiElement {
            name: "minimap".into(),
            bounds: [0.8, 0.0, 0.2, 0.2],
            data_values: 50,
            pixel_area: 1000.0,
            data_ink_area: 600.0,
            critical: false,
        },
    ]
}

/// StarCraft-style RTS HUD: minimap, unit list, resources, command card.
#[must_use]
pub fn rts_hud_elements() -> Vec<UiElement> {
    vec![
        UiElement {
            name: "minimap".into(),
            bounds: [0.75, 0.0, 0.25, 0.25],
            data_values: 200,
            pixel_area: 2500.0,
            data_ink_area: 1800.0,
            critical: true,
        },
        UiElement {
            name: "unit_list".into(),
            bounds: [0.0, 0.0, 0.15, 0.5],
            data_values: 30,
            pixel_area: 1500.0,
            data_ink_area: 800.0,
            critical: true,
        },
        UiElement {
            name: "resources".into(),
            bounds: [0.3, 0.0, 0.3, 0.03],
            data_values: 4,
            pixel_area: 300.0,
            data_ink_area: 250.0,
            critical: true,
        },
        UiElement {
            name: "command_card".into(),
            bounds: [0.0, 0.7, 0.2, 0.3],
            data_values: 12,
            pixel_area: 2000.0,
            data_ink_area: 600.0,
            critical: true,
        },
    ]
}

/// Minecraft-style sandbox HUD: hotbar, health, crosshair.
#[must_use]
pub fn sandbox_hud_elements() -> Vec<UiElement> {
    vec![
        UiElement {
            name: "hotbar".into(),
            bounds: [0.3, 0.95, 0.4, 0.05],
            data_values: 9,
            pixel_area: 500.0,
            data_ink_area: 400.0,
            critical: true,
        },
        UiElement {
            name: "health".into(),
            bounds: [0.3, 0.9, 0.1, 0.03],
            data_values: 2,
            pixel_area: 80.0,
            data_ink_area: 70.0,
            critical: true,
        },
        UiElement {
            name: "crosshair".into(),
            bounds: [0.49, 0.49, 0.02, 0.02],
            data_values: 1,
            pixel_area: 10.0,
            data_ink_area: 9.0,
            critical: false,
        },
    ]
}

/// Diablo-style RPG HUD: health/mana orbs, action bar, buffs, quest tracker, chat.
#[must_use]
pub fn rpg_hud_elements() -> Vec<UiElement> {
    vec![
        UiElement {
            name: "health_orb".into(),
            bounds: [0.02, 0.85, 0.08, 0.12],
            data_values: 1,
            pixel_area: 400.0,
            data_ink_area: 200.0,
            critical: true,
        },
        UiElement {
            name: "mana_orb".into(),
            bounds: [0.9, 0.85, 0.08, 0.12],
            data_values: 1,
            pixel_area: 400.0,
            data_ink_area: 200.0,
            critical: true,
        },
        UiElement {
            name: "action_bar".into(),
            bounds: [0.2, 0.92, 0.6, 0.06],
            data_values: 10,
            pixel_area: 2000.0,
            data_ink_area: 800.0,
            critical: true,
        },
        UiElement {
            name: "buff_icons".into(),
            bounds: [0.7, 0.0, 0.3, 0.04],
            data_values: 8,
            pixel_area: 600.0,
            data_ink_area: 450.0,
            critical: false,
        },
        UiElement {
            name: "quest_tracker".into(),
            bounds: [0.75, 0.1, 0.25, 0.15],
            data_values: 3,
            pixel_area: 800.0,
            data_ink_area: 300.0,
            critical: false,
        },
        UiElement {
            name: "chat_window".into(),
            bounds: [0.0, 0.6, 0.25, 0.2],
            data_values: 5,
            pixel_area: 1200.0,
            data_ink_area: 900.0,
            critical: false,
        },
    ]
}

/// Tetris-style puzzle HUD: board, score, timer.
#[must_use]
pub fn puzzle_hud_elements() -> Vec<UiElement> {
    vec![
        UiElement {
            name: "board".into(),
            bounds: [0.15, 0.1, 0.7, 0.75],
            data_values: 64,
            pixel_area: 30000.0,
            data_ink_area: 28000.0,
            critical: true,
        },
        UiElement {
            name: "score".into(),
            bounds: [0.02, 0.02, 0.1, 0.04],
            data_values: 1,
            pixel_area: 100.0,
            data_ink_area: 80.0,
            critical: true,
        },
        UiElement {
            name: "timer".into(),
            bounds: [0.88, 0.02, 0.1, 0.04],
            data_values: 1,
            pixel_area: 100.0,
            data_ink_area: 80.0,
            critical: true,
        },
    ]
}
