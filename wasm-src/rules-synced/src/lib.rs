extern crate alloc;

pub mod abduction;
pub mod civilians;
pub mod cus_scripts;
pub mod definitions;
pub mod defs;
pub mod diagnostics;
pub mod drones;
pub mod fall_damage;
pub mod gadgets;
#[cfg(feature = "headless-integration")]
pub mod integration;
pub mod scenario;
pub mod script_bridge;
pub mod state;
pub mod ufo_controller;
pub mod upgrades;
#[cfg(feature = "e2e")]
pub mod visual_integration;
pub mod weapon_multishot;
pub mod weapons;

spring::export_core_cus!(cus_scripts::Area17CusModule);

spring_addons::export_rules_gadgets! {
    state: state::Global,
    setup: gadgets::setup,
    scratch: 8192,
}
