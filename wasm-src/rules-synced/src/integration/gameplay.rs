use spring::commands;
use springcabal::message::Encoder;

use super::{TestState, with_test};
use crate::definitions;
use crate::defs::{self, UnitKind};
use crate::fall_damage;
use crate::state::{self, Global, TechId};
use spring::Float3;

const START: i32 = 215;
const LAUNCH: i32 = START + 5;
const ARMOR_CHECK: i32 = START + 5;
const COLLISION: i32 = START + 10;
const COLLISION_CHECK: i32 = COLLISION + 2;
const DEADLINE: i32 = START + 140;
const HEAL_FRACTION: f32 = 0.05;
const LAUNCH_SPEED: f32 = 8.0;
const OFFSET: f32 = 500.0;
const COLLISION_DAMAGE: f32 = 10.0;
const UNPAIRED_MULTIPLIER: f32 = 2.5;
const CAPTURE_ALTITUDE: f32 = 200.0;
const CAPTURE_SPOTS: [(f32, f32); 6] = [
    (-OFFSET, OFFSET),
    (OFFSET, -OFFSET),
    (0.0, 2.0 * OFFSET),
    (2.0 * OFFSET, 0.0),
    (0.0, -2.0 * OFFSET),
    (-2.0 * OFFSET, 0.0),
];
const CAPTURE_CLEARANCE: f32 = 300.0;
const VELOCITY_THRESHOLD: f32 = 3.0;
const DAMAGE_PER_MASS: f32 = 0.6;
const TANK_MASS: f32 = 100.0;

#[derive(Default)]
pub(super) struct Gameplay {
    health_before_armor: f32,
    max_before_armor: f32,
    capture: Option<Capture>,
    fall: Option<Fall>,
    collision: Option<Fall>,
    fall_launched: bool,
}

struct Capture {
    unit: i32,
    hover: Float3,
    biomass: f32,
    health: f32,
    max_health: f32,
}

struct Fall {
    unit: i32,
    health: f32,
}

pub(super) fn run(test: &mut TestState, global: &Global, elapsed: i32) {
    if !(START..=DEADLINE).contains(&elapsed) {
        return;
    }
    let ufo_id = global.ufo.access().id;
    match elapsed {
        START => start(test, ufo_id),
        LAUNCH => {
            check_armor(test, ufo_id);
            launch(test);
        }
        COLLISION => start_collision(test, global, ufo_id),
        COLLISION_CHECK => check_collision(test),
        _ => {}
    }
    if elapsed > ARMOR_CHECK {
        check_capture(test, global, ufo_id, elapsed);
        check_fall(test, elapsed);
    }
}

fn start(test: &mut TestState, ufo_id: i32) {
    check_footprint(test);

    let Some((_, max_health)) = definitions::unit_health(ufo_id) else {
        test.fail("could not read the respawned UFO's health");
        return;
    };
    definitions::set_health(ufo_id, max_health * 0.5);
    test.gameplay.health_before_armor = max_health * 0.5;
    test.gameplay.max_before_armor = max_health;
    let upgrade = Encoder::new("upgrade").field(TechId::Armor.name()).finish();
    super::send_input(test, &upgrade);

    let Some(ufo) = definitions::unit_position(ufo_id) else {
        test.fail("could not read the respawned UFO's position");
        return;
    };
    let (x, z) = (ufo.x + OFFSET, ufo.z + OFFSET);
    let y = spring::get_ground_height(x, z).unwrap_or(ufo.y);
    definitions::create_at_position(UnitKind::Tank, x, y, z, state::PLAYER_TEAM, 0.0).then(
        |tank| {
            let Some(tank) = tank else {
                with_test(|test| test.fail("could not create the fall-damage tank"));
                return;
            };
            let health = definitions::unit_health(tank).map_or(0.0, |(health, _)| health);
            with_test(|test| test.gameplay.fall = Some(Fall { unit: tank, health }));
        },
    );
}

fn check_footprint(test: &mut TestState) {
    let block = |kind| {
        defs::unit_def(kind)
            .and_then(defs::unit_data)
            .and_then(|physics| physics.terrain_block)
    };
    let big_tank = block(UnitKind::Building4BigTank);
    if big_tank != Some((48.0, 48.0)) || block(UnitKind::Building3) != Some((71.0, 71.0)) {
        test.fail(alloc::format!(
            "terrainblock custom params were not read: Building4BigTank={big_tank:?}"
        ));
        return;
    }
    test.pass("building footprints come from the terrainblock custom params");
}

fn check_armor(test: &mut TestState, ufo_id: i32) {
    let Some((health, max_health)) = definitions::unit_health(ufo_id) else {
        test.fail("could not read the UFO's health after the armor upgrade");
        return;
    };
    let before = &test.gameplay;
    let ratio = health / max_health.max(1.0);
    if max_health <= before.max_before_armor || (ratio - 0.5).abs() > 0.01 {
        test.fail(alloc::format!(
            "the armor upgrade changed the UFO's health ratio: {:.1}/{:.1} -> {health:.1}/{max_health:.1}",
            before.health_before_armor,
            before.max_before_armor,
        ));
        return;
    }
    test.pass(alloc::format!(
        "the armor upgrade raised max health to {max_health:.1} and kept the UFO at half health"
    ));
}

fn launch(test: &mut TestState) {
    if let Some(fall) = &test.gameplay.fall {
        test.gameplay.fall_launched = definitions::add_impulse(
            fall.unit,
            Float3 {
                x: 0.0,
                y: LAUNCH_SPEED,
                z: 0.0,
            },
        );
    }
}

fn start_collision(test: &mut TestState, global: &Global, ufo_id: i32) {
    start_capture(test, global, ufo_id);

    let Some(ufo) = definitions::unit_position(ufo_id) else {
        return;
    };
    let (x, z) = (ufo.x - OFFSET, ufo.z - OFFSET);
    let y = spring::get_ground_height(x, z).unwrap_or(ufo.y);
    definitions::create_at_position(UnitKind::Tank, x, y, z, state::PLAYER_TEAM, 0.0).then(
        |unit| {
            let Some(unit) = unit else {
                with_test(|test| test.fail("could not create the collision tank"));
                return;
            };
            let Some((health, _)) = definitions::unit_health(unit) else {
                with_test(|test| test.fail("could not read the collision tank's health"));
                return;
            };
            commands::add_unit_damage(
                unit,
                COLLISION_DAMAGE,
                0.0,
                fall_damage::OBJECT_COLLISION,
                -1,
                spring::Float3::ZERO,
            )
            .then(move |hit| {
                with_test(|test| {
                    if !hit.unwrap_or(false) {
                        test.fail("the engine refused the collision damage");
                        return;
                    }
                    test.gameplay.collision = Some(Fall { unit, health });
                });
            });
        },
    );
}

fn check_collision(test: &mut TestState) {
    let Some(collision) = test.gameplay.collision.take() else {
        return;
    };
    let after = definitions::unit_health(collision.unit).map_or(0.0, |(health, _)| health);
    let expected = COLLISION_DAMAGE * UNPAIRED_MULTIPLIER;
    let taken = collision.health - after;
    if (taken - expected).abs() > 0.1 {
        test.fail(alloc::format!(
            "an engine object collision cost the tank {taken:.1} health, expected {expected:.1}"
        ));
        return;
    }
    test.pass(alloc::format!(
        "an engine object collision without an attacker cost the tank {expected:.1} health a frame later"
    ));
}

fn start_capture(test: &mut TestState, global: &Global, ufo_id: i32) {
    let (Some(ufo), Some((health, max_health))) = (
        definitions::unit_base_position(ufo_id),
        definitions::unit_health(ufo_id),
    ) else {
        test.fail("could not read the UFO before the capture");
        return;
    };
    let Some((x, z)) = CAPTURE_SPOTS
        .iter()
        .map(|(dx, dz)| (ufo.x + dx, ufo.z + dz))
        .find(|(x, z)| {
            spring::get_units_in_cylinder(*x, *z, CAPTURE_CLEARANCE, -1)
                .is_ok_and(|units| units.is_empty())
        })
    else {
        test.fail("found no empty spot for the capture");
        return;
    };
    let y = spring::get_ground_height(x, z).unwrap_or(0.0);
    let hover = Float3 {
        x,
        y: y + CAPTURE_ALTITUDE,
        z,
    };
    if !definitions::set_position(ufo_id, hover) {
        test.fail("could not move the UFO for the capture");
        return;
    }
    let _ = definitions::set_velocity(ufo_id, Float3::default());
    let biomass = global.economy.access().biomass;
    definitions::create_at_position(UnitKind::Civilian, x, y, z, state::ENEMY_TEAM, 0.0).then(
        move |unit| {
            with_test(|test| {
                let Some(unit) = unit else {
                    test.fail("could not create the civilian to capture");
                    return;
                };
                test.gameplay.capture = Some(Capture {
                    unit,
                    hover,
                    biomass,
                    health,
                    max_health,
                });
            });
        },
    );
}

fn check_capture(test: &mut TestState, global: &Global, ufo_id: i32, elapsed: i32) {
    let Some(capture) = &test.gameplay.capture else {
        return;
    };
    if spring::get_unit_def_id(capture.unit).is_ok() {
        if elapsed == DEADLINE {
            test.fail("the beam never captured the civilian under the UFO");
            return;
        }
        let _ = definitions::set_position(ufo_id, capture.hover);
        let _ = definitions::set_velocity(ufo_id, Float3::default());
        return;
    }
    let reward = defs::unit_def(UnitKind::Civilian)
        .and_then(|def| spring::unit_defs::get_unit_def_custom_param(def, "biomass").ok())
        .and_then(|value| value.trim().parse::<f32>().ok())
        .unwrap_or(-1.0);
    let biomass = global.economy.access().biomass - capture.biomass;
    let health = definitions::unit_health(ufo_id).map_or(0.0, |(health, _)| health);
    let heal = capture.max_health * HEAL_FRACTION;
    let healed = health - capture.health;
    let message = alloc::format!(
        "capture: biomass +{biomass:.1} (def {reward:.1}), UFO health +{healed:.1} (expected {heal:.1})"
    );
    test.gameplay.capture = None;
    if (biomass - reward).abs() > 0.5 || (healed - heal).abs() > 1.0 {
        test.fail(message);
        return;
    }
    test.pass(message);
}

fn check_fall(test: &mut TestState, elapsed: i32) {
    let Some(fall) = &test.gameplay.fall else {
        return;
    };
    if !test.gameplay.fall_launched {
        test.fail("the engine rejected the fall-damage tank's impulse");
        return;
    }
    let health = definitions::unit_health(fall.unit).map_or(0.0, |(health, _)| health);
    let loss = fall.health - health;
    if loss <= 0.0 {
        if elapsed == DEADLINE {
            test.fail("the tank took no damage from landing");
        }
        return;
    }
    test.gameplay.fall = None;
    let expected = (LAUNCH_SPEED - VELOCITY_THRESHOLD) * TANK_MASS * DAMAGE_PER_MASS;
    let message = alloc::format!(
        "the tank took {loss:.1} fall damage on landing, about {expected:.0} expected"
    );
    if !(expected * 0.5..=expected * 1.5).contains(&loss) {
        test.fail(message);
        return;
    }
    test.pass(message);
}
