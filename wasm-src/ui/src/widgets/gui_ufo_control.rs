use area17_shared::message as msg;
use spring::ResultExt;
use spring_addons::event::{EventResult, KeyEvent};
use spring_addons::ui::Widget;
use springcabal::menu;
use springcabal::message::{self, Encoder, send_to_rules};
use springcabal_control::keyboard::KeyMovement;
use springcabal_control::protocol;

use crate::catalog::{ABILITIES, WEAPONS};
use crate::commands;
use crate::pointer::{ground_at, mouse_state, process_mouse_move, ui_owns_pointer};
use crate::state::Global;
use crate::{audio, diagnostics};
use spring::Float3;

pub struct GuiUfoControlWidget {
    movement: KeyMovement,
    last_target: core::cell::Cell<Option<Float3>>,
    fire_sent: core::cell::Cell<Option<Option<Float3>>>,
    left_mouse_down: core::cell::Cell<bool>,
    rml_pointer_active: core::cell::Cell<bool>,
}

impl Default for GuiUfoControlWidget {
    fn default() -> Self {
        Self {
            movement: KeyMovement::new(),
            last_target: core::cell::Cell::new(None),
            fire_sent: core::cell::Cell::new(None),
            left_mouse_down: core::cell::Cell::new(false),
            rml_pointer_active: core::cell::Cell::new(false),
        }
    }
}

impl GuiUfoControlWidget {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Widget<Global> for GuiUfoControlWidget {
    fn name(&self) -> &'static str {
        "GuiUfoControl"
    }

    fn init(&self, _ctx: &spring_addons::AddonContext<'_, Global>) {
        hide_existing_drones();
    }

    fn recv_from_synced(&self, _ctx: &spring_addons::AddonContext<'_, Global>, message: &[u8]) {
        let Some(unit_id) = core::str::from_utf8(message)
            .ok()
            .and_then(|message| message::fields(message, msg::DRONE))
            .and_then(|mut fields| fields.i32())
        else {
            return;
        };
        spring::unsynced_ctrl::set_unit_no_select(unit_id, true)
            .log_err_once("Area 17 drone selection: set unit no select");
    }

    fn update(&self, ctx: &spring_addons::AddonContext<'_, Global>, _dt: f32) {
        let global = ctx.global();
        self.update_movement();
        let Some((x, y, left)) = mouse_state() else {
            self.update_fire(global, false);
            return;
        };
        let was_left_mouse_down = self.left_mouse_down.get();
        let rml_pointer_active = self.rml_pointer_active.get();
        self.left_mouse_down.set(left);

        let context = global.hud.access().context;
        process_mouse_move(global, context, x, y);
        self.update_aim(x, y);
        if left && !rml_pointer_active && !was_left_mouse_down {
            diagnostics::input("mouse fire start");
        }
        self.update_fire(global, left && !rml_pointer_active);
    }

    fn mouse_move(
        &self,
        ctx: &spring_addons::AddonContext<'_, Global>,
        x: i32,
        y: i32,
        _dx: i32,
        _dy: i32,
        _button: i32,
    ) -> EventResult {
        ui_owns_pointer(ctx.global(), x, y).into()
    }

    fn mouse_press(
        &self,
        ctx: &spring_addons::AddonContext<'_, Global>,
        x: i32,
        y: i32,
        button: i32,
    ) -> EventResult {
        let global = ctx.global();
        let context = global.hud.access().context;
        process_mouse_move(global, context, x, y);
        if context != 0 {
            let _ = spring::context_process_mouse_button_down(context, button - 1, 0);
        }
        if button != spring::mouse::LEFT {
            return ui_owns_pointer(global, x, y).into();
        }
        if ui_owns_pointer(global, x, y) {
            self.rml_pointer_active.set(true);
            return EventResult::Handled;
        }
        self.left_mouse_down.set(true);
        self.update_aim(x, y);
        diagnostics::input("mouse fire press");
        self.update_fire(global, true);
        EventResult::Handled
    }

    fn mouse_release(
        &self,
        ctx: &spring_addons::AddonContext<'_, Global>,
        x: i32,
        y: i32,
        button: i32,
    ) -> EventResult {
        let global = ctx.global();
        let context = global.hud.access().context;
        process_mouse_move(global, context, x, y);
        if context != 0 {
            let _ = spring::context_process_mouse_button_up(context, button - 1, 0);
        }
        if button == spring::mouse::LEFT {
            self.left_mouse_down.set(false);
            self.rml_pointer_active.set(false);
        }
        EventResult::Ignored
    }

    fn key_press(
        &self,
        ctx: &spring_addons::AddonContext<'_, Global>,
        event: &KeyEvent<'_>,
    ) -> EventResult {
        let global = ctx.global();

        if menu::is_quit_to_menu(event) {
            menu::quit_to_menu();
            return EventResult::Handled;
        }

        if self.movement.key(event, true) {
            return EventResult::Handled;
        }
        if event.matches("t") {
            if event.is_repeat {
                return EventResult::Handled;
            }
            let mut panel = global.panel.access_mut();
            panel.upgrade_panel_open = !panel.upgrade_panel_open;
            audio::play_ui("sounds/click.wav");
            return EventResult::Handled;
        }
        if event.matches("escape") {
            if global.panel.access().upgrade_panel_open {
                if event.is_repeat {
                    return EventResult::Handled;
                }
                global.panel.access_mut().upgrade_panel_open = false;
                return EventResult::Handled;
            }
            return EventResult::Ignored;
        }
        if let Some(ability) = ABILITIES.iter().find(|ability| event.matches(ability.key)) {
            if event.is_repeat {
                return EventResult::Handled;
            }
            send_to_rules(&Encoder::new(msg::ABILITY).field(ability.name()).finish());
            diagnostics::input(ability.name());
            return EventResult::Handled;
        }
        if let Some(index) = weapon_key(event) {
            if event.is_repeat {
                return EventResult::Handled;
            }
            commands::select_weapon(global, index);
            return EventResult::Handled;
        }
        EventResult::Ignored
    }

    fn key_release(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global>,
        event: &KeyEvent<'_>,
    ) -> EventResult {
        self.movement.key(event, false).into()
    }
}

impl GuiUfoControlWidget {
    fn update_aim(&self, x: i32, y: i32) {
        let Some(target) = ground_at(x, y) else {
            return;
        };
        let changed = self.last_target.get().is_none_or(|old| {
            (old.x - target.x).abs() > 0.5
                || (old.y - target.y).abs() > 0.5
                || (old.z - target.z).abs() > 0.5
        });
        if !changed {
            return;
        }
        self.last_target.set(Some(target));
        diagnostics::mouse_target(x, y, target);
        send_to_rules(&commands::with_point(Encoder::new(msg::AIM), target));
    }

    fn update_fire(&self, global: &Global, held: bool) {
        let order = held
            .then(|| self.last_target.get())
            .flatten()
            .map(|target| commands::fire_order(global, target));
        if self.fire_sent.get() == order {
            return;
        }
        self.fire_sent.set(order);
        match order {
            Some(order) => commands::fire(order),
            None => commands::cease_fire(),
        }
    }

    fn update_movement(&self) {
        self.movement
            .send(|direction| send_to_rules(&protocol::encode_direction(direction)));
    }
}

fn weapon_key(event: &KeyEvent<'_>) -> Option<usize> {
    WEAPONS.iter().position(|weapon| event.matches(weapon.key))
}

fn hide_existing_drones() {
    let Some(drone_def_id) = spring::get_unit_def_id_by_name("drone").ok() else {
        return;
    };
    let Ok(units) = spring::get_all_units() else {
        return;
    };
    for unit_id in units {
        if spring::units_info::get_unit_def_id(unit_id).ok() == Some(drone_def_id) {
            spring::unsynced_ctrl::set_unit_no_select(unit_id, true)
                .log_err_once("Area 17 drone selection: set unit no select");
        }
    }
}
