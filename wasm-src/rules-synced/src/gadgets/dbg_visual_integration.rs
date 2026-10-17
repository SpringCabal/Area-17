use core::cell::RefCell;

use spring_addons::rules::Gadget;

use crate::state::Global;
use crate::visual_integration::{self, TestState};
use spring::Float3;

#[derive(Default)]
pub struct DbgVisualIntegrationGadget {
    test: RefCell<TestState>,
}

impl DbgVisualIntegrationGadget {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Gadget<Global> for DbgVisualIntegrationGadget {
    fn name(&self) -> &'static str {
        "DbgVisualIntegration"
    }

    fn game_frame(&self, ctx: &spring_addons::AddonContext<'_, Global>, frame: i32) {
        let spring = ctx.spring();
        visual_integration::update(spring, &mut self.test.borrow_mut(), ctx.global(), frame);
    }

    fn handle_lua_msg(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global>,
        _player_id: i32,
        _script: i32,
        _mode: i32,
        data: &[u8],
    ) {
        if let Ok(message) = core::str::from_utf8(data) {
            self.test.borrow_mut().lua_message(message);
        }
    }

    fn projectile_created(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global>,
        _projectile_id: i32,
        owner_id: i32,
        weapon_def_id: i32,
    ) {
        self.test
            .borrow_mut()
            .projectile_created(owner_id, weapon_def_id);
    }

    fn explosion(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global>,
        weapon_def_id: i32,
        pos: (f32, f32, f32),
        _owner_id: i32,
        _projectile_id: i32,
    ) -> bool {
        let spring = _ctx.spring();
        self.test.borrow_mut().explosion(
            spring,
            weapon_def_id,
            Float3 {
                x: pos.0,
                y: pos.1,
                z: pos.2,
            },
        );
        false
    }
}
