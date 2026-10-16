use alloc::rc::Rc;
use alloc::{format, string::String};
use core::cell::RefCell;
use spring::{ResultExt, commands};

use crate::definitions;
use crate::defs::{self, UnitKind};
use crate::diagnostics;
use crate::state::{self, Global};
use spring::Float3;
use spring::cmd;
use spring::rules_params::los;

#[derive(Clone, Copy)]
pub struct InitialUnit {
    pub name: &'static str,
    pub tooltip: &'static str,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub angle: f32,
    pub team: i32,
    pub neutral: bool,
    pub experience: f32,
    pub rules_n: f32,
    pub firestate: i32,
    pub movestate: i32,
    pub cloak: bool,
    pub repeat: bool,
}

include!(concat!(env!("OUT_DIR"), "/scenario_generated.rs"));

const RESPAWNED_UFO: InitialUnit = InitialUnit {
    name: UnitKind::Ufo.def_name(),
    tooltip: "UFO - ufo",
    x: 0.0,
    y: 0.0,
    z: 0.0,
    angle: 0.0,
    team: state::PLAYER_TEAM,
    neutral: false,
    experience: 0.0,
    rules_n: 0.0,
    firestate: 0,
    movestate: 0,
    cloak: false,
    repeat: false,
};

#[derive(Default)]
struct LoadReport {
    ufo_configured: bool,
    created: usize,
    failed: usize,
    failed_names: String,
}

impl LoadReport {
    fn fail(&mut self, name: &str) {
        self.failed += 1;
        if self.failed <= 8 {
            if !self.failed_names.is_empty() {
                self.failed_names.push_str(", ");
            }
            self.failed_names.push_str(name);
        }
    }
}

pub fn load() {
    let report = Rc::new(RefCell::new(LoadReport::default()));
    for unit in INITIAL_UNITS {
        let Some(kind) = UnitKind::from_def_name(unit.name) else {
            report.borrow_mut().fail(unit.name);
            continue;
        };
        let report = Rc::clone(&report);
        definitions::create_at_height(kind, unit.x, unit.y, unit.z, unit.team, unit.angle).then(
            move |unit_id| {
                let Some(unit_id) = unit_id else {
                    report.borrow_mut().fail(unit.name);
                    return;
                };
                let first_ufo = {
                    let mut report = report.borrow_mut();
                    report.created += 1;
                    let first_ufo = kind == UnitKind::Ufo && !report.ufo_configured;
                    report.ufo_configured |= first_ufo;
                    first_ufo
                };
                if first_ufo {
                    crate::with_global(|global| {
                        let mut ufo = global.ufo.access_mut();
                        ufo.id = unit_id;
                        ufo.spawn_position = Some(Float3 {
                            x: unit.x,
                            y: unit.y,
                            z: unit.z,
                        });
                    });
                    initialize_ufo_health(unit_id);
                    let _ = definitions::set_unit_param(unit_id, "selfMoveSpeedChange", 1.0);
                }
                configure_unit(unit_id, unit);
            },
        );
    }
    commands::run(move || {
        let report = report.borrow();
        if report.failed == 0 {
            diagnostics::info(&format!(
                "Area 17 scenario spawned all {} initial units",
                report.created
            ));
        } else {
            spring::log_warning!(
                crate::diagnostics::SECTION,
                "Area 17 scenario spawned {}/{} initial units; failed={} ({})",
                report.created,
                INITIAL_UNITS.len(),
                report.failed,
                report.failed_names
            );
        }
    });
}

pub fn respawn(global: &Global, frame: i32) {
    let position = {
        let mut ufo = global.ufo.access_mut();
        let Some(respawn_frame) = ufo.respawn_frame else {
            return;
        };
        if respawn_frame > frame {
            return;
        }
        let Some(position) = ufo.spawn_position else {
            ufo.respawn_frame = None;
            return;
        };
        ufo.respawn_frame = None;
        position
    };
    definitions::create_at_height(
        UnitKind::Ufo,
        position.x,
        position.y,
        position.z,
        state::PLAYER_TEAM,
        0.0,
    )
    .then(move |unit_id| {
        let Some(unit_id) = unit_id else {
            spring::log_warning!(crate::diagnostics::SECTION, "Area 17 UFO respawn failed");
            crate::with_global(|global| global.ufo.access_mut().respawn_frame = Some(frame + 1));
            return;
        };
        crate::with_global(|global| global.ufo.access_mut().id = unit_id);
        initialize_ufo_health(unit_id);
        let _ = definitions::set_unit_param(unit_id, "selfMoveSpeedChange", 1.0);
        configure_unit(
            unit_id,
            &InitialUnit {
                x: position.x,
                y: position.y,
                z: position.z,
                ..RESPAWNED_UFO
            },
        );
        diagnostics::info("Area 17 UFO respawned");
    });
}

fn initialize_ufo_health(unit_id: i32) {
    let max_health = defs::unit_def(UnitKind::Ufo)
        .and_then(defs::unit_data)
        .map_or(1_000.0, |physics| physics.health);
    let health = max_health.max(1.0);
    spring::set_unit_max_health(unit_id, health)
        .log_err_once("Area 17 scenario: set unit max health");
    let _ = definitions::set_health(unit_id, health);
}

fn configure_unit(unit_id: i32, unit: &InitialUnit) {
    if !unit.tooltip.is_empty() {
        spring::set_unit_tooltip(unit_id, unit.tooltip)
            .log_err_once("Area 17 scenario: set unit tooltip");
    }
    spring::set_unit_neutral(unit_id, unit.neutral)
        .log_err_once("Area 17 scenario: set unit neutral");
    spring::set_unit_experience(unit_id, unit.experience)
        .log_err_once("Area 17 scenario: set unit experience");
    spring::set_unit_rules_param_f32(unit_id, "n", unit.rules_n, los::PRIVATE)
        .log_err_once("Area 17 scenario: set unit rules param f32");
    set_initial_state(
        unit_id,
        unit.firestate,
        unit.movestate,
        unit.cloak,
        unit.repeat,
    );
}

fn set_initial_state(unit_id: i32, firestate: i32, movestate: i32, cloak: bool, repeat: bool) {
    for (command, value) in [
        (cmd::FIRE_STATE, firestate as f32),
        (cmd::MOVE_STATE, movestate as f32),
        (cmd::CLOAK, cloak as i32 as f32),
        (cmd::REPEAT, repeat as i32 as f32),
    ] {
        let accepted = spring::give_order_to_unit(
            unit_id,
            cmd::INSERT,
            &[0.0, command as f32, 0.0, value],
            cmd::options::ALT_KEY,
            -1,
        )
        .unwrap_or(false);
        if !accepted {
            spring::log_warning!(
                crate::diagnostics::SECTION,
                "Area 17 unit {unit_id} rejected its initial state {command}={value}"
            );
        }
    }
}
