use spring_addons::rules::Gadget;

use crate::state::Global;
use crate::upgrades;

#[derive(Default)]
pub struct UpgradesGadget;

impl UpgradesGadget {
    pub fn new() -> Self {
        Self
    }
}

impl Gadget<Global> for UpgradesGadget {
    fn name(&self) -> &'static str {
        "Upgrades"
    }

    fn init(&self, _ctx: &spring_addons::AddonContext<'_, Global>) {
        spring::commands::run(|| crate::with_global(upgrades::initialize));
    }

    fn game_frame(&self, ctx: &spring_addons::AddonContext<'_, Global>, _frame: i32) {
        upgrades::update(ctx.global());
    }
}
