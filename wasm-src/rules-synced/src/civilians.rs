use alloc::collections::BTreeSet;
use alloc::rc::Rc;
use alloc::vec::Vec;
use core::cell::RefCell;

use spring::cmd;
use spring::commands::{self, Pending};
use spring::consts::GAME_SPEED;
use spring::random::synced;
use spring::rules_params::los;
use spring::{Float3, ResultExt};

use crate::defs::{self, UnitKind};
use crate::{definitions, diagnostics, state};
use state::Global;

const WANDER_RADIUS: f32 = 800.0;
const RUN_AWAY_DISTANCE: f32 = 450.0;

const BASE_SPEED_MULTIPLIER: f32 = 1.0;
const FLEE_SPEED_MULTIPLIER: f32 = 2.5;
const UFO_DIST_SPAWN_SQ: f32 = 3_500.0 * 3_500.0;
const UFO_DIST_ACTIVE_SQ: f32 = 3_000.0 * 3_000.0;
const SPAWNER_PERIOD: i32 = 5;
const SPAWN_ATTEMPTS: usize = 8;

#[derive(Clone, Copy, Debug)]
pub struct Spawner {
    pub unit_id: i32,
    pub unit_def_id: i32,
    pub x: f32,
    pub z: f32,
    pub active: bool,
    pub initialized: bool,
    pub stock: [f32; 4],
    pub wanderers: [i32; 4],
}

#[derive(Clone, Copy, Debug)]
pub struct Wanderer {
    pub unit_id: i32,
    pub spawn_id: i32,
    pub spec: usize,
    pub home_x: f32,
    pub home_z: f32,
    pub wandering: bool,
}

#[derive(Default)]
pub struct Civilians {
    pub spawners: Vec<Spawner>,
    pub wanderers: Vec<Wanderer>,
    idle: BTreeSet<i32>,
    ufo_xz: Option<(f32, f32)>,
}

#[derive(Clone, Copy)]
struct SpawnSpec {
    kind: UnitKind,
    wanted: i32,
    stock_rate: f32,
}

const BUILDING1: [SpawnSpec; 4] = [
    SpawnSpec {
        kind: UnitKind::Civilian,
        wanted: 3,
        stock_rate: 0.3 / GAME_SPEED as f32,
    },
    SpawnSpec {
        kind: UnitKind::Scientist,
        wanted: 1,
        stock_rate: 0.3 / GAME_SPEED as f32,
    },
    SpawnSpec {
        kind: UnitKind::Civilian1,
        wanted: 2,
        stock_rate: 0.3 / GAME_SPEED as f32,
    },
    SpawnSpec {
        kind: UnitKind::Scientist2,
        wanted: 1,
        stock_rate: 0.3 / GAME_SPEED as f32,
    },
];
const BUILDING2: [SpawnSpec; 4] = [
    SpawnSpec {
        kind: UnitKind::Civilian,
        wanted: 2,
        stock_rate: 0.3 / GAME_SPEED as f32,
    },
    SpawnSpec {
        kind: UnitKind::Scientist,
        wanted: 1,
        stock_rate: 0.3 / GAME_SPEED as f32,
    },
    SpawnSpec {
        kind: UnitKind::Civilian1,
        wanted: 3,
        stock_rate: 0.3 / GAME_SPEED as f32,
    },
    SpawnSpec {
        kind: UnitKind::Scientist2,
        wanted: 1,
        stock_rate: 0.3 / GAME_SPEED as f32,
    },
];
const BUILDING3: [SpawnSpec; 2] = [
    SpawnSpec {
        kind: UnitKind::Civilian,
        wanted: 1,
        stock_rate: 0.3 / GAME_SPEED as f32,
    },
    SpawnSpec {
        kind: UnitKind::Civilian1,
        wanted: 1,
        stock_rate: 0.3 / GAME_SPEED as f32,
    },
];
const BUILDING4: [SpawnSpec; 1] = [SpawnSpec {
    kind: UnitKind::SmallTank,
    wanted: 3,
    stock_rate: 0.06 / GAME_SPEED as f32,
}];
const BUILDING4_BIG_TANK: [SpawnSpec; 1] = [SpawnSpec {
    kind: UnitKind::Tank,
    wanted: 2,
    stock_rate: 0.03 / GAME_SPEED as f32,
}];

pub fn update(global: &Global, civilians: &Rc<RefCell<Civilians>>, frame: i32) {
    let ufo_id = global.ufo.access().id;
    let cloaked = global.tech.access().ability_durations[state::Ability::Cloak.index()] > 0;
    if let Some(position) = (ufo_id >= 0)
        .then(|| definitions::unit_position(ufo_id))
        .flatten()
    {
        civilians.borrow_mut().ufo_xz = Some((position.x, position.z));
    }
    if civilians.borrow().ufo_xz.is_none() {
        return;
    }
    let mut spawning = false;
    if frame.rem_euclid(SPAWNER_PERIOD) == 0 {
        let spawner_count = civilians.borrow().spawners.len();
        for index in 0..spawner_count {
            spawning |= update_spawner(civilians, index);
        }
    }
    let scare_radius = if cloaked {
        0.0
    } else {
        state::UFO_SCARE_RADIUS
    };
    let civilians = Rc::clone(civilians);
    let send_off = move || {
        let wanderer_count = civilians.borrow().wanderers.len();
        for index in 0..wanderer_count {
            check_flee(&civilians, index, scare_radius);
        }
        let idle = core::mem::take(&mut civilians.borrow_mut().idle);
        for unit_id in idle {
            wander(&civilians, unit_id);
        }
    };
    if spawning {
        commands::run(send_off);
    } else {
        send_off();
    }
}

pub fn unit_created(civilians: &mut Civilians, unit_id: i32, unit_def_id: i32) {
    if spawner_specs(unit_def_id).is_some()
        && civilians
            .spawners
            .iter()
            .all(|spawner| spawner.unit_id != unit_id)
        && let Some(position) = definitions::unit_position(unit_id)
    {
        civilians.spawners.push(Spawner {
            unit_id,
            unit_def_id,
            x: position.x,
            z: position.z,
            active: false,
            initialized: false,
            stock: [0.0; 4],
            wanderers: [0; 4],
        });
    }
}

pub fn unit_destroyed(civilians: &mut Civilians, unit_id: i32) {
    civilians
        .spawners
        .retain(|spawner| spawner.unit_id != unit_id);
    civilians.idle.remove(&unit_id);
    if let Some(index) = civilians
        .wanderers
        .iter()
        .position(|wanderer| wanderer.unit_id == unit_id)
    {
        let wanderer = civilians.wanderers.remove(index);
        if let Some(spawner) = civilians
            .spawners
            .iter_mut()
            .find(|spawner| spawner.unit_id == wanderer.spawn_id)
        {
            spawner.wanderers[wanderer.spec] -= 1;
        }
    }
}

pub fn unit_idle(civilians: &RefCell<Civilians>, unit_id: i32) {
    set_idle(civilians, unit_id);
}

fn set_idle(civilians: &RefCell<Civilians>, unit_id: i32) {
    let was_wandering = {
        let mut civilians = civilians.borrow_mut();
        let Some(index) = civilians
            .wanderers
            .iter()
            .position(|wanderer| wanderer.unit_id == unit_id)
        else {
            return;
        };
        let wanderer = civilians.wanderers[index];
        civilians.wanderers[index].wandering = true;
        let resting = civilians
            .spawners
            .iter()
            .any(|spawner| spawner.unit_id == wanderer.spawn_id && !spawner.active);
        if !resting {
            civilians.idle.insert(unit_id);
        }
        wanderer.wandering
    };
    if !was_wandering {
        set_speed(unit_id, BASE_SPEED_MULTIPLIER);
    }
}

fn wander(civilians: &RefCell<Civilians>, unit_id: i32) {
    let Some(wanderer) = civilians
        .borrow()
        .wanderers
        .iter()
        .find(|wanderer| wanderer.unit_id == unit_id)
        .copied()
    else {
        return;
    };
    let (x, z) = random_destination(wanderer.home_x, wanderer.home_z, WANDER_RADIUS);
    let _ = give_fight(unit_id, x, z);
}

fn check_flee(civilians: &RefCell<Civilians>, index: usize, scare_radius: f32) {
    let (wanderer, (ufo_x, ufo_z)) = {
        let civilians = civilians.borrow();
        (
            civilians.wanderers[index],
            civilians.ufo_xz.unwrap_or_default(),
        )
    };
    if !wanderer.wandering {
        return;
    }
    let Some(position) = definitions::unit_base_position(wanderer.unit_id) else {
        return;
    };
    let away = Float3::new(position.x - ufo_x, 0.0, position.z - ufo_z);
    if away.length_squared() >= scare_radius * scare_radius {
        return;
    }
    let away = away.normalized() * RUN_AWAY_DISTANCE;
    let unit_id = wanderer.unit_id;
    give_fight(unit_id, position.x + away.x, position.z + away.z)
        .then(move |()| set_speed(unit_id, FLEE_SPEED_MULTIPLIER));
    {
        let mut civilians = civilians.borrow_mut();
        civilians.wanderers[index].wandering = false;
        civilians.idle.remove(&unit_id);
    }
    diagnostics::debug(&alloc::format!(
        "civilian flee unit={} from=({:.1},{:.1})",
        wanderer.unit_id,
        position.x,
        position.z,
    ));
}

fn update_spawner(civilians: &Rc<RefCell<Civilians>>, index: usize) -> bool {
    let (spawner, (ufo_x, ufo_z)) = {
        let civilians = civilians.borrow();
        (
            civilians.spawners[index],
            civilians.ufo_xz.unwrap_or_default(),
        )
    };
    let Some(specs) = spawner_specs(spawner.unit_def_id) else {
        return false;
    };
    let distance_sq = (spawner.x - ufo_x).powi(2) + (spawner.z - ufo_z).powi(2);
    let active = distance_sq <= UFO_DIST_ACTIVE_SQ;
    if active != spawner.active {
        civilians.borrow_mut().spawners[index].active = active;
        set_spawner_active(civilians, spawner.unit_id, active);
    }
    let mut spawned = false;
    if !spawner.initialized {
        for (spec_index, spec) in specs.iter().enumerate() {
            for _ in 0..spec.wanted {
                spawned |= spawn_wanderer(civilians, spawner.unit_id, spec_index, spec.kind);
            }
        }
        civilians.borrow_mut().spawners[index].initialized = true;
        return spawned;
    }
    if distance_sq <= UFO_DIST_SPAWN_SQ {
        return false;
    }
    for (spec_index, spec) in specs.iter().enumerate() {
        let ready = {
            let mut civilians = civilians.borrow_mut();
            let spawner = &mut civilians.spawners[index];
            if spawner.wanderers[spec_index] >= spec.wanted {
                continue;
            }
            spawner.stock[spec_index] += spec.stock_rate * SPAWNER_PERIOD as f32;
            let ready = spawner.stock[spec_index] > 1.0;
            if ready {
                spawner.stock[spec_index] -= 1.0;
            }
            ready
        };
        if ready {
            spawned |= spawn_wanderer(civilians, spawner.unit_id, spec_index, spec.kind);
        }
    }
    spawned
}

fn set_spawner_active(civilians: &RefCell<Civilians>, spawner_id: i32, active: bool) {
    let units: Vec<i32> = civilians
        .borrow()
        .wanderers
        .iter()
        .filter(|wanderer| wanderer.spawn_id == spawner_id)
        .map(|wanderer| wanderer.unit_id)
        .collect();
    for unit_id in units {
        if active {
            set_idle(civilians, unit_id);
        } else {
            stop(unit_id);
        }
    }
}

fn spawn_wanderer(
    civilians: &Rc<RefCell<Civilians>>,
    spawner_id: i32,
    spec: usize,
    kind: UnitKind,
) -> bool {
    let Some(spawner) = civilians
        .borrow()
        .spawners
        .iter()
        .find(|spawner| spawner.unit_id == spawner_id)
        .copied()
    else {
        return false;
    };
    for _ in 0..SPAWN_ATTEMPTS {
        let (x, z) = random_destination(spawner.x, spawner.z, WANDER_RADIUS);
        let y = spring::get_ground_height(x, z).unwrap_or(0.0);
        if !definitions::position_is_buildable(kind, x, y, z) {
            continue;
        }
        let angle = synced::float() * 360.0;
        let civilians = Rc::clone(civilians);
        definitions::create_at_position(kind, x, y, z, state::ENEMY_TEAM, angle).then(
            move |unit_id| {
                let Some(unit_id) = unit_id else {
                    return;
                };
                {
                    let mut civilians = civilians.borrow_mut();
                    civilians.wanderers.push(Wanderer {
                        unit_id,
                        spawn_id: spawner_id,
                        spec,
                        home_x: spawner.x,
                        home_z: spawner.z,
                        wandering: false,
                    });
                    if let Some(spawner) = civilians
                        .spawners
                        .iter_mut()
                        .find(|spawner| spawner.unit_id == spawner_id)
                    {
                        spawner.wanderers[spec] += 1;
                    }
                }
                set_idle(&civilians, unit_id);
            },
        );
        return true;
    }
    false
}

fn spawner_specs(unit_def_id: i32) -> Option<&'static [SpawnSpec]> {
    Some(match defs::kind(unit_def_id)? {
        UnitKind::Building1 => &BUILDING1,
        UnitKind::Building2 => &BUILDING2,
        UnitKind::Building3 => &BUILDING3,
        UnitKind::Building4 => &BUILDING4,
        UnitKind::Building4BigTank => &BUILDING4_BIG_TANK,
        _ => return None,
    })
}

fn give_fight(unit_id: i32, x: f32, z: f32) -> Pending<()> {
    commands::queue(move || {
        let Some((x, z)) = definitions::clamp_xz(x, z) else {
            return;
        };
        spring::give_order_to_unit(unit_id, cmd::FIGHT, &[x, 0.0, z], 0, -1)
            .log_err_once("Area 17 civilian fight order");
    })
}

fn stop(unit_id: i32) {
    definitions::stop(unit_id).then(move |accepted| {
        if !accepted {
            spring::log_warning!(
                crate::diagnostics::SECTION,
                "Area 17 civilian {unit_id} rejected its stop order"
            );
        }
    });
}

fn set_speed(unit_id: i32, multiplier: f32) {
    if !definitions::set_move_speed_multiplier(unit_id, multiplier) {
        spring::log_warning!(
            crate::diagnostics::SECTION,
            "Area 17 civilian {unit_id} rejected its speed change"
        );
    }
    spring::set_unit_rules_param_f32(unit_id, "selfMoveSpeedChange", multiplier, los::PRIVATE)
        .log_err_once("Area 17 civilian speed param");
}

fn random_destination(home_x: f32, home_z: f32, radius: f32) -> (f32, f32) {
    let distance = radius * synced::float();
    let angle = core::f32::consts::TAU * synced::float();
    (
        home_x + angle.cos() * distance,
        home_z + angle.sin() * distance,
    )
}
