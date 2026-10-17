use spring::ResultExt;
use spring_addons::rules::{Gadget, UnitDestroyedEvent};

use crate::definitions;
use crate::defs::{self, UnitKind};
use crate::state::Global;

fn is_explosive(spring: &spring::Spring<'_, spring::RulesSynced>, unit_def_id: i32) -> bool {
    matches!(
        defs::kind(spring, unit_def_id),
        Some(
            UnitKind::Building1
                | UnitKind::Building2
                | UnitKind::Building3
                | UnitKind::Building4
                | UnitKind::Building4BigTank
                | UnitKind::StaticAa
                | UnitKind::StrongAa
                | UnitKind::Tank
                | UnitKind::SmallTank
        )
    )
}

#[derive(Default)]
pub struct UnitBuildingsGadget;

impl UnitBuildingsGadget {
    pub fn new() -> Self {
        Self
    }
}

impl Gadget<Global> for UnitBuildingsGadget {
    fn name(&self) -> &'static str {
        "UnitBuildings"
    }

    fn unit_destroyed(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global>,
        event: &UnitDestroyedEvent,
    ) {
        let spring = _ctx.spring();
        if !is_explosive(spring, event.unit_def_id) {
            return;
        }
        let Some(position) = definitions::unit_position(spring, event.unit_id) else {
            return;
        };
        spring
            .sound()
            .play_sound_file(
                "sounds/explosion.wav",
                2.0,
                position,
                spring::Float3::default(),
                0,
            )
            .log_err_once(spring, "Area 17 building explosion: play sound file");
    }
}
