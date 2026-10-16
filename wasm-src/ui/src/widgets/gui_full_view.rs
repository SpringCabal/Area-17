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

    fn init(&self, _ctx: &spring_addons::AddonContext<'_, Global>) {
        spring::selection::select_unit_array(&[], false)
            .log_err_once("Area 17 clear RTS selection");
        for unit in spring::get_all_units().unwrap_or_default() {
            spring::unsynced_ctrl::set_unit_no_select(unit, true)
                .log_err_once("Area 17 disable RTS selection");
        }
    }

    fn unit_created(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global>,
        unit: i32,
        _def: i32,
        _team: i32,
        _builder: i32,
    ) {
        spring::unsynced_ctrl::set_unit_no_select(unit, true)
            .log_err_once("Area 17 disable RTS selection");
    }

    fn update(&self, _ctx: &spring_addons::AddonContext<'_, Global>, _dt: f32) {
        if self.done.get() {
            return;
        }
        let cheating = spring::game::is_cheating_enabled(0).unwrap_or(false);
        if !cheating {
            let _ = spring::send_commands("cheat", "");
        }
        let full_view = spring::send_commands("globallos", "");
        if !cheating {
            let _ = spring::send_commands("cheat", "");
        }
        self.done.set(full_view.is_ok());
    }
}
