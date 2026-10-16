use spring_addons::Resource;

#[derive(Clone, Copy, Debug, Default)]
pub struct TechState {
    pub level: i32,
    pub unlocked: bool,
    pub enabled: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct AbilityState {
    pub cooldown: i32,
    pub duration: i32,
}

#[derive(Clone, Debug, Default)]
pub struct ViewState {
    pub width: i32,
    pub height: i32,
}

#[derive(Clone, Debug)]
pub struct SimulationState {
    pub status_initialized: bool,
    pub game_over: bool,
    pub game_won: bool,
    pub ufo_id: i32,
    pub health: f32,
    pub max_health: f32,
    pub shield: f32,
    pub abduction_radius: f32,
    pub current_weapon: Option<usize>,
    pub tech: [TechState; 13],
    pub abilities: [AbilityState; 4],
    pub biomass: f32,
    pub research: f32,
    pub metal: f32,
}

impl Default for SimulationState {
    fn default() -> Self {
        Self {
            status_initialized: false,
            game_over: false,
            game_won: false,
            ufo_id: -1,
            health: 0.0,
            max_health: 1.0,
            shield: 0.0,
            abduction_radius: 0.0,
            current_weapon: None,
            tech: [TechState::default(); 13],
            abilities: [AbilityState::default(); 4],
            biomass: 0.0,
            research: 0.0,
            metal: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Panel {
    pub upgrade_panel_open: bool,
}

#[derive(Default)]
pub struct Global {
    pub view: Resource<ViewState>,
    pub simulation: Resource<SimulationState>,
    pub panel: Resource<Panel>,
    pub hud: Resource<crate::rml::Hud>,
}
