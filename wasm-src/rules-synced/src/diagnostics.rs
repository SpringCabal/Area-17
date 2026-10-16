#[cfg(feature = "diagnostics")]
use alloc::format;

pub const SECTION: &str = "area17-wasm";

#[cfg(feature = "diagnostics")]
#[derive(Clone, Copy)]
pub struct FrameSnapshot {
    pub ufo_id: i32,
    pub position: spring::Float3,
    pub aim: spring::Float3,
    pub health: f32,
    pub max_health: f32,
}

pub fn info(message: &str) {
    #[cfg(feature = "diagnostics")]
    spring::log_info!(SECTION, "{message}");
    #[cfg(not(feature = "diagnostics"))]
    let _ = message;
}

pub fn debug(message: &str) {
    #[cfg(feature = "diagnostics")]
    spring::log_debug!(SECTION, "{message}");
    #[cfg(not(feature = "diagnostics"))]
    let _ = message;
}

#[cfg(feature = "diagnostics")]
pub fn frame(frame: i32, snapshot: FrameSnapshot) {
    if frame.rem_euclid(300) == 0 {
        info(&format!(
            "ufo frame={frame} id={} pos=({:.1},{:.1},{:.1}) aim=({:.1},{:.1},{:.1}) health={:.1}/{:.1}",
            snapshot.ufo_id,
            snapshot.position.x,
            snapshot.position.y,
            snapshot.position.z,
            snapshot.aim.x,
            snapshot.aim.y,
            snapshot.aim.z,
            snapshot.health,
            snapshot.max_health,
        ));
    }
}
