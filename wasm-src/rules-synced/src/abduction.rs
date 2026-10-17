use crate::defs::{self, UnitKind};
use crate::{definitions, script_bridge, state};
use spring::{Float3, ResultExt};
use state::{BeamState, Global};

const GRAB_DISTANCE: f32 = 40.0;
const SFX_CHANNEL: i32 = 1;
const HEAL_FRACTION: f32 = 0.05;

pub fn update(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &Global,
    gravity_per_frame: f32,
) {
    let ufo_id = global.ufo.access().id;
    let (beam, radius, pull_speed, accel) = {
        let tech = global.tech.access();
        (
            tech.beam,
            tech.abduction_radius,
            tech.abduction_speed,
            tech.abduction_accel,
        )
    };
    if ufo_id < 0 || beam == BeamState::GodGun {
        return;
    }
    let Some(ufo_position) = definitions::unit_base_position(spring, ufo_id) else {
        return;
    };
    let ufo_velocity = definitions::unit_velocity(spring, ufo_id).unwrap_or_default();

    let candidates = spring
        .units_query()
        .get_units_in_cylinder(ufo_position.x, ufo_position.z, radius, -1)
        .unwrap_or_default();
    let mut beam_active = false;
    for unit_id in candidates {
        let kind = defs::unit_kind(spring, unit_id);
        if unit_id == ufo_id
            || !can_abduct(kind)
            || spring
                .units_info()
                .get_unit_is_dead(unit_id)
                .unwrap_or(true)
        {
            continue;
        }
        let Some(unit_position) = definitions::unit_position(spring, unit_id) else {
            continue;
        };
        let height = definitions::unit_height(spring, unit_id);
        if ufo_position.y - unit_position.y < GRAB_DISTANCE + height * 2.0 {
            capture(spring, global, unit_id, kind, unit_position);
        } else {
            beam_active = true;
            script_bridge::abduction_float(spring, unit_id);
            float_unit(
                spring,
                pull_speed,
                definitions::unit_def_id(spring, unit_id)
                    .and_then(|id| defs::unit_data(spring, id))
                    .map_or(1.0, |data| data.abduct_multiplier),
                accel,
                unit_id,
                ufo_position,
                ufo_velocity,
                unit_position,
                gravity_per_frame,
            );
        }
    }
    let beam = if beam_active {
        BeamState::Lifting
    } else {
        BeamState::Idle
    };
    global.tech.access_mut().beam = beam;
}

fn capture(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &Global,
    unit_id: i32,
    kind: Option<UnitKind>,
    unit_position: Float3,
) {
    let [biomass, research, metal] = definitions::unit_def_id(spring, unit_id)
        .and_then(|id| defs::unit_data(spring, id))
        .map_or([0.0; 3], |data| data.capture_value);
    {
        let mut economy = global.economy.access_mut();
        economy.biomass += biomass;
        economy.research += research;
        economy.metal += metal;
    }
    let ufo_id = global.ufo.access().id;
    let healed = definitions::heal_fraction(spring, ufo_id, HEAL_FRACTION);
    let sound = if kind == Some(UnitKind::Alien) {
        spring.sound().play_sound_file(
            "sounds/yay.wav",
            100.0,
            Float3::ZERO,
            Float3::ZERO,
            SFX_CHANNEL,
        )
    } else {
        spring.sound().play_sound_file(
            "sounds/swoop.wav",
            50.0,
            unit_position,
            Float3::ZERO,
            SFX_CHANNEL,
        )
    };
    sound.log_err_once(spring, "Area 17 capture sound");
    definitions::destroy_reclaimed(spring, unit_id).then(move |spring, removed| {
        if !removed {
            spring::log_warning!(
                spring,
                crate::diagnostics::SECTION,
                "Area 17 could not remove captured unit {unit_id}"
            );
        }
    });
    if !healed {
        spring::log_warning!(
            spring,
            crate::diagnostics::SECTION,
            "Area 17 capture could not apply the UFO heal"
        );
    }
    if kind == Some(UnitKind::Endcube) {
        let mut session = global.session.access_mut();
        session.game_over = true;
        session.game_won = true;
    }
}

fn float_unit(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    abduction_speed: f32,
    multiplier: f32,
    abduction_accel: f32,
    unit_id: i32,
    ufo_position: Float3,
    ufo_velocity: Float3,
    unit_position: Float3,
    gravity_per_frame: f32,
) {
    let velocity = definitions::unit_velocity(spring, unit_id).unwrap_or_default();
    let desired = (ufo_position - unit_position).normalized() * (abduction_speed * multiplier);
    let mut correction = Float3 {
        x: desired.x - velocity.x + ufo_velocity.x,
        y: desired.y - velocity.y + gravity_per_frame,
        z: desired.z - velocity.z + ufo_velocity.z,
    };
    let vertical_accel = gravity_per_frame + 0.4 * multiplier * abduction_accel;
    correction.y = correction.y.min(vertical_accel);
    let horizontal = (correction.x * correction.x + correction.z * correction.z).sqrt();
    let max_horizontal = 2.0 * abduction_accel * multiplier;
    if horizontal > max_horizontal {
        let scale = max_horizontal / horizontal;
        correction.x *= scale;
        correction.z *= scale;
    }
    let _ = definitions::set_rotation(
        spring,
        unit_id,
        Float3 {
            x: 0.0,
            y: -spring
                .units_info()
                .get_unit_heading(unit_id, true)
                .unwrap_or(0.0),
            z: 0.0,
        },
    );
    let _ = definitions::add_impulse(spring, unit_id, Float3::new(0.0, -4.0, 0.0));
    let _ = definitions::add_impulse(
        spring,
        unit_id,
        Float3 {
            x: correction.x,
            y: 4.0 + correction.y,
            z: correction.z,
        },
    );
}

fn can_abduct(kind: Option<UnitKind>) -> bool {
    !matches!(
        kind,
        Some(
            UnitKind::Drone
                | UnitKind::Helipad
                | UnitKind::Building1
                | UnitKind::Building2
                | UnitKind::Building3
                | UnitKind::Building4
                | UnitKind::Building4BigTank
                | UnitKind::StaticAa
                | UnitKind::StrongAa
                | UnitKind::UfoWreck
        )
    )
}
