use alloc::vec::Vec;

use spring::rml_ui::{RmlDataFieldDef, RmlDataValue};

pub const BOOL: u8 = 0;
pub const INT: u8 = 1;
pub const STRING: u8 = 3;
pub const PIXELS: u8 = 5;
pub const PERCENT: u8 = 6;

pub fn bind_rows(
    spring: &spring::Spring<'_, spring::Ui>,
    model: u64,
    name: &str,
    fields: &[(&str, u8)],
) -> Option<u64> {
    let fields: Vec<_> = fields
        .iter()
        .map(|&(name, type_)| RmlDataFieldDef {
            name: name.into(),
            type_,
        })
        .collect();
    let binding = spring
        .rml_ui()
        .data_model_bind_rows(model, name, &fields)
        .ok()?;
    (binding.success && binding.rows_handle != 0).then_some(binding.rows_handle)
}

pub fn bind_bool(spring: &spring::Spring<'_, spring::Ui>, model: u64, name: &str) -> Option<u64> {
    let binding = spring
        .rml_ui()
        .data_model_bind_bool(model, name, false)
        .ok()?;
    (binding.success && binding.variable_handle != 0).then_some(binding.variable_handle)
}

pub fn bind_string(spring: &spring::Spring<'_, spring::Ui>, model: u64, name: &str) -> Option<u64> {
    let binding = spring
        .rml_ui()
        .data_model_bind_string(model, name, "")
        .ok()?;
    (binding.success && binding.variable_handle != 0).then_some(binding.variable_handle)
}

pub fn boolean(value: bool) -> RmlDataValue {
    RmlDataValue {
        type_: BOOL,
        bool_value: value,
        ..Default::default()
    }
}

pub fn int(value: i32) -> RmlDataValue {
    RmlDataValue {
        type_: INT,
        int_value: value,
        ..Default::default()
    }
}

pub fn text(value: &str) -> RmlDataValue {
    RmlDataValue {
        type_: STRING,
        string_value: value.into(),
        ..Default::default()
    }
}

pub fn pixels(value: f32) -> RmlDataValue {
    RmlDataValue {
        type_: PIXELS,
        float_value: value,
        ..Default::default()
    }
}

pub fn percent(value: f32) -> RmlDataValue {
    RmlDataValue {
        type_: PERCENT,
        float_value: value,
        ..Default::default()
    }
}
