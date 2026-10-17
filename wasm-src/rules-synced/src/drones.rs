use alloc::rc::Rc;
use core::cell::RefCell;
use spring::ResultExt;
use spring::commands::Pending;

use crate::defs::{self, UnitKind};
use crate::{definitions, state, upgrades};
use spring::cmd;
use spring::consts::GAME_SPEED;

use springcabal::message::Encoder;
use state::{Global, Tech};

const DRONE_RADIUS: f32 = 300.0;
const MAX_DRONE_RADIUS_SQ: f32 = 1_000.0 * 1_000.0;
const ORDER_PERIOD: i32 = 15;
const SPAWN_DEPTH: f32 = 100.0;

#[derive(Clone, Copy, Debug)]
pub struct DroneState {
    pub unit_id: i32,
}

#[derive(Default)]
pub struct Drones {
    pub drones: Vec<DroneState>,
    pub spawn_progress: f32,
}

pub fn update(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &Global,
    drones: &Rc<RefCell<Drones>>,
    frame: i32,
) {
    let ufo_id = global.ufo.access().id;
    let (wanted, drone_level) = {
        let tech = global.tech.access();
        (
            wanted_drones(&tech),
            state::tech(&tech, state::TechId::CarrierDrones).level,
        )
    };
    let drone_count = drones.borrow().drones.len();
    if ufo_id < 0 {
        return;
    }
    if drone_count < wanted {
        let drone_period = (5_i32 - drone_level).max(1) as f32 * GAME_SPEED as f32;
        let ready = {
            let mut drones = drones.borrow_mut();
            drones.spawn_progress += 1.0 / drone_period;
            drones.spawn_progress >= 1.0
        };
        if ready && let Some(spawn) = spawn_drone(spring, global) {
            let drones = Rc::clone(drones);
            spawn.then(move |_spring, drone| {
                if drone.is_some() {
                    drones.borrow_mut().spawn_progress -= 1.0;
                }
            });
        }
    }
    if frame.rem_euclid(ORDER_PERIOD) == 0 {
        update_movement(spring, global, drones);
    }
}

pub fn unit_created(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    drones: &mut Drones,
    unit_id: i32,
    unit_def_id: i32,
) {
    if defs::kind(spring, unit_def_id) == Some(UnitKind::Drone)
        && drones.drones.iter().all(|drone| drone.unit_id != unit_id)
    {
        drones.drones.push(DroneState { unit_id });
        let message = Encoder::new("AREA17_DRONE").field(unit_id).finish();
        springcabal::message::send_to_ui(spring, &message);
    }
}

pub fn unit_destroyed(drones: &mut Drones, unit_id: i32) {
    drones.drones.retain(|drone| drone.unit_id != unit_id);
}

fn spawn_drone(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &Global,
) -> Option<Pending<spring::RulesSynced, Option<i32>>> {
    let ufo_id = global.ufo.access().id;
    let position = definitions::unit_position(spring, ufo_id)?;
    Some(definitions::create_at_position(
        spring,
        UnitKind::Drone,
        position.x,
        position.y - SPAWN_DEPTH,
        position.z,
        state::PLAYER_TEAM,
        0.0,
    ))
}

fn update_movement(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &Global,
    drones: &RefCell<Drones>,
) {
    let ufo_id = global.ufo.access().id;
    let drones = drones.borrow().drones.clone();
    let Some(ufo) = definitions::unit_position(spring, ufo_id) else {
        return;
    };
    for drone in &drones {
        let Some(position) = definitions::unit_position(spring, drone.unit_id) else {
            continue;
        };
        let distance_sq = (position.x - ufo.x) * (position.x - ufo.x)
            + (position.z - ufo.z) * (position.z - ufo.z);
        let command = if distance_sq > MAX_DRONE_RADIUS_SQ {
            cmd::MOVE
        } else {
            cmd::FIGHT
        };
        let Some((x, z)) = random_destination(spring, ufo.x, ufo.z) else {
            continue;
        };
        crate::with_addon_context(|ctx| {
            ctx.commands()
                .give_order_to_unit(drone.unit_id, command, &[x, 0.0, z], 0, -1)
        })
        .then(|spring, accepted| {
            accepted.log_err_once(spring, "Area 17 drones: give order to unit");
        });
    }
}

fn wanted_drones(tech: &Tech) -> usize {
    if !upgrades::is_enabled(tech, state::TechId::CarrierDrones) {
        0
    } else {
        (state::tech(tech, state::TechId::CarrierDrones).level + 1) as usize
    }
}

fn random_destination(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    x: f32,
    z: f32,
) -> Option<(f32, f32)> {
    let radius = DRONE_RADIUS
        * (1.0
            - spring
                .synced_random()
                .next_float()
                .expect("synced random float")
                .powi(2));
    let angle = core::f32::consts::TAU
        * spring
            .synced_random()
            .next_float()
            .expect("synced random float");
    definitions::clamp_xz(spring, x + angle.cos() * radius, z + angle.sin() * radius)
}
