use crate::state::{BeamState, Weapon};
use spring::Float3;

#[derive(Clone, Copy, Debug)]
pub struct UfoControl {
    pub aim: Float3,
    pub weapon: Option<Weapon>,
    pub beam: BeamState,
}

fn call(
    unit: i32,
    name: &str,
    args: &[f32],
    returns: usize,
) -> Option<spring::UnitScriptCallResult> {
    match spring::call_unit_script(unit, name, args, returns) {
        Ok(result) if result.function_found && result.success => Some(result),
        result => {
            spring::log_warning!("area17", "Lua unit script {unit} {name}: {result:?}");
            None
        }
    }
}

pub fn abduction_float(unit: i32) {
    call(unit, "WasmAbductionFloat", &[], 0);
}
pub fn alien_land(unit: i32) {
    call(unit, "WasmAlienLand", &[], 0);
}
pub fn set_ufo_control(unit: i32, control: UfoControl) {
    let weapon = control
        .weapon
        .map_or(0.0, |weapon| (weapon.index() + 1) as f32);
    let beam = match control.beam {
        BeamState::Idle => 0.0,
        BeamState::Lifting => 1.0,
        BeamState::GodGun => -1.0,
    };
    call(
        unit,
        "WasmUfoControl",
        &[control.aim.x, control.aim.y, control.aim.z, weapon, beam],
        0,
    );
}
#[cfg(feature = "headless-integration")]
pub fn human_walks(unit: i32) -> Option<bool> {
    call(unit, "WasmHumanWalks", &[], 1)
        .and_then(|result| result.ret_values.first().map(|value| *value > 0.0))
}
