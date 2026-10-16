use area17_shared::TechId;

pub struct TechDef {
    pub title: &'static str,
    pub icon: &'static str,
    pub disabled_icon: &'static str,
    pub position: (i32, i32),
}

pub const TECH_DEFS: [TechDef; 13] = [
    TechDef {
        title: "Armor",
        icon: "../Images/armour.png",
        disabled_icon: "../Images/armour_off.png",
        position: (0, 0),
    },
    TechDef {
        title: "Cone upgrade",
        icon: "../Images/coneUpgrade.png",
        disabled_icon: "../Images/coneUpgrade_off.png",
        position: (80, 0),
    },
    TechDef {
        title: "Pulse laser",
        icon: "../Images/laser.png",
        disabled_icon: "../Images/laser_off.png",
        position: (0, 60),
    },
    TechDef {
        title: "Black hole",
        icon: "../Images/blackHole.png",
        disabled_icon: "../Images/blackHole_off.png",
        position: (80, 60),
    },
    TechDef {
        title: "Napalm",
        icon: "../Images/incendiary.png",
        disabled_icon: "../Images/incendiary_off.png",
        position: (0, 120),
    },
    TechDef {
        title: "Gravity beam",
        icon: "../Images/gravityBeam.png",
        disabled_icon: "../Images/gravityBeam_off.png",
        position: (80, 120),
    },
    TechDef {
        title: "Shield",
        icon: "../Images/shield.png",
        disabled_icon: "../Images/shield_off.png",
        position: (0, 210),
    },
    TechDef {
        title: "Anti missile point defense",
        icon: "../Images/antiMissilePointDefense.png",
        disabled_icon: "../Images/antiMissilePointDefense_off.png",
        position: (80, 180),
    },
    TechDef {
        title: "Drones",
        icon: "../Images/carrierDrones.png",
        disabled_icon: "../Images/carrierDrones_off.png",
        position: (80, 240),
    },
    TechDef {
        title: "Haste",
        icon: "../Images/haste.png",
        disabled_icon: "../Images/haste_off.png",
        position: (0, 330),
    },
    TechDef {
        title: "Cloak",
        icon: "../Images/cloak.png",
        disabled_icon: "../Images/cloak_off.png",
        position: (80, 300),
    },
    TechDef {
        title: "Teleport",
        icon: "../Images/teleport.png",
        disabled_icon: "../Images/teleport_off.png",
        position: (80, 360),
    },
    TechDef {
        title: "God gun",
        icon: "../Images/cleanse.png",
        disabled_icon: "../Images/cleanse_off.png",
        position: (160, 180),
    },
];

pub fn find_tech_index(name: &str) -> Option<usize> {
    TechId::from_name(name).map(TechId::index)
}

pub fn tech_unlock_cost(index: usize) -> f32 {
    TechId::ALL[index].unlock_cost()
}

pub fn tech_upgrade_cost(index: usize, level: i32) -> f32 {
    TechId::ALL[index].upgrade_cost(level)
}

pub fn can_activate(simulation: &crate::state::SimulationState, index: usize) -> bool {
    let tech = simulation.tech[index];
    tech.enabled
        && if tech.unlocked {
            tech.level < TechId::ALL[index].max_level()
                && simulation.biomass >= tech_upgrade_cost(index, tech.level)
        } else {
            simulation.research >= tech_unlock_cost(index)
        }
}
