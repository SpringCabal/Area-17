use super::*;

pub type Area17CusModule = CusHost<Area17UnitScript>;

impl UnitDefScript for Area17UnitScript {
    fn capabilities(
        spring: &spring::Spring<'_, spring::RulesSynced>,
        def: i32,
    ) -> Option<ScriptCapabilities> {
        script_kind_for_def(spring, def).map(capabilities)
    }
    fn for_def(
        spring: &spring::Spring<'_, spring::RulesSynced>,
        def: i32,
        ctx: &mut InitCtx<'_>,
        _: &mut (),
    ) -> Self {
        Self::for_kind(script_kind_for_def(spring, def), ctx)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct UfoControl {
    pub aim: Float3,
    pub weapon: Option<Weapon>,
    pub beam: BeamState,
}

pub fn attach_script(spring: &spring::Spring<'_, spring::RulesSynced>, unit: i32, def: i32) {
    crate::with_cus_module(move |host, spring| host.unit_created(spring, UnitId(unit), def))
        .log_err_once(spring, "attaching an Area 17 unit script");
}

fn call_script(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit: i32,
    name: &'static str,
    call: fn(&mut Area17UnitScript, &UnitCtx),
) {
    crate::with_cus_module(move |host, spring| {
        if host
            .with_script(spring, UnitId(unit), |ctx, script| call(script, ctx))?
            .is_none()
        {
            spring::log_warning!(spring, "area17", "{name}: unit {unit} has no unit script");
        }
        Ok(())
    })
    .log_err_once(spring, "calling an Area 17 unit script");
}

pub fn abduction_float(spring: &spring::Spring<'_, spring::RulesSynced>, unit: i32) {
    call_script(
        spring,
        unit,
        "Abduction_float",
        Area17UnitScript::abduction_float,
    );
}

pub fn alien_land(spring: &spring::Spring<'_, spring::RulesSynced>, unit: i32) {
    call_script(spring, unit, "Alien_land", Area17UnitScript::alien_land);
}

pub fn set_ufo_control(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit: i32,
    control: UfoControl,
) {
    crate::with_cus_module(|host, spring| {
        host.with_script(spring, UnitId(unit), |ctx, script| {
            if let Script::Ufo(ufo) = &script.0 {
                ufo.borrow_mut().control = Some(control);
                ufo::aim(ctx, ufo);
            }
        })
    })
    .log_err_once(spring, "updating Area 17 UFO script control");
}

#[cfg(feature = "headless-integration")]
pub fn human_walks(spring: &spring::Spring<'_, spring::RulesSynced>, unit: i32) -> Option<bool> {
    crate::with_cus_module(|host, spring| {
        Ok(host.with_state(UnitId(unit), |script| match &script.0 {
            Script::Human(human) => {
                let human = human.borrow();
                spring::log_info!(
                    spring,
                    "r7dbg",
                    "R7DBG human {unit} pose={:?} floating={}",
                    human.pose,
                    human.floating
                );
                Some(human.pose == human::Pose::Walk && !human.floating)
            }
            _ => None,
        }))
    })
    .log_err_once(spring, "reading Area 17 human script state")
    .flatten()
    .flatten()
}
