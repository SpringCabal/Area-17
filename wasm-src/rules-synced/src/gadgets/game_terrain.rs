use alloc::collections::BTreeSet;
use alloc::vec::Vec;
use core::cell::RefCell;
use spring::ResultExt;

use spring::consts::SQUARE_SIZE;
use spring_addons::rules::Gadget;

use crate::defs;
use crate::state::{Global, Weapon};
use crate::{definitions, upgrades};

const FIRE_TERRAIN: i32 = 10;
const IMPASSABLE_TERRAIN: i32 = 11;
const NORMAL_TERRAIN: i32 = 1;
const FIRE_DURATION_FRAMES: i32 = 150;
const FIRE_SQUARE_REACH: i32 = 3 * SQUARE_SIZE;

#[derive(Clone, Copy)]
struct TemporaryFire {
    x: i32,
    z: i32,
    expires: i32,
}

#[derive(Default)]
struct Terrain {
    fire: Vec<TemporaryFire>,
    blocked: BTreeSet<(i32, i32)>,
}

#[derive(Default)]
pub struct GameTerrainGadget {
    terrain: RefCell<Terrain>,
}

impl GameTerrainGadget {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Gadget<Global> for GameTerrainGadget {
    fn name(&self) -> &'static str {
        "GameTerrain"
    }

    fn init(&self, _ctx: &spring_addons::AddonContext<'_, Global>) {
        let spring = _ctx.spring();
        spring
            .terrain_control()
            .set_terrain_type_data(
                IMPASSABLE_TERRAIN,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                false,
                "impassable",
            )
            .log_err_once(spring, "Area 17 terrain: set terrain type data");
        spring
            .terrain_control()
            .set_terrain_type_data(FIRE_TERRAIN, 0.0, 0.0, 0.8, 0.8, 0.0, false, "fire")
            .log_err_once(spring, "Area 17 terrain: set terrain type data");
        spring
            .terrain_control()
            .set_terrain_type_data(NORMAL_TERRAIN, 1.0, 1.0, 1.0, 1.0, 0.0, true, "normal")
            .log_err_once(spring, "Area 17 terrain: set terrain type data");
    }

    fn game_frame(&self, _ctx: &spring_addons::AddonContext<'_, Global>, frame: i32) {
        let spring = _ctx.spring();
        let mut expired = Vec::new();
        self.terrain.borrow_mut().fire.retain(|entry| {
            let keep = entry.expires >= frame;
            if !keep {
                expired.push((entry.x, entry.z));
            }
            keep
        });
        for (x, z) in expired {
            set_square(spring, x, z, NORMAL_TERRAIN);
        }
    }

    fn unit_created(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global>,
        unit: i32,
        def: i32,
        _team: i32,
        _builder: i32,
    ) {
        let spring = _ctx.spring();
        let Some((half_x, half_z)) =
            defs::unit_data(spring, def).and_then(|physics| physics.terrain_block)
        else {
            return;
        };
        let Some(position) = definitions::unit_position(spring, unit) else {
            return;
        };
        let left = snap((position.x - half_x).max(0.0));
        let right = snap(position.x + half_x);
        let top = snap((position.z - half_z).max(0.0));
        let bottom = snap(position.z + half_z);
        let mut changed = Vec::new();
        {
            let mut terrain = self.terrain.borrow_mut();
            for x in (left..=right).step_by(SQUARE_SIZE as usize) {
                for z in (top..=bottom).step_by(SQUARE_SIZE as usize) {
                    if terrain.blocked.insert((x, z)) {
                        changed.push((x, z));
                    }
                }
            }
        }
        for (x, z) in changed {
            set_square(spring, x, z, IMPASSABLE_TERRAIN);
        }
    }

    fn explosion(
        &self,
        ctx: &spring_addons::AddonContext<'_, Global>,
        weapon_def_id: i32,
        pos: (f32, f32, f32),
        _owner_id: i32,
        _projectile_id: i32,
    ) -> bool {
        let spring = ctx.spring();
        if weapon_def_id != defs::weapon(spring, Weapon::IncendiaryBeamLaser) {
            return false;
        }
        let multiplier = upgrades::incendiary_duration_multiplier(&ctx.global().tech.access());
        let expires = spring.game().current_frame().unwrap_or(0)
            + (FIRE_DURATION_FRAMES as f32 * multiplier) as i32;
        let center_x = snap(pos.0);
        let center_z = snap(pos.2);
        let mut changes = Vec::new();
        {
            let mut terrain = self.terrain.borrow_mut();
            let reach = (-FIRE_SQUARE_REACH..=FIRE_SQUARE_REACH).step_by(SQUARE_SIZE as usize);
            for offset_x in reach.clone() {
                for offset_z in reach.clone() {
                    let square = (center_x + offset_x, center_z + offset_z);
                    if !terrain.blocked.contains(&square) {
                        changes.push(square);
                    }
                }
            }
            terrain
                .fire
                .retain(|entry| !changes.contains(&(entry.x, entry.z)));
            terrain.fire.extend(
                changes
                    .iter()
                    .map(|&(x, z)| TemporaryFire { x, z, expires }),
            );
        }
        for (x, z) in changes {
            set_square(spring, x, z, FIRE_TERRAIN);
        }
        false
    }
}

fn set_square(spring: &spring::Spring<'_, spring::RulesSynced>, x: i32, z: i32, terrain_type: i32) {
    spring
        .terrain_control()
        .set_map_square_terrain_type(x, z, terrain_type)
        .log_err_once(spring, "Area 17 terrain: set map square terrain type");
}

fn snap(value: f32) -> i32 {
    (((value + 4.0) / 8.0).floor() * 8.0) as i32
}
