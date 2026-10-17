#[cfg(feature = "diagnostics")]
use alloc::format;

pub const SECTION: &str = "area17-wasm-ui";

pub fn info(_spring: &spring::Spring<'_, spring::Ui>, message: &str) {
    #[cfg(feature = "diagnostics")]
    spring::log_info!(_spring, SECTION, "{message}");
    #[cfg(not(feature = "diagnostics"))]
    let _ = message;
}

pub fn debug(_spring: &spring::Spring<'_, spring::Ui>, message: &str) {
    #[cfg(feature = "diagnostics")]
    spring::log_debug!(_spring, SECTION, "{message}");
    #[cfg(not(feature = "diagnostics"))]
    let _ = message;
}

pub fn mouse_target(
    _spring: &spring::Spring<'_, spring::Ui>,
    x: i32,
    y: i32,
    target: spring::Float3,
) {
    #[cfg(feature = "diagnostics")]
    debug(
        _spring,
        &format!(
            "aim frame={} screen=({x},{y}) world=({:.1},{:.1},{:.1})",
            _spring.game().current_frame().unwrap_or(0),
            target.x,
            target.y,
            target.z
        ),
    );
    #[cfg(not(feature = "diagnostics"))]
    let _ = (x, y, target);
}

pub fn input(_spring: &spring::Spring<'_, spring::Ui>, message: &str) {
    #[cfg(feature = "diagnostics")]
    debug(
        _spring,
        &format!(
            "input frame={} {message}",
            _spring.game().current_frame().unwrap_or(0)
        ),
    );
    #[cfg(not(feature = "diagnostics"))]
    let _ = message;
}

pub fn status(
    _spring: &spring::Spring<'_, spring::Ui>,
    ufo_id: i32,
    health: f32,
    max_health: f32,
    biomass: f32,
    game_over: bool,
    game_won: bool,
) {
    #[cfg(feature = "diagnostics")]
    debug(
        _spring,
        &format!(
            "status frame={} ufo={ufo_id} health={health:.1}/{max_health:.1} biomass={biomass:.0} game_over={game_over} game_won={game_won}",
            _spring.game().current_frame().unwrap_or(0)
        ),
    );
    #[cfg(not(feature = "diagnostics"))]
    let _ = (ufo_id, health, max_health, biomass, game_over, game_won);
}
