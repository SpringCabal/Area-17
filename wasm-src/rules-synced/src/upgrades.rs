use crate::{definitions, defs, diagnostics, state, weapon_multishot};
use spring::ResultExt;
use state::{Ability, BeamState, Economy, Global, Tech, TechId};

const BASE_SHIELD_REGEN: f32 = 1.5;
const MAX_SHIELD_POWER: f32 = 1_000.0;

pub fn update(spring: &spring::Spring<'_, spring::RulesSynced>, global: &Global) {
    if global.ufo.access().id < 0 {
        return;
    }
    apply_effects(spring, global);
    update_shield(spring, global);
    update_ability_timers(spring, global);
}

pub fn unlock(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &Global,
    tech_id: TechId,
) -> bool {
    {
        let mut tech = global.tech.access_mut();
        if !can_unlock(&tech, &global.economy.access(), tech_id) {
            return false;
        }
        global.economy.access_mut().research -= tech_id.unlock_cost();
        state::tech_mut(&mut tech, tech_id).unlocked = true;
        refresh_enabled(&mut tech);
    }
    apply_effects(spring, global);
    diagnostics::info(spring, "Area 17 technology unlocked");
    true
}

pub fn upgrade(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &Global,
    tech_id: TechId,
) -> bool {
    {
        let mut tech = global.tech.access_mut();
        if !can_upgrade(&tech, &global.economy.access(), tech_id) {
            return false;
        }
        global.economy.access_mut().biomass -= upgrade_cost(&tech, tech_id);
        state::tech_mut(&mut tech, tech_id).level += 1;
    }
    apply_effects(spring, global);
    true
}

pub fn is_enabled(tech: &Tech, tech_id: TechId) -> bool {
    let state = state::tech(tech, tech_id);
    state.enabled && state.unlocked
}

pub fn incendiary_damage_multiplier(tech: &Tech) -> f32 {
    1.0 + tech_value(tech, TechId::IncendiaryBeamLaser) / 100.0
}

pub fn incendiary_duration_multiplier(tech: &Tech) -> f32 {
    incendiary_damage_multiplier(tech)
}

pub fn black_hole_radius_multiplier(tech: &Tech) -> f32 {
    1.0 + tech_value(tech, TechId::BlackHoleGun) / 100.0
}

pub fn black_hole_duration_multiplier(tech: &Tech) -> f32 {
    black_hole_radius_multiplier(tech)
}

pub fn gravity_impulse_multiplier(tech: &Tech) -> f32 {
    1.0 + tech_value(tech, TechId::GravityBeam) / 100.0
}

pub fn activate_ability(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &Global,
    ability: Ability,
) {
    let tech_id = ability.tech();
    let ufo_id = global.ufo.access().id;
    let index = ability.index();
    let level = {
        let tech = global.tech.access();
        if !is_enabled(&tech, tech_id) || ufo_id < 0 {
            return;
        }
        if tech.ability_cooldowns[index] > 0 || tech.ability_durations[index] > 0 {
            return;
        }
        state::tech(&tech, tech_id).level
    };
    let multiplier = tech_id.value(level) / 100.0;
    match ability {
        Ability::Haste => {
            set_ufo_multiplier(spring, ufo_id, 1.6 * (1.0 + multiplier));
            play_sound(spring, ufo_id, "sounds/haste.wav", 40.0);
        }
        Ability::Cloak => {
            spring
                .unit_control()
                .set_unit_cloak(
                    ufo_id,
                    spring::unit_control::NumberOrBool {
                        number: 4.0,
                        boolean: false,
                        use_boolean: false,
                    },
                    spring::unit_control::NumberOrBool::default(),
                )
                .log_err_once(spring, "Area 17 upgrades: set unit cloak");
            play_sound(spring, ufo_id, "sounds/cloak.wav", 40.0);
        }
        Ability::Teleport => {
            teleport(spring, ufo_id, 800.0 * (1.0 + multiplier));
            play_sound(spring, ufo_id, "sounds/teleport.wav", 40.0);
        }
        Ability::IndependenceDayGun => {
            global.tech.access_mut().beam = BeamState::GodGun;
        }
    }
    {
        let mut tech = global.tech.access_mut();
        tech.ability_cooldowns[index] = ability.cooldown(level);
        tech.ability_durations[index] = ability.duration(level);
    }
}

pub fn initialize(global: &Global) {
    {
        let mut tech = global.tech.access_mut();
        for tech_id in TechId::ALL {
            let entry = state::tech_mut(&mut tech, tech_id);
            entry.enabled = false;
            entry.unlocked = tech_id.starts_unlocked();
        }
        refresh_enabled(&mut tech);
        tech.shield_power = MAX_SHIELD_POWER;
        tech.beam = BeamState::Idle;
    }
    *global.economy.access_mut() = Economy::default();
}

fn refresh_enabled(tech: &mut Tech) {
    for tech_id in TechId::ALL {
        let enabled = tech_id.dependencies().iter().all(|dependency| {
            let dependency_state = state::tech(tech, *dependency);
            dependency_state.enabled && dependency_state.unlocked
        });
        state::tech_mut(tech, tech_id).enabled = enabled;
    }
}

fn apply_effects(spring: &spring::Spring<'_, spring::RulesSynced>, global: &Global) {
    let ufo_id = global.ufo.access().id;
    if ufo_id < 0 {
        return;
    }
    let (armor_multiplier, cone_multiplier, point_defense) = {
        let tech = global.tech.access();
        if tech.effects_applied && tech.effects_ufo_id == ufo_id && tech.effects_tech == tech.tech {
            return;
        }
        (
            1.0 + tech_value(&tech, TechId::Armor) / 100.0,
            1.0 + tech_value(&tech, TechId::ConeUpgrade) / 100.0,
            is_enabled(&tech, TechId::AntiMissilePointDefense),
        )
    };

    if let Some(base_health) = definitions::unit_def_id(spring, ufo_id)
        .and_then(|id| defs::unit_data(spring, id))
        .map(|physics| physics.health)
    {
        let health_ratio = match definitions::unit_health(spring, ufo_id) {
            Some((health, max_health)) if max_health > 0.0 => (health / max_health).clamp(0.0, 1.0),
            _ => 1.0,
        };
        let max_health = base_health * armor_multiplier;
        spring
            .unit_control()
            .set_unit_max_health(ufo_id, max_health)
            .log_err_once(spring, "Area 17 upgrades: set unit max health");
        let _ = definitions::set_health(spring, ufo_id, max_health * health_ratio);
    }
    spring
        .rules_params()
        .set_unit_rules_param_f32(
            ufo_id,
            "PDisUnlocked",
            if point_defense { 1.0 } else { 0.0 },
            0,
        )
        .log_err_once(spring, "Area 17 upgrades: set unit rules param f32");
    if !weapon_multishot::apply(spring, global) {
        spring::log_warning!(
            spring,
            crate::diagnostics::SECTION,
            "Area 17 engine rejected the upgraded Pulse Laser or Point Defense"
        );
    }

    let mut tech = global.tech.access_mut();
    tech.abduction_radius = state::DEFAULT_ABDUCTION_RADIUS * cone_multiplier;
    tech.abduction_speed = state::DEFAULT_ABDUCTION_SPEED * cone_multiplier;
    tech.abduction_accel = cone_multiplier;
    tech.effects_ufo_id = ufo_id;
    tech.effects_tech = tech.tech;
    tech.effects_applied = true;
}

fn update_shield(spring: &spring::Spring<'_, spring::RulesSynced>, global: &Global) {
    let ufo_id = global.ufo.access().id;
    let (enabled, shield_multiplier, stored_power) = {
        let tech = global.tech.access();
        (
            is_enabled(&tech, TechId::Shield),
            1.0 + tech_value(&tech, TechId::Shield) / 100.0,
            tech.shield_power,
        )
    };
    if !enabled {
        global.tech.access_mut().shield_power = 0.0;
        spring
            .unit_control()
            .set_unit_shield_state(ufo_id, -1, false, 0.0)
            .log_err_once(spring, "Area 17 upgrades: set unit shield state");
        return;
    }
    let live_power = spring
        .units_info()
        .get_unit_shield_state(ufo_id, -1)
        .ok()
        .flatten()
        .map(|shield| shield.shield_power)
        .unwrap_or(stored_power);
    let shield_power = (live_power + BASE_SHIELD_REGEN * shield_multiplier).min(MAX_SHIELD_POWER);
    global.tech.access_mut().shield_power = shield_power;
    spring
        .unit_control()
        .set_unit_shield_state(ufo_id, -1, true, shield_power)
        .log_err_once(spring, "Area 17 upgrades: set unit shield state");
}

fn update_ability_timers(spring: &spring::Spring<'_, spring::RulesSynced>, global: &Global) {
    for ability in Ability::ALL {
        let index = ability.index();
        let expiring = {
            let mut tech = global.tech.access_mut();
            tech.ability_cooldowns[index] = tech.ability_cooldowns[index].saturating_sub(1).max(0);
            tech.ability_durations[index] == 1
        };
        if expiring {
            disable_ability(spring, global, ability);
        }
        let mut tech = global.tech.access_mut();
        tech.ability_durations[index] = tech.ability_durations[index].saturating_sub(1).max(0);
    }
}

fn disable_ability(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &Global,
    ability: Ability,
) {
    let ufo_id = global.ufo.access().id;
    match ability {
        Ability::Haste => set_ufo_multiplier(spring, ufo_id, 1.0),
        Ability::Cloak => {
            spring
                .unit_control()
                .set_unit_cloak(
                    ufo_id,
                    spring::unit_control::NumberOrBool::default(),
                    spring::unit_control::NumberOrBool::default(),
                )
                .log_err_once(spring, "Area 17 upgrades: set unit cloak");
        }
        Ability::Teleport => {}
        Ability::IndependenceDayGun => {
            global.tech.access_mut().beam = BeamState::Idle;
        }
    }
}

fn set_ufo_multiplier(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    ufo_id: i32,
    multiplier: f32,
) {
    definitions::set_gunship_attributes(spring, ufo_id, multiplier, multiplier * multiplier);
    spring
        .rules_params()
        .set_unit_rules_param_f32(
            ufo_id,
            "selfMoveSpeedChange",
            multiplier,
            spring::rules_params::los::PUBLIC,
        )
        .log_err_once(spring, "Area 17 upgrades: set unit rules param f32");
    spring
        .rules_params()
        .set_unit_rules_param_f32(
            ufo_id,
            "selfMaxAccelerationChange",
            multiplier,
            spring::rules_params::los::PUBLIC,
        )
        .log_err_once(spring, "Area 17 upgrades: set unit rules param f32");
}

fn teleport(spring: &spring::Spring<'_, spring::RulesSynced>, ufo_id: i32, distance: f32) {
    let Some(position) = definitions::unit_position(spring, ufo_id) else {
        return;
    };
    let Some(velocity) = definitions::unit_velocity(spring, ufo_id) else {
        return;
    };
    let length = (velocity.x * velocity.x + velocity.z * velocity.z).sqrt();
    if length <= f32::EPSILON {
        return;
    }
    let Some((x, z)) = definitions::clamp_xz(
        spring,
        position.x + velocity.x / length * distance,
        position.z + velocity.z / length * distance,
    ) else {
        return;
    };
    let old_ground = spring
        .terrain()
        .get_ground_height(position.x, position.z)
        .unwrap_or(0.0);
    let new_ground = spring.terrain().get_ground_height(x, z).unwrap_or(0.0);
    let _ = definitions::set_position(
        spring,
        ufo_id,
        spring::Float3 {
            x,
            y: position.y - old_ground + new_ground,
            z,
        },
    );
}

fn play_sound(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    ufo_id: i32,
    file: &str,
    volume: f32,
) {
    let position = definitions::unit_position(spring, ufo_id).unwrap_or_default();
    spring
        .sound()
        .play_sound_file(file, volume, position, spring::Float3::default(), 0)
        .log_err_once(spring, "Area 17 upgrades: play sound file");
}

fn can_unlock(tech: &Tech, economy: &Economy, tech_id: TechId) -> bool {
    let state = state::tech(tech, tech_id);
    !state.unlocked && state.enabled && economy.research >= tech_id.unlock_cost()
}

fn can_upgrade(tech: &Tech, economy: &Economy, tech_id: TechId) -> bool {
    let state = state::tech(tech, tech_id);
    state.unlocked
        && state.enabled
        && state.level < tech_id.max_level()
        && economy.biomass >= upgrade_cost(tech, tech_id)
}

fn upgrade_cost(tech: &Tech, tech_id: TechId) -> f32 {
    tech_id.upgrade_cost(state::tech(tech, tech_id).level)
}

fn tech_value(tech: &Tech, tech_id: TechId) -> f32 {
    tech_id.value(state::tech(tech, tech_id).level)
}

pub fn tech_value_percent(tech: &Tech, tech_id: TechId) -> f32 {
    tech_value(tech, tech_id) / 100.0
}
