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

pub fn setup(handler: &mut WidgetHandler<Global>) {
    register_widgets!(handler, [springcabal::controls::GameControls::new()]);
    if springcabal::verification::world_only() {
        gallery::setup(handler);
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
