use springcabal::check::Suite;

use crate::{definitions, defs, state, upgrades};
use state::{Ability, Global, TechId, Weapon};

const SECTION: &str = "area17-visual-test";
const TAG: &str = "[Area17-Visual-Test]";
const COMPLETE_AFTER_FRAMES: i32 = 420;
const BASELINE_CLEAR_FRAME: i32 = 10;
const BASELINE_RESTORE_FRAME: i32 = 30;
const GOD_GUN_START_FRAME: i32 = 45;
const GOD_GUN_STOP_FRAME: i32 = 180;
const GOD_GUN_SETTLE_FRAME: i32 = 220;

pub struct TestState {
    start_frame: Option<i32>,
    ufo_id: i32,
    weapon_ids: [i32; 4],
    weapons_fired: [bool; 4],
    abilities_activated: [bool; 4],
    abilities_requested: [bool; 4],
    target_accepted: bool,
    origin: Option<spring::Float3>,
    crater_impact: Option<spring::Float3>,
    crater_before: f32,
    crater_depth: f32,
    suite: Suite,
}

impl Default for TestState {
    fn default() -> Self {
        Self {
            start_frame: None,
            ufo_id: 0,
            weapon_ids: [0; 4],
            weapons_fired: [false; 4],
            abilities_activated: [false; 4],
            abilities_requested: [false; 4],
            target_accepted: false,
            origin: None,
            crater_impact: None,
            crater_before: 0.0,
            crater_depth: 0.0,
            suite: Suite::new(TAG, "AREA17 VISUAL INTEGRATION TESTS", 1),
        }
    }
}

impl TestState {
    pub fn projectile_created(&mut self, owner_id: i32, weapon_def_id: i32) {
        if owner_id != self.ufo_id || self.suite.is_finished() {
            return;
        }
        for (index, expected) in self.weapon_ids.iter().enumerate() {
            if weapon_def_id == *expected {
                self.weapons_fired[index] = true;
            }
        }
    }

    pub fn lua_message(&mut self, message: &str) {
        if let Some(ability) =
            springcabal::message::fields(message, area17_shared::message::ABILITY)
                .and_then(|mut fields| fields.next())
                .and_then(Ability::from_name)
        {
            self.abilities_requested[ability.index()] = true;
        }
    }

    pub fn explosion(&mut self, weapon_id: i32, position: spring::Float3) {
        if self.crater_impact.is_some() || !defs::god_gun_weapons().contains(&weapon_id) {
            return;
        }
        let ground = spring::get_ground_height(position.x, position.z).unwrap_or(position.y);
        if position.y - ground <= 2.0 {
            self.crater_before = ground;
            self.crater_impact = Some(position);
        }
    }
}

fn clear_crater_site(ufo_id: i32) {
    let Some(position) = definitions::unit_position(ufo_id) else {
        return;
    };
    let units = spring::get_units_in_cylinder(position.x, position.z, 640.0, -1);
    for unit_id in units.unwrap_or_default() {
        if unit_id != ufo_id {
            let _ = definitions::destroy_reclaimed(unit_id);
        }
    }
}

pub fn update(test: &mut TestState, global: &Global, frame: i32) {
    test.target_accepted |= global.ufo.access().target_accepted;
    update_test(test, global, frame);
}

fn update_test(test: &mut TestState, global: &Global, frame: i32) {
    let ufo_id = global.ufo.access().id;
    if test.suite.is_finished() || ufo_id < 0 {
        return;
    }
    let start = *test.start_frame.get_or_insert_with(|| {
        clear_crater_site(ufo_id);
        prepare(global);
        test.ufo_id = ufo_id;
        for (index, weapon) in Weapon::ALL.iter().enumerate() {
            test.weapon_ids[index] = defs::weapon(*weapon);
        }
        test.origin = definitions::unit_position(ufo_id);
        let _ = definitions::set_velocity(ufo_id, spring::Float3::default());
        spring::log_info!(SECTION, "{TAG} Input fixture ready");
        frame
    });

    let elapsed = frame - start;
    run_crater_sequence(test, global, elapsed);

    for ability in Ability::ALL {
        let tech = global.tech.access();
        test.abilities_activated[ability.index()] |= tech.ability_durations[ability.index()] > 0
            || tech.ability_cooldowns[ability.index()] > 0;
    }

    if elapsed >= COMPLETE_AFTER_FRAMES {
        let activated_by_ui = [Ability::Haste, Ability::Cloak, Ability::Teleport]
            .iter()
            .all(|ability| test.abilities_activated[ability.index()]);
        let passed = test.target_accepted
            && test.weapons_fired.iter().all(|fired| *fired)
            && test.abilities_requested.iter().all(|requested| *requested)
            && activated_by_ui
            && test.crater_depth >= 100.0;
        if passed {
            test.suite.check(
                true,
                "UI input reached the rules through the engine: four weapons fired, four abilities requested, and haste, cloak and teleport activated; the God Gun crater scenario completed",
            );
        } else {
            let message = alloc::format!(
                "input: target={} weapons={:?} requested={:?} activated={:?} crater_depth={:.1}",
                test.target_accepted,
                test.weapons_fired,
                test.abilities_requested,
                test.abilities_activated,
                test.crater_depth,
            );
            test.suite.check(false, message);
        }
        test.suite.finish();
    }
}

fn run_crater_sequence(test: &mut TestState, global: &Global, elapsed: i32) {
    let ufo_id = global.ufo.access().id;
    if elapsed == BASELINE_CLEAR_FRAME {
        if let Some(origin) = test.origin {
            move_ufo(ufo_id, origin, -900.0);
        }
        return;
    }
    if elapsed == BASELINE_RESTORE_FRAME {
        if let Some(origin) = test.origin {
            move_ufo(ufo_id, origin, 0.0);
        }
        return;
    }
    if elapsed == GOD_GUN_START_FRAME {
        let index = Ability::IndependenceDayGun.index();
        {
            let mut tech = global.tech.access_mut();
            tech.ability_cooldowns[index] = 0;
            tech.ability_durations[index] = 0;
        }
        upgrades::activate_ability(global, Ability::IndependenceDayGun);
        return;
    }
    if elapsed == GOD_GUN_STOP_FRAME {
        global.tech.access_mut().ability_durations[Ability::IndependenceDayGun.index()] = 1;
        return;
    }
    if elapsed != GOD_GUN_SETTLE_FRAME {
        return;
    }
    let Some(impact) = test.crater_impact else {
        spring::log_warning!(SECTION, "{TAG} God Gun never reached bare ground");
        return;
    };
    let current = spring::get_ground_height(impact.x, impact.z).unwrap_or(test.crater_before);
    test.crater_depth = (current - test.crater_before).abs();
    move_ufo(ufo_id, impact, -900.0);
    spring::log_info!(
        SECTION,
        "{TAG} God Gun crater ready for rendered capture: depth={:.1} at ({:.1}, {:.1})",
        test.crater_depth,
        impact.x,
        impact.z,
    );
}

fn move_ufo(unit_id: i32, origin: spring::Float3, x_offset: f32) {
    let position = spring::Float3 {
        x: origin.x + x_offset,
        y: spring::get_ground_height(origin.x + x_offset, origin.z).unwrap_or(origin.y) + 180.0,
        z: origin.z,
    };
    let _ = definitions::set_position(unit_id, position);
    let _ = definitions::set_velocity(unit_id, spring::Float3::default());
}

fn prepare(global: &Global) {
    {
        let mut economy = global.economy.access_mut();
        economy.research = 1_000_000.0;
        economy.biomass = 1_000_000.0;
    }
    for tech in TechId::ALL {
        if !state::tech(&global.tech.access(), tech).unlocked {
            let _ = upgrades::unlock(global, tech);
        }
    }
}
