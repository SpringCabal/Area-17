use alloc::format;
use alloc::vec::Vec;

use spring::ResultExt;
use spring::consts::GAME_SPEED;

use super::data;
use super::state::Hud;
use super::tech_data::find_tech_index;
use crate::catalog::{ABILITIES, WEAPONS};
use crate::state::SimulationState;

const WEAPON_PANEL_BOTTOM_BASE: i32 = 600;
const WEAPON_PANEL_SLOT_STEP: i32 = 50;
const ABILITY_PANEL_BOTTOM_BASE: i32 = 200;
const ABILITY_PANEL_SLOT_STEP: i32 = 50;
const WEAPON_FIELDS: [(&str, u8); 3] = [
    ("label", data::STRING),
    ("state", data::STRING),
    ("bottom", data::PIXELS),
];
const ABILITY_FIELDS: [(&str, u8); 5] = [
    ("label", data::STRING),
    ("state", data::STRING),
    ("bottom", data::PIXELS),
    ("progress", data::PERCENT),
    ("caption", data::STRING),
];

pub fn bind(spring: &spring::Spring<'_, spring::Ui>, model: u64) -> Option<(u64, u64)> {
    Some((
        data::bind_rows(spring, model, "weapons", &WEAPON_FIELDS)?,
        data::bind_rows(spring, model, "abilities", &ABILITY_FIELDS)?,
    ))
}

pub fn set_weapon_panel(
    spring: &spring::Spring<'_, spring::Ui>,
    hud: &Hud,
    simulation: &SimulationState,
) {
    let Some(bindings) = hud.bindings else {
        return;
    };
    let mut values = Vec::with_capacity(WEAPONS.len() * WEAPON_FIELDS.len());
    for (index, weapon) in WEAPONS.iter().enumerate() {
        let unlocked =
            find_tech_index(weapon.name()).is_some_and(|tech| simulation.tech[tech].unlocked);
        if !unlocked {
            continue;
        }
        let state = if simulation.current_weapon == Some(index) {
            "selected"
        } else {
            "disabled"
        };
        values.extend([
            data::text(&format!("[{}] {}", weapon.key, weapon.title)),
            data::text(state),
            data::pixels((WEAPON_PANEL_BOTTOM_BASE - index as i32 * WEAPON_PANEL_SLOT_STEP) as f32),
        ]);
    }
    spring
        .rml_ui()
        .data_model_set_rows(bindings.weapons, &values)
        .log_err_once(spring, "updating Area 17 weapon slots");
}

pub fn set_ability_panel(
    spring: &spring::Spring<'_, spring::Ui>,
    hud: &Hud,
    simulation: &SimulationState,
) {
    let Some(bindings) = hud.bindings else {
        return;
    };
    let mut values = Vec::with_capacity(ABILITIES.len() * ABILITY_FIELDS.len());
    for (index, ability) in ABILITIES.iter().enumerate() {
        let Some(tech) = find_tech_index(ability.name()).map(|tech| simulation.tech[tech]) else {
            continue;
        };
        if !tech.unlocked {
            continue;
        }
        let timers = simulation.abilities[index];
        let (state, percent, caption) = if timers.duration > 0 {
            (
                "active",
                progress_percent(timers.duration, ability.ability.duration(tech.level)),
                format!("Active [{}s]", seconds_remaining(timers.duration)),
            )
        } else if timers.cooldown > 0 {
            (
                "cooldown",
                cooldown_percent(timers.cooldown, ability.ability.cooldown(tech.level)),
                format!("CD [{}s]", seconds_remaining(timers.cooldown)),
            )
        } else {
            ("ready", 100.0, "=READY=".into())
        };
        values.extend([
            data::text(&format!("[{}] {}", ability.key_label, ability.title)),
            data::text(state),
            data::pixels(
                (ABILITY_PANEL_BOTTOM_BASE - index as i32 * ABILITY_PANEL_SLOT_STEP) as f32,
            ),
            data::percent(percent),
            data::text(&caption),
        ]);
    }
    spring
        .rml_ui()
        .data_model_set_rows(bindings.abilities, &values)
        .log_err_once(spring, "updating Area 17 ability slots");
}

fn progress_percent(value: i32, maximum: i32) -> f32 {
    if maximum <= 0 {
        100.0
    } else {
        (100.0 * value as f32 / maximum as f32).clamp(0.0, 100.0)
    }
}

fn cooldown_percent(remaining: i32, maximum: i32) -> f32 {
    if maximum <= 0 {
        100.0
    } else {
        (100.0 * (maximum - remaining).max(0) as f32 / maximum as f32).clamp(0.0, 100.0)
    }
}

fn seconds_remaining(frames: i32) -> i32 {
    (frames.max(0) + GAME_SPEED - 1) / GAME_SPEED
}
