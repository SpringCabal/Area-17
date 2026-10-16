use alloc::vec::Vec;

use spring::commands;

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

pub(super) fn unit_created(test: &mut TestState, unit_id: i32, unit_def_id: i32) {
    if test.orphans.building.is_some()
        && matches!(
            defs::kind(unit_def_id),
            Some(UnitKind::Civilian | UnitKind::Civilian1)
        )
    {
        test.orphans.civilians.push(unit_id);
    }
}

pub(super) fn run(test: &mut TestState, global: &Global, elapsed: i32) {
    match elapsed {
        BUILD => build(test, global),
        FLOAT => {
            if let Some(loner) = test.orphans.loner {
                script_bridge::abduction_float(loner);
            }
        }
        WALK => {
            if let Some(loner) = test.orphans.loner
                && let Some(position) = definitions::unit_base_position(loner)
            {
                let Some((x, z)) = definitions::clamp_xz(position.x + 300.0, position.z) else {
                    test.fail("could not clamp the civilian test destination");
                    return;
                };
                let _ = commands::give_order_to_unit(loner, spring::cmd::MOVE, &[x, 0.0, z], 0, -1);
            }
        }
        DESTROY => destroy(test),
        CHECK => check(test),
        _ => {}
    }
}

fn build(test: &mut TestState, global: &Global) {
    let ufo = definitions::unit_position(global.ufo.access().id).unwrap_or_default();
    let fits_civilian = |x: f32, z: f32| {
        let y = spring::get_ground_height(x, z).unwrap_or(0.0);
        definitions::position_is_buildable(UnitKind::Civilian, x, y, z)
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
        test.fail("found no spot for the civilians' test building");
        return;
    };
    let y = spring::get_ground_height(x, z).unwrap_or(0.0);
    definitions::create_at_height(UnitKind::Building3, x, y, z, state::ENEMY_TEAM, 0.0).then(
        |building| {
            with_test(|test| match building {
                Some(building) => test.orphans.building = Some(building),
                None => test.fail("could not create the civilians' test building"),
            });
        },
    );
    let (x, z) = (x - 300.0, z - 300.0);
    let y = spring::get_ground_height(x, z).unwrap_or(0.0);
    definitions::create_at_position(UnitKind::Scientist, x, y, z, state::ENEMY_TEAM, 0.0)
        .then(|loner| with_test(|test| test.orphans.loner = loner));
}

fn destroy(test: &mut TestState) {
    let Some(building) = test.orphans.building.take() else {
        return;
    };
    if test.orphans.civilians.is_empty() {
        test.fail("the test building spawned no civilians");
        return;
    }
    for civilian in &test.orphans.civilians {
        let _ = definitions::stop(*civilian);
    }
    super::remove_unit(building);
}

fn check(test: &mut TestState) {
    let civilians = core::mem::take(&mut test.orphans.civilians);
    let alive: Vec<i32> = civilians
        .into_iter()
        .filter(|unit| spring::get_unit_def_id(*unit).is_ok())
        .collect();
    let moving = alive
        .iter()
        .filter(|unit| spring::get_command_queue(**unit, 1).is_ok_and(|queue| !queue.is_empty()))
        .count();
    let message = alloc::format!(
        "{moving} of {} civilians of a destroyed building got a new destination when idle",
        alive.len()
    );
    for unit in &alive {
        super::remove_unit(*unit);
    }
    if alive.is_empty() || moving != alive.len() {
        test.fail(message);
        return;
    }
    test.pass(message);
    let Some(loner) = test.orphans.loner.take() else {
        test.fail("could not create the civilian for the float check");
        return;
    };
    super::remove_unit(loner);
    let walks = script_bridge::human_walks(loner);
    if walks != Some(true) {
        test.fail(alloc::format!(
            "a civilian the beam floated kept its float animation once it walked ({walks:?})"
        ));
        return;
    }
    test.pass("a civilian the beam floated plays its walk animation once it walks");
}
