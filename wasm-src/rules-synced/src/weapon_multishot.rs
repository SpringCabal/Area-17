use crate::{state, upgrades};
use state::{Global, ufo_slots};

pub fn apply(spring: &spring::Spring<'_, spring::RulesSynced>, global: &Global) -> bool {
    let ufo_id = global.ufo.access().id;
    if ufo_id < 0 {
        return false;
    }
    let tech = global.tech.access();
    let mut applied = true;
    let pulse_level = state::tech(&tech, state::TechId::PulseLaser).level;
    if pulse_level > 0 {
        applied &= spring
            .unit_control()
            .set_unit_weapon_state(
                ufo_id,
                ufo_slots::PULSE + 1,
                "projectiles",
                pulse_level as f32,
            )
            .unwrap_or(false);
    }

    let point_defense_level = state::tech(&tech, state::TechId::AntiMissilePointDefense).level;
    if point_defense_level > 0 {
        let Some(definition) = crate::definitions::unit_def_id(spring, ufo_id)
            .and_then(|def| spring.unit_defs().get_unit_def_by_id(def).ok())
        else {
            spring::log_warning!(
                spring,
                "area17",
                "Point defense: UFO definition is unavailable"
            );
            return false;
        };
        let multiplier =
            1.0 + upgrades::tech_value_percent(&tech, state::TechId::AntiMissilePointDefense);
        for slot in ufo_slots::POINT_DEFENSE {
            let Some(weapon) = definition
                .weapons
                .weapon_def_i_ds
                .get(slot as usize)
                .and_then(|def| spring.weapon_defs().get_weapon_def_by_id(*def).ok())
            else {
                spring::log_warning!(
                    spring,
                    "area17",
                    "Point defense: weapon slot {slot} definition is unavailable"
                );
                applied = false;
                continue;
            };
            let reload = weapon.info.reload_time / multiplier;
            applied &= spring
                .unit_control()
                .set_unit_weapon_state(ufo_id, slot + 1, "reloadTime", reload)
                .unwrap_or(false);
        }
    }
    applied
}
