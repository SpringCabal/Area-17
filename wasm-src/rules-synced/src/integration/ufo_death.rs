use spring::commands;

use super::{SECTION, TestState};
use crate::defs::{self, UnitKind};
use crate::state::{Ability, BeamState, Global};
use crate::upgrades;

const DEATH_FRAME: i32 = 100;
const ALIEN_CHECK_FRAME: i32 = DEATH_FRAME + 15;
const RESPAWN_CHECK_FRAME: i32 = DEATH_FRAME + 110;
const BIOMASS_PENALTY: f32 = 10_000.0;
const RESEARCH_PENALTY: f32 = 5_000.0;

pub(super) fn run(test: &mut TestState, global: &Global, elapsed: i32) -> bool {
    match elapsed {
        DEATH_FRAME => {
            let ufo = global.ufo.access().id;
            test.death_ufo = ufo;
            global.session.access_mut().game_won = true;
            upgrades::activate_ability(global, Ability::Haste);
            upgrades::activate_ability(global, Ability::Cloak);
            {
                let tech = global.tech.access();
                if tech.ability_durations[Ability::Haste.index()] == 0
                    || tech.ability_durations[Ability::Cloak.index()] == 0
                {
                    test.fail("death regression did not activate haste and cloak");
                    return true;
                }
            }
            destroy_ufo(ufo);
            return true;
        }
        frame if frame == DEATH_FRAME + 1 => check_death(test, global),
        ALIEN_CHECK_FRAME => check_alien(test),
        RESPAWN_CHECK_FRAME => check_respawn(test, global),
        _ => {}
    }
    false
}

fn destroy_ufo(ufo: i32) {
    commands::queue(move || {
        let economy = crate::with_global(|global| *global.economy.access());
        super::with_test(|test| test.death_economy = (economy.biomass, economy.research));
        spring::destroy_unit(
            ufo,
            spring::rules_synced::unit_control::DestroyUnitOptions {
                selfd: false,
                reclaimed: false,
                attacker_id: -1,
                recycle_id: false,
            },
        )
    })
    .then(move |destroyed| {
        if !destroyed.unwrap_or(false) {
            spring::log_error!(SECTION, "could not destroy unit {ufo}");
        }
    });
}

fn check_death(test: &mut TestState, global: &Global) {
    let (ufo_id, respawn_scheduled) = {
        let ufo = global.ufo.access();
        (ufo.id, ufo.respawn_frame.is_some())
    };
    let economy = *global.economy.access();
    let (biomass, research) = test.death_economy;
    let penalized = (economy.biomass - (biomass - BIOMASS_PENALTY)).abs() < 0.5
        && (economy.research - (research - RESEARCH_PENALTY)).abs() < 0.5;
    let tech = global.tech.access();
    let abilities_cancelled = tech.ability_durations.iter().all(|duration| *duration == 0)
        && tech.ability_cooldowns[Ability::Haste.index()] > 0
        && tech.ability_cooldowns[Ability::Cloak.index()] > 0
        && tech.beam == BeamState::Idle;
    if ufo_id >= 0 || !respawn_scheduled || !penalized || !abilities_cancelled {
        test.fail(alloc::format!(
            "UFO death was not handled: ufo={ufo_id} respawn_scheduled={respawn_scheduled} biomass {biomass:.0}->{:.0} research {research:.0}->{:.0}",
            economy.biomass,
            economy.research,
        ));
        return;
    }
    test.pass("UFO death penalized resources, scheduled a respawn and cancelled abilities while retaining cooldowns");
}

fn check_alien(test: &mut TestState) {
    let alien = spring::get_all_units()
        .unwrap_or_default()
        .into_iter()
        .any(|unit| defs::unit_kind(unit) == Some(UnitKind::Alien));
    if !alien {
        test.fail("no alien dropped over the UFO wreck");
        return;
    }
    test.pass("the alien dropped over the UFO wreck");
}

fn check_respawn(test: &mut TestState, global: &Global) {
    let ufo_id = global.ufo.access().id;
    let max_health = crate::definitions::unit_health(ufo_id).map_or(0.0, |(_, max)| max);
    let live = ufo_id >= 0 && spring::get_unit_def_id(ufo_id).is_ok();
    let won = core::mem::replace(&mut global.session.access_mut().game_won, false);
    if !live || ufo_id == test.death_ufo || max_health <= 0.0 || !won {
        test.fail(alloc::format!(
            "the UFO did not respawn cleanly: ufo={ufo_id} (destroyed {}) max_health={max_health:.1} game_won={won}",
            test.death_ufo,
        ));
        return;
    }
    let speed = spring::get_unit_rules_param_f32(ufo_id, "selfMoveSpeedChange").unwrap_or(0.0);
    if (speed - 1.0).abs() > f32::EPSILON
        || global
            .tech
            .access()
            .ability_durations
            .iter()
            .any(|duration| *duration != 0)
    {
        test.fail(alloc::format!("the replacement UFO retained an active ability or haste speed: speed={speed} durations={:?}", global.tech.access().ability_durations));
        return;
    }
    global.tech.access_mut().ability_cooldowns.fill(0);
    test.pass("a new UFO respawned without active abilities, and the game stayed won");
}
