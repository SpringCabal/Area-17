use alloc::vec::Vec;
use core::cell::{Cell, RefCell};

use spring_addons::rules::{Gadget, UnitPreDamagedEvent};

use crate::definitions;
#[cfg(feature = "diagnostics")]
use crate::diagnostics;
use crate::state::Global;
use crate::weapons::{self, Impulses, LingeringArea};

#[derive(Default)]
pub struct WeaponHandlersGadget {
    areas: RefCell<Vec<LingeringArea>>,
    impulses: RefCell<Impulses>,
    gravity_per_frame: Cell<Option<f32>>,
}

impl WeaponHandlersGadget {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Gadget<Global> for WeaponHandlersGadget {
    fn name(&self) -> &'static str {
        "WeaponHandlers"
    }

    fn game_frame(&self, ctx: &spring_addons::AddonContext<'_, Global>, frame: i32) {
        let spring = ctx.spring();
        let gravity = self.gravity_per_frame.get().unwrap_or_else(|| {
            let gravity = definitions::gravity_per_frame(spring);
            self.gravity_per_frame.set(Some(gravity));
            gravity
        });
        weapons::apply_impulses(spring, self.impulses.take(), gravity);
        weapons::update(
            spring,
            ctx.global(),
            &mut self.areas.borrow_mut(),
            &self.impulses,
            frame,
        );
    }

    fn unit_pre_damaged(
        &self,
        ctx: &spring_addons::AddonContext<'_, Global>,
        event: &UnitPreDamagedEvent,
    ) -> Option<spring::DamageResult> {
        let spring = ctx.spring();
        let damage = weapons::pre_damaged(
            spring,
            ctx.global(),
            &self.impulses,
            event.unit_id,
            event.unit_def_id,
            event.damage,
            event.weapon_def_id,
            event.attacker_id,
        )?;

        #[cfg(feature = "diagnostics")]
        if event.unit_id == ctx.global().ufo.access().id {
            let frame = spring.game().current_frame().unwrap_or(0);
            if frame.rem_euclid(spring::consts::GAME_SPEED) == 0 {
                diagnostics::debug(
                    spring,
                    &alloc::format!(
                        "ufo damage frame={frame} unit={} weapon={} attacker={} incoming={:.2} applied={damage:.2}",
                        event.unit_id,
                        event.weapon_def_id,
                        event.attacker_id,
                        event.damage,
                    ),
                );
            }
        }

        Some(spring::DamageResult {
            new_damage: damage,
            impulse_mult: 1.0,
        })
    }

    fn explosion(
        &self,
        ctx: &spring_addons::AddonContext<'_, Global>,
        weapon_def_id: i32,
        pos: (f32, f32, f32),
        owner_id: i32,
        _projectile_id: i32,
    ) -> bool {
        let spring = ctx.spring();
        let area = weapons::explosion(
            spring,
            &ctx.global().tech.access(),
            weapon_def_id,
            spring::Float3::new(pos.0, pos.1, pos.2),
            owner_id,
        );
        if let Some(area) = area {
            self.areas.borrow_mut().push(area);
        }
        false
    }
}
