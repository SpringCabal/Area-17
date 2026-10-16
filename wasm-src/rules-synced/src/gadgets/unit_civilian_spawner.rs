use alloc::rc::Rc;
use core::cell::RefCell;

use spring_addons::rules::{Gadget, UnitDestroyedEvent};

use crate::civilians::{self, Civilians};
use crate::state::Global;

#[derive(Default)]
pub struct UnitCivilianSpawnerGadget {
    civilians: Rc<RefCell<Civilians>>,
}

impl UnitCivilianSpawnerGadget {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Gadget<Global> for UnitCivilianSpawnerGadget {
    fn name(&self) -> &'static str {
        "UnitCivilianSpawner"
    }

    fn game_frame(&self, ctx: &spring_addons::AddonContext<'_, Global>, frame: i32) {
        civilians::update(ctx.global(), &self.civilians, frame);
        #[cfg(feature = "diagnostics")]
        if frame.rem_euclid(300) == 0 {
            let civilians = self.civilians.borrow();
            crate::diagnostics::info(&alloc::format!(
                "civilians frame={frame} spawners={} wanderers={}",
                civilians.spawners.len(),
                civilians.wanderers.len(),
            ));
        }
    }

    fn unit_created(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global>,
        unit: i32,
        def: i32,
        _team: i32,
        _builder: i32,
    ) {
        civilians::unit_created(&mut self.civilians.borrow_mut(), unit, def);
    }

    fn unit_destroyed(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global>,
        event: &UnitDestroyedEvent,
    ) {
        civilians::unit_destroyed(&mut self.civilians.borrow_mut(), event.unit_id);
    }

    fn unit_idle(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global>,
        unit: i32,
        _def: i32,
        _team: i32,
    ) {
        civilians::unit_idle(&self.civilians, unit);
    }
}
