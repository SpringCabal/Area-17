use super::*;

pub type Area17CusModule = CusHost<Area17UnitScript>;

impl UnitDefScript for Area17UnitScript {
    fn capabilities(def: i32) -> Option<ScriptCapabilities> {
        script_kind_for_def(def).map(capabilities)
    }

    fn for_def(def: i32, ctx: &mut InitCtx<'_>, _: &mut ()) -> Self {
        Self::for_kind(script_kind_for_def(def), ctx)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct UfoControl {
    pub aim: Float3,
    pub weapon: Option<Weapon>,
    pub beam: BeamState,
}

pub fn attach_script(unit: i32, def: i32) {
    crate::with_cus_module_or_defer(move |host| host.unit_created(UnitId(unit), def));
}

fn call_script(unit: i32, name: &'static str, call: fn(&mut Area17UnitScript, &UnitCtx)) {
    crate::with_cus_module_or_defer(move |host| {
        if host
            .with_script(UnitId(unit), |ctx, script| call(script, ctx))
            .is_none()
        {
            spring::log_warning!("area17", "{name}: unit {unit} has no unit script");
        }
    });
}

pub fn abduction_float(unit: i32) {
    call_script(unit, "Abduction_float", Area17UnitScript::abduction_float);
}

pub fn alien_land(unit: i32) {
    call_script(unit, "Alien_land", Area17UnitScript::alien_land);
}

pub fn set_ufo_control(unit: i32, control: UfoControl) {
    let _ = crate::with_cus_module(|host| {
        host.with_script(UnitId(unit), |ctx, script| {
            if let Script::Ufo(ufo) = &script.0 {
                ufo.borrow_mut().control = Some(control);
                ufo::aim(ctx, ufo);
            }
        })
    });
}

#[cfg(feature = "headless-integration")]
pub fn human_walks(unit: i32) -> Option<bool> {
    crate::with_cus_module(|host| {
        host.with_state(UnitId(unit), |script| match &script.0 {
            Script::Human(human) => {
                let human = human.borrow();
                spring::log_info!(
                    "r7dbg",
                    "R7DBG human {unit} pose={:?} floating={}",
                    human.pose,
                    human.floating
                );
                Some(human.pose == human::Pose::Walk && !human.floating)
            }
            _ => None,
        })
    })
    .flatten()
    .flatten()
}
