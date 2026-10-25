use core::cell::Cell;

use spring_addons::rules::Gadget;

use crate::scenario;
use crate::state::Global;

#[derive(Default)]
pub struct GameLoadGadget {
    loaded: Cell<bool>,
}

impl GameLoadGadget {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Gadget<Global> for GameLoadGadget {
    fn name(&self) -> &'static str {
        "GameLoad"
    }

    fn game_frame(&self, ctx: &spring_addons::AddonContext<'_, Global>, frame: i32) {
        let spring = ctx.spring();
        // Addon init now runs while the module is loading, before its engine
        // event subscription is installed. Populate once it can receive every
        // UnitCreated event, including script attachment and unit setup.
        if !self.loaded.replace(true) {
            scenario::load(spring);
        }
        scenario::respawn(spring, ctx.global(), frame);
    }
}
