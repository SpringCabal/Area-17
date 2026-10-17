use spring::ResultExt;
use spring_addons::rules::Gadget;

use crate::cus_scripts;
use crate::state::Global;

#[derive(Default)]
pub struct UnitScriptsGadget;

impl UnitScriptsGadget {
    pub fn new() -> Self {
        Self
    }
}

impl Gadget<Global> for UnitScriptsGadget {
    fn name(&self) -> &'static str {
        "UnitScripts"
    }

    fn init(&self, _ctx: &spring_addons::AddonContext<'_, Global>) {
        let _spring = _ctx.spring();
        crate::with_cus_module(|host, spring| host.attach_existing(spring))
            .log_err_once(_ctx.spring(), "attaching existing Area 17 unit scripts");
    }

    fn game_frame(&self, _ctx: &spring_addons::AddonContext<'_, Global>, frame: i32) {
        crate::with_cus_module(move |host, _| {
            host.game_frame(frame);
            Ok(())
        })
        .log_err_once(_ctx.spring(), "running Area 17 unit scripts");
    }

    fn unit_created(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global>,
        unit: i32,
        def: i32,
        _team: i32,
        _builder: i32,
    ) {
        cus_scripts::attach_script(_ctx.spring(), unit, def);
    }
}
