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
        scenario::load();
    }

    fn game_frame(&self, ctx: &spring_addons::AddonContext<'_, Global>, frame: i32) {
        scenario::respawn(ctx.global(), frame);
    }
}
