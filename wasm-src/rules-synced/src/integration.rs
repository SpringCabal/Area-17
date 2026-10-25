mod civilians;
mod gameplay;
mod god_gun;
mod ufo_death;
mod weapons;

use core::cell::RefCell;
use core::fmt::Display;

use springcabal::check::Suite;

use crate::{definitions, defs, state, upgrades, weapon_multishot};
use state::{Global, TechId, Weapon};

const SECTION: &str = "area17-headless-test";
const EXPECTED_CHECKS: u32 = 23;

struct Tally(Suite);

impl Default for Tally {
    fn default() -> Self {
        Self(Suite::new(
            "[Area17-Rust-Test]",
            "AREA17 RUST INTEGRATION TESTS",
            EXPECTED_CHECKS,
        ))
    }
}
std::thread_local! {
    static TEST: RefCell<TestState> = RefCell::new(TestState::default());
}

pub fn with_test<R>(f: impl FnOnce(&mut TestState) -> R) -> R {
    TEST.with(|test| f(&mut test.borrow_mut()))
}

#[derive(Default)]
pub struct TestState {
    started_frame: Option<i32>,
    stage: usize,
    expected_weapon_id: i32,
    expected_weapon_slot: i32,
    expected_owner_id: i32,
    observed_weapon_id: i32,
    observed_owner_id: i32,
    target_accepted: bool,
    projectile_seen: bool,
    expected_projectiles: usize,
    observed_projectiles: usize,
    upgrades_verified: bool,
    shield_regen_verified: bool,
    god_gun_impact: Option<spring::Float3>,
    god_gun_impact_altitude: f32,
    god_gun_impact_frame: i32,
    god_gun_min_altitude: f32,
    god_gun_explosions: usize,
    god_gun_ground_before: [f32; god_gun::TERRAIN_SAMPLES],
    god_gun_first_impact_verified: bool,
    death_ufo: i32,
    death_economy: (f32, f32),
    gameplay: gameplay::Gameplay,
    orphans: civilians::Orphans,
    gravity_target: Option<weapons::GravityTarget>,
    tally: Tally,
}

impl TestState {
    fn finished(&self) -> bool {
        self.tally.0.is_finished()
    }

    fn pass(&mut self, spring: &spring::Spring<'_, spring::RulesSynced>, message: impl Display) {
        self.tally.0.check(spring, true, message);
    }

    fn fail(&mut self, spring: &spring::Spring<'_, spring::RulesSynced>, message: impl Display) {
        self.tally.0.check(spring, false, message);
        self.tally.0.finish(spring);
    }

    fn finish(&mut self, spring: &spring::Spring<'_, spring::RulesSynced>) {
        self.tally.0.finish(spring);
    }

    pub fn projectile_created(&mut self, owner_id: i32, weapon_def_id: i32) {
        if !self.finished() && owner_id == self.expected_owner_id {
            self.observed_weapon_id = weapon_def_id;
            self.observed_owner_id = owner_id;
            if weapon_def_id == self.expected_weapon_id {
                self.projectile_seen = true;
                self.observed_projectiles += 1;
            }
        }
    }

    pub fn explosion(
        &mut self,
        spring: &spring::Spring<'_, spring::RulesSynced>,
        weapon_id: i32,
        position: spring::Float3,
    ) {
        god_gun::explosion(spring, self, weapon_id, position);
    }

    pub fn unit_created(
        &mut self,
        spring: &spring::Spring<'_, spring::RulesSynced>,
        unit_id: i32,
        unit_def_id: i32,
    ) {
        civilians::unit_created(spring, self, unit_id, unit_def_id);
    }

    pub fn pre_damaged(
        &mut self,
        spring: &spring::Spring<'_, spring::RulesSynced>,
        unit_id: i32,
        weapon_def_id: i32,
    ) {
        weapons::pre_damaged(spring, self, unit_id, weapon_def_id);
    }
}

fn send_input(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    test: &mut TestState,
    message: &str,
) {
    if !spring
        .messages()
        .send_lua_rules_msg(message)
        .unwrap_or(false)
    {
        test.fail(
            spring,
            alloc::format!("the engine refused the rules message {message}"),
        );
    }
}

fn remove_unit(spring: &spring::Spring<'_, spring::RulesSynced>, unit: i32) {
    definitions::destroy_reclaimed(spring, unit).then(move |spring, removed| {
        if !removed {
            spring::log_error!(spring, SECTION, "could not remove test unit {unit}");
        }
    });
}

pub fn update(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    test: &mut TestState,
    global: &Global,
    frame: i32,
) {
    if test.finished() {
        return;
    }
    test.target_accepted |= global.ufo.access().target_accepted;
    if let Some(start) = test.started_frame
        && ufo_death::run(spring, test, global, frame - start)
    {
        return;
    }
    if global.ufo.access().id >= 0 {
        update_test(spring, test, global, frame);
    }
}

fn update_test(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    test: &mut TestState,
    global: &Global,
    frame: i32,
) {
    let start = match test.started_frame {
        Some(start) => start,
        None => {
            spring::log_info!(
                spring,
                SECTION,
                "[Area17-Rust-Test] Starting integration test suite"
            );
            test.started_frame = Some(frame);
            prepare(spring, test, global);
            frame
        }
    };
    let elapsed = frame - start;

    if elapsed == 1 && !test.upgrades_verified {
        let missing = crate::with_cus_module(|host, spring| {
            Ok(spring
                .units_query()
                .get_all_units()?
                .into_iter()
                .filter(|unit| {
                    defs::unit_kind(spring, *unit).is_some()
                        && !host.is_attached(spring::UnitId(*unit))
                })
                .collect::<alloc::vec::Vec<_>>())
        });
        match missing {
            Ok(missing) if missing.is_empty() => {
                test.pass(
                    spring,
                    "every initial scenario unit has its CUS script attached",
                );
            }
            Ok(missing) => {
                test.fail(
                    spring,
                    alloc::format!(
                        "{} initial scenario units have no CUS script (first IDs: {:?})",
                        missing.len(),
                        &missing[..missing.len().min(8)],
                    ),
                );
                return;
            }
            Err(error) => {
                test.fail(
                    spring,
                    alloc::format!("could not inspect initial scripts: {error:?}"),
                );
                return;
            }
        }
        weapons::verify_weapon_behaviours(spring, test);
        if test.finished() {
            return;
        }
        verify_upgrades(spring, test, global);
        test.upgrades_verified = true;
        if test.finished() {
            return;
        }
    }
    if elapsed == 2 {
        let _ = spring
            .unit_control()
            .set_unit_shield_state(global.ufo.access().id, -1, true, 10.0);
    }
    if elapsed == 3 && !test.shield_regen_verified {
        let Some(shield) = spring
            .units_info()
            .get_unit_shield_state(global.ufo.access().id, -1)
            .ok()
            .flatten()
        else {
            test.fail(
                spring,
                "could not read the live shield state from the engine",
            );
            return;
        };
        if !(11.4..11.6).contains(&shield.shield_power) {
            test.fail(spring,
                "shield regeneration overwrote damage instead of regenerating from the live shield state",
            );
            return;
        }
        test.shield_regen_verified = true;
        test.pass(
            spring,
            "shield regeneration starts from the live engine shield power",
        );
    }

    gameplay::run(spring, test, global, elapsed);
    civilians::run(spring, test, global, elapsed);
    if test.finished() {
        return;
    }
    if test.stage < Weapon::ALL.len() {
        weapons::run_weapon_stage(spring, test, global, elapsed);
        return;
    }
    if test.stage == Weapon::ALL.len() {
        weapons::run_haste_stage(spring, test, global, elapsed);
        return;
    }
    god_gun::run(spring, test, global, elapsed);
}

fn prepare(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    test: &mut TestState,
    global: &Global,
) {
    {
        let mut economy = global.economy.access_mut();
        economy.research = 1_000_000.0;
        economy.biomass = 1_000_000.0;
    }
    for tech in TechId::ALL {
        if !state::tech(&global.tech.access(), tech).unlocked
            && !upgrades::unlock(spring, global, tech)
        {
            test.fail(spring, "could not unlock integration-test technology");
        }
    }
    for _ in 0..2 {
        if !upgrades::upgrade(spring, global, TechId::PulseLaser)
            || !upgrades::upgrade(spring, global, TechId::AntiMissilePointDefense)
        {
            test.fail(spring, "could not upgrade integration-test technology");
        }
    }
}

fn verify_upgrades(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    test: &mut TestState,
    global: &Global,
) {
    let Ok(map) = spring.game().get_game_map_info() else {
        test.fail(spring, "could not read map-damage state from the engine");
        return;
    };
    let width = map.map_size_x as f32;
    let height = map.map_size_z as f32;
    if definitions::clamp_xz(spring, width + 800.0, -500.0) != Some((width - 1.0, 1.0))
        || definitions::clamp_xz(spring, -500.0, height + 800.0) != Some((1.0, height - 1.0))
    {
        test.fail(
            spring,
            "movement destinations did not clamp to the engine map dimensions",
        );
        return;
    }
    if !map.map_damage {
        test.fail(
            spring,
            "the engine started Area 17 with map damage disabled",
        );
        return;
    }
    let Ok(god_gun) = spring
        .units_weapons()
        .get_unit_weapon_damages(global.ufo.access().id, 9)
    else {
        test.fail(
            spring,
            "could not read God Gun damage settings from the engine",
        );
        return;
    };
    if (god_gun.crater_boost - 2.5).abs() > f32::EPSILON
        || (god_gun.crater_mult - 5.0).abs() > f32::EPSILON
    {
        test.fail(
            spring,
            "God Gun crater settings do not match the Lua weapon definition",
        );
        return;
    }
    if !weapon_multishot::apply(spring, global) {
        test.fail(
            spring,
            "the engine rejected one or more upgraded Pulse Laser or Point Defense slots",
        );
        return;
    }
    test.pass(spring,
        "map damage is enabled, God Gun crater strength is loaded, and the engine accepts every upgraded weapon slot",
    );

    let before = global.economy.access().biomass;
    let upgraded = upgrades::upgrade(spring, global, TechId::IndependenceDayGun);
    let cost = before - global.economy.access().biomass;
    let message = alloc::format!("upgrading the God Gun cost {cost:.0} biomass");
    if !upgraded || cost != 37_500.0 {
        test.fail(spring, message);
        return;
    }
    test.pass(spring, message);
}
