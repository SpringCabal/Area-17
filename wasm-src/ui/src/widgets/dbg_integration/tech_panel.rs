use crate::rml::tech_data::can_activate;
use crate::rml::{self, dom};
use crate::state::{Global, SimulationState, TechState};
use area17_shared::TechId;

pub(super) fn enabled() -> bool {
    spring::game_options::get_mod_option("tech_ui_test")
        .ok()
        .flatten()
        .as_deref()
        == Some("1")
}

fn fixture() -> SimulationState {
    let mut simulation = SimulationState {
        biomass: 4_000.0,
        research: 500.0,
        ..Default::default()
    };
    for tech in [
        TechId::Armor,
        TechId::ConeUpgrade,
        TechId::PulseLaser,
        TechId::BlackHoleGun,
        TechId::IncendiaryBeamLaser,
        TechId::Shield,
        TechId::Haste,
    ] {
        simulation.tech[tech.index()].enabled = true;
    }
    for (tech, level) in [
        (TechId::Armor, 3),
        (TechId::ConeUpgrade, 0),
        (TechId::PulseLaser, 2),
    ] {
        simulation.tech[tech.index()] = TechState {
            level,
            unlocked: true,
            enabled: true,
        };
    }
    simulation
}

pub(super) fn update(global: &Global) {
    if global.hud.access().context == 0 {
        return;
    }
    let simulation = fixture();
    global.panel.access_mut().upgrade_panel_open = true;
    rml::refresh_hud(&mut global.hud.access_mut(), &simulation, true);
    rml::update_context(global);
    let text = |id, expected| {
        let element = dom::element(&global.hud.access(), id);
        element != 0 && spring::element_get_inner_rml(element).is_ok_and(|value| value == expected)
    };
    let hover = |index| {
        let card = dom::element(&global.hud.access(), &alloc::format!("tech-card-{index}"));
        let dispatched = spring::element_dispatch_event(card, "mouseover") == Ok(true);
        let received = global.hud.access().hovered_tech == Some(index);
        rml::refresh_hud(&mut global.hud.access_mut(), &simulation, true);
        rml::update_context(global);
        dispatched && received
    };
    let pulse = hover(TechId::PulseLaser.index())
        && text("tech-tooltip-level", "Level 2 / 5")
        && text("tech-tooltip-effect", "Current total: +200% damage")
        && text("tech-tooltip-cost", "Next upgrade: 3000 biomass")
        && text("tech-tooltip-status", "Ready to upgrade");
    let armor = hover(TechId::Armor.index())
        && text("tech-level-0", "3/10")
        && text("tech-level-2", "2/5")
        && text("tech-tooltip-level", "Level 3 / 10")
        && text("tech-tooltip-effect", "Current total: +120% armor")
        && text("tech-tooltip-cost", "Next upgrade: 4500 biomass")
        && text("tech-tooltip-status", "Insufficient biomass (have 4000)");
    let mut purchased = simulation.clone();
    purchased.biomass = 4_500.0;
    let exact_cost = can_activate(&purchased, TechId::Armor.index());
    purchased.tech[TechId::Armor.index()].level = 10;
    let maximum = !can_activate(&purchased, TechId::Armor.index());
    purchased.research = 1_000.0;
    let unlock = can_activate(&purchased, TechId::IncendiaryBeamLaser.index());
    let valid = pulse
        && armor
        && exact_cost
        && maximum
        && unlock
        && !can_activate(&simulation, TechId::Armor.index())
        && can_activate(&simulation, TechId::PulseLaser.index())
        && !can_activate(&simulation, TechId::IncendiaryBeamLaser.index())
        && !can_activate(&simulation, TechId::GravityBeam.index());
    if valid {
        spring::log_once!(
            spring::log_level::NOTICE,
            crate::diagnostics::SECTION,
            "Area 17 technology fixture passed: Armor 3/10 +120%, Pulse 2/5, costs and affordability"
        );
    } else {
        spring::log_once!(
            spring::log_level::ERROR,
            crate::diagnostics::SECTION,
            "Area 17 technology fixture failed: level, total effect, cost or affordability binding"
        );
    }
}
