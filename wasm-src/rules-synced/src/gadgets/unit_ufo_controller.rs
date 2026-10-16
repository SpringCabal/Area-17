use alloc::rc::Rc;
use core::cell::RefCell;
use spring::ResultExt;

use spring::cmd;
use spring::commands;
use spring_addons::rules::{Gadget, UnitDestroyedEvent};

use crate::defs::{self, UnitKind};
use crate::state::{BeamState, Global};
use crate::ufo_controller::{self, Alien};
use crate::{definitions, diagnostics};

const UFO_RESPAWN_DELAY_FRAMES: i32 = 105;
const ALIEN_SPAWN_DELAY_FRAMES: i32 = 10;

#[derive(Default)]
pub struct UnitUfoControllerGadget {
    alien: Rc<RefCell<Alien>>,
}

impl UnitUfoControllerGadget {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Gadget<Global> for UnitUfoControllerGadget {
    fn name(&self) -> &'static str {
        "UnitUfoController"
    }

    fn handle_lua_msg(
        &self,
        ctx: &spring_addons::AddonContext<'_, Global>,
        _player_id: i32,
        _script: i32,
        _mode: i32,
        data: &[u8],
    ) {
        if let Ok(message) = core::str::from_utf8(data) {
            ufo_controller::handle_message(ctx.global(), message);
        }
    }

    fn game_frame(&self, ctx: &spring_addons::AddonContext<'_, Global>, frame: i32) {
        let global = ctx.global();
        ufo_controller::apply_pending_input(global);
        ufo_controller::update_destroyed_sequence(global, &self.alien, frame);
        ufo_controller::update(global);
    }

    fn unit_created(
        &self,
        ctx: &spring_addons::AddonContext<'_, Global>,
        unit: i32,
        def: i32,
        _team: i32,
        _builder: i32,
    ) {
        if defs::kind(def) != Some(UnitKind::Ufo) {
            return;
        }
        ctx.global().ufo.access_mut().id = unit;
        commands::give_order_to_unit(unit, cmd::IDLEMODE, &[0.0], 0, -1).then(|accepted| {
            if !accepted.unwrap_or(false) {
                spring::log_warning!(
                    crate::diagnostics::SECTION,
                    "Area 17 UFO rejected idle mode 0; it may land by itself"
                );
            }
        });
        spring::set_unit_rules_param_f32(unit, "selfMoveSpeedChange", 1.0, 0)
            .log_err_once("Area 17 UFO controller: set unit rules param f32");
    }

    fn unit_destroyed(
        &self,
        ctx: &spring_addons::AddonContext<'_, Global>,
        event: &UnitDestroyedEvent,
    ) {
        let unit_id = event.unit_id;
        {
            let mut alien = self.alien.borrow_mut();
            if alien.id == unit_id {
                alien.id = -1;
                alien.land_frame = None;
            }
        }
        if ctx.global().ufo.access().id != unit_id {
            return;
        }

        let frame = spring::game::current_frame();
        let destroyed_position = definitions::unit_position(unit_id);
        {
            let mut ufo = ctx.global().ufo.access_mut();
            ufo.id = -1;
            ufo.respawn_frame = Some(frame + UFO_RESPAWN_DELAY_FRAMES);
            ufo.destroyed_position = destroyed_position;
        }
        *self.alien.borrow_mut() = Alien {
            spawn_frame: Some(frame + ALIEN_SPAWN_DELAY_FRAMES),
            ..Alien::default()
        };
        {
            let mut tech = ctx.global().tech.access_mut();
            tech.ability_durations.fill(0);
            tech.beam = BeamState::Idle;
        }
        let [biomass, research, metal] = defs::unit_def(UnitKind::Alien)
            .and_then(defs::unit_data)
            .map_or([0.0; 3], |data| data.capture_value);
        {
            let mut economy = ctx.global().economy.access_mut();
            economy.biomass = (economy.biomass - biomass).max(0.0);
            economy.research = (economy.research - research).max(0.0);
            economy.metal = (economy.metal - metal).max(0.0);
        }
        diagnostics::info("Area 17 UFO destroyed; respawn scheduled");
    }
}
