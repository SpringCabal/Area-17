use alloc::string::String;
use area17_shared::message as msg;

use springcabal::message::{Encoder, send_to_rules};

use crate::catalog::WEAPONS;
use crate::rml::tech_data::{can_activate, find_tech_index};
use crate::state::Global;
use crate::{audio, diagnostics};
use spring::Float3;

pub fn with_point(message: Encoder, point: Float3) -> String {
    message.fields([point.x, point.y, point.z]).finish()
}

pub fn fire_order(global: &Global, target: Float3) -> Option<Float3> {
    let current_weapon = global.simulation.access().current_weapon;
    (!current_weapon.is_some_and(|index| WEAPONS[index].is_beam)).then_some(target)
}

pub fn fire(spring: &spring::Spring<'_, spring::Ui>, order: Option<Float3>) {
    let message = Encoder::new(msg::FIRE);
    send_to_rules(
        spring,
        &match order {
            None => message.finish(),
            Some(target) => with_point(message, target),
        },
    );
}

pub fn cease_fire(spring: &spring::Spring<'_, spring::Ui>) {
    send_to_rules(spring, msg::CEASE_FIRE);
}

pub fn select_weapon(spring: &spring::Spring<'_, spring::Ui>, global: &Global, index: usize) {
    let name = WEAPONS[index].name();
    let enabled = find_tech_index(name).is_some_and(|tech| {
        let tech = global.simulation.access().tech[tech];
        tech.enabled && tech.unlocked
    });
    if !enabled {
        diagnostics::debug(
            spring,
            "weapon selection ignored because technology is locked",
        );
        return;
    }
    global.simulation.access_mut().current_weapon = Some(index);
    send_to_rules(
        spring,
        &Encoder::new(msg::CHANGE_WEAPON).field(name).finish(),
    );
    audio::play_ui(spring, "sounds/select.wav");
}

pub fn activate_tech(spring: &spring::Spring<'_, spring::Ui>, global: &Global, index: usize) {
    let Some(name) = area17_shared::TechId::ALL
        .get(index)
        .map(|tech| tech.name())
    else {
        return;
    };
    let simulation = global.simulation.access();
    if !can_activate(&simulation, index) {
        return;
    }
    let tech = simulation.tech[index];
    drop(simulation);
    if tech.unlocked {
        send_to_rules(spring, &Encoder::new(msg::UPGRADE).field(name).finish());
        audio::play_ui(spring, "sounds/click.wav");
    } else {
        send_to_rules(spring, &Encoder::new(msg::UNLOCK).field(name).finish());
        audio::play_ui(spring, "sounds/select.wav");
    }
}

pub fn toggle_upgrade_panel(spring: &spring::Spring<'_, spring::Ui>, global: &Global) {
    #[cfg(feature = "diagnostics")]
    diagnostics::info(spring, "Area 17 RmlUi callback: technology toggle");
    let mut panel = global.panel.access_mut();
    panel.upgrade_panel_open = !panel.upgrade_panel_open;
    audio::play_ui(spring, "sounds/click.wav");
}

pub fn close_upgrade_panel(spring: &spring::Spring<'_, spring::Ui>, global: &Global) {
    #[cfg(feature = "diagnostics")]
    diagnostics::info(spring, "Area 17 RmlUi callback: technology close");
    global.panel.access_mut().upgrade_panel_open = false;
    audio::play_ui(spring, "sounds/click.wav");
}

#[cfg(feature = "e2e")]
pub fn grant_test_resources(spring: &spring::Spring<'_, spring::Ui>) {
    diagnostics::info(spring, "Area 17 e2e: test resources requested");
    send_to_rules(spring, "cheatResources");
    audio::play_ui(spring, "sounds/select.wav");
}
