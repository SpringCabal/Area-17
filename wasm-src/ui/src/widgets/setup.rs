use spring::default_interface::{
    CLOCK, CONSOLE, MINIMAP, PLAYER_INFO, RESOURCE_BAR, TOOLTIP, UNIT_ICONS,
};
use spring_addons::register_widgets;
use spring_addons::ui::{HideDefaultInterface, WidgetHandler};
use springcabal::music::MusicLoop;

use super::*;
use crate::state::Global;

const MUSIC: &str = "sounds/music.ogg";
const MUSIC_VOLUME: f32 = 0.7;

pub const HIDDEN_INTERFACE: u32 =
    CONSOLE | RESOURCE_BAR | TOOLTIP | CLOCK | PLAYER_INFO | MINIMAP | UNIT_ICONS;

pub fn setup(handler: &mut WidgetHandler<Global>, spring: &spring::Spring<'_, spring::Ui>) {
    #[cfg(feature = "diagnostics")]
    {
        use spring::ResultExt;
        // Diagnostic builds must enable the sections their verification markers
        // use; the engine's default NOTICE filter otherwise hides INFO lines.
        for section in [crate::diagnostics::SECTION, "area17-wasm"] {
            spring
                .config()
                .set_log_section_filter_level(section, spring::log_level::INFO)
                .log_err_once(spring, "enabling Area 17 diagnostics");
        }
    }

    register_widgets!(handler, [springcabal::controls::GameControls::new()]);
    if springcabal::verification::world_only(spring) {
        gallery::setup(spring, handler);
        return;
    }
    register_widgets!(
        handler,
        [
            ApiSyncWidget::new(),
            HideDefaultInterface::new(HIDDEN_INTERFACE & !CONSOLE),
            GuiFullViewWidget::new(),
            GuiCameraWidget::new(),
            GuiUfoControlWidget::new(),
            MusicLoop::new(MUSIC, MUSIC_VOLUME),
            GfxVolumesWidget::new(),
            GuiRmlWidget::new(),
        ]
    );
    #[cfg(feature = "e2e")]
    register_widgets!(handler, [DbgIntegrationWidget::new()]);
}
