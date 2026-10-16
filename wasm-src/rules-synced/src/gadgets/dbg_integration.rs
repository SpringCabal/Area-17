use spring_addons::rules::Gadget;

use crate::integration::{self, with_test};
use crate::state::Global;
use spring::Float3;

#[derive(Default)]
pub struct DbgIntegrationGadget;

impl DbgIntegrationGadget {
    pub fn new() -> Self {
        Self
    }
}

impl Gadget<Global> for DbgIntegrationGadget {
    fn name(&self) -> &'static str {
        "DbgIntegration"
    }

    fn game_frame(&self, ctx: &spring_addons::AddonContext<'_, Global>, frame: i32) {
        with_test(|test| integration::update(test, ctx.global(), frame));
    }

    fn projectile_created(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global>,
        _projectile_id: i32,
        owner_id: i32,
        weapon_def_id: i32,
    ) {
        with_test(|test| test.projectile_created(owner_id, weapon_def_id));
    }

    fn unit_created(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global>,
        unit: i32,
        def: i32,
        _team: i32,
        _builder: i32,
    ) {
        with_test(|test| test.unit_created(unit, def));
    }

    fn unit_pre_damaged(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global>,
        event: &spring_addons::rules::UnitPreDamagedEvent,
    ) -> Option<spring::DamageResult> {
        with_test(|test| test.pre_damaged(event.unit_id, event.weapon_def_id));
        None
    }

    fn explosion(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global>,
        weapon_def_id: i32,
        pos: (f32, f32, f32),
        _owner_id: i32,
        _projectile_id: i32,
    ) -> bool {
        with_test(|test| {
            test.explosion(
                weapon_def_id,
                Float3 {
                    x: pos.0,
                    y: pos.1,
                    z: pos.2,
                },
            );
        });
        false
    }
}
