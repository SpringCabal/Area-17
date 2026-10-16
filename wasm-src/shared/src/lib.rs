#![no_std]

const GAME_SPEED: i32 = 30;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TechId {
    Armor,
    ConeUpgrade,
    PulseLaser,
    BlackHoleGun,
    IncendiaryBeamLaser,
    GravityBeam,
    Shield,
    AntiMissilePointDefense,
    CarrierDrones,
    Haste,
    Cloak,
    Teleport,
    IndependenceDayGun,
}

impl TechId {
    pub const ALL: [Self; 13] = [
        Self::Armor,
        Self::ConeUpgrade,
        Self::PulseLaser,
        Self::BlackHoleGun,
        Self::IncendiaryBeamLaser,
        Self::GravityBeam,
        Self::Shield,
        Self::AntiMissilePointDefense,
        Self::CarrierDrones,
        Self::Haste,
        Self::Cloak,
        Self::Teleport,
        Self::IndependenceDayGun,
    ];

    pub const fn index(self) -> usize {
        self as usize
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|tech| tech.name() == name)
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Armor => "armor",
            Self::ConeUpgrade => "coneUpgrade",
            Self::PulseLaser => "pulseLaser",
            Self::BlackHoleGun => "blackHoleGun",
            Self::IncendiaryBeamLaser => "incendiaryBeamLaser",
            Self::GravityBeam => "gravityBeam",
            Self::Shield => "shield",
            Self::AntiMissilePointDefense => "antiMissilePointDefense",
            Self::CarrierDrones => "carrierDrones",
            Self::Haste => "haste",
            Self::Cloak => "cloak",
            Self::Teleport => "teleport",
            Self::IndependenceDayGun => "independenceDayGun",
        }
    }

    pub const fn description(self) -> &'static str {
        match self {
            Self::Armor => "Armor upgrade (+40% armor)",
            Self::ConeUpgrade => "Advanced gravitons (+40% beamup cone and size)",
            Self::PulseLaser => "Pulse laser (+100% damage)",
            Self::BlackHoleGun => "Black hole (+20% duration, radius)",
            Self::IncendiaryBeamLaser => "Incendiary beam laser (+40% fire duration, damage)",
            Self::GravityBeam => "Gravity beam (+40% strength)",
            Self::Shield => "Shield (+50% recharge speed)",
            Self::AntiMissilePointDefense => "Anti missile point defense (+15% fire rate)",
            Self::CarrierDrones => "Carrier drones (+1 drone)",
            Self::Haste => "Haste (+10% speed and duration)",
            Self::Cloak => "Cloak (+15% shorter cooldown)",
            Self::Teleport => "Teleport (+30% distance)",
            Self::IndependenceDayGun => "Independence day gun (+100% duration)",
        }
    }

    pub const fn value_per_level(self) -> f32 {
        match self {
            Self::Armor | Self::ConeUpgrade | Self::IncendiaryBeamLaser | Self::GravityBeam => 40.0,
            Self::PulseLaser | Self::IndependenceDayGun => 100.0,
            Self::BlackHoleGun => 20.0,
            Self::Shield => 50.0,
            Self::AntiMissilePointDefense | Self::Cloak => 15.0,
            Self::CarrierDrones => 1.0,
            Self::Haste => 10.0,
            Self::Teleport => 30.0,
        }
    }

    pub fn value(self, level: i32) -> f32 {
        self.value_per_level() * level as f32
    }

    pub const fn tier(self) -> i32 {
        match self {
            Self::Armor
            | Self::PulseLaser
            | Self::IncendiaryBeamLaser
            | Self::Shield
            | Self::Haste => 1,
            Self::IndependenceDayGun => 3,
            _ => 2,
        }
    }

    pub const fn max_level(self) -> i32 {
        match self {
            Self::Armor | Self::Shield => 10,
            Self::IndependenceDayGun => 1,
            _ => 5,
        }
    }

    pub const fn starts_unlocked(self) -> bool {
        matches!(self, Self::Armor | Self::ConeUpgrade)
    }

    pub const fn dependencies(self) -> &'static [Self] {
        match self {
            Self::Armor
            | Self::PulseLaser
            | Self::IncendiaryBeamLaser
            | Self::Shield
            | Self::Haste => &[],
            Self::ConeUpgrade => &[Self::Armor],
            Self::BlackHoleGun => &[Self::PulseLaser],
            Self::GravityBeam => &[Self::IncendiaryBeamLaser],
            Self::AntiMissilePointDefense | Self::CarrierDrones => &[Self::Shield],
            Self::Cloak | Self::Teleport => &[Self::Haste],
            Self::IndependenceDayGun => &[
                Self::BlackHoleGun,
                Self::GravityBeam,
                Self::AntiMissilePointDefense,
                Self::CarrierDrones,
                Self::Cloak,
                Self::Teleport,
            ],
        }
    }

    pub fn unlock_cost(self) -> f32 {
        let tier = self.tier();
        let tier_multiplier = if tier > 1 { 5 * tier } else { 1 };
        (tier_multiplier * 1_000) as f32
    }

    pub fn upgrade_cost(self, level: i32) -> f32 {
        let tier = self.tier();
        let level_multiplier = if tier == 3 {
            5 * 3
        } else if level > 0 {
            level * 3
        } else {
            1
        };
        ((2 * tier - 1) * level_multiplier * 500) as f32
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Weapon {
    IncendiaryBeamLaser,
    GravityBeam,
    PulseLaser,
    BlackHoleGun,
}

impl Weapon {
    pub const ALL: [Self; 4] = [
        Self::IncendiaryBeamLaser,
        Self::GravityBeam,
        Self::PulseLaser,
        Self::BlackHoleGun,
    ];

    pub const fn index(self) -> usize {
        self as usize
    }

    pub const fn tech(self) -> TechId {
        match self {
            Self::IncendiaryBeamLaser => TechId::IncendiaryBeamLaser,
            Self::GravityBeam => TechId::GravityBeam,
            Self::PulseLaser => TechId::PulseLaser,
            Self::BlackHoleGun => TechId::BlackHoleGun,
        }
    }

    pub const fn name(self) -> &'static str {
        self.tech().name()
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|weapon| weapon.name() == name)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ability {
    Haste,
    Cloak,
    Teleport,
    IndependenceDayGun,
}

impl Ability {
    pub const ALL: [Self; 4] = [
        Self::Haste,
        Self::Cloak,
        Self::Teleport,
        Self::IndependenceDayGun,
    ];

    pub const fn index(self) -> usize {
        self as usize
    }

    pub const fn tech(self) -> TechId {
        match self {
            Self::Haste => TechId::Haste,
            Self::Cloak => TechId::Cloak,
            Self::Teleport => TechId::Teleport,
            Self::IndependenceDayGun => TechId::IndependenceDayGun,
        }
    }

    pub const fn name(self) -> &'static str {
        self.tech().name()
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|ability| ability.name() == name)
    }

    pub const fn base_duration(self) -> i32 {
        match self {
            Self::Haste | Self::Cloak => 5 * GAME_SPEED,
            Self::Teleport => 0,
            Self::IndependenceDayGun => 10 * GAME_SPEED,
        }
    }

    pub const fn base_cooldown(self) -> i32 {
        match self {
            Self::Haste => 20 * GAME_SPEED,
            Self::Cloak | Self::IndependenceDayGun => 60 * GAME_SPEED,
            Self::Teleport => 15 * GAME_SPEED,
        }
    }

    pub fn duration(self, level: i32) -> i32 {
        let base = self.base_duration();
        match self {
            Self::Haste | Self::IndependenceDayGun => {
                round_frames(base as f32 * (1.0 + self.tech().value(level) / 100.0))
            }
            Self::Cloak | Self::Teleport => base,
        }
    }

    pub fn cooldown(self, level: i32) -> i32 {
        let base = self.base_cooldown();
        match self {
            Self::Cloak => round_frames(base as f32 * (1.0 - self.tech().value(level) / 100.0)),
            Self::Haste | Self::Teleport | Self::IndependenceDayGun => base,
        }
    }
}

fn round_frames(frames: f32) -> i32 {
    if frames <= 0.0 {
        0
    } else {
        (frames + 0.5) as i32
    }
}

pub mod message {
    pub const STATUS: &str = "AREA17_STATUS";
    pub const DRONE: &str = "AREA17_DRONE";

    pub const AIM: &str = "aimWeapon";
    pub const FIRE: &str = "fireWeapon";
    pub const CEASE_FIRE: &str = "ceaseFire";
    pub const CHANGE_WEAPON: &str = "changeWeapon";
    pub const ABILITY: &str = "ability";
    pub const UNLOCK: &str = "unlock";
    pub const UPGRADE: &str = "upgrade";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upgrade_costs_follow_tech_lua() {
        assert_eq!(TechId::Armor.upgrade_cost(0), 500.0);
        assert_eq!(TechId::Armor.upgrade_cost(1), 1_500.0);
        assert_eq!(TechId::Armor.upgrade_cost(4), 6_000.0);
        assert_eq!(TechId::GravityBeam.upgrade_cost(0), 1_500.0);
        assert_eq!(TechId::GravityBeam.upgrade_cost(2), 9_000.0);
        assert_eq!(TechId::IndependenceDayGun.upgrade_cost(0), 37_500.0);
    }

    #[test]
    fn unlock_costs_follow_tech_lua() {
        assert_eq!(TechId::PulseLaser.unlock_cost(), 1_000.0);
        assert_eq!(TechId::Cloak.unlock_cost(), 10_000.0);
        assert_eq!(TechId::IndependenceDayGun.unlock_cost(), 15_000.0);
    }

    #[test]
    fn value_per_level_is_the_number_in_the_description() {
        for tech in TechId::ALL {
            let description = tech.description();
            let start = description.find(|c: char| c.is_ascii_digit()).unwrap();
            let end = description[start..]
                .find(|c: char| !c.is_ascii_digit())
                .map_or(description.len(), |end| start + end);
            let number: f32 = description[start..end].parse().unwrap();
            assert_eq!(number, tech.value_per_level(), "{}", tech.name());
        }
    }

    #[test]
    fn tiers_match_the_dependencies() {
        for tech in TechId::ALL {
            let expected = match tech.dependencies().len() {
                0 => 1,
                1 => 2,
                _ => 3,
            };
            assert_eq!(tech.tier(), expected, "{}", tech.name());
        }
    }

    #[test]
    fn names_round_trip() {
        for tech in TechId::ALL {
            assert_eq!(TechId::from_name(tech.name()), Some(tech));
            assert_eq!(TechId::ALL[tech.index()], tech);
        }
        for weapon in Weapon::ALL {
            assert_eq!(Weapon::from_name(weapon.name()), Some(weapon));
        }
        for ability in Ability::ALL {
            assert_eq!(Ability::from_name(ability.name()), Some(ability));
            assert_eq!(Ability::ALL[ability.index()], ability);
        }
    }

    #[test]
    fn ability_timings_scale_with_their_technology() {
        assert_eq!(Ability::Haste.duration(0), 150);
        assert_eq!(Ability::Haste.duration(2), 180);
        assert_eq!(Ability::Cloak.cooldown(2), 1_260);
        assert_eq!(Ability::Cloak.duration(2), 150);
        assert_eq!(Ability::IndependenceDayGun.duration(1), 600);
        assert_eq!(Ability::Teleport.cooldown(5), 450);
    }
}
