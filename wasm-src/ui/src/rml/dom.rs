use alloc::format;
use alloc::string::String;

use super::state::Hud;

pub fn element(spring: &spring::Spring<'_, spring::Ui>, hud: &Hud, id: &str) -> u64 {
    if hud.document == 0 {
        return 0;
    }
    spring
        .rml_ui()
        .element_get_element_by_id(hud.document, id)
        .ok()
        .filter(|element| element.exists)
        .map_or(0, |element| element.element_handle)
}

pub fn health_color(percent: f32) -> String {
    let t = (percent / 100.0).clamp(0.0, 1.0);
    let (red, green, blue) = if t < 0.5 {
        let local = t * 2.0;
        (1.0 - 0.2 * local, 0.6 * local, 0.0)
    } else {
        let local = (t - 0.5) * 2.0;
        (0.8 * (1.0 - local), 0.6 + 0.15 * local, 0.0)
    };
    rgb(red, green, blue)
}

pub fn shield_color(percent: f32) -> String {
    let t = (percent / 100.0).clamp(0.0, 1.0);
    if t < 0.5 {
        let local = t * 2.0;
        rgb(1.0 - 0.8 * local, 0.0, 0.8 * local)
    } else {
        let local = (t - 0.5) * 2.0;
        rgb(0.2 * (1.0 - local), 0.0, 0.8 + 0.2 * local)
    }
}

pub fn rgb(red: f32, green: f32, blue: f32) -> String {
    format!(
        "rgb({},{},{})",
        (red.clamp(0.0, 1.0) * 255.0).round() as i32,
        (green.clamp(0.0, 1.0) * 255.0).round() as i32,
        (blue.clamp(0.0, 1.0) * 255.0).round() as i32,
    )
}
