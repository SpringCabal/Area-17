use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use area17_shared::TechId;
use spring::ResultExt;

use super::data;
use super::state::Hud;
use super::tech_data::{TECH_DEFS, can_activate, tech_unlock_cost, tech_upgrade_cost};
use crate::state::{Global, SimulationState};

const CARD_OFFSET_LEFT: i32 = 10;
const CARD_OFFSET_TOP: i32 = 50;
const CARD_FIELDS: [(&str, u8); 8] = [
    ("index", data::INT),
    ("class", data::STRING),
    ("icon", data::STRING),
    ("left", data::PIXELS),
    ("top", data::PIXELS),
    ("locked", data::BOOL),
    ("level", data::INT),
    ("maximum", data::INT),
];

#[derive(Clone, Copy)]
pub struct Bindings {
    open: u64,
    available: u64,
    cards: u64,
    tooltip_visible: u64,
    tooltip_title: u64,
    tooltip_level: u64,
    tooltip_description: u64,
    tooltip_status: u64,
    tooltip_class: u64,
    tooltip_cost: u64,
}

pub fn bind(model: u64) -> Option<Bindings> {
    Some(Bindings {
        open: data::bind_bool(model, "tech_open")?,
        available: data::bind_bool(model, "tech_available")?,
        cards: data::bind_rows(model, "technology", &CARD_FIELDS)?,
        tooltip_visible: data::bind_bool(model, "tooltip_visible")?,
        tooltip_title: data::bind_string(model, "tooltip_title")?,
        tooltip_level: data::bind_string(model, "tooltip_level")?,
        tooltip_description: data::bind_string(model, "tooltip_description")?,
        tooltip_status: data::bind_string(model, "tooltip_status")?,
        tooltip_class: data::bind_string(model, "tooltip_class")?,
        tooltip_cost: data::bind_string(model, "tooltip_cost")?,
    })
}

pub fn set_tech_toggle(hud: &Hud, simulation: &SimulationState) {
    let Some(bindings) = hud.bindings else {
        return;
    };
    let available = simulation.tech.iter().enumerate().any(|(index, tech)| {
        tech.enabled && !tech.unlocked && simulation.research >= tech_unlock_cost(index)
    });
    let _ = spring::data_model_set_bool(bindings.tech.available, available);
}

pub fn set_tech_panel(hud: &mut Hud, simulation: &SimulationState, open: bool) {
    hud.tech_open = Some(open);
    if !open {
        hud.hovered_tech = None;
    }
    let Some(bindings) = hud.bindings.map(|bindings| bindings.tech) else {
        return;
    };
    let _ = spring::data_model_set_bool(bindings.open, open);
    if open {
        let mut values = Vec::with_capacity(TECH_DEFS.len() * CARD_FIELDS.len());
        for (index, def) in TECH_DEFS.iter().enumerate() {
            let value = simulation.tech[index];
            let state = card_state(simulation, index);
            let icon = if value.enabled {
                def.icon
            } else {
                def.disabled_icon
            };
            let (left, top) = def.position;
            let max_level = TechId::ALL[index].max_level();
            values.extend([
                data::int(index as i32),
                data::text(&format!("tech-card {state}")),
                data::text(icon),
                data::pixels((CARD_OFFSET_LEFT + left) as f32),
                data::pixels((CARD_OFFSET_TOP + top) as f32),
                data::boolean(!value.unlocked),
                data::int(value.level),
                data::int(max_level),
            ]);
        }
        spring::data_model_set_rows(bindings.cards, &values)
            .log_err_once("updating Area 17 technology cards");
    }
    refresh_tooltip(hud, simulation);
}

pub fn hover(global: &Global, index: usize) {
    let mut hud = global.hud.access_mut();
    if hud.tech_open != Some(true) {
        return;
    }
    hud.hovered_tech = Some(index);
    refresh_tooltip(&hud, &global.simulation.access());
}

pub fn leave(global: &Global, index: usize) {
    let mut hud = global.hud.access_mut();
    if hud.hovered_tech == Some(index) {
        hud.hovered_tech = None;
        refresh_tooltip(&hud, &global.simulation.access());
    }
}

fn refresh_tooltip(hud: &Hud, simulation: &SimulationState) {
    let Some(bindings) = hud.bindings.map(|bindings| bindings.tech) else {
        return;
    };
    let Some(index) = hud.hovered_tech else {
        let _ = spring::data_model_set_bool(bindings.tooltip_visible, false);
        return;
    };
    let def = &TECH_DEFS[index];
    let value = simulation.tech[index];
    let tech = TechId::ALL[index];
    let complete = value.level >= tech.max_level();
    let (action, amount, resource, available) = if value.unlocked {
        (
            "Next upgrade",
            tech_upgrade_cost(index, value.level),
            "biomass",
            simulation.biomass,
        )
    } else {
        (
            "Unlock",
            tech_unlock_cost(index),
            "research",
            simulation.research,
        )
    };
    let cost = if complete {
        String::new()
    } else {
        format!("{action}: {amount:.0} {resource}")
    };
    let status = if !value.enabled {
        String::from("Requires prerequisite technology")
    } else if complete {
        String::from("Fully upgraded")
    } else if can_activate(simulation, index) {
        format!(
            "Ready to {}",
            if value.unlocked { "upgrade" } else { "unlock" }
        )
    } else {
        format!("Insufficient {resource} (have {available:.0})")
    };
    let _ = spring::data_model_set_string(bindings.tooltip_title, def.title);
    let _ = spring::data_model_set_string(
        bindings.tooltip_level,
        &format!("Level {} / {}", value.level, tech.max_level()),
    );
    let _ = spring::data_model_set_string(
        bindings.tooltip_description,
        &tech_effect(tech, value.level),
    );
    let _ = spring::data_model_set_string(bindings.tooltip_cost, &cost);
    let _ = spring::data_model_set_string(bindings.tooltip_status, &status);
    let _ = spring::data_model_set_string(bindings.tooltip_class, card_state(simulation, index));
    let _ = spring::data_model_set_bool(bindings.tooltip_visible, true);
}

fn card_state(simulation: &SimulationState, index: usize) -> &'static str {
    let tech = simulation.tech[index];
    if !tech.enabled {
        "disabled"
    } else if tech.level >= TechId::ALL[index].max_level() {
        "complete"
    } else if can_activate(simulation, index) {
        "affordable"
    } else {
        "unaffordable"
    }
}

fn tech_effect(tech: TechId, level: i32) -> String {
    let value = tech.value(level);
    let effect = match tech {
        TechId::Armor => "armor",
        TechId::ConeUpgrade => "beamup cone and size",
        TechId::PulseLaser => "damage",
        TechId::BlackHoleGun => "duration and radius",
        TechId::IncendiaryBeamLaser => "fire duration and damage",
        TechId::GravityBeam => "strength",
        TechId::Shield => "recharge speed",
        TechId::AntiMissilePointDefense => "fire rate",
        TechId::CarrierDrones => return format!("Current total: {value:.0} drones"),
        TechId::Haste => "speed and duration",
        TechId::Cloak => "shorter cooldown",
        TechId::Teleport => "distance",
        TechId::IndependenceDayGun => "duration",
    };
    format!("Current total: +{value:.0}% {effect}")
}
