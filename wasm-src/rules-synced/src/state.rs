use alloc::vec::Vec;
use spring_addons::Resource;

use crate::fall_damage::FallImmunity;
use spring::Float3;

pub use area17_shared::{Ability, TechId, Weapon};

pub const PLAYER_TEAM: i32 = 0;
pub const ENEMY_TEAM: i32 = 1;

pub const UFO_SCARE_RADIUS: f32 = 500.0;
pub const DEFAULT_ABDUCTION_RADIUS: f32 = 70.0;
pub const DEFAULT_ABDUCTION_SPEED: f32 = 7.0;

// Zero-based slots for CUS callbacks and weapon-definition arrays.
// Spring weapon getters and setters take slot + 1.
pub mod ufo_slots {
    use core::ops::Range;

    use super::Weapon;

    pub const fn of(weapon: Weapon) -> Range<i32> {
        match weapon {
            Weapon::IncendiaryBeamLaser => INCENDIARY,
            Weapon::GravityBeam => GRAVITY,
            Weapon::PulseLaser => PULSE..PULSE + 1,
            Weapon::BlackHoleGun => BLACK_HOLE..BLACK_HOLE + 1,
        }
    }

    pub const INCENDIARY: Range<i32> = 0..4;
    pub const GRAVITY: Range<i32> = 4..8;
    pub const GOD_GUN: i32 = 8;
    pub const POINT_DEFENSE: Range<i32> = 12..16;
    pub const BLACK_HOLE: i32 = 16;
    pub const PULSE: i32 = 18;
    pub const SHIELD: i32 = 19;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BeamState {
    GodGun,
    #[default]
    Idle,
    Lifting,
}

impl BeamState {
    pub const fn value(self) -> f32 {
        match self {
            Self::GodGun => -1.0,
            Self::Idle => 0.0,
            Self::Lifting => 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TechState {
    pub level: i32,
    pub unlocked: bool,
    pub enabled: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum PendingInput {
    Fire(Option<Float3>),
    CeaseFire,
    Ability(Ability),
    ChangeWeapon(Weapon),
    Unlock(TechId),
    Upgrade(TechId),
}

#[derive(Clone, Copy, Debug)]
pub struct WeaponTarget {
    pub position: Float3,
    pub use_aim_target: bool,
}

pub struct Ufo {
    pub id: i32,
    pub spawn_position: Option<Float3>,
    pub respawn_frame: Option<i32>,
    pub destroyed_position: Option<Float3>,
    pub moving: bool,
    pub aim: Float3,
    pub aim_target: Option<Float3>,
    pub movement: (f32, f32),
    pub weapon_target: Option<WeaponTarget>,
    pub target_accepted: bool,
    pub current_weapon: Option<Weapon>,
}

impl Default for Ufo {
    fn default() -> Self {
        Self {
            id: -1,
            spawn_position: None,
            respawn_frame: None,
            destroyed_position: None,
            moving: false,
            aim: Float3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            aim_target: None,
            movement: (0.0, 0.0),
            weapon_target: None,
            target_accepted: false,
            current_weapon: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Economy {
    pub biomass: f32,
    pub research: f32,
    pub metal: f32,
}

pub struct Tech {
    pub tech: [TechState; 13],
    pub effects_ufo_id: i32,
    pub effects_tech: [TechState; 13],
    pub effects_applied: bool,
    pub shield_power: f32,
    pub beam: BeamState,
    pub ability_cooldowns: [i32; 4],
    pub ability_durations: [i32; 4],
    pub abduction_radius: f32,
    pub abduction_speed: f32,
    pub abduction_accel: f32,
}

impl Default for Tech {
    fn default() -> Self {
        Self {
            tech: [TechState::default(); 13],
            effects_ufo_id: -1,
            effects_tech: [TechState::default(); 13],
            effects_applied: false,
            shield_power: 1_000.0,
            beam: BeamState::Idle,
            ability_cooldowns: [0; 4],
            ability_durations: [0; 4],
            abduction_radius: DEFAULT_ABDUCTION_RADIUS,
            abduction_speed: DEFAULT_ABDUCTION_SPEED,
            abduction_accel: 1.0,
        }
    }
}

#[derive(Default)]
pub struct Input {
    pub pending: Vec<PendingInput>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Session {
    pub game_over: bool,
    pub game_won: bool,
}

pub fn tech(tech: &Tech, id: TechId) -> TechState {
    tech.tech[id.index()]
}

pub fn tech_mut(tech: &mut Tech, id: TechId) -> &mut TechState {
    &mut tech.tech[id.index()]
}

#[derive(Default)]
pub struct Global {
    pub ufo: Resource<Ufo>,
    pub economy: Resource<Economy>,
    pub tech: Resource<Tech>,
    pub input: Resource<Input>,
    pub session: Resource<Session>,
    pub fall_immunity: Resource<FallImmunity>,
}
