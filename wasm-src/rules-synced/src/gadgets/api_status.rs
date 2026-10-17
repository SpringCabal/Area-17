use alloc::format;
use alloc::string::String;

use spring_addons::rules::Gadget;
use springcabal::message::{Encoder, SendOnChange, send_to_ui};

use crate::definitions;
use crate::script_bridge::{self, UfoControl};
use crate::state::{self, Global, Weapon};
use area17_shared::message;
use spring::Float3;

const STATUS_PERIOD: i32 = 15;

#[derive(Default)]
pub struct ApiStatusGadget {
    status: SendOnChange<String>,
}

impl ApiStatusGadget {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Gadget<Global> for ApiStatusGadget {
    fn name(&self) -> &'static str {
        "ApiStatus"
    }

    fn game_frame(&self, ctx: &spring_addons::AddonContext<'_, Global>, frame: i32) {
        let spring = ctx.spring();
        let global = ctx.global();
        let ufo_id = global.ufo.access().id;
        if let Some(position) = (ufo_id >= 0)
            .then(|| definitions::unit_position(spring, ufo_id))
            .flatten()
        {
            publish_control(spring, global, position);
        }
        if frame.rem_euclid(STATUS_PERIOD) != 0 {
            return;
        }
        self.status
            .send_if_changed(status(spring, global), |status| send_to_ui(spring, status));
    }
}

fn publish_control(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &Global,
    position: Float3,
) {
    let ufo = global.ufo.access();
    let beam = global.tech.access().beam;
    script_bridge::set_ufo_control(
        spring,
        ufo.id,
        UfoControl {
            aim: position + ufo.aim,
            weapon: ufo.current_weapon,
            beam,
        },
    );
}

fn status(spring: &spring::Spring<'_, spring::RulesSynced>, global: &Global) -> String {
    let (ufo_id, current_weapon) = {
        let ufo = global.ufo.access();
        (ufo.id, ufo.current_weapon)
    };
    let (health, max_health) = (ufo_id >= 0)
        .then(|| definitions::unit_health(spring, ufo_id))
        .flatten()
        .filter(|(_, max_health)| *max_health > 0.0)
        .unwrap_or((0.0, 1.0));
    let economy = *global.economy.access();
    let session = *global.session.access();
    let tech = global.tech.access();
    let mut message = Encoder::new(message::STATUS)
        .kv("ufo", ufo_id)
        .kv("health", format_args!("{health:.1}"))
        .kv("max_health", format_args!("{max_health:.1}"))
        .kv("biomass", format_args!("{:.0}", economy.biomass))
        .kv("research", format_args!("{:.0}", economy.research))
        .kv("metal", format_args!("{:.0}", economy.metal))
        .kv("shield", format_args!("{:.1}", tech.shield_power))
        .kv("radius", format_args!("{:.1}", tech.abduction_radius))
        .kv_flag("game_over", session.game_over)
        .kv_flag("game_won", session.game_won)
        .kv("weapon", current_weapon.map_or("", Weapon::name));
    for ability in state::Ability::ALL {
        let index = ability.index();
        message = message
            .kv(
                &format!("{}CD", ability.name()),
                tech.ability_cooldowns[index],
            )
            .kv(
                &format!("{}Duration", ability.name()),
                tech.ability_durations[index],
            );
    }
    for tech_id in state::TechId::ALL {
        let entry = state::tech(&tech, tech_id);
        message = message.kv(
            &format!("tech_{}", tech_id.name()),
            format_args!(
                "{},{},{}",
                entry.level,
                u8::from(entry.unlocked),
                u8::from(entry.enabled)
            ),
        );
    }
    message.finish()
}
