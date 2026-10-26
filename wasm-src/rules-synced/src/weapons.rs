use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::cell::RefCell;
use spring::ResultExt;

use spring::Float3;

use crate::defs::{self, AREA_DAMAGE_PERIOD, UnitData};
use crate::state::{Global, Tech, Weapon};
use crate::{definitions, upgrades};

const GRAVITY_BASELINE: f32 = 120.0;
const GROUND_PUSH_GRAVITY_FACTOR: f32 = 1.12;
const UNSTICK_IMPULSE: f32 = 4.0;
const GUNSHIP_VERTICAL_MULTIPLIER: f32 = 0.25;
const IMPULSE_MULTIPLIER: [f32; 3] = [0.02, 0.004, 0.0036];
const NO_EXTRA_MULTIPLIER: [f32; 3] = [1.0; 3];
const BLACK_HOLE_IMPULSE_MULTIPLIER: [f32; 3] = [0.22, 0.7, 1.0];
const BLACK_HOLE_FALL_IMMUNITY_FRAMES: i32 = 10;
const BLACK_HOLE_EFFECT: &str = "genericshellexplosion-large-purple";
const BLACK_HOLE_EFFECT_PERIOD: i32 = 10;
const FIRE_EFFECT: &str = "genericshellexplosion-small";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MoveType {
    FixedWing,
    Gunship,
    Ground,
}

impl MoveType {
    fn of(physics: &UnitData) -> Option<Self> {
        if physics.can_fly {
            Some(if physics.can_hover {
                Self::Gunship
            } else {
                Self::FixedWing
            })
        } else if physics.is_building || physics.speed == 0.0 {
            None
        } else {
            Some(Self::Ground)
        }
    }

    const fn index(self) -> usize {
        match self {
            Self::FixedWing => 0,
            Self::Gunship => 1,
            Self::Ground => 2,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct PendingImpulse {
    move_type: MoveType,
    impulse: Float3,
    push_off_ground: bool,
    unstick: bool,
}

#[derive(Default)]
pub struct Impulses(BTreeMap<i32, PendingImpulse>);

#[derive(Clone, Copy, Debug)]
pub struct LingeringArea {
    weapon_id: i32,
    position: Float3,
    radius: f32,
    damage: f32,
    time_loss: f32,
    range_falloff: f32,
    impulse: bool,
    expires: f32,
    owner_id: i32,
    repeat_effect: Option<f32>,
}

pub fn apply_impulses(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    impulses: Impulses,
    gravity_per_frame: f32,
) {
    for (unit_id, pending) in impulses.0 {
        let mut impulse = pending.impulse;
        match pending.move_type {
            MoveType::Gunship => {
                if let Some(velocity) = definitions::unit_velocity(spring, unit_id) {
                    definitions::set_velocity(spring, unit_id, velocity + impulse);
                }
            }
            MoveType::FixedWing => {
                definitions::add_impulse(spring, unit_id, impulse);
            }
            MoveType::Ground => {
                if pending.push_off_ground {
                    impulse.y += GROUND_PUSH_GRAVITY_FACTOR * gravity_per_frame;
                }
                if pending.unstick {
                    definitions::add_impulse(
                        spring,
                        unit_id,
                        Float3::new(UNSTICK_IMPULSE, 0.0, 0.0),
                    );
                }
                definitions::add_impulse(spring, unit_id, impulse);
                if pending.unstick {
                    definitions::add_impulse(
                        spring,
                        unit_id,
                        Float3::new(-UNSTICK_IMPULSE, 0.0, 0.0),
                    );
                }
            }
        }
    }
}

pub fn update(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &Global,
    areas: &mut Vec<LingeringArea>,
    impulses: &RefCell<Impulses>,
    frame: i32,
) {
    if frame.rem_euclid(BLACK_HOLE_EFFECT_PERIOD) == 0 {
        for area in areas.iter() {
            if let Some(size) = area.repeat_effect
                && area.expires >= frame as f32
            {
                spawn_effect(spring, BLACK_HOLE_EFFECT, area.position, size);
            }
        }
    }
    if frame.rem_euclid(AREA_DAMAGE_PERIOD) != 0 {
        return;
    }
    for area in areas.iter_mut() {
        apply_area(spring, global, impulses, area, frame);
        area.damage -= area.time_loss;
    }
    areas.retain(|area| (frame as f32) < area.expires);
}

pub fn explosion(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    tech: &Tech,
    weapon_id: i32,
    position: Float3,
    owner_id: i32,
) -> Option<LingeringArea> {
    let area = defs::weapon_behaviour(spring, weapon_id)?.area?;
    let fire = weapon_id == defs::weapon(spring, Weapon::IncendiaryBeamLaser);
    let black_hole = weapon_id == defs::weapon(spring, Weapon::BlackHoleGun);
    let (radius_multiplier, damage_multiplier, duration_multiplier) = if fire {
        (
            1.0,
            upgrades::incendiary_damage_multiplier(tech),
            upgrades::incendiary_duration_multiplier(tech),
        )
    } else if black_hole {
        (
            upgrades::black_hole_radius_multiplier(tech),
            1.0,
            upgrades::black_hole_duration_multiplier(tech),
        )
    } else {
        (1.0, 1.0, 1.0)
    };
    let duration = area.duration_frames * duration_multiplier;
    if fire {
        spawn_effect(spring, FIRE_EFFECT, position, duration);
    } else if black_hole {
        spawn_effect(spring, BLACK_HOLE_EFFECT, position, duration);
    }
    Some(LingeringArea {
        weapon_id,
        position,
        radius: area.radius * radius_multiplier,
        damage: area.damage * damage_multiplier,
        time_loss: area.time_loss / duration_multiplier,
        range_falloff: area.range_falloff,
        impulse: area.impulse,
        expires: spring.game().current_frame().unwrap_or(0) as f32 + duration,
        owner_id,
        repeat_effect: black_hole.then_some(duration),
    })
}

pub fn pre_damaged(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &Global,
    impulses: &RefCell<Impulses>,
    unit_id: i32,
    unit_def_id: i32,
    damage: f32,
    weapon_def_id: i32,
    attacker_id: i32,
) -> Option<f32> {
    let behaviour = defs::weapon_behaviour(spring, weapon_def_id)?;
    if let Some(frames) = behaviour.fall_immunity {
        global
            .fall_immunity
            .access_mut()
            .grant(unit_id, spring.game().current_frame().unwrap_or(0) + frames);
    }
    let mut damage = damage;
    if let Some(impulse) = behaviour.impulse
        && let Some(attacker) = definitions::unit_mid_position(spring, attacker_id)
    {
        if let Some(target) = definitions::unit_mid_position(spring, unit_id) {
            let multiplier = if weapon_def_id == defs::weapon(spring, Weapon::GravityBeam) {
                upgrades::gravity_impulse_multiplier(&global.tech.access())
            } else {
                1.0
            };
            queue_impulse(
                spring,
                impulses,
                unit_id,
                unit_def_id,
                target - attacker,
                impulse.magnitude * multiplier,
                ImpulseOptions {
                    push_off_ground: false,
                    unstick: true,
                    multipliers: NO_EXTRA_MULTIPLIER,
                },
            );
        }
        if !impulse.normal_damage {
            damage = 0.0;
        }
    }
    if behaviour.area.is_some_and(|area| area.impulse) {
        damage = 0.0;
    }
    Some(damage)
}

fn apply_area(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &Global,
    impulses: &RefCell<Impulses>,
    area: &LingeringArea,
    frame: i32,
) {
    let units = spring
        .units_query()
        .get_units_in_sphere(
            area.position.x,
            area.position.y,
            area.position.z,
            area.radius,
            -1,
        )
        .unwrap_or_default();
    for unit_id in units {
        let Some(position) = definitions::unit_position(spring, unit_id) else {
            continue;
        };
        let mut damage = area.damage;
        if area.range_falloff != 0.0 {
            damage -= damage * area.range_falloff * position.distance(area.position) / area.radius;
        }
        if area.impulse {
            let Some(unit_def_id) = definitions::unit_def_id(spring, unit_id) else {
                continue;
            };
            queue_impulse(
                spring,
                impulses,
                unit_id,
                unit_def_id,
                area.position - position,
                damage,
                ImpulseOptions {
                    push_off_ground: true,
                    unstick: false,
                    multipliers: BLACK_HOLE_IMPULSE_MULTIPLIER,
                },
            );
            global
                .fall_immunity
                .access_mut()
                .grant(unit_id, frame + BLACK_HOLE_FALL_IMMUNITY_FRAMES);
            apply_air_drag(spring, unit_id, unit_def_id, damage);
        } else {
            let (weapon_id, owner_id) = (area.weapon_id, area.owner_id);
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
                        .add_unit_damage(unit_id, damage, 0.0, weapon_id, owner_id, Float3::ZERO)
                        .log_err_once(spring, "Area 17 weapons: add unit damage");
                })
            });
        }
    }
}

#[derive(Clone, Copy)]
struct ImpulseOptions {
    push_off_ground: bool,
    unstick: bool,
    multipliers: [f32; 3],
}

fn queue_impulse(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    impulses: &RefCell<Impulses>,
    unit_id: i32,
    unit_def_id: i32,
    direction: Float3,
    magnitude: f32,
    options: ImpulseOptions,
) {
    let Some(physics) = defs::unit_data(spring, unit_def_id) else {
        return;
    };
    let Some(move_type) = MoveType::of(&physics) else {
        return;
    };
    let distance = direction.length();
    if distance <= f32::EPSILON || physics.mass <= f32::EPSILON {
        return;
    }
    let index = move_type.index();
    let scale = magnitude * GRAVITY_BASELINE / distance
        * IMPULSE_MULTIPLIER[index]
        * options.multipliers[index]
        / physics.mass;
    let mut impulse = direction * scale;
    let mut push_off_ground = false;
    match move_type {
        MoveType::FixedWing => {}
        MoveType::Gunship => impulse.y *= GUNSHIP_VERTICAL_MULTIPLIER,
        MoveType::Ground => {
            impulse.y += magnitude.abs() / (20.0 * physics.mass);
            push_off_ground = options.push_off_ground && definitions::is_on_ground(spring, unit_id);
        }
    }
    let mut impulses = impulses.borrow_mut();
    let pending = impulses.0.entry(unit_id).or_insert(PendingImpulse {
        move_type,
        impulse: Float3::ZERO,
        push_off_ground: false,
        unstick: false,
    });
    pending.impulse += impulse;
    pending.push_off_ground |= push_off_ground;
    pending.unstick |= options.unstick;
}

fn apply_air_drag(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit_id: i32,
    unit_def_id: i32,
    damage: f32,
) {
    let Some(physics) = defs::unit_data(spring, unit_def_id) else {
        return;
    };
    let Some(velocity) = definitions::unit_velocity(spring, unit_id) else {
        return;
    };
    let factor = damage / (damage + physics.mass.powf(1.5));
    definitions::set_velocity(spring, unit_id, velocity * (1.0 - factor));
}

fn spawn_effect(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    name: &str,
    position: Float3,
    size: f32,
) {
    spring
        .effects_control()
        .spawn_ceg(
            &spring::effects_control::DefRef {
                name: name.to_owned(),
                id: -1,
            },
            position,
            Float3::new(0.0, 1.0, 0.0),
            size,
            size,
            1.0,
        )
        .log_err_once(spring, "Area 17 weapons: spawn ceg");
}
