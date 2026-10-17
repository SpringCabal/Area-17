use alloc::format;
use alloc::string::String;

use super::hud::{create_bindings, refresh_hud, update_events};
use super::state::{Bindings, Hud};
use super::tech_data::TECH_DEFS;
use super::{data, dom, tech};
use crate::commands;
use crate::diagnostics;

const CONTEXT_NAME: &str = "area17-wasm";
const DATA_MODEL_NAME: &str = "area17";
const DOCUMENT_PATH: &str = "LuaUI/wasm/area17.rml";
const CONTEXT_VALIDATION_INTERVAL_TICKS: i32 = 30;
const SETUP_RETRY_INTERVAL_TICKS: i32 = 30;

pub fn initialize(spring: &spring::Spring<'_, spring::Ui>, global: &crate::state::Global) {
    let (context, document, should_validate) = {
        let mut rml = global.hud.access_mut();
        if rml.setup_retry_ticks > 0 {
            rml.setup_retry_ticks -= 1;
            return;
        }
        let should_validate = rml.context_check_ticks >= CONTEXT_VALIDATION_INTERVAL_TICKS;
        if should_validate {
            rml.context_check_ticks = 0;
        } else if rml.context != 0 {
            rml.context_check_ticks += 1;
        }
        (rml.context, rml.document, should_validate)
    };
    if context != 0 {
        if !should_validate {
            return;
        }
        match context_is_live(spring, context, document) {
            Some(true) => return,
            Some(false) => {
                let _ = spring.rml_ui().remove_context(context);
                reset_handles(spring, &mut global.hud.access_mut());
                return;
            }
            None => return,
        }
    }
    let (context, document, bindings) = match create_hud(spring) {
        Ok(Some(handles)) => handles,
        Ok(None) => return,
        Err(error) => {
            global.hud.access_mut().setup_retry_ticks = SETUP_RETRY_INTERVAL_TICKS;
            spring::log_once!(
                spring,
                spring::log_level::WARNING,
                crate::diagnostics::SECTION,
                "Area 17 HUD setup: {error}; retrying"
            );
            return;
        }
    };
    let tech_open = global.panel.access().upgrade_panel_open;
    let mut hud = global.hud.access_mut();
    hud.context = context;
    hud.document = document;
    hud.bindings = Some(bindings);
    hud.context_check_ticks = 0;
    {
        let simulation = global.simulation.access();
        refresh_hud(spring, &mut hud, &simulation, tech_open);
        update_events(spring, &mut hud, &simulation, tech_open, 0.0);
    }
    let health_handle = dom::element(spring, &hud, "health-bar");
    let resource_handle = dom::element(spring, &hud, "biomass");
    diagnostics::info(
        spring,
        &format!(
            "Area 17 HUD handles context={context} document={document} health={health_handle} biomass={resource_handle}"
        ),
    );
    diagnostics::info(spring, "Area 17 RmlUi HUD initialized");
}

fn create_hud(
    spring: &spring::Spring<'_, spring::Ui>,
) -> Result<Option<(u64, u64, Bindings)>, String> {
    match spring.rml_ui().is_ready() {
        Ok(true) => {}
        Ok(false) => return Ok(None),
        Err(error) => {
            return Err(format!(
                "readiness query failed (error code {})",
                error.code
            ));
        }
    }
    let context = match spring.rml_ui().create_context(CONTEXT_NAME) {
        Ok(value) if value.success && value.context_handle != 0 => value.context_handle,
        Ok(value) => {
            return Err(format!(
                "context creation returned no context (handle={}, success={})",
                value.context_handle, value.success
            ));
        }
        Err(error) => {
            return Err(format!(
                "context creation failed (error code {})",
                error.code
            ));
        }
    };
    match load_hud_document(spring, context) {
        Ok((document, bindings)) => Ok(Some((context, document, bindings))),
        Err(error) => {
            let _ = spring.rml_ui().remove_context(context);
            Err(error)
        }
    }
}

fn load_hud_document(
    spring: &spring::Spring<'_, spring::Ui>,
    context: u64,
) -> Result<(u64, Bindings), String> {
    let data_model = create_data_model(spring, context).ok_or("data-model creation failed")?;
    let bindings = create_bindings(spring, data_model).ok_or("HUD value binding failed")?;
    let document = match spring
        .rml_ui()
        .context_load_document(context, DOCUMENT_PATH)
    {
        Ok(value) if value.success && value.document_handle != 0 => value.document_handle,
        Ok(_) => return Err(format!("{DOCUMENT_PATH} loaded no document")),
        Err(error) => {
            return Err(format!(
                "{DOCUMENT_PATH} failed to load (error code {})",
                error.code
            ));
        }
    };
    for (id, markup) in [
        (
            "weapons",
            include_str!("../../../../LuaUI/wasm/weapons.rml"),
        ),
        (
            "abilities",
            include_str!("../../../../LuaUI/wasm/abilities.rml"),
        ),
        (
            "tech-toggle",
            include_str!("../../../../LuaUI/wasm/tech-toggle.rml"),
        ),
        (
            "tech-panel",
            include_str!("../../../../LuaUI/wasm/tech-panel.rml"),
        ),
        ("events", include_str!("../../../../LuaUI/wasm/events.rml")),
        (
            "endgame",
            include_str!("../../../../LuaUI/wasm/endgame.rml"),
        ),
    ] {
        let element = spring
            .rml_ui()
            .element_get_element_by_id(document, id)
            .map_err(|_| format!("{id} placeholder could not be found"))?;
        if !element.exists
            || spring
                .rml_ui()
                .element_set_inner_rml(element.element_handle, markup)
                != Ok(true)
        {
            return Err(format!("{id} template could not be loaded"));
        }
    }
    if spring.rml_ui().document_show(document, Default::default()) != Ok(true) {
        return Err(format!("{DOCUMENT_PATH} could not be shown"));
    }
    Ok((document, bindings))
}

fn context_is_live(
    spring: &spring::Spring<'_, spring::Ui>,
    context: u64,
    document: u64,
) -> Option<bool> {
    let found = spring.rml_ui().get_context(CONTEXT_NAME).ok()?;
    if !found.exists || found.context_handle != context || document == 0 {
        return Some(false);
    }
    match spring.rml_ui().document_get_context(document) {
        Ok(owner) => Some(owner.exists && owner.context_handle == context),
        Err(error) if error.code == spring::ErrorCode::InvalidArgument as i32 => Some(false),
        Err(_) => None,
    }
}

fn reset_handles(spring: &spring::Spring<'_, spring::Ui>, hud: &mut Hud) {
    hud.context = 0;
    hud.document = 0;
    hud.bindings = None;
    hud.hovered_tech = None;
    hud.tech_open = None;
    hud.context_check_ticks = CONTEXT_VALIDATION_INTERVAL_TICKS;
    diagnostics::debug(
        spring,
        "Area 17 RmlUi handles were stale; rebuilding the HUD",
    );
}

fn create_data_model(spring: &spring::Spring<'_, spring::Ui>, context: u64) -> Option<u64> {
    let model = spring
        .rml_ui()
        .context_create_data_model(context, DATA_MODEL_NAME)
        .ok()?;
    if !model.success || model.data_model_handle == 0 {
        return None;
    }
    let model = model.data_model_handle;

    if !bind_closure(
        spring,
        model,
        "techToggle",
        &[],
        commands::toggle_upgrade_panel,
    ) || !bind_closure(
        spring,
        model,
        "techClose",
        &[],
        commands::close_upgrade_panel,
    ) {
        return None;
    }
    for (name, handler) in [
        (
            "techActivate",
            commands::activate_tech
                as fn(&spring::Spring<'_, spring::Ui>, &crate::state::Global, usize),
        ),
        ("techHover", tech::hover),
        ("techLeave", tech::leave),
    ] {
        if !bind_index_event(spring, model, name, handler) {
            return None;
        }
    }
    Some(model)
}

fn bind_closure(
    spring: &spring::Spring<'_, spring::Ui>,
    model: u64,
    name: &str,
    field_types: &[u8],
    handler: impl Fn(&spring::Spring<'_, spring::Ui>, &crate::state::Global)
    + spring::RetainedCapture
    + 'static,
) -> bool {
    spring
        .rml_ui()
        .data_model_bind_event(model, name, field_types, move |spring, _| {
            crate::with_global(|global| handler(spring, global));
            Ok(())
        })
        .is_ok_and(|registration| registration.success && registration.event_handle != 0)
}

fn bind_index_event(
    spring: &spring::Spring<'_, spring::Ui>,
    model: u64,
    name: &str,
    handler: fn(&spring::Spring<'_, spring::Ui>, &crate::state::Global, usize),
) -> bool {
    spring
        .rml_ui()
        .data_model_bind_event(model, name, &[data::INT], move |spring, event| {
            if let Some(spring::callbacks::Value::Int(index)) = event.values.first()
                && let Ok(index) = usize::try_from(*index)
                && index < TECH_DEFS.len()
            {
                crate::with_global(|global| handler(spring, global, index));
            }
            Ok(())
        })
        .is_ok_and(|registration| registration.success && registration.event_handle != 0)
}

pub fn update_context(spring: &spring::Spring<'_, spring::Ui>, global: &crate::state::Global) {
    let context = global.hud.access().context;
    if context != 0 {
        let _ = spring.rml_ui().context_update(context);
    }
}

pub fn render(
    spring: &spring::Spring<'_, spring::Ui>,
    global: &crate::state::Global,
    width: i32,
    height: i32,
) {
    initialize(spring, global);
    {
        let mut view = global.view.access_mut();
        view.width = width;
        view.height = height;
    }
    let context = global.hud.access().context;
    if context == 0 {
        return;
    }
    #[cfg(feature = "diagnostics")]
    if !core::mem::replace(&mut global.hud.access_mut().render_reported, true) {
        crate::diagnostics::info(spring, "Area 17 RmlUi HUD render pass reached");
    }
    if spring
        .rml_ui()
        .context_set_dimensions(context, width, height)
        != Ok(true)
    {
        spring::log_once!(
            spring,
            spring::log_level::WARNING,
            crate::diagnostics::SECTION,
            "Area 17 HUD context dimensions could not be set"
        );
        return;
    }
    if spring.rml_ui().context_render(context) != Ok(true) {
        spring::log_once!(
            spring,
            spring::log_level::WARNING,
            crate::diagnostics::SECTION,
            "Area 17 HUD context could not be rendered"
        );
    }
}
