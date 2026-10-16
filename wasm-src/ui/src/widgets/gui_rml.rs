use spring_addons::ui::Widget;

use crate::rml;
use crate::state::Global;

#[derive(Default)]
pub struct GuiRmlWidget;

impl GuiRmlWidget {
    pub fn new() -> Self {
        Self
    }
}

impl Widget<Global> for GuiRmlWidget {
    fn name(&self) -> &'static str {
        "GuiRml"
    }

    fn init(&self, ctx: &spring_addons::AddonContext<'_, Global>) {
        rml::initialize(ctx.global());
    }

    fn update(&self, ctx: &spring_addons::AddonContext<'_, Global>, dt: f32) {
        let global = ctx.global();
        rml::initialize(global);
        let dt = dt.max(0.0);
        let tech_open = global.panel.access().upgrade_panel_open;
        rml::update_events(
            &mut global.hud.access_mut(),
            &global.simulation.access(),
            tech_open,
            dt,
        );
        rml::update_context(global);
    }

    fn draw_screen(&self, ctx: &spring_addons::AddonContext<'_, Global>, width: i32, height: i32) {
        rml::render(ctx.global(), width, height);
    }
}
