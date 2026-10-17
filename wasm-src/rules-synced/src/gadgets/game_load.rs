use spring_addons::rules::Gadget;

use crate::scenario;
use crate::state::Global;

#[derive(Default)]
pub struct GameLoadGadget;

impl GameLoadGadget {
    pub fn new() -> Self {
        Self
    }
}

impl Gadget<Global> for GameLoadGadget {
    fn name(&self) -> &'static str {
        "GameLoad"
    }

    fn init(&self, _ctx: &spring_addons::AddonContext<'_, Global>) {
        let spring = _ctx.spring();
        scenario::load(spring);
    }

    fn game_frame(&self, ctx: &spring_addons::AddonContext<'_, Global>, frame: i32) {
        let spring = ctx.spring();
        scenario::respawn(spring, ctx.global(), frame);
    }
}
