use core::cell::Cell;
use spring::ResultExt;

use spring_addons::ui::Widget;

use crate::state::Global;

#[derive(Default)]
pub struct GuiFullViewWidget {
    done: Cell<bool>,
}

impl GuiFullViewWidget {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Widget<Global> for GuiFullViewWidget {
    fn name(&self) -> &'static str {
        "GuiFullView"
    }

    fn init(&self, _ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>) {
        let spring = _ctx.spring();
        spring
            .selection()
            .select_unit_array(&[], false)
            .log_err_once(spring, "Area 17 clear RTS selection");
        for unit in spring.units_query().get_all_units().unwrap_or_default() {
            spring
                .unsynced_ctrl()
                .set_unit_no_select(unit, true)
                .log_err_once(spring, "Area 17 disable RTS selection");
        }
    }

    fn unit_created(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>,
        unit: i32,
        _def: i32,
        _team: i32,
        _builder: i32,
    ) {
        let spring = _ctx.spring();
        spring
            .unsynced_ctrl()
            .set_unit_no_select(unit, true)
            .log_err_once(spring, "Area 17 disable RTS selection");
    }

    fn update(&self, _ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>, _dt: f32) {
        let spring = _ctx.spring();
        if self.done.get() {
            return;
        }
        let cheating = spring.game().is_cheating_enabled().unwrap_or(false);
        if !cheating {
            let _ = spring.messages().send_commands("cheat", "");
        }
        let full_view = spring.messages().send_commands("globallos", "");
        if !cheating {
            let _ = spring.messages().send_commands("cheat", "");
        }
        self.done.set(full_view.is_ok());
    }
}
