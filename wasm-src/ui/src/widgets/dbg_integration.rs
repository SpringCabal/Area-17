mod tech_panel;

use area17_shared::message as msg;
use core::cell::Cell;

use spring_addons::ui::Widget;
use springcabal::message::{Encoder, send_to_rules};

use crate::catalog::{ABILITIES, WEAPONS};
use crate::commands;
use crate::state::Global;
use spring::{Float3, ResultExt};

const TARGET: Float3 = Float3 {
    x: 14_702.2,
    y: 89.5,
    z: 1_550.9,
};

const TECH_TOGGLE_FRAME: i32 = 30;
const FIRST_WEAPON_FRAME: i32 = 45;
const WEAPON_STAGE_FRAMES: i32 = 30;
const ABILITY_FRAMES: [i32; 4] = [180, 165, 195, 210];

const SETTLE_UPDATES: u32 = 2;
const PAUSED_UPDATES: u32 = 10;

#[derive(Clone, Copy)]
enum Step {
    ToggleTech,
    SelectWeapon(usize),
    Fire(usize),
    CeaseFire,
    Ability(usize),
}

fn steps() -> alloc::vec::Vec<(i32, Step)> {
    let mut steps = alloc::vec![(TECH_TOGGLE_FRAME, Step::ToggleTech)];
    for index in 0..WEAPONS.len() {
        let select_frame = FIRST_WEAPON_FRAME + index as i32 * WEAPON_STAGE_FRAMES;
        let fire_frame = select_frame + WEAPON_STAGE_FRAMES / 2;
        steps.push((select_frame, Step::SelectWeapon(index)));
        steps.push((fire_frame, Step::Fire(index)));
        steps.push((fire_frame + WEAPON_STAGE_FRAMES / 2, Step::CeaseFire));
    }
    for (index, frame) in ABILITY_FRAMES.iter().enumerate() {
        steps.push((*frame, Step::Ability(index)));
    }
    steps.sort_by_key(|(frame, _)| *frame);
    steps
}

#[derive(Default)]
pub struct DbgIntegrationWidget {
    next: Cell<usize>,
    tech_fixture_frames: Cell<u32>,
    hud_verified: Cell<bool>,
    pauses_requested: Cell<bool>,
    paused_updates: Cell<u32>,
    resume_in: Cell<Option<u32>>,
}

impl DbgIntegrationWidget {
    pub fn new() -> Self {
        Self::default()
    }

    fn request_pauses(&self, spring: &spring::Spring<'_, spring::Ui>, steps: &[(i32, Step)]) {
        if self.pauses_requested.replace(true) {
            return;
        }
        let mut last = None;
        for (frame, _) in steps {
            if last != Some(*frame) {
                command(spring, &alloc::format!("pauseatframe {frame}"));
                last = Some(*frame);
            }
        }
    }

    fn resume_when_due(&self, spring: &spring::Spring<'_, spring::Ui>) -> bool {
        let Some(updates) = self.resume_in.get() else {
            return false;
        };
        if updates > 0 {
            self.resume_in.set(Some(updates - 1));
        } else {
            self.resume_in.set(None);
            self.paused_updates.set(0);
            command(spring, "pause 0");
        }
        true
    }

    fn settled_in_pause(&self, spring: &spring::Spring<'_, spring::Ui>) -> bool {
        if !paused(spring) {
            self.paused_updates.set(0);
            return false;
        }
        let updates = self.paused_updates.get() + 1;
        self.paused_updates.set(updates);
        updates > SETTLE_UPDATES
    }
}

fn run(spring: &spring::Spring<'_, spring::Ui>, global: &Global, step: Step) {
    match step {
        Step::ToggleTech => click_tech_toggle(spring, global),
        Step::SelectWeapon(index) => {
            if index == 0 {
                commands::grant_test_resources(spring);
            }
            global.simulation.access_mut().current_weapon = Some(index);
            send_to_rules(
                spring,
                &Encoder::new(msg::CHANGE_WEAPON)
                    .field(WEAPONS[index].name())
                    .finish(),
            );
        }
        Step::Fire(index) => {
            commands::fire(spring, (!WEAPONS[index].is_beam).then_some(TARGET));
            crate::diagnostics::input(spring, "mouse fire press");
        }
        Step::CeaseFire => commands::cease_fire(spring),
        Step::Ability(index) => {
            let name = ABILITIES[index].name();
            send_to_rules(spring, &Encoder::new(msg::ABILITY).field(name).finish());
            crate::diagnostics::input(spring, name);
        }
    }
}

fn aim_at_target(spring: &spring::Spring<'_, spring::Ui>) {
    send_to_rules(
        spring,
        &commands::with_point(Encoder::new(msg::AIM), TARGET),
    );
}

fn click_tech_toggle(spring: &spring::Spring<'_, spring::Ui>, global: &Global) {
    let button = crate::rml::dom::element(spring, &global.hud.access(), "tech-toggle");
    let clicked = button != 0 && spring.rml_ui().element_click(button).unwrap_or(false);
    if !clicked {
        spring::log_warning!(
            spring,
            crate::diagnostics::SECTION,
            "Area 17 e2e: could not click the technology button"
        );
    }
}

fn paused(spring: &spring::Spring<'_, spring::Ui>) -> bool {
    spring
        .display()
        .get_game_speed()
        .is_ok_and(|speed| speed.paused)
}

fn command(spring: &spring::Spring<'_, spring::Ui>, command: &str) {
    spring
        .messages()
        .send_commands(command, "")
        .log_err(spring, command);
}

impl Widget<Global> for DbgIntegrationWidget {
    fn name(&self) -> &'static str {
        "DbgIntegration"
    }

    fn update(&self, ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>, _dt: f32) {
        let spring = ctx.spring();
        if tech_panel::enabled(spring) {
            tech_panel::update(spring, ctx.global());
            return;
        }
        let frame = spring.game().current_frame().unwrap_or(0);
        if frame < 0 {
            return;
        }
        if !self.hud_verified.get() && ctx.global().hud.access().context != 0 {
            self.hud_verified.set(true);
            verify_hud_refresh(spring, ctx.global());
        }
        aim_at_target(spring);

        let steps = steps();
        self.request_pauses(spring, &steps);
        if self.resume_when_due(spring) {
            return;
        }
        let Some(&(due, _)) = steps.get(self.next.get()) else {
            return;
        };
        if frame < due || (frame == due && !self.settled_in_pause(spring)) {
            return;
        }
        if frame > due {
            spring::log_warning!(
                spring,
                crate::diagnostics::SECTION,
                "Area 17 e2e missed the pause at frame {due} (now {frame})"
            );
        } else {
            self.resume_in.set(Some(PAUSED_UPDATES));
        }
        while let Some(&(step_frame, step)) = steps.get(self.next.get()) {
            if step_frame != due {
                break;
            }
            run(spring, ctx.global(), step);
            self.next.set(self.next.get() + 1);
        }
    }
    fn draw_screen(
        &self,
        ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>,
        _width: i32,
        _height: i32,
    ) {
        let spring = ctx.spring();
        if tech_panel::enabled(spring) && ctx.global().hud.access().context != 0 {
            let frame = self.tech_fixture_frames.get() + 1;
            self.tech_fixture_frames.set(frame);
            if frame == 3 {
                command(spring, "screenshot png");
                spring::log_notice!(
                    spring,
                    crate::diagnostics::SECTION,
                    "Area 17 technology fixture screenshot captured"
                );
            } else if frame == 30 {
                command(spring, "quitforce");
            }
        }
    }
}

fn verify_hud_refresh(spring: &spring::Spring<'_, spring::Ui>, global: &Global) {
    use crate::rml::{self, dom};

    let previous_open = global.panel.access().upgrade_panel_open;
    let set_open = |open| {
        global.panel.access_mut().upgrade_panel_open = open;
        rml::refresh_hud(
            spring,
            &mut global.hud.access_mut(),
            &global.simulation.access(),
            open,
        );
        rml::update_context(spring, global);
    };
    let hover = || {
        let card = dom::element(spring, &global.hud.access(), "tech-card-0");
        let dispatched =
            card != 0 && spring.rml_ui().element_dispatch_event(card, "mouseover") == Ok(true);
        rml::update_context(spring, global);
        let hud = global.hud.access();
        let tooltip = dom::element(spring, &hud, "tech-tooltip");
        dispatched
            && hud.hovered_tech == Some(0)
            && spring.rml_ui().element_is_visible(tooltip) == Ok(true)
            && spring
                .rml_ui()
                .element_get_inner_rml(tooltip)
                .is_ok_and(|markup| markup.contains(rml::tech_data::TECH_DEFS[0].title))
    };

    set_open(true);
    let first_hover = hover();
    set_open(false);
    let cleared_on_close = global.hud.access().hovered_tech.is_none();
    set_open(true);
    let reopened_hover = hover();
    let card = dom::element(spring, &global.hud.access(), "tech-card-0");
    let left = card != 0 && spring.rml_ui().element_dispatch_event(card, "mouseout") == Ok(true);
    rml::update_context(spring, global);
    let cleared_on_leave = {
        let hud = global.hud.access();
        let tooltip = dom::element(spring, &hud, "tech-tooltip");
        hud.hovered_tech.is_none() && spring.rml_ui().element_is_visible(tooltip) == Ok(false)
    };
    set_open(previous_open);
    if first_hover && cleared_on_close && reopened_hover && left && cleared_on_leave {
        spring::log_info!(
            spring,
            crate::diagnostics::SECTION,
            "Area 17 HUD regression passed: native technology hover callbacks and reopened tooltip bindings work"
        );
    } else {
        spring::log_error!(
            spring,
            crate::diagnostics::SECTION,
            "Area 17 HUD regression failed: first={first_hover} closed={cleared_on_close} reopened={reopened_hover} left={left} cleared={cleared_on_leave}"
        );
    }
}
