use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::cell::RefCell;

use spring_addons::Resource;

use crate::definitions;
use crate::defs::{self, UnitKind};
use crate::state::Global;
use spring::{ResultExt, cmd};

const FALL_DAMAGE: f32 = 1.0;
const VELOCITY_THRESHOLD: f32 = 3.0;
const TANGENT_DAMAGE: f32 = 0.5;
const ELASTICITY: f32 = 0.5;
const FRICTION: f32 = 0.8;
const DEBRIS_SPRING_DAMAGE_MULTIPLIER: f32 = 2.5;
pub const GROUND_COLLISION: i32 = -2;
pub const OBJECT_COLLISION: i32 = -3;
pub const DEBRIS: i32 = -1;
const CRUSHED: i32 = -7;

pub fn handles(weapon_def_id: i32) -> bool {
    (OBJECT_COLLISION..=DEBRIS).contains(&weapon_def_id)
}

#[derive(Clone, Debug)]
pub struct CollisionData {
    pub unit_id: i32,
    unit_def_id: i32,
    given_damage: Vec<f32>,
}

#[derive(Default)]
pub struct FallImmunity {
    until: BTreeMap<i32, i32>,
}

impl FallImmunity {
    pub fn grant(&mut self, unit_id: i32, until_frame: i32) {
        let until = self.until.entry(unit_id).or_insert(until_frame);
        *until = (*until).max(until_frame);
    }

    pub fn is_immune(&self, unit_id: i32, frame: i32) -> bool {
        self.until
            .get(&unit_id)
            .is_some_and(|until| *until >= frame)
    }

    pub fn forget(&mut self, unit_id: i32) {
        self.until.remove(&unit_id);
    }

    pub fn expire(&mut self, frame: i32) {
        self.until.retain(|_, until| *until >= frame);
    }
}

fn is_permanently_immune(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit_def_id: i32,
) -> bool {
    defs::kind(spring, unit_def_id) == Some(UnitKind::Ufo)
}

pub fn resolve_collisions(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &Global,
    collisions: Vec<CollisionData>,
    frame: i32,
) {
    let immunity = &global.fall_immunity;
    for collision in collisions {
        interrupt_order(spring, collision.unit_id);
        for damage in collision.given_damage {
            apply_collision_damage(
                spring,
                immunity,
                collision.unit_id,
                collision.unit_def_id,
                DEBRIS_SPRING_DAMAGE_MULTIPLIER * damage,
                frame,
            );
        }
    }
}

pub fn ground_collision(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit_id: i32,
    unit_def_id: i32,
    immune: bool,
) -> spring::DamageResult {
    if immune {
        return no_damage();
    }
    ground_collision_damage(spring, unit_id, unit_def_id)
}

pub fn debris() -> spring::DamageResult {
    spring::DamageResult {
        new_damage: 20.0,
        impulse_mult: 1.0,
    }
}

pub fn no_damage() -> spring::DamageResult {
    spring::DamageResult {
        new_damage: 0.0,
        impulse_mult: 1.0,
    }
}

fn ground_collision_damage(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit_id: i32,
    unit_def_id: i32,
) -> spring::DamageResult {
    if is_permanently_immune(spring, unit_def_id) {
        return no_damage();
    }
    let velocity = definitions::unit_velocity(spring, unit_id).unwrap_or_default();
    let position = definitions::unit_position(spring, unit_id).unwrap_or_default();
    let normal = spring
        .terrain()
        .get_ground_normal(position.x, position.z, false)
        .map_or(spring::Float3::new(0.0, 1.0, 0.0), |value| value.normal);

    let normal = -normal.normalized();
    let normal_velocity = normal * velocity.dot(normal);
    let tangent_velocity = velocity - normal_velocity;
    let bounced = tangent_velocity * FRICTION - normal_velocity * ELASTICITY;
    let _ = definitions::set_velocity(spring, unit_id, spring::Float3::ZERO);
    spring
        .unit_control()
        .add_unit_impulse(unit_id, bounced, 0.0)
        .log_err_once(spring, "Area 17 fall damage: add unit impulse");
    interrupt_order(spring, unit_id);

    let impact_speed = (normal_velocity + tangent_velocity * TANGENT_DAMAGE).length() * FALL_DAMAGE;
    let physics = defs::unit_data(spring, unit_def_id);
    let mass = physics.map_or(100.0, |physics| physics.mass);
    let static_multiplier = if physics.is_none_or(|physics| physics.speed == 0.0) {
        10.0
    } else {
        1.0
    };
    let scale = mass * 0.6 * static_multiplier;
    let outside_map = outside_map_damage(spring, unit_id, unit_def_id);
    let new_damage = (impact_speed - VELOCITY_THRESHOLD).max(0.0) * scale + outside_map;
    spring::DamageResult {
        new_damage,
        impulse_mult: 1.0,
    }
}

fn interrupt_order(_spring: &spring::Spring<'_, spring::RulesSynced>, unit_id: i32) {
    for _ in 0..2 {
        crate::with_addon_context(|ctx| {
            ctx.commands()
                .give_order_to_unit(unit_id, cmd::WAIT, &[], 0, -1)
        })
        .then(|spring, accepted| {
            accepted.log_err_once(spring, "Area 17 fall damage: give order to unit");
        });
    }
}

fn outside_map_damage(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit_id: i32,
    unit_def_id: i32,
) -> f32 {
    let Some(position) = definitions::unit_position(spring, unit_id) else {
        return 0.0;
    };
    let Some((width, height)) = definitions::map_size(spring) else {
        return 0.0;
    };
    let outside = (-position.x)
        .max(-position.z)
        .max(position.x - width)
        .max(position.z - height)
        .max(0.0);
    let health = defs::unit_data(spring, unit_def_id).map_or(0.0, |physics| physics.health);
    outside * health / 800.0
}

pub fn record_collision(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    collisions: &RefCell<Vec<CollisionData>>,
    unit_id: i32,
    unit_def_id: i32,
    damage: f32,
) {
    if is_permanently_immune(spring, unit_def_id) {
        return;
    }
    let mut collisions = collisions.borrow_mut();
    if let Some(collision) = collisions
        .iter_mut()
        .find(|collision| collision.unit_id == unit_id)
    {
        collision.given_damage.push(damage);
        return;
    }
    collisions.push(CollisionData {
        unit_id,
        unit_def_id,
        given_damage: vec![damage],
    });
}

fn apply_collision_damage(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    immunity: &Resource<FallImmunity>,
    unit_id: i32,
    unit_def_id: i32,
    damage: f32,
    frame: i32,
) {
    if damage <= 0.0
        || is_permanently_immune(spring, unit_def_id)
        || immunity.access().is_immune(unit_id, frame)
    {
        return;
    }
    crate::with_addon_context(|ctx| {
        ctx.commands().run(move |spring| {
            if spring
                .units_info()
                .get_unit_is_dead(unit_id)
                .unwrap_or(true)
            {
                return;
            }
            spring
                .unit_control()
                .add_unit_damage(unit_id, damage, 0.0, CRUSHED, -1, spring::Float3::ZERO)
                .log_err_once(spring, "Area 17 collision damage");
        })
    });
}
