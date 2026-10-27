use core::cell::Cell;
use spring::{Float3, ResultExt};
use spring_addons::UnitDestroyedEvent;
use spring_addons::ui::Widget;
use springcabal_control::camera::{Aim, CameraFollow, FocusY, Follow, Smoothing, View};

use crate::state::Global;

const VIEW: View = View::overhead(2_000.0, 0.4).pitch(-0.921_061, -0.389_418);
const START: Float3 = Float3::new(14_000.0, 25.0, 1_600.0);

const FOLLOW: Follow = Follow::state(VIEW, 0.4)
    .start(VIEW, START)
    .aim(Aim::ViewPosition)
    .lead(12.0)
    .smoothing(Smoothing::AverageMove { updates: 12 })
    .focus_y(FocusY::Offset(25.0))
    .snap_updates(1);

pub struct GuiCameraWidget {
    camera: CameraFollow,
    ufo_def: Cell<Option<i32>>,
    controlled: Cell<Option<i32>>,
}

impl Default for GuiCameraWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl GuiCameraWidget {
    pub fn new() -> Self {
        Self {
            camera: CameraFollow::new(FOLLOW),
            ufo_def: Cell::new(None),
            controlled: Cell::new(None),
        }
    }
}

impl Widget<Global> for GuiCameraWidget {
    fn name(&self) -> &'static str {
        "GuiCamera"
    }

    fn init(&self, ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>) {
        let spring = ctx.spring();
        self.ufo_def.set(
            spring
                .unit_defs()
                .get_unit_def_id_by_name("ufo")
                .log_err_once(spring, "resolving the camera's UFO definition"),
        );
        for unit in spring.units_query().get_all_units().unwrap_or_default() {
            if spring.units_info().get_unit_def_id(unit).ok() == self.ufo_def.get() {
                self.controlled.set(Some(unit));
            }
        }
    }

    fn unit_created(
        &self,
        ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>,
        unit: i32,
        def: i32,
        _team: i32,
        _builder: i32,
    ) {
        if self.ufo_def.get() == Some(def) {
            self.controlled.set(Some(unit));
            // Lua starts the follow history in UnitCreated, before the first
            // status packet or UI update can advance the UFO's drawn position.
            self.camera.update(ctx.spring(), Some(unit), 0.0);
        }
    }

    fn unit_destroyed(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>,
        event: &UnitDestroyedEvent,
    ) {
        if self.controlled.get() == Some(event.unit_id) {
            self.controlled.set(None);
        }
    }

    fn update(&self, ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>, dt: f32) {
        self.camera.update(ctx.spring(), self.controlled.get(), dt);
    }
}
