use spring_addons::rules::Gadget;

use crate::definitions;
use crate::diagnostics;
use crate::state::Global;
use spring::Float3;

#[derive(Default)]
pub struct DbgDiagnosticsGadget;

impl DbgDiagnosticsGadget {
    pub fn new() -> Self {
        Self
    }
}

impl Gadget<Global> for DbgDiagnosticsGadget {
    fn name(&self) -> &'static str {
        "DbgDiagnostics"
    }

    fn game_frame(&self, ctx: &spring_addons::AddonContext<'_, Global>, frame: i32) {
        let (ufo_id, aim) = {
            let ufo = ctx.global().ufo.access();
            (ufo.id, ufo.aim)
        };
        let (health, max_health) = definitions::unit_health(ufo_id).unwrap_or_default();
        diagnostics::frame(
            frame,
            diagnostics::FrameSnapshot {
                ufo_id,
                position: ufo_position(ufo_id),
                aim,
                health,
                max_health,
            },
        );
    }
}

fn ufo_position(ufo_id: i32) -> Float3 {
    (ufo_id >= 0)
        .then(|| definitions::unit_position(ufo_id))
        .flatten()
        .unwrap_or_default()
}
