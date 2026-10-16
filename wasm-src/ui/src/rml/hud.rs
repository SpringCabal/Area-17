use alloc::format;
use alloc::vec::Vec;

use spring::ResultExt;
use spring::owned::rml_ui::{RmlDataFieldDef, RmlDataValue};

use super::dom::{health_color, shield_color};
use super::loadout::{self, set_ability_panel, set_weapon_panel};
use super::state::{Bindings, Hud, UiEvent};
use super::tech::{self, set_tech_panel, set_tech_toggle};
use crate::state::SimulationState;

const MAX_EVENT_LINES: usize = 8;
const EVENT_EXPIRATION_SECONDS: f32 = 2.2;
const EVENT_FADE_IN_FRACTION: f32 = 0.12;
const EVENT_START_TOP_PERCENT: f32 = 82.0;
const EVENT_TRAVEL_PERCENT: f32 = 62.0;
const MAX_SHIELD_CAPACITY: f32 = 1_000.0;

const FIELD_FLOAT: u8 = 2;
const FIELD_STRING: u8 = 3;
const FIELD_COLOR: u8 = 4;
const FIELD_PERCENT: u8 = 6;
const EVENT_FIELDS: [(&str, u8); 4] = [
    ("text", FIELD_STRING),
    ("color", FIELD_COLOR),
    ("top", FIELD_PERCENT),
    ("opacity", FIELD_FLOAT),
];

pub fn refresh_hud(hud: &mut Hud, simulation: &SimulationState, tech_open: bool) {
    if let Some(bindings) = hud.bindings {
        let _ = spring::data_model_set_int(bindings.biomass, simulation.biomass.round() as i32);
        let _ = spring::data_model_set_int(bindings.research, simulation.research.round() as i32);
        set_health_bar(bindings, simulation);
        set_endgame(bindings, simulation.game_over, simulation.game_won);
    }
    set_tech_toggle(hud, simulation);
    set_weapon_panel(hud, simulation);
    set_ability_panel(hud, simulation);
    set_tech_panel(hud, simulation, tech_open);
}

pub fn update_events(
    hud: &mut Hud,
    simulation: &SimulationState,
    tech_open: bool,
    delta_seconds: f32,
) {
    let had_events = !hud.events.is_empty();
    for event in &mut hud.events {
        event.age_seconds += delta_seconds;
    }
    hud.events
        .retain(|event| event.age_seconds < EVENT_EXPIRATION_SECONDS);
    if hud.tech_open != Some(tech_open) {
        set_tech_panel(hud, simulation, tech_open);
    }
    let Some(bindings) = hud.bindings.filter(|_| had_events) else {
        return;
    };
    let mut values = Vec::with_capacity(hud.events.len() * EVENT_FIELDS.len());
    for event in hud.events.iter().rev() {
        let age = (event.age_seconds / EVENT_EXPIRATION_SECONDS).clamp(0.0, 1.0);
        let top = EVENT_START_TOP_PERCENT - age * EVENT_TRAVEL_PERCENT;
        let opacity = if age < EVENT_FADE_IN_FRACTION {
            age / EVENT_FADE_IN_FRACTION
        } else {
            1.0 - (age - EVENT_FADE_IN_FRACTION) / (1.0 - EVENT_FADE_IN_FRACTION)
        };
        let color =
            u32::from_str_radix(event.color.trim_start_matches('#'), 16).unwrap_or(0xffffff);
        values.extend([
            RmlDataValue {
                type_: FIELD_STRING,
                string_value: event.text.clone(),
                ..Default::default()
            },
            RmlDataValue {
                type_: FIELD_COLOR,
                red: (color >> 16) as u8,
                green: (color >> 8) as u8,
                blue: color as u8,
                alpha: u8::MAX,
                ..Default::default()
            },
            RmlDataValue {
                type_: FIELD_PERCENT,
                float_value: top,
                ..Default::default()
            },
            RmlDataValue {
                type_: FIELD_FLOAT,
                float_value: opacity,
                ..Default::default()
            },
        ]);
    }
    spring::data_model_set_rows(bindings.events, &values)
        .log_err_once("updating Area 17 floating messages");
}

pub fn add_event(hud: &mut Hud, text: &str, color: &'static str) {
    hud.events.push(UiEvent {
        text: text.into(),
        color,
        age_seconds: 0.0,
    });
    if hud.events.len() > MAX_EVENT_LINES {
        hud.events.remove(0);
    }
}

fn set_health_bar(bindings: Bindings, simulation: &SimulationState) {
    let health = simulation.health;
    let max_health = simulation.max_health;
    let shield = simulation.shield;
    let health_percent = (100.0 * health / max_health.max(1.0)).clamp(0.0, 100.0);
    let shield_percent = (100.0 * shield / MAX_SHIELD_CAPACITY).clamp(0.0, 100.0);
    let _ = spring::data_model_set_percent(bindings.health, health_percent);
    let _ = spring::data_model_set_percent(bindings.shield, shield_percent);
    let _ = spring::data_model_set_string(bindings.health_color, &health_color(health_percent));
    let _ = spring::data_model_set_string(bindings.shield_color, &shield_color(shield_percent));
    let _ = spring::data_model_set_string(
        bindings.health_caption,
        &format!("{:.0}/{:.0}", health.max(0.0), max_health),
    );
    let _ = spring::data_model_set_string(
        bindings.shield_caption,
        &format!("{:.0}/{MAX_SHIELD_CAPACITY:.0}", shield.max(0.0)),
    );
}

fn set_endgame(bindings: Bindings, game_over: bool, game_won: bool) {
    let title = if game_won {
        "THE QUEEN IS SAVED"
    } else {
        "THE UFO HAS BEEN DESTROYED"
    };
    let _ = spring::data_model_set_bool(bindings.game_over, game_over);
    let _ = spring::data_model_set_string(bindings.endgame_title, title);
}

pub fn create_bindings(model: u64) -> Option<Bindings> {
    let bind_int = |name: &str| {
        let binding = spring::data_model_bind_int(model, name, 0).ok()?;
        (binding.success && binding.variable_handle != 0).then_some(binding.variable_handle)
    };
    let bind_percent = |name: &str| {
        let binding = spring::data_model_bind_percent(model, name, 0.0).ok()?;
        (binding.success && binding.variable_handle != 0).then_some(binding.variable_handle)
    };
    let bind_string = |name: &str, initial: &str| {
        let binding = spring::data_model_bind_string(model, name, initial).ok()?;
        (binding.success && binding.variable_handle != 0).then_some(binding.variable_handle)
    };
    let game_over = spring::data_model_bind_bool(model, "game_over", false).ok()?;
    if !game_over.success || game_over.variable_handle == 0 {
        return None;
    }
    let fields = EVENT_FIELDS.map(|(name, type_)| RmlDataFieldDef {
        name: name.into(),
        type_,
    });
    let events = spring::data_model_bind_rows(model, "events", &fields).ok()?;
    if !events.success || events.rows_handle == 0 {
        return None;
    }
    let (weapons, abilities) = loadout::bind(model)?;
    Some(Bindings {
        biomass: bind_int("biomass")?,
        research: bind_int("research")?,
        health: bind_percent("health")?,
        health_color: bind_string("health_color", &health_color(0.0))?,
        health_caption: bind_string("health_caption", "0/0")?,
        shield: bind_percent("shield")?,
        shield_color: bind_string("shield_color", &shield_color(0.0))?,
        shield_caption: bind_string("shield_caption", "0/1000")?,
        game_over: game_over.variable_handle,
        endgame_title: bind_string("endgame_title", "")?,
        events: events.rows_handle,
        weapons,
        abilities,
        tech: tech::bind(model)?,
    })
}
