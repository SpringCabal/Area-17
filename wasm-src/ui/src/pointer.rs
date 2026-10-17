use crate::state::Global;
use spring::Float3;

pub fn ui_owns_pointer(
    spring: &spring::Spring<'_, spring::Ui>,
    global: &Global,
    x: i32,
    y: i32,
) -> bool {
    let context = global.hud.access().context;
    if context == 0 {
        return false;
    }
    process_mouse_move(spring, global, context, x, y);
    spring
        .rml_ui()
        .context_is_mouse_interacting(context)
        .unwrap_or(false)
}

pub fn process_mouse_move(
    spring: &spring::Spring<'_, spring::Ui>,
    global: &Global,
    context: u64,
    x: i32,
    y: i32,
) {
    if context != 0 {
        let rml_y = global.view.access().height - y - 1;
        let _ = spring
            .rml_ui()
            .context_process_mouse_move(context, x as f32, rml_y as f32, 0);
    }
}

pub fn mouse_state(spring: &spring::Spring<'_, spring::Ui>) -> Option<(i32, i32, bool)> {
    let mouse = spring.input().get_mouse_state().ok()?;
    if mouse.offscreen || !mouse.x.is_finite() || !mouse.y.is_finite() {
        return None;
    }
    Some((mouse.x.round() as i32, mouse.y.round() as i32, mouse.left))
}

pub fn ground_at(
    spring: &spring::Spring<'_, spring::Ui>,
    screen_x: i32,
    screen_y: i32,
) -> Option<Float3> {
    spring
        .camera()
        .ground_at(screen_x as f32, screen_y as f32)
        .ok()
        .flatten()
}
