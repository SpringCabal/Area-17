use crate::defs::UnitKind;
use alloc::rc::Rc;
use core::cell::RefCell;
use spring::ResultExt;

use area17_shared::message as msg;
use springcabal::message::{self, Fields};
use springcabal_control::goal::{Bounds, Clamp, MoveGoal};
use springcabal_control::protocol;

use crate::{definitions, diagnostics, script_bridge, state, upgrades};
use spring::Float3;
use state::{Ability, Global, Input, PendingInput, Ufo, Weapon, WeaponTarget};

const MOVE_RANGE: f32 = 1_000.0;
const BRAKE_RANGE: f32 = 20.0;
const BRAKE_SPEED: f32 = 6.0;
const STEER: MoveGoal = MoveGoal::ahead(MOVE_RANGE)
    .radius(16.0)
    .raw(false)
    .bounds(Bounds::Map {
        clearance: 1.0,
        clamp: Clamp::PerAxis,
    });
const AIM_STEP: f32 = 40.0;
const ALIEN_DROP_HEIGHT: f32 = 100.0;
const ALIEN_LAND_DELAY_FRAMES: i32 = 10;

pub fn handle_message(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &Global,
    message: &str,
) {
    let command = message
        .split_once('|')
        .map_or(message, |(command, _)| command);
    let Some(mut fields) = message::fields(message, command) else {
        return;
    };
    match command {
        protocol::DIRECTION => {
            if let Some(direction) = protocol::decode_direction(message) {
                global.ufo.access_mut().movement = direction;
            }
        }
        msg::AIM => handle_aim(&mut global.ufo.access_mut(), &mut fields),
        msg::FIRE => queue_fire(&mut global.input.access_mut(), &mut fields),
        msg::CEASE_FIRE => global
            .input
            .access_mut()
            .pending
            .push(PendingInput::CeaseFire),
        msg::ABILITY => queue_ability(&mut global.input.access_mut(), fields.next()),
        msg::CHANGE_WEAPON => queue_weapon(&mut global.input.access_mut(), fields.next()),
        msg::UNLOCK => queue_tech(&mut global.input.access_mut(), fields.next(), false),
        msg::UPGRADE => queue_tech(&mut global.input.access_mut(), fields.next(), true),
        #[cfg(feature = "e2e")]
        "cheatResources" => grant_test_resources(spring, global),
        _ => diagnostics::debug(spring, "ignored unknown Area 17 input message"),
    }
}

pub fn apply_pending_input(spring: &spring::Spring<'_, spring::RulesSynced>, global: &Global) {
    let inputs = core::mem::take(&mut global.input.access_mut().pending);
    for input in inputs {
        match input {
            PendingInput::Fire(position) => {
                let mut ufo = global.ufo.access_mut();
                if ufo.current_weapon.is_none() {
                    continue;
                }
                let target = WeaponTarget {
                    position: position.unwrap_or_default(),
                    use_aim_target: position.is_none(),
                };
                ufo.weapon_target = Some(target);
                if ufo.id >= 0 {
                    let point = target_position(spring, &ufo, target);
                    ufo.target_accepted = definitions::set_target(spring, ufo.id, Some(point));
                }
            }
            PendingInput::CeaseFire => {
                let mut ufo = global.ufo.access_mut();
                ufo.weapon_target = None;
                if ufo.id >= 0 {
                    let _ = definitions::set_target(spring, ufo.id, None);
                }
            }
            PendingInput::Ability(ability) => upgrades::activate_ability(spring, global, ability),
            PendingInput::ChangeWeapon(weapon) => {
                global.ufo.access_mut().current_weapon = Some(weapon);
            }
            PendingInput::Unlock(tech) => {
                let _ = upgrades::unlock(spring, global, tech);
            }
            PendingInput::Upgrade(tech) => {
                let _ = upgrades::upgrade(spring, global, tech);
            }
        }
    }
}

pub fn update(spring: &spring::Spring<'_, spring::RulesSynced>, global: &Global) {
    let ufo_id = global.ufo.access().id;
    if ufo_id < 0 {
        return;
    }
    let Some(position) = definitions::unit_position(spring, ufo_id) else {
        return;
    };

    update_aim(&mut global.ufo.access_mut(), position);
    update_movement(spring, global, position);
    update_weapon_target(spring, global, position);
}

pub struct Alien {
    pub id: i32,
    pub spawn_frame: Option<i32>,
    pub land_frame: Option<i32>,
}

impl Default for Alien {
    fn default() -> Self {
        Self {
            id: -1,
            spawn_frame: None,
            land_frame: None,
        }
    }
}

pub fn update_destroyed_sequence(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &Global,
    alien: &Rc<RefCell<Alien>>,
    frame: i32,
) {
    let spawn_at = {
        let mut alien = alien.borrow_mut();
        if alien
            .spawn_frame
            .is_some_and(|spawn_frame| spawn_frame <= frame)
        {
            alien.spawn_frame = None;
            global.ufo.access().destroyed_position
        } else {
            None
        }
    };
    if let Some(position) = spawn_at {
        let alien = Rc::clone(alien);
        definitions::create_at_position(
            spring,
            UnitKind::Alien,
            position.x,
            position.y + ALIEN_DROP_HEIGHT,
            position.z,
            state::ENEMY_TEAM,
            0.0,
        )
        .then(move |spring, alien_id| {
            let Some(alien_id) = alien_id else {
                spring::log_warning!(
                    spring,
                    crate::diagnostics::SECTION,
                    "Area 17 could not spawn the alien over the UFO wreck"
                );
                return;
            };
            {
                let mut alien = alien.borrow_mut();
                alien.id = alien_id;
                alien.land_frame = Some(frame + ALIEN_LAND_DELAY_FRAMES);
            }
            script_bridge::abduction_float(spring, alien_id);
            diagnostics::info(spring, "Area 17 alien reveal spawned");
        });
    }

    let landing = {
        let mut alien = alien.borrow_mut();
        let due = alien
            .land_frame
            .is_some_and(|land_frame| land_frame <= frame);
        if due {
            alien.land_frame = None;
        }
        (due && alien.id >= 0).then_some(alien.id)
    };
    if let Some(alien_id) = landing {
        script_bridge::alien_land(spring, alien_id);
        diagnostics::info(spring, "Area 17 alien reveal landed");
    }
}

fn handle_aim(ufo: &mut Ufo, fields: &mut Fields<'_>) {
    if let Some(point) = parse_point(fields) {
        ufo.aim_target = Some(point);
    }
}

fn queue_fire(input: &mut Input, fields: &mut Fields<'_>) {
    let position = if fields.is_done() {
        None
    } else {
        let Some(point) = parse_point(fields) else {
            return;
        };
        Some(point)
    };
    input.pending.push(PendingInput::Fire(position));
}

fn queue_ability(input: &mut Input, name: Option<&str>) {
    let Some(ability) = name.and_then(Ability::from_name) else {
        return;
    };
    input.pending.push(PendingInput::Ability(ability));
}

fn queue_weapon(input: &mut Input, name: Option<&str>) {
    let Some(weapon) = name.and_then(Weapon::from_name) else {
        return;
    };
    input.pending.push(PendingInput::ChangeWeapon(weapon));
}

fn queue_tech(input: &mut Input, name: Option<&str>, upgrade: bool) {
    let Some(tech) = name.and_then(state::TechId::from_name) else {
        return;
    };
    input.pending.push(if upgrade {
        PendingInput::Upgrade(tech)
    } else {
        PendingInput::Unlock(tech)
    });
}

fn update_aim(ufo: &mut Ufo, ufo_position: Float3) {
    let Some(target) = ufo.aim_target else {
        return;
    };
    let desired = target - ufo_position;
    let delta = desired - ufo.aim;
    let length = delta.length();
    ufo.aim = if length <= AIM_STEP {
        desired
    } else {
        ufo.aim + delta * (AIM_STEP / length)
    };
}

fn update_movement(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &Global,
    ufo_position: Float3,
) {
    let (ufo_id, steering, x, z) = {
        let mut ufo = global.ufo.access_mut();
        let ufo_id = ufo.id;
        let steering = ufo.movement != (0.0, 0.0);
        let (x, z) = if steering {
            ufo.movement
        } else {
            let velocity = definitions::unit_velocity(spring, ufo_id).unwrap_or_default();
            let speed = (velocity.x * velocity.x + velocity.z * velocity.z).sqrt();
            if ufo.moving || speed > BRAKE_SPEED {
                ufo.moving = false;
                (velocity.x, velocity.z)
            } else {
                return;
            }
        };
        (ufo_id, steering, x, z)
    };
    if (x * x + z * z).sqrt() <= f32::EPSILON {
        return;
    }
    let speed_multiplier = spring
        .rules_params()
        .get_unit_rules_param_f32(ufo_id, "selfMoveSpeedChange")
        .unwrap_or(1.0)
        .max(0.0);

    let range = if steering { MOVE_RANGE } else { BRAKE_RANGE } * speed_multiplier;
    STEER
        .reach(range)
        .apply_from(spring, ufo_id, ufo_position, (x, z))
        .log_err_once(spring, "Area 17 UFO controller: STEER");
    global.ufo.access_mut().moving = steering;
}

fn update_weapon_target(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &Global,
    ufo_position: Float3,
) {
    let mut ufo = global.ufo.access_mut();
    let Some(target) = ufo.weapon_target else {
        return;
    };
    let position = target_position_at(target, &ufo, ufo_position);
    ufo.target_accepted = definitions::set_target(spring, ufo.id, Some(position));
}

fn target_position(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    ufo: &Ufo,
    target: WeaponTarget,
) -> Float3 {
    definitions::unit_position(spring, ufo.id)
        .map(|position| target_position_at(target, ufo, position))
        .unwrap_or(target.position)
}

fn target_position_at(target: WeaponTarget, ufo: &Ufo, ufo_position: Float3) -> Float3 {
    if target.use_aim_target {
        ufo_position + ufo.aim
    } else {
        target.position
    }
}

#[cfg(feature = "e2e")]
fn grant_test_resources(spring: &spring::Spring<'_, spring::RulesSynced>, global: &Global) {
    {
        let mut economy = global.economy.access_mut();
        economy.biomass += 10_000.0;
        economy.research += 10_000.0;
        economy.metal += 10_000.0;
    }
    diagnostics::info(spring, "Area 17 test resources granted");
}

fn parse_point(fields: &mut Fields<'_>) -> Option<Float3> {
    let (x, y, z) = fields.xyz()?;
    Some(Float3 { x, y, z })
}
