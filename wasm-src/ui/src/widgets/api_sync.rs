use area17_shared::message as msg;
use spring_addons::ui::Widget;
use springcabal::message;

use crate::catalog::{find_ability, find_weapon};
use crate::rml::Hud;
use crate::rml::tech_data::find_tech_index;
use crate::state::{Global, SimulationState};
use crate::{diagnostics, rml, state};

const RESOURCE_DIFF_THRESHOLD: f32 = 0.5;
const COLOR_RESOURCE_GAIN: &str = "#64e890";
const COLOR_RESOURCE_LOSS: &str = "#ff8064";

enum Resource {
    Biomass,
    Research,
    Metal,
}

#[derive(Default)]
pub struct ApiSyncWidget;

impl ApiSyncWidget {
    pub fn new() -> Self {
        Self
    }
}

impl Widget<Global> for ApiSyncWidget {
    fn name(&self) -> &'static str {
        "ApiSync"
    }

    fn recv_from_synced(
        &self,
        ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>,
        message: &[u8],
    ) {
        let spring = ctx.spring();
        let Ok(message) = core::str::from_utf8(message) else {
            return;
        };
        if let Some(fields) = message::key_values(message, msg::STATUS) {
            let tech_open = ctx.global().panel.access().upgrade_panel_open;
            Self::handle_status_message(
                spring,
                &mut ctx.global().simulation.access_mut(),
                &mut ctx.global().hud.access_mut(),
                tech_open,
                fields,
            );
        }
    }
}

impl ApiSyncWidget {
    fn handle_status_message<'a>(
        spring: &spring::Spring<'_, spring::Ui>,
        simulation: &mut SimulationState,
        hud: &mut Hud,
        tech_open: bool,
        fields: impl Iterator<Item = (&'a str, &'a str)>,
    ) {
        let first_status = !simulation.status_initialized;
        for (key, value) in fields {
            match key {
                "ufo" => set_i32(value, |v| simulation.ufo_id = v),
                "health" => set_f32(value, |v| simulation.health = v),
                "max_health" => set_f32(value, |v| simulation.max_health = v.max(1.0)),
                "biomass" => {
                    Self::update_resource(simulation, hud, value, Resource::Biomass, first_status)
                }
                "research" => {
                    Self::update_resource(simulation, hud, value, Resource::Research, first_status)
                }
                "metal" => {
                    Self::update_resource(simulation, hud, value, Resource::Metal, first_status)
                }
                "shield" => set_f32(value, |v| simulation.shield = v),
                "radius" => set_f32(value, |v| simulation.abduction_radius = v),
                "weapon" => simulation.current_weapon = find_weapon(value),
                "game_over" => set_bool(value, |v| simulation.game_over = v),
                "game_won" => set_bool(value, |v| simulation.game_won = v),
                key if key.ends_with("CD") => Self::update_ability(simulation, key, value, true),
                key if key.ends_with("Duration") => {
                    Self::update_ability(simulation, key, value, false)
                }
                key if key.starts_with("tech_") => Self::update_tech(simulation, key, value),
                _ => {}
            }
        }
        rml::refresh_hud(spring, hud, simulation, tech_open);
        simulation.status_initialized = true;
        diagnostics::status(
            spring,
            simulation.ufo_id,
            simulation.health,
            simulation.max_health,
            simulation.biomass,
            simulation.game_over,
            simulation.game_won,
        );
    }

    fn update_resource(
        simulation: &mut SimulationState,
        hud: &mut Hud,
        value: &str,
        resource: Resource,
        first_status: bool,
    ) {
        let Some(parsed) = parse_f32(value) else {
            return;
        };
        let (old, label) = match resource {
            Resource::Biomass => (simulation.biomass, "Biomass"),
            Resource::Research => (simulation.research, "Research"),
            Resource::Metal => (simulation.metal, "Metal"),
        };
        match resource {
            Resource::Biomass => simulation.biomass = parsed,
            Resource::Research => simulation.research = parsed,
            Resource::Metal => simulation.metal = parsed,
        }
        if !first_status && (parsed - old).abs() >= RESOURCE_DIFF_THRESHOLD {
            let delta = parsed - old;
            rml::add_event(
                hud,
                &alloc::format!("{label} {delta:+.0}"),
                if delta > 0.0 {
                    COLOR_RESOURCE_GAIN
                } else {
                    COLOR_RESOURCE_LOSS
                },
            );
        }
    }

    fn update_ability(simulation: &mut SimulationState, key: &str, value: &str, cooldown: bool) {
        let name = key
            .strip_suffix("CD")
            .or_else(|| key.strip_suffix("Duration"))
            .unwrap_or(key);
        let Some(index) = find_ability(name) else {
            return;
        };
        let Some(value) = parse_i32(value) else {
            return;
        };
        if cooldown {
            simulation.abilities[index].cooldown = value;
        } else {
            simulation.abilities[index].duration = value;
        }
    }

    fn update_tech(simulation: &mut SimulationState, key: &str, value: &str) {
        let Some(name) = key.strip_prefix("tech_") else {
            return;
        };
        let Some(index) = find_tech_index(name) else {
            return;
        };
        let mut values = value.split(',');
        let level = parse_i32(values.next().unwrap_or("0")).unwrap_or(0);
        let unlocked = values.next() == Some("1");
        let enabled = values.next() == Some("1");
        simulation.tech[index] = state::TechState {
            level,
            unlocked,
            enabled,
        };
    }
}

fn parse_i32(value: &str) -> Option<i32> {
    value.parse().ok()
}

fn parse_f32(value: &str) -> Option<f32> {
    value.parse().ok()
}

fn set_i32(value: &str, setter: impl FnOnce(i32)) {
    if let Some(value) = parse_i32(value) {
        setter(value);
    }
}

fn set_f32(value: &str, setter: impl FnOnce(f32)) {
    if let Some(value) = parse_f32(value) {
        setter(value);
    }
}

fn set_bool(value: &str, setter: impl FnOnce(bool)) {
    if let Some(value) = parse_i32(value) {
        setter(value != 0);
    }
}
