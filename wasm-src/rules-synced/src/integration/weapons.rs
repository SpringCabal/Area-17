use springcabal::message::Encoder;

use super::{TestState, with_test};
use crate::definitions;
use crate::defs::{self, UnitKind};
use crate::state::{self, Global, Weapon};

const WEAPON_SETTLE_FRAMES: i32 = 30;
const FIRE_FRAME: i32 = 35;
const FIRE_OBSERVE_FRAMES: i32 = 65;
const INPUT_START_FRAME: i32 = 360;
const INPUT_STAGE_FRAMES: i32 = 80;

pub(super) fn run_weapon_stage(test: &mut TestState, global: &Global, elapsed: i32) {
    let weapon = Weapon::ALL[test.stage];
    let stage_start = INPUT_START_FRAME + (test.stage as i32 * INPUT_STAGE_FRAMES);
    if elapsed == stage_start {
        test.expected_weapon_id = defs::weapon(weapon);
        test.expected_weapon_slot = state::ufo_slots::of(weapon).start + 1;
        test.expected_owner_id = global.ufo.access().id;
        test.observed_weapon_id = -1;
        test.observed_owner_id = -1;
        test.expected_projectiles = usize::from(weapon == Weapon::PulseLaser) + 1;
        test.observed_projectiles = 0;
        test.target_accepted = false;
        global.ufo.access_mut().target_accepted = false;
        if test.expected_weapon_id < 0 {
            test.fail("could not resolve integration-test weapon definition");
            return;
        }
        if weapon == Weapon::GravityBeam {
            place_gravity_target(test, global);
        }
        let aim = with_ground_target(global, test.stage, Encoder::new("aimWeapon"));
        super::send_input(test, &aim);
        let change = Encoder::new("changeWeapon").field(weapon.name()).finish();
        super::send_input(test, &change);
        return;
    }
    let age = elapsed - stage_start;
    if age == WEAPON_SETTLE_FRAMES {
        if global.ufo.access().current_weapon != Some(weapon) {
            test.fail("unsynced weapon-selection message did not update Rust game state");
            return;
        }
        let selected_slot =
            spring::get_unit_rules_param_f32(global.ufo.access().id, "active_weapon_slot")
                .unwrap_or_default() as i32;
        if selected_slot != test.expected_weapon_slot {
            test.fail(alloc::format!(
                "Lua UFO script did not select the requested weapon (expected slot={}, observed slot={selected_slot})",
                test.expected_weapon_slot,
            ));
        }
        return;
    }
    if age == FIRE_FRAME {
        if let Some(target) = &mut test.gravity_target {
            target.start_distance = distance_to_ufo(global, target.unit);
            target.health = definitions::unit_health(target.unit).map_or(0.0, |(health, _)| health);
        }
        let fire = with_ground_target(global, test.stage, Encoder::new("fireWeapon"));
        super::send_input(test, &fire);
        return;
    }
    if age >= FIRE_OBSERVE_FRAMES {
        if !test.target_accepted {
            test.fail("engine rejected the selected weapon's ground target");
            return;
        }
        if !test.projectile_seen || test.observed_projectiles < test.expected_projectiles {
            test.fail(alloc::format!(
                "selected weapon did not produce its expected engine projectiles (expected at least {} of {} from owner={}, observed {} of {} from owner={})",
                test.expected_projectiles,
                test.expected_weapon_id,
                test.expected_owner_id,
                test.observed_projectiles,
                test.observed_weapon_id,
                test.observed_owner_id,
            ));
            return;
        }
        super::send_input(test, "ceaseFire");
        test.pass("selected weapon reached the unit script and fired its expected projectile");
        if weapon == Weapon::GravityBeam && !check_gravity_target(test, global) {
            return;
        }
        test.stage += 1;
        test.projectile_seen = false;
    }
}

pub(super) fn run_haste_stage(test: &mut TestState, global: &Global, elapsed: i32) {
    if elapsed == 700 {
        super::send_input(test, "ability|haste");
        return;
    }
    if elapsed < 725 {
        return;
    }
    let active = global.tech.access().ability_durations[state::Ability::Haste.index()] > 0;
    let multiplier =
        spring::get_unit_rules_param_f32(global.ufo.access().id, "selfMoveSpeedChange")
            .unwrap_or_default();
    if !active || multiplier <= 1.0 {
        test.fail("haste did not activate its duration and speed multiplier");
        return;
    }
    let ufo = global.ufo.access().id;
    let max_speed = spring::move_ctrl::get_unit_move_type_data(ufo)
        .ok()
        .map(|data| data.max_speed);
    let expected = definitions::unit_def_id(ufo)
        .and_then(defs::unit_data)
        .map(|def| def.speed * 0.05 * multiplier);
    let (Some(max_speed), Some(expected)) = (max_speed, expected) else {
        test.fail("haste: no move type data for the UFO");
        return;
    };
    if (max_speed - expected).abs() > 0.01 * expected {
        test.fail(alloc::format!(
            "haste set the UFO's top speed to {max_speed}, not {expected}"
        ));
        return;
    }
    let Some(definition) =
        definitions::unit_def_id(ufo).and_then(|def| spring::get_unit_def_by_id(def).ok())
    else {
        test.fail("haste: UFO weapon definitions unavailable");
        return;
    };
    let reload_multiplier = 1.0
        + crate::upgrades::tech_value_percent(
            &global.tech.access(),
            state::TechId::AntiMissilePointDefense,
        );
    for slot in state::ufo_slots::POINT_DEFENSE {
        let expected = definition
            .weapons
            .weapon_def_i_ds
            .get(slot as usize)
            .and_then(|def| spring::weapon_defs::get_weapon_def_by_id(*def).ok())
            .map(|weapon| weapon.info.reload_time / reload_multiplier);
        let actual = spring::units_weapons::get_unit_weapon_state(ufo, slot + 1, "reloadTime")
            .ok()
            .map(|weapon| weapon.reload_time);
        if !matches!((actual, expected), (Some(actual), Some(expected))
            if (actual - expected).abs() <= 1.0 / 30.0)
        {
            test.fail(alloc::format!(
                "haste lost point-defense reload in slot {slot}: {actual:?}, expected {expected:?}"
            ));
            return;
        }
    }
    test.pass("haste activated its duration and movement speed while preserving upgraded point-defense reloads");
    test.stage += 1;
}

fn with_ground_target(global: &Global, stage: usize, message: Encoder) -> alloc::string::String {
    let position = definitions::unit_position(global.ufo.access().id).unwrap_or_default();
    let (x, z) = ground_target_xz(position, stage);
    let y = spring::get_ground_height(x, z).unwrap_or(position.y);
    message.fields([x, y, z]).finish()
}

pub(super) struct GravityTarget {
    unit: i32,
    start_distance: f32,
    health: f32,
    hits: [u32; 2],
}

pub(super) fn pre_damaged(test: &mut TestState, unit_id: i32, weapon_def_id: i32) {
    let Some(target) = &mut test.gravity_target else {
        return;
    };
    if unit_id != target.unit {
        return;
    }
    if weapon_def_id == defs::weapon(Weapon::GravityBeam) {
        target.hits[0] += 1;
    } else if Some(weapon_def_id) == gravity_silent_def() {
        target.hits[1] += 1;
    }
}

fn gravity_silent_def() -> Option<i32> {
    spring::weapon_defs::get_weapon_def_id("ufo_gravitybeamsilent").ok()
}

fn place_gravity_target(test: &mut TestState, global: &Global) {
    let position = definitions::unit_position(global.ufo.access().id).unwrap_or_default();
    let (x, z) = ground_target_xz(position, test.stage);
    let y = spring::get_ground_height(x, z).unwrap_or(position.y);
    definitions::create_at_position(UnitKind::Tank, x, y, z, state::PLAYER_TEAM, 0.0).then(
        |unit| {
            with_test(|test| {
                let Some(unit) = unit else {
                    test.fail("could not create the gravity beam's target tank");
                    return;
                };
                test.gravity_target = Some(GravityTarget {
                    unit,
                    start_distance: 0.0,
                    health: 0.0,
                    hits: [0; 2],
                });
            });
        },
    );
}

fn check_gravity_target(test: &mut TestState, global: &Global) -> bool {
    let Some(target) = test.gravity_target.take() else {
        test.fail("the gravity beam's target tank was never placed");
        return false;
    };
    super::remove_unit(target.unit);
    let distance = distance_to_ufo(global, target.unit);
    let health = definitions::unit_health(target.unit).map_or(0.0, |(health, _)| health);
    let message = alloc::format!(
        "gravity beam: {} hits from the def with sound and {} from its silent copies; the tank came {:.0} elmos closer and lost {:.1} health",
        target.hits[0],
        target.hits[1],
        target.start_distance - distance,
        target.health - health,
    );
    if target.hits[0] == 0
        || target.hits[1] < 2 * target.hits[0]
        || target.start_distance - distance < MIN_GRAVITY_PULL
        || health >= target.health
    {
        test.fail(message);
        return false;
    }
    test.pass(message);
    true
}

const MIN_GRAVITY_PULL: f32 = 50.0;

fn distance_to_ufo(global: &Global, unit: i32) -> f32 {
    let ufo = definitions::unit_position(global.ufo.access().id).unwrap_or_default();
    definitions::unit_position(unit).map_or(0.0, |position| position.distance_xz(ufo))
}

pub(super) fn verify_weapon_behaviours(test: &mut TestState) {
    let behaviour = |name: &str| {
        spring::weapon_defs::get_weapon_def_id(name)
            .ok()
            .and_then(defs::weapon_behaviour)
    };
    let pairs = [
        ("ufo_gravitybeam", "ufo_gravitybeamsilent"),
        ("ufo_incendiarybeamlaser", "ufo_incendiarybeamlasersilent"),
    ];
    for (loud, silent) in pairs {
        let (loud_behaviour, silent_behaviour) = (behaviour(loud), behaviour(silent));
        if loud_behaviour.is_none() || loud_behaviour != silent_behaviour {
            test.fail(alloc::format!(
                "{silent} does not behave like {loud}: {silent_behaviour:?} vs {loud_behaviour:?}"
            ));
            return;
        }
    }
    let gravity = behaviour("ufo_gravitybeamsilent").and_then(|behaviour| behaviour.impulse);
    let fire = behaviour("ufo_incendiarybeamlasersilent").and_then(|behaviour| behaviour.area);
    let black_hole = behaviour("ufo_blackholegun");
    if gravity.is_none_or(|impulse| impulse.magnitude != -30.0 || !impulse.normal_damage)
        || fire.is_none_or(|area| area.radius != 40.0 || area.impulse)
        || black_hole.is_none_or(|behaviour| {
            behaviour.fall_immunity != Some(120) || !behaviour.area.is_some_and(|area| area.impulse)
        })
    {
        test.fail(alloc::format!(
            "weapon custom params were misread: gravity {gravity:?}, napalm {fire:?}, black hole {black_hole:?}"
        ));
        return;
    }
    test.pass(
        "every UFO beam slot, silent or not, gets its behaviour from its def's custom params",
    );
}

fn ground_target_xz(ufo: spring::Float3, stage: usize) -> (f32, f32) {
    const OFFSETS: [(f32, f32); 4] = [
        (600.0, 600.0),
        (-600.0, 600.0),
        (600.0, -600.0),
        (-600.0, -600.0),
    ];
    let (x, z) = OFFSETS[stage % OFFSETS.len()];
    (ufo.x + x, ufo.z + z)
}
