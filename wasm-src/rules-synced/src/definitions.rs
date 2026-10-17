use crate::defs::{self, UnitKind};
use spring::cmd;
use spring::commands::Pending;
use spring::consts::GAME_SPEED;
use spring::los_state;
use spring::{Float3, ResultExt};

const GAME_SPEED_SQUARED: f32 = (GAME_SPEED * GAME_SPEED) as f32;
const DEFAULT_GRAVITY_PER_FRAME: f32 = -130.0 / GAME_SPEED_SQUARED;
const LUA_UNIT_SPEED_FACTOR: f32 = 0.05;
const LOS_AND_RADAR: u8 = (los_state::IN_LOS | los_state::IN_RADAR) as u8;
const LOS_ALLY_TEAM: i32 = 0;

pub fn gravity_per_frame(spring: &spring::Spring<'_, spring::RulesSynced>) -> f32 {
    spring
        .game()
        .get_game_map_info()
        .map(|info| info.gravity / GAME_SPEED_SQUARED)
        .unwrap_or(DEFAULT_GRAVITY_PER_FRAME)
}

pub fn set_unit_param(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit_id: i32,
    name: &str,
    value: f32,
) -> bool {
    spring
        .rules_params()
        .set_unit_rules_param_f32(unit_id, name, value, spring::rules_params::los::PRIVATE)
        .log_err_once(spring, "Area 17 set unit param")
        .unwrap_or(false)
}

pub fn unit_position(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit_id: i32,
) -> Option<Float3> {
    spring
        .units_info()
        .get_unit_position(
            unit_id,
            spring::units_info::GetUnitPositionOptions {
                mid_pos: true,
                aim_pos: false,
            },
        )
        .ok()
}

pub fn unit_base_position(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit_id: i32,
) -> Option<Float3> {
    spring
        .units_info()
        .get_unit_position(unit_id, Default::default())
        .ok()
}

pub fn unit_velocity(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit_id: i32,
) -> Option<Float3> {
    spring.units_info().get_unit_velocity(unit_id).ok()
}

pub fn unit_def_id(spring: &spring::Spring<'_, spring::RulesSynced>, unit_id: i32) -> Option<i32> {
    spring.units_info().get_unit_def_id(unit_id).ok()
}

pub fn unit_height(spring: &spring::Spring<'_, spring::RulesSynced>, unit_id: i32) -> f32 {
    spring.units_info().get_unit_height(unit_id).unwrap_or(40.0)
}

pub fn unit_health(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit_id: i32,
) -> Option<(f32, f32)> {
    spring
        .units_info()
        .get_unit_health(unit_id)
        .ok()
        .map(|health| (health.health, health.max_health))
}

pub fn create_at_height(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    kind: UnitKind,
    x: f32,
    y: f32,
    z: f32,
    team: i32,
    angle: f32,
) -> Pending<spring::RulesSynced, Option<i32>> {
    create_at(spring, kind, x, y, z, team, angle, true, false)
}

pub fn create_at_position(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    kind: UnitKind,
    x: f32,
    y: f32,
    z: f32,
    team: i32,
    angle: f32,
) -> Pending<spring::RulesSynced, Option<i32>> {
    create_at(spring, kind, x, y, z, team, angle, false, true)
}

pub fn position_is_buildable(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    kind: UnitKind,
    x: f32,
    y: f32,
    z: f32,
) -> bool {
    let Some(unit_def_id) = defs::unit_def(spring, kind) else {
        return false;
    };
    spring
        .utils()
        .test_build_order(unit_def_id, spring::Float3 { x, y, z }, 0)
        .is_ok_and(|result| result.status != 0)
}

fn create_at(
    _spring: &spring::Spring<'_, spring::RulesSynced>,
    kind: UnitKind,
    x: f32,
    y: f32,
    z: f32,
    team: i32,
    angle: f32,
    flatten_ground: bool,
    apply_direction: bool,
) -> Pending<spring::RulesSynced, Option<i32>> {
    crate::with_addon_context(|ctx| {
        ctx.commands().queue(move |spring| {
            let id = defs::unit_def(spring, kind)?;
            let unit_id = spring
                .unit_control()
                .create_unit(
                    &spring::unit_control::DefRef {
                        name: kind.def_name().into(),
                        id,
                    },
                    spring::Float3 { x, y, z },
                    0,
                    team,
                    spring::unit_control::CreateUnitOptions {
                        unit_id: -1,
                        builder_id: -1,
                        build: false,
                        flatten_ground,
                    },
                )
                .log_err_once(spring, "Area 17 create unit")?;

            if apply_direction {
                let heading = angle.to_radians();
                spring
                    .unit_control()
                    .set_unit_direction(
                        unit_id,
                        spring::Float3 {
                            x: heading.sin(),
                            y: 0.0,
                            z: heading.cos(),
                        },
                        spring::Float3 {
                            x: heading.cos(),
                            y: 0.0,
                            z: -heading.sin(),
                        },
                    )
                    .log_err_once(spring, "Area 17 unit setup: set unit direction");
            }
            Some(unit_id)
        })
    })
}

pub fn make_visible(spring: &spring::Spring<'_, spring::RulesSynced>, unit_id: i32) {
    spring
        .unit_control()
        .set_unit_always_visible(unit_id, true)
        .log_err_once(spring, "Area 17 unit setup: set unit always visible");
    spring
        .unit_control()
        .set_unit_los_state(unit_id, LOS_ALLY_TEAM, LOS_AND_RADAR)
        .log_err_once(spring, "Area 17 unit setup: set unit los state");
}

pub fn set_move_speed_multiplier(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit_id: i32,
    multiplier: f32,
) -> bool {
    let Some(unit_def_id) = unit_def_id(spring, unit_id) else {
        return false;
    };
    let Some(base_speed) = defs::unit_data(spring, unit_def_id).map(|physics| physics.speed) else {
        return false;
    };
    spring
        .move_ctrl()
        .set_ground_move_type_max_speed(unit_id, base_speed * LUA_UNIT_SPEED_FACTOR * multiplier)
        .log_err_once(spring, "Area 17 set move speed multiplier")
        .unwrap_or(false)
}

pub fn set_gunship_attributes(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit_id: i32,
    speed: f32,
    acceleration: f32,
) {
    use spring::move_ctrl::MoveTypeNumericField as Field;
    let Some(data) = unit_def_id(spring, unit_id).and_then(|id| defs::unit_data(spring, id)) else {
        return;
    };
    let speed = speed.max(0.0);
    let rate = speed.max(0.001) * acceleration.max(0.001);
    for (field, value) in [
        (
            Field::MoveTypeMaxSpeed,
            data.speed * LUA_UNIT_SPEED_FACTOR * speed,
        ),
        (Field::MoveTypeGunshipAccRate, data.acceleration * rate),
        (Field::MoveTypeGunshipDecRate, data.brake_rate * rate),
    ] {
        spring
            .move_ctrl()
            .set_move_type_numeric(unit_id, field, value)
            .log_err_once(spring, "Area 17 set gunship attributes");
    }
}

pub fn stop(
    _spring: &spring::Spring<'_, spring::RulesSynced>,
    unit_id: i32,
) -> Pending<spring::RulesSynced, bool> {
    crate::with_addon_context(|ctx| {
        ctx.commands().queue(move |spring| {
            spring
                .unit_control()
                .give_order_to_unit(unit_id, cmd::STOP, &[], 0, -1)
                .log_err_once(spring, "Area 17 stop")
                .unwrap_or(false)
        })
    })
}

pub fn set_target(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit_id: i32,
    position: Option<Float3>,
) -> bool {
    let target = position.map_or(
        spring::unit_control::UnitTargetRef {
            target_id: -1,
            pos: spring::Float3::default(),
            is_ground_target: false,
        },
        |position| spring::unit_control::UnitTargetRef {
            target_id: -1,
            pos: position,
            is_ground_target: true,
        },
    );
    spring
        .unit_control()
        .set_unit_target(
            unit_id,
            target,
            spring::unit_control::SetUnitTargetOptions {
                manual_fire: true,
                user_target: false,
            },
            -1,
        )
        .log_err_once(spring, "Area 17 set target")
        .unwrap_or(false)
}

pub fn set_position(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit_id: i32,
    position: Float3,
) -> bool {
    spring
        .unit_control()
        .set_unit_position(unit_id, position)
        .log_err_once(spring, "Area 17 set position")
        .unwrap_or(false)
}

pub fn add_impulse(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit_id: i32,
    impulse: Float3,
) -> bool {
    spring
        .unit_control()
        .add_unit_impulse(unit_id, impulse, 0.0)
        .log_err_once(spring, "Area 17 add impulse")
        .unwrap_or(false)
}

pub fn set_velocity(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit_id: i32,
    velocity: Float3,
) -> bool {
    spring
        .unit_control()
        .set_unit_velocity(unit_id, velocity)
        .log_err_once(spring, "Area 17 set velocity")
        .unwrap_or(false)
}

pub fn is_on_ground(spring: &spring::Spring<'_, spring::RulesSynced>, unit_id: i32) -> bool {
    let Some(position) = unit_base_position(spring, unit_id) else {
        return false;
    };
    spring
        .terrain()
        .get_ground_height(position.x, position.z)
        .is_ok_and(|ground| position.y - ground < 1.0)
}

pub fn set_rotation(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit_id: i32,
    rotation: Float3,
) -> bool {
    spring
        .unit_control()
        .set_unit_rotation(unit_id, rotation)
        .log_err_once(spring, "Area 17 set rotation")
        .unwrap_or(false)
}

pub fn heal_fraction(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit_id: i32,
    fraction: f32,
) -> bool {
    let Some((health, max_health)) = unit_health(spring, unit_id) else {
        return false;
    };
    spring
        .unit_control()
        .set_unit_health(
            unit_id,
            spring::unit_control::UnitHealthValue {
                health: (health + max_health * fraction).min(max_health),
                capture: 0.0,
                paralyze: 0.0,
                build: 1.0,
                use_amounts: false,
            },
        )
        .log_err_once(spring, "Area 17 heal fraction")
        .unwrap_or(false)
}

pub fn set_health(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit_id: i32,
    health: f32,
) -> bool {
    let Some((_, max_health)) = unit_health(spring, unit_id) else {
        return false;
    };
    spring
        .unit_control()
        .set_unit_health(
            unit_id,
            spring::unit_control::UnitHealthValue {
                health: health.clamp(0.0, max_health),
                capture: 0.0,
                paralyze: 0.0,
                build: 1.0,
                use_amounts: false,
            },
        )
        .log_err_once(spring, "Area 17 set health")
        .unwrap_or(false)
}

pub fn configure_unit_geometry(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit_id: i32,
    unit_def_id: i32,
) {
    let Some(data) = defs::unit_data(spring, unit_def_id) else {
        return;
    };
    if let Some((mid, aim)) = data.mid_and_aim {
        spring
            .unit_control()
            .set_unit_mid_and_aim_pos(unit_id, mid, aim, true)
            .log_err_once(spring, "Area 17 unit mid and aim points");
    }
    if let Some((radius, height)) = data.radius_and_height {
        spring
            .unit_control()
            .set_unit_radius_and_height(unit_id, radius, height)
            .log_err_once(spring, "Area 17 unit radius and height");
    }
}

pub fn destroy_reclaimed(
    _spring: &spring::Spring<'_, spring::RulesSynced>,
    unit_id: i32,
) -> Pending<spring::RulesSynced, bool> {
    crate::with_addon_context(|ctx| {
        ctx.commands().queue(move |spring| {
            spring
                .unit_control()
                .destroy_unit(
                    unit_id,
                    spring::unit_control::DestroyUnitOptions {
                        selfd: false,
                        reclaimed: true,
                        attacker_id: -1,
                        recycle_id: false,
                    },
                )
                .log_err_once(spring, "Area 17 destroy reclaimed")
                .unwrap_or(false)
        })
    })
}

pub fn distance(a: Float3, b: Float3) -> f32 {
    a.distance(b)
}

pub fn map_size(spring: &spring::Spring<'_, spring::RulesSynced>) -> Option<(f32, f32)> {
    spring
        .game()
        .map_size_elmos()
        .log_err_once(spring, "Area 17 map dimensions")
        .map(|(x, z)| (x as f32, z as f32))
}

pub fn clamp_xz(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    x: f32,
    z: f32,
) -> Option<(f32, f32)> {
    let (width, height) = map_size(spring)?;
    Some((x.clamp(1.0, width - 1.0), z.clamp(1.0, height - 1.0)))
}
