use alloc::vec::Vec;
use core::cell::RefCell;

use spring_addons::rules::{Gadget, UnitDestroyedEvent, UnitPreDamagedEvent};

use crate::fall_damage::{self, CollisionData};
use crate::state::Global;

#[derive(Default)]
pub struct UnitFallDamageGadget {
    collisions: RefCell<Vec<CollisionData>>,
}

impl UnitFallDamageGadget {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Gadget<Global> for UnitFallDamageGadget {
    fn name(&self) -> &'static str {
        "UnitFallDamage"
    }

    fn game_frame(&self, ctx: &spring_addons::AddonContext<'_, Global>, frame: i32) {
        let global = ctx.global();
        global.fall_immunity.access_mut().expire(frame);
        let collisions = self.collisions.take();
        fall_damage::resolve_collisions(global, collisions, frame);
    }

    fn unit_destroyed(
        &self,
        ctx: &spring_addons::AddonContext<'_, Global>,
        event: &UnitDestroyedEvent,
    ) {
        ctx.global()
            .fall_immunity
            .access_mut()
            .forget(event.unit_id);
        self.collisions
            .borrow_mut()
            .retain(|collision| collision.unit_id != event.unit_id);
    }

    fn unit_pre_damaged(
        &self,
        ctx: &spring_addons::AddonContext<'_, Global>,
        event: &UnitPreDamagedEvent,
    ) -> Option<spring::DamageResult> {
        let from_engine = event.attacker_id < 0;
        Some(match event.weapon_def_id {
            fall_damage::OBJECT_COLLISION if from_engine => {
                fall_damage::record_collision(
                    &self.collisions,
                    event.unit_id,
                    event.unit_def_id,
                    event.damage,
                );
                fall_damage::no_damage()
            }
            fall_damage::GROUND_COLLISION if from_engine => {
                let frame = spring::game::current_frame();
                let immune = ctx
                    .global()
                    .fall_immunity
                    .access()
                    .is_immune(event.unit_id, frame);
                fall_damage::ground_collision(event.unit_id, event.unit_def_id, immune)
            }
            fall_damage::DEBRIS => fall_damage::debris(),
            weapon if fall_damage::handles(weapon) => spring::DamageResult::unchanged(event.damage),
            _ => return None,
        })
    }
}
