use alloc::vec::Vec;

use super::{TestState, with_test};
use crate::defs::{self, UnitKind};
use crate::state::{self, Global};
use crate::{definitions, script_bridge};

const BUILD: i32 = 600;
const FLOAT: i32 = BUILD + 6;
const DESTROY: i32 = BUILD + 10;
const WALK: i32 = BUILD + 12;
const CHECK: i32 = DESTROY + 30;
const OFFSETS: [(f32, f32); 12] = [
    (1_500.0, 0.0),
    (0.0, 1_500.0),
    (-1_500.0, 0.0),
    (0.0, -1_500.0),
    (1_200.0, 1_200.0),
    (-1_200.0, 1_200.0),
    (1_200.0, -1_200.0),
    (-1_200.0, -1_200.0),
    (2_200.0, 0.0),
    (0.0, 2_200.0),
    (-2_200.0, 0.0),
    (0.0, -2_200.0),
];
const CLEARANCE: [(f32, f32); 5] = [
    (0.0, 300.0),
    (300.0, 0.0),
    (0.0, -300.0),
    (-300.0, 0.0),
    (400.0, 400.0),
];

#[derive(Default)]
pub(super) struct Orphans {
    building: Option<i32>,
    civilians: Vec<i32>,
    loner: Option<i32>,
}

pub(super) fn unit_created(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    test: &mut TestState,
    unit_id: i32,
    unit_def_id: i32,
) {
    if test.orphans.building.is_some()
        && matches!(
            defs::kind(spring, unit_def_id),
            Some(UnitKind::Civilian | UnitKind::Civilian1)
        )
    {
        test.orphans.civilians.push(unit_id);
    }
}

pub(super) fn run(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    test: &mut TestState,
    global: &Global,
    elapsed: i32,
) {
    match elapsed {
        BUILD => build(spring, test, global),
        FLOAT => {
            if let Some(loner) = test.orphans.loner {
                script_bridge::abduction_float(spring, loner);
            }
        }
        WALK => {
            if let Some(loner) = test.orphans.loner
                && let Some(position) = definitions::unit_position(spring, loner)
            {
                let Some((x, z)) = definitions::clamp_xz(spring, position.x + 300.0, position.z)
                else {
                    test.fail(spring, "could not clamp the civilian test destination");
                    return;
                };
                let _ = crate::with_addon_context(|ctx| {
                    ctx.commands()
                        .give_order_to_unit(loner, spring::cmd::MOVE, &[x, 0.0, z], 0, -1)
                });
            }
        }
        DESTROY => destroy(spring, test),
        CHECK => check(spring, test),
        _ => {}
    }
}

fn build(spring: &spring::Spring<'_, spring::RulesSynced>, test: &mut TestState, global: &Global) {
    let ufo = definitions::unit_position(spring, global.ufo.access().id).unwrap_or_default();
    let fits_civilian = |x: f32, z: f32| {
        let y = spring.terrain().get_ground_height(x, z).unwrap_or(0.0);
        definitions::position_is_buildable(spring, UnitKind::Civilian, x, y, z)
    };
    let Some((x, z)) = OFFSETS
        .iter()
        .map(|(dx, dz)| (ufo.x + dx, ufo.z + dz))
        .find(|(x, z)| {
            CLEARANCE
                .iter()
                .all(|(dx, dz)| fits_civilian(x + dx, z + dz))
        })
    else {
        test.fail(spring, "found no spot for the civilians' test building");
        return;
    };
    let y = spring.terrain().get_ground_height(x, z).unwrap_or(0.0);
    definitions::create_at_height(spring, UnitKind::Building3, x, y, z, state::ENEMY_TEAM, 0.0)
        .then(|spring, building| {
            with_test(|test| match building {
                Some(building) => test.orphans.building = Some(building),
                None => test.fail(spring, "could not create the civilians' test building"),
            });
        });
    let (x, z) = (x - 300.0, z - 300.0);
    let y = spring.terrain().get_ground_height(x, z).unwrap_or(0.0);
    definitions::create_at_position(spring, UnitKind::Scientist, x, y, z, state::ENEMY_TEAM, 0.0)
        .then(|_spring, loner| with_test(|test| test.orphans.loner = loner));
}

fn destroy(spring: &spring::Spring<'_, spring::RulesSynced>, test: &mut TestState) {
    let Some(building) = test.orphans.building.take() else {
        return;
    };
    if test.orphans.civilians.is_empty() {
        test.fail(spring, "the test building spawned no civilians");
        return;
    }
    for civilian in &test.orphans.civilians {
        let _ = definitions::stop(spring, *civilian);
    }
    super::remove_unit(spring, building);
}

fn check(spring: &spring::Spring<'_, spring::RulesSynced>, test: &mut TestState) {
    let civilians = core::mem::take(&mut test.orphans.civilians);
    let alive: Vec<i32> = civilians
        .into_iter()
        .filter(|unit| spring.units_info().get_unit_def_id(*unit).is_ok())
        .collect();
    let moving = alive
        .iter()
        .filter(|unit| {
            spring
                .units_commands()
                .get_command_queue(**unit, 1)
                .is_ok_and(|queue| !queue.is_empty())
        })
        .count();
    let message = alloc::format!(
        "{moving} of {} civilians of a destroyed building got a new destination when idle",
        alive.len()
    );
    for unit in &alive {
        super::remove_unit(spring, *unit);
    }
    if alive.is_empty() || moving != alive.len() {
        test.fail(spring, message);
        return;
    }
    test.pass(spring, message);
    let Some(loner) = test.orphans.loner.take() else {
        test.fail(spring, "could not create the civilian for the float check");
        return;
    };
    super::remove_unit(spring, loner);
    let walks = script_bridge::human_walks(spring, loner);
    if walks != Some(true) {
        test.fail(
            spring,
            alloc::format!(
                "a civilian the beam floated kept its float animation once it walked ({walks:?})"
            ),
        );
        return;
    }
    test.pass(
        spring,
        "a civilian the beam floated plays its walk animation once it walks",
    );
}
