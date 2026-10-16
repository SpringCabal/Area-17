use alloc::string::String;
use alloc::vec::Vec;

#[derive(Clone, Copy)]
pub struct Bindings {
    pub biomass: u64,
    pub research: u64,
    pub health: u64,
    pub health_color: u64,
    pub health_caption: u64,
    pub shield: u64,
    pub shield_color: u64,
    pub shield_caption: u64,
    pub game_over: u64,
    pub endgame_title: u64,
    pub events: u64,
    pub weapons: u64,
    pub abilities: u64,
    pub tech: super::tech::Bindings,
}

pub struct UiEvent {
    pub text: String,
    pub color: &'static str,
    pub age_seconds: f32,
}

#[derive(Default)]
pub struct Hud {
    pub context: u64,
    pub document: u64,
    pub bindings: Option<Bindings>,
    pub events: Vec<UiEvent>,
    pub context_check_ticks: i32,
    pub tech_open: Option<bool>,
    pub hovered_tech: Option<usize>,
    pub render_reported: bool,
    pub setup_retry_ticks: i32,
}
