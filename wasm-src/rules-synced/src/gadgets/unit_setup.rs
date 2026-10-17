use spring::ResultExt;
use spring_addons::rules::Gadget;

use crate::definitions;
use crate::defs::{self, UnitKind};
use crate::state::Global;

#[derive(Default)]
pub struct UnitSetupGadget;

impl UnitSetupGadget {
    pub fn new() -> Self {
        Self
    }
}

impl Gadget<Global> for UnitSetupGadget {
    fn name(&self) -> &'static str {
        "UnitSetup"
    }

    fn unit_created(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global>,
        unit: i32,
        def: i32,
        _team: i32,
        _builder: i32,
    ) {
        let spring = _ctx.spring();
        definitions::make_visible(spring, unit);
        definitions::configure_unit_geometry(spring, unit, def);
        match defs::kind(spring, def) {
            Some(UnitKind::Ufo | UnitKind::Drone) => {
                definitions::set_gunship_attributes(spring, unit, 1.0, 1.0);
            }
            Some(
                UnitKind::Tank
                | UnitKind::SmallTank
                | UnitKind::Civilian
                | UnitKind::Civilian1
                | UnitKind::Scientist
                | UnitKind::Scientist2
                | UnitKind::Alien
                | UnitKind::Endcube,
            ) => {
                definitions::set_move_speed_multiplier(spring, unit, 1.0);
            }
            _ => {}
        }
    }

    fn projectile_created(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global>,
        projectile_id: i32,
        owner_id: i32,
        _weapon_def_id: i32,
    ) {
        let spring = _ctx.spring();
        if defs::unit_kind(spring, owner_id).is_some_and(UnitKind::is_static_turret) {
            spring
                .projectile_control()
                .set_projectile_always_visible(projectile_id, true)
                .log_err_once(spring, "Area 17 unit setup: set projectile always visible");
        }
    }
}
