use spring::Float3;
use spring_addons::ui::Widget;
use springcabal_control::camera::{Aim, CameraFollow, FocusY, Follow, Smoothing, View};

use crate::state::Global;

const VIEW: View = View::overhead(2_000.0, 0.4).pitch(-0.921_061, -0.389_418);
const START: Float3 = Float3::new(14_000.0, 25.0, 1_600.0);

const FOLLOW: Follow = Follow::state(VIEW, 0.4)
    .start(VIEW, START)
    .aim(Aim::ViewMidPosition)
    .lead(12.0)
    .smoothing(Smoothing::AverageMove { updates: 12 })
    .focus_y(FocusY::Offset(25.0))
    .snap_updates(0);

pub struct GuiCameraWidget {
    camera: CameraFollow,
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
        }
    }
}

impl Widget<Global> for GuiCameraWidget {
    fn name(&self) -> &'static str {
        "GuiCamera"
    }

    fn update(&self, ctx: &spring_addons::AddonContext<'_, Global>, dt: f32) {
        let ufo_id = ctx.global().simulation.access().ufo_id;
        self.camera.update((ufo_id >= 0).then_some(ufo_id), dt);
    }
}
