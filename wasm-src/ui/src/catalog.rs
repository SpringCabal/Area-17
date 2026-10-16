use area17_shared::{Ability, Weapon};

pub struct WeaponDef {
    pub weapon: Weapon,
    pub title: &'static str,
    pub key: &'static str,
    pub is_beam: bool,
}

impl WeaponDef {
    pub fn name(&self) -> &'static str {
        self.weapon.name()
    }
}

pub const WEAPONS: [WeaponDef; 4] = [
    WeaponDef {
        weapon: Weapon::IncendiaryBeamLaser,
        title: "NAPALM",
        key: "1",
        is_beam: true,
    },
    WeaponDef {
        weapon: Weapon::GravityBeam,
        title: "GRAVITY BEAM",
        key: "2",
        is_beam: true,
    },
    WeaponDef {
        weapon: Weapon::PulseLaser,
        title: "PULSE LASER",
        key: "3",
        is_beam: false,
    },
    WeaponDef {
        weapon: Weapon::BlackHoleGun,
        title: "BLACK HOLE",
        key: "4",
        is_beam: false,
    },
];

pub fn find_weapon(name: &str) -> Option<usize> {
    Weapon::from_name(name).map(Weapon::index)
}

pub struct AbilityDef {
    pub ability: Ability,
    pub title: &'static str,
    pub key_label: &'static str,
    pub key: &'static str,
}

impl AbilityDef {
    pub fn name(&self) -> &'static str {
        self.ability.name()
    }
}

pub const ABILITIES: [AbilityDef; 4] = [
    AbilityDef {
        ability: Ability::Haste,
        title: "Haste",
        key_label: "Q",
        key: "q",
    },
    AbilityDef {
        ability: Ability::Cloak,
        title: "Cloak",
        key_label: "C",
        key: "c",
    },
    AbilityDef {
        ability: Ability::Teleport,
        title: "Teleport",
        key_label: "E",
        key: "e",
    },
    AbilityDef {
        ability: Ability::IndependenceDayGun,
        title: "God gun",
        key_label: "SPACE",
        key: "space",
    },
];

pub fn find_ability(name: &str) -> Option<usize> {
    Ability::from_name(name).map(Ability::index)
}
