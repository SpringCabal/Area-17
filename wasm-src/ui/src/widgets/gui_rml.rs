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

    fn init(&self, ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>) {
        let spring = ctx.spring();
        rml::initialize(spring, ctx.global());
    }

    fn update(&self, ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>, dt: f32) {
        let spring = ctx.spring();
        let global = ctx.global();
        rml::initialize(spring, global);
        let dt = dt.max(0.0);
        let tech_open = global.panel.access().upgrade_panel_open;
        rml::update_events(
            spring,
            &mut global.hud.access_mut(),
            &global.simulation.access(),
            tech_open,
            dt,
        );
        rml::update_context(spring, global);
    }

    fn draw_screen(
        &self,
        ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>,
        width: i32,
        height: i32,
    ) {
        let spring = ctx.spring();
        rml::render(spring, ctx.global(), width, height);
    }
}
