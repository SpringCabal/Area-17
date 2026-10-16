use alloc::string::String;
use alloc::vec::Vec;
use springcabal::defs::DefCache;

use spring::Float3;
use spring::consts::GAME_SPEED;

use crate::state::Weapon;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnitKind {
    Ufo,
    UfoWreck,
    Alien,
    Endcube,
    Helipad,
    Drone,
    Civilian,
    Civilian1,
    Scientist,
    Scientist2,
    Tank,
    SmallTank,
    Building1,
    Building2,
    Building3,
    Building4,
    Building4BigTank,
    StaticAa,
    StrongAa,
}

impl UnitKind {
    pub const ALL: [Self; 19] = [
        Self::Ufo,
        Self::UfoWreck,
        Self::Alien,
        Self::Endcube,
        Self::Helipad,
        Self::Drone,
        Self::Civilian,
        Self::Civilian1,
        Self::Scientist,
        Self::Scientist2,
        Self::Tank,
        Self::SmallTank,
        Self::Building1,
        Self::Building2,
        Self::Building3,
        Self::Building4,
        Self::Building4BigTank,
        Self::StaticAa,
        Self::StrongAa,
    ];

    pub const fn def_name(self) -> &'static str {
        match self {
            Self::Ufo => "ufo",
            Self::UfoWreck => "ufo_wreck",
            Self::Alien => "alien",
            Self::Endcube => "endcube",
            Self::Helipad => "helipad",
            Self::Drone => "drone",
            Self::Civilian => "civilian",
            Self::Civilian1 => "civilian1",
            Self::Scientist => "scientist",
            Self::Scientist2 => "scientist2",
            Self::Tank => "tank",
            Self::SmallTank => "small_tank",
            Self::Building1 => "building1",
            Self::Building2 => "building2",
            Self::Building3 => "building3",
            Self::Building4 => "building4",
            Self::Building4BigTank => "building4bigtank",
            Self::StaticAa => "static_aa",
            Self::StrongAa => "strong_aa",
        }
    }

    pub fn from_def_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.def_name() == name)
    }

    const fn index(self) -> usize {
        self as usize
    }

    pub const fn is_static_turret(self) -> bool {
        matches!(self, Self::StaticAa | Self::StrongAa)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct UnitData {
    pub mass: f32,
    pub can_fly: bool,
    pub can_move: bool,
    pub can_hover: bool,
    pub is_building: bool,
    pub speed: f32,
    pub acceleration: f32,
    pub brake_rate: f32,
    pub health: f32,
    pub height: f32,
    pub terrain_block: Option<(f32, f32)>,
    pub capture_value: [f32; 3],
    pub abduct_multiplier: f32,
    pub mid_and_aim: Option<(Float3, Float3)>,
    pub radius_and_height: Option<(f32, f32)>,
}

impl UnitData {
    fn load(def_id: i32) -> Option<Self> {
        let definition = spring::get_unit_def_by_id(def_id).ok()?;
        let dimensions = spring::utils::get_unit_def_dimensions(def_id).ok();
        Some(Self {
            mass: definition.physics.mass,
            can_fly: definition.physics.can_fly,
            can_move: definition.physics.can_move,
            can_hover: definition.physics.can_hover,
            is_building: definition.classify.is_building,
            speed: spring::get_unit_def_speed(def_id).ok()?,
            acceleration: definition.physics.acceleration,
            brake_rate: definition.physics.brake_rate,
            health: spring::get_unit_def_health(def_id).ok()?,
            height: dimensions.map_or(0.0, |dimensions| dimensions.height),
            terrain_block: terrain_block(def_id),
            capture_value: ["biomass", "research", "metal"]
                .map(|key| custom_param(def_id, key).unwrap_or(0.0)),
            abduct_multiplier: custom_param(def_id, "abduct_mult").unwrap_or(1.0),
            mid_and_aim: mid_and_aim(def_id, dimensions),
            radius_and_height: radius_and_height(def_id, dimensions),
        })
    }
}

fn terrain_block(def_id: i32) -> Option<(f32, f32)> {
    let x = custom_param(def_id, "terrainblock_x")?;
    let z = custom_param(def_id, "terrainblock_z").unwrap_or(x);
    Some((x / 2.0, z / 2.0))
}

fn mid_and_aim(
    def_id: i32,
    dimensions: Option<spring::utils::UnitDefDimensions>,
) -> Option<(Float3, Float3)> {
    let mid = custom_param_xyz(def_id, "midposoffset");
    let aim = custom_param_xyz(def_id, "aimposoffset");
    if mid.is_none() && aim.is_none() {
        return None;
    }
    let mid = mid.unwrap_or(Float3::ZERO);
    let aim = aim.unwrap_or(mid);
    let model_mid = dimensions.map_or(Float3::ZERO, |dimensions| {
        Float3::new(dimensions.midx, dimensions.midy, dimensions.midz)
    });
    Some((mid + model_mid, aim + model_mid))
}

fn radius_and_height(
    def_id: i32,
    dimensions: Option<spring::utils::UnitDefDimensions>,
) -> Option<(f32, f32)> {
    let radius = custom_param(def_id, "modelradius");
    let height = custom_param(def_id, "modelheight");
    if radius.is_none() && height.is_none() {
        return None;
    }
    Some((
        radius.or(dimensions.map(|dimensions| dimensions.radius))?,
        height.or(dimensions.map(|dimensions| dimensions.height))?,
    ))
}

fn custom_param_xyz(def_id: i32, key: &str) -> Option<Float3> {
    let text = spring::unit_defs::get_unit_def_custom_param(def_id, key).ok()?;
    let mut numbers = text
        .split_whitespace()
        .map(|value| value.parse::<f32>().ok());
    let (x, y, z) = (numbers.next()??, numbers.next()??, numbers.next()??);
    Some(Float3::new(x, y, z))
}

fn custom_param(def_id: i32, key: &str) -> Option<f32> {
    spring::unit_defs::get_unit_def_custom_param(def_id, key)
        .ok()?
        .trim()
        .parse()
        .ok()
}

pub const AREA_DAMAGE_PERIOD: i32 = 2;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WeaponBehaviour {
    pub impulse: Option<Impulse>,
    pub area: Option<AreaDamage>,
    pub fall_immunity: Option<i32>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Impulse {
    pub magnitude: f32,
    pub normal_damage: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AreaDamage {
    pub damage: f32,
    pub radius: f32,
    pub impulse: bool,
    pub duration_frames: f32,
    pub range_falloff: f32,
    pub time_loss: f32,
}

impl WeaponBehaviour {
    fn load(weapon_def_id: i32) -> Self {
        let keys = spring::weapon_defs::get_weapon_def_custom_param_keys(weapon_def_id)
            .unwrap_or_default();
        let has = |key: &str| keys.iter().any(|candidate| candidate == key);
        let number = |key: &str| weapon_custom_param(weapon_def_id, key);
        let impulse = number("impulse").map(|magnitude| Impulse {
            magnitude,
            normal_damage: has("normaldamage"),
        });
        let area = has("area_damage")
            .then(|| {
                let dps = number("area_damage_dps")?;
                let duration_frames = number("area_damage_duration")? * GAME_SPEED as f32;
                let damage = dps * AREA_DAMAGE_PERIOD as f32 / GAME_SPEED as f32;
                let time_falloff = number("area_damage_time_falloff")?;
                Some(AreaDamage {
                    damage,
                    radius: number("area_damage_radius")?,
                    impulse: weapon_custom_param_text(weapon_def_id, "area_damage_is_impulse")
                        .as_deref()
                        == Some("1"),
                    duration_frames,
                    range_falloff: number("area_damage_range_falloff")?,
                    time_loss: damage * time_falloff * AREA_DAMAGE_PERIOD as f32 / duration_frames,
                })
            })
            .flatten();
        Self {
            impulse,
            area,
            fall_immunity: number("falldamageimmunity").map(|frames| frames as i32),
        }
    }
}

fn weapon_custom_param_text(weapon_def_id: i32, key: &str) -> Option<String> {
    spring::weapon_defs::get_weapon_def_custom_param(weapon_def_id, key).ok()
}

fn weapon_custom_param(weapon_def_id: i32, key: &str) -> Option<f32> {
    weapon_custom_param_text(weapon_def_id, key)?
        .trim()
        .parse()
        .ok()
}

struct Defs {
    unit_ids: [i32; UnitKind::ALL.len()],
    by_def: Vec<Option<(UnitKind, UnitData)>>,
    weapon_ids: [i32; Weapon::ALL.len()],
    god_gun_ids: [i32; 2],
    weapon_behaviours: Vec<WeaponBehaviour>,
}

impl Defs {
    fn load() -> Self {
        let unit_ids = UnitKind::ALL
            .map(|kind| spring::get_unit_def_id_by_name(kind.def_name()).unwrap_or(-1));
        let mut by_def = Vec::new();
        for (kind, def_id) in UnitKind::ALL.into_iter().zip(unit_ids) {
            let Ok(index) = usize::try_from(def_id) else {
                continue;
            };
            let Some(physics) = UnitData::load(def_id) else {
                continue;
            };
            if by_def.len() <= index {
                by_def.resize(index + 1, None);
            }
            by_def[index] = Some((kind, physics));
        }
        Self {
            unit_ids,
            by_def,
            weapon_ids: Weapon::ALL.map(|weapon| weapon_id(weapon_def_name(weapon))),
            god_gun_ids: [
                weapon_id("ufo_independencedaygun"),
                weapon_id("ufo_independencedaygunsilent"),
            ],
            weapon_behaviours: weapon_behaviours(),
        }
    }

    fn entry(&self, def_id: i32) -> Option<(UnitKind, UnitData)> {
        let index = usize::try_from(def_id).ok()?;
        self.by_def.get(index).copied().flatten()
    }
}

fn weapon_behaviours() -> Vec<WeaponBehaviour> {
    let ids = spring::weapon_defs::get_weapon_def_i_ds(0).unwrap_or_default();
    let mut behaviours = Vec::new();
    for id in ids {
        let Ok(index) = usize::try_from(id) else {
            continue;
        };
        if behaviours.len() <= index {
            behaviours.resize(index + 1, WeaponBehaviour::default());
        }
        behaviours[index] = WeaponBehaviour::load(id);
    }
    behaviours
}

fn weapon_id(name: &str) -> i32 {
    spring::weapon_defs::get_weapon_def_id(name).unwrap_or(-1)
}

static DEFS: DefCache<Defs> = DefCache::new(Defs::load);

fn with<R>(f: impl FnOnce(&Defs) -> R) -> R {
    f(DEFS.get())
}

pub fn unit_def(kind: UnitKind) -> Option<i32> {
    with(|defs| Some(defs.unit_ids[kind.index()]).filter(|id| *id >= 0))
}

pub fn kind(def_id: i32) -> Option<UnitKind> {
    with(|defs| defs.entry(def_id).map(|(kind, _)| kind))
}

pub fn unit_kind(unit_id: i32) -> Option<UnitKind> {
    spring::get_unit_def_id(unit_id)
        .ok()
        .and_then(|def| kind(def.0))
}

pub fn unit_data(def_id: i32) -> Option<UnitData> {
    with(|defs| defs.entry(def_id).map(|(_, physics)| physics)).or_else(|| UnitData::load(def_id))
}

pub fn weapon(weapon: Weapon) -> i32 {
    with(|defs| defs.weapon_ids[weapon.index()])
}

const fn weapon_def_name(weapon: Weapon) -> &'static str {
    match weapon {
        Weapon::IncendiaryBeamLaser => "ufo_incendiarybeamlaser",
        Weapon::GravityBeam => "ufo_gravitybeam",
        Weapon::PulseLaser => "ufo_pulselaser",
        Weapon::BlackHoleGun => "ufo_blackholegun",
    }
}

pub fn weapon_behaviour(weapon_def_id: i32) -> Option<WeaponBehaviour> {
    let index = usize::try_from(weapon_def_id).ok()?;
    with(|defs| defs.weapon_behaviours.get(index).copied())
        .filter(|behaviour| *behaviour != WeaponBehaviour::default())
}

pub fn god_gun_weapons() -> [i32; 2] {
    with(|defs| defs.god_gun_ids)
}
