use super::TestState;
use crate::state::{self, BeamState, Global};
use crate::{definitions, defs};

const GOD_GUN_OBSERVE_FRAME: i32 = 860;
const GOD_GUN_TERRAIN_SETTLE_FRAMES: i32 = 12;
const MIN_GOD_GUN_CRATER_DEPTH: f32 = 100.0;
pub(super) const TERRAIN_SAMPLES: usize = 17;
const TERRAIN_SAMPLE_OFFSETS: [(f32, f32); TERRAIN_SAMPLES] = [
    (0.0, 0.0),
    (-64.0, 0.0),
    (64.0, 0.0),
    (0.0, -64.0),
    (0.0, 64.0),
    (-64.0, -64.0),
    (-64.0, 64.0),
    (64.0, -64.0),
    (64.0, 64.0),
    (-128.0, 0.0),
    (128.0, 0.0),
    (0.0, -128.0),
    (0.0, 128.0),
    (-128.0, -128.0),
    (-128.0, 128.0),
    (128.0, -128.0),
    (128.0, 128.0),
];

pub(super) fn explosion(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    test: &mut TestState,
    weapon_id: i32,
    position: spring::Float3,
) {
    if !defs::god_gun_weapons(spring).contains(&weapon_id) {
        return;
    }
    test.god_gun_explosions += 1;
    let ground = spring
        .terrain()
        .get_ground_height(position.x, position.z)
        .unwrap_or(position.y);
    let altitude = position.y - ground;
    test.god_gun_min_altitude = test.god_gun_min_altitude.min(altitude);
    if test.god_gun_impact.is_some() || altitude > 2.0 {
        return;
    }
    test.god_gun_impact_altitude = altitude;
    test.god_gun_impact_frame = spring.game().current_frame().unwrap_or(0);
    test.god_gun_ground_before = sample_ground(spring, position);
    test.god_gun_impact = Some(position);
}

pub(super) fn run(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    test: &mut TestState,
    global: &Global,
    elapsed: i32,
) {
    if elapsed == 735 {
        let ufo_id = global.ufo.access().id;
        if let Some(position) = definitions::unit_position(spring, ufo_id) {
            let doomed: alloc::vec::Vec<i32> = spring
                .units_query()
                .get_units_in_cylinder(position.x, position.z, 384.0, -1)
                .unwrap_or_default()
                .into_iter()
                .filter(|unit_id| *unit_id != ufo_id)
                .collect();
            for unit_id in doomed {
                let _ = definitions::destroy_reclaimed(spring, unit_id);
            }
        }
        return;
    }
    if elapsed == 740 {
        test.god_gun_impact = None;
        test.god_gun_impact_frame = -1;
        test.god_gun_min_altitude = f32::MAX;
        test.god_gun_explosions = 0;
        test.god_gun_first_impact_verified = false;
        super::send_input(spring, test, "ability|independenceDayGun");
        return;
    }
    if !test.god_gun_first_impact_verified && !verify_first_impact(spring, test, elapsed) {
        return;
    }
    if elapsed < GOD_GUN_OBSERVE_FRAME {
        return;
    }
    let active =
        global.tech.access().ability_durations[state::Ability::IndependenceDayGun.index()] > 0;
    let script_beam = spring
        .rules_params()
        .get_unit_rules_param_f32(global.ufo.access().id, "beam_enabled")
        .unwrap_or_default();
    if !active || global.tech.access().beam != BeamState::GodGun || script_beam != -1.0 {
        test.fail(
            spring,
            "God Gun did not activate the Rust state and Lua model script",
        );
        return;
    }
    let Some(impact) = test.god_gun_impact else {
        test.fail(spring, "God Gun produced no engine Explosion callin");
        return;
    };
    let crater_depth = terrain_deformation(spring, impact, test.god_gun_ground_before);
    if crater_depth < MIN_GOD_GUN_CRATER_DEPTH {
        test.fail(spring, alloc::format!(
            "God Gun crater was too shallow at its actual impact: {crater_depth:.1}, expected at least {MIN_GOD_GUN_CRATER_DEPTH:.1}; impact altitude={:.1}, explosions={} ",
            test.god_gun_impact_altitude,
            test.god_gun_explosions,
        ));
        return;
    }
    test.pass(spring, alloc::format!(
            "God Gun produced {} native explosions at ({:.1}, {:.1}, {:.1}), {:.1} above terrain, and deformed that impact area by {crater_depth:.1} height units",
            test.god_gun_explosions,
            impact.x,
            impact.y,
            impact.z,
            test.god_gun_impact_altitude,
        ),
    );
    test.finish(spring);
}

fn verify_first_impact(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    test: &mut TestState,
    elapsed: i32,
) -> bool {
    let Some(impact) = test.god_gun_impact else {
        if elapsed < GOD_GUN_OBSERVE_FRAME {
            return false;
        }
        test.fail(
            spring,
            alloc::format!(
                "God Gun produced no ground impact; closest explosion remained {:.1} above terrain",
                test.god_gun_min_altitude,
            ),
        );
        return false;
    };
    if spring.game().current_frame().unwrap_or(0) - test.god_gun_impact_frame
        < GOD_GUN_TERRAIN_SETTLE_FRAMES
    {
        return false;
    }
    let deformation = terrain_deformation(spring, impact, test.god_gun_ground_before);
    if deformation <= 0.1 {
        test.fail(
            spring,
            alloc::format!(
                "God Gun's direct ground impact did not deform terrain; impact altitude={:.1}",
                test.god_gun_impact_altitude,
            ),
        );
        return false;
    }
    test.god_gun_first_impact_verified = true;
    test.pass(spring, alloc::format!(
            "God Gun's direct ground impact deformed terrain by {deformation:.1} height units from {:.1} above ground",
            test.god_gun_impact_altitude,
        ),
    );
    true
}

fn sample_ground(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    position: spring::Float3,
) -> [f32; TERRAIN_SAMPLES] {
    core::array::from_fn(|index| {
        let (x, z) = TERRAIN_SAMPLE_OFFSETS[index];
        spring
            .terrain()
            .get_ground_height(position.x + x, position.z + z)
            .unwrap_or_default()
    })
}

fn terrain_deformation(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    position: spring::Float3,
    before: [f32; TERRAIN_SAMPLES],
) -> f32 {
    sample_ground(spring, position)
        .into_iter()
        .zip(before)
        .map(|(after, before)| (after - before).abs())
        .fold(0.0, f32::max)
}
