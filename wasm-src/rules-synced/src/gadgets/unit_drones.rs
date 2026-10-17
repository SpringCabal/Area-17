use alloc::rc::Rc;
use core::cell::RefCell;

use spring_addons::rules::{Gadget, UnitDestroyedEvent};

use crate::drones::{self, Drones};
use crate::state::Global;

#[derive(Default)]
pub struct UnitDronesGadget {
    drones: Rc<RefCell<Drones>>,
}

impl UnitDronesGadget {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Gadget<Global> for UnitDronesGadget {
    fn name(&self) -> &'static str {
        "UnitDrones"
    }

    fn game_frame(&self, ctx: &spring_addons::AddonContext<'_, Global>, frame: i32) {
        let spring = ctx.spring();
        drones::update(spring, ctx.global(), &self.drones, frame);
        #[cfg(feature = "diagnostics")]
        if frame.rem_euclid(300) == 0 {
            crate::diagnostics::info(
                spring,
                &alloc::format!(
                    "drones frame={frame} count={}",
                    self.drones.borrow().drones.len(),
                ),
            );
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
        let spring = _ctx.spring();
        drones::unit_created(spring, &mut self.drones.borrow_mut(), unit, def);
    }

    fn unit_destroyed(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global>,
        event: &UnitDestroyedEvent,
    ) {
        drones::unit_destroyed(&mut self.drones.borrow_mut(), event.unit_id);
    }
}
