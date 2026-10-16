extern crate alloc;

pub mod audio;
pub mod catalog;
pub mod commands;
pub mod diagnostics;
pub mod pointer;
pub mod rml;
pub mod state;
pub mod widgets;

spring_addons::export_ui_widgets! {
    state: state::Global,
    setup: widgets::setup,
    scratch: 4096,
}
