use core::cell::Cell;

use spring_addons::rules::Gadget;

use crate::abduction;
use crate::definitions;
use crate::state::Global;

#[derive(Default)]
pub struct UnitUfoAbductionGadget {
    gravity_per_frame: Cell<Option<f32>>,
}

impl UnitUfoAbductionGadget {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Gadget<Global> for UnitUfoAbductionGadget {
    fn name(&self) -> &'static str {
        "UnitUfoAbduction"
    }

    fn game_frame(&self, ctx: &spring_addons::AddonContext<'_, Global>, _frame: i32) {
        let spring = ctx.spring();
        let gravity = self.gravity_per_frame.get().unwrap_or_else(|| {
            let gravity = definitions::gravity_per_frame(spring);
            self.gravity_per_frame.set(Some(gravity));
            gravity
        });
        abduction::update(spring, ctx.global(), gravity);
    }
}
