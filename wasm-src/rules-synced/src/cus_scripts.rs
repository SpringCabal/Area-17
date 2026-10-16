mod human;
mod keyframes;
mod module;
mod props;
mod ufo;

#[cfg(feature = "headless-integration")]
pub use module::human_walks;
pub use module::{
    Area17CusModule, UfoControl, abduction_float, alien_land, attach_script, set_ufo_control,
};

use alloc::rc::Rc;
use core::cell::RefCell;
use spring::ResultExt;

use spring::UnitId;
use spring::cus::host::{CusHost, UnitDefScript};
use spring::cus::{
    Angle, AngularSpeed, Axis, InitCtx, Piece, ScriptCapabilities, SfxFlags, SignalMask,
    TaskDefinition, UnitCtx, UnitScript, WeaponId, WreckLevel,
};

use crate::defs::{self, UnitKind};
use crate::state::{BeamState, Weapon, ufo_slots};
use human::HumanState;
use props::{BuildingState, DronePieces, HelipadState, StaticAaPieces, TankPieces};
use spring::Float3;
use ufo::UfoState;

const PUBLIC: i32 = spring::rules_params::los::PUBLIC;
const SFX_FIRST_EXPLOSION_GENERATOR: i32 = 1024;
const SFX_FIRE_WEAPON: i32 = 2048;
const SIG_HUMAN: SignalMask = SignalMask::new(1);
const SIG_INDEPENDENCE: SignalMask = SignalMask::new(2);
const SIG_HELIPAD: SignalMask = SignalMask::new(4);
const SIG_AIM: SignalMask = SignalMask::new(8);

fn piece(ctx: &InitCtx<'_>, name: &str) -> Piece {
    let direct = ctx.piece(name);
    if direct != Piece::INVALID {
        return direct;
    }
    match name {
        "footRight" => ctx.piece("footRIght"),
        "footRIght" => ctx.piece("footRight"),
        _ => Piece::INVALID,
    }
}

fn turn(ctx: &UnitCtx, piece: Piece, axis: Axis, target: f32, speed: f32) {
    if piece == Piece::INVALID {
        return;
    }
    if speed == 0.0 {
        ctx.turn_now(piece, axis, Angle::radians(target));
    } else {
        ctx.turn(
            piece,
            axis,
            Angle::radians(target),
            AngularSpeed::radians_per_second(speed),
        );
    }
}

fn move_piece(ctx: &UnitCtx, piece: Piece, axis: Axis, target: f32, speed: f32) {
    if piece == Piece::INVALID {
        return;
    }
    if speed == 0.0 {
        ctx.move_now(piece, axis, target);
    } else {
        ctx.move_piece(piece, axis, target, speed);
    }
}

fn explode(ctx: &UnitCtx, piece: Piece, flags: SfxFlags) {
    if piece != Piece::INVALID {
        ctx.emit_sfx(piece, SFX_FIRST_EXPLOSION_GENERATOR);
        ctx.explode(piece, flags);
    }
}

fn play_sound(name: &str, volume: f32, unit: UnitId) {
    let position = spring::get_unit_position(unit, false, false).unwrap_or_default();
    spring::sound::play_sound_file(name, volume, position, spring::Float3::default(), 0)
        .log_err_once("Area 17 unit script: play sound file");
}

fn script_kind_for_def(def: i32) -> Option<ScriptKind> {
    Some(match defs::kind(def)? {
        UnitKind::Ufo => ScriptKind::Ufo,
        UnitKind::Drone => ScriptKind::Drone,
        UnitKind::Tank | UnitKind::SmallTank => ScriptKind::Tank,
        UnitKind::StaticAa | UnitKind::StrongAa => ScriptKind::StaticAa,
        UnitKind::Building1
        | UnitKind::Building2
        | UnitKind::Building3
        | UnitKind::Building4
        | UnitKind::Building4BigTank => ScriptKind::Building,
        UnitKind::Helipad => ScriptKind::Helipad,
        UnitKind::UfoWreck => ScriptKind::UfoWreck,
        UnitKind::Alien => ScriptKind::Alien,
        UnitKind::Civilian
        | UnitKind::Civilian1
        | UnitKind::Scientist
        | UnitKind::Scientist2
        | UnitKind::Endcube => ScriptKind::Human,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ScriptKind {
    Ufo,
    Human,
    Alien,
    Drone,
    Tank,
    StaticAa,
    Building,
    Helipad,
    UfoWreck,
}

fn capabilities(kind: ScriptKind) -> ScriptCapabilities {
    let mut bits = ScriptCapabilities::CREATE | ScriptCapabilities::KILLED;
    match kind {
        ScriptKind::Ufo | ScriptKind::Tank => {
            bits |= ScriptCapabilities::QUERY_WEAPON
                | ScriptCapabilities::AIM_FROM_WEAPON
                | ScriptCapabilities::AIM_WEAPON
                | ScriptCapabilities::SHOT;
        }
        ScriptKind::Human | ScriptKind::Alien => {
            bits |= ScriptCapabilities::START_MOVING
                | ScriptCapabilities::STOP_MOVING
                | ScriptCapabilities::LANDED;
        }
        ScriptKind::Drone | ScriptKind::StaticAa => {
            bits |= ScriptCapabilities::QUERY_WEAPON
                | ScriptCapabilities::AIM_FROM_WEAPON
                | ScriptCapabilities::AIM_WEAPON;
        }
        ScriptKind::Helipad => {
            bits |= ScriptCapabilities::ACTIVATE | ScriptCapabilities::DEACTIVATE;
        }
        ScriptKind::Building | ScriptKind::UfoWreck => {}
    }
    ScriptCapabilities::new(bits)
}

pub struct Area17UnitScript(Script);

enum Script {
    Ufo(Rc<RefCell<UfoState>>),
    Human(Rc<RefCell<HumanState>>),
    Drone(DronePieces),
    Tank(TankPieces),
    StaticAa(StaticAaPieces),
    Building(BuildingState),
    Helipad(Rc<RefCell<HelipadState>>),
    Inert,
}

impl Area17UnitScript {
    fn for_kind(kind: Option<ScriptKind>, ctx: &mut InitCtx<'_>) -> Self {
        Self(match kind {
            Some(ScriptKind::Ufo) => Script::Ufo(Rc::new(RefCell::new(UfoState::new(ctx)))),
            Some(ScriptKind::Human) => {
                Script::Human(Rc::new(RefCell::new(HumanState::new(ctx, false))))
            }
            Some(ScriptKind::Alien) => {
                Script::Human(Rc::new(RefCell::new(HumanState::new(ctx, true))))
            }
            Some(ScriptKind::Drone) => Script::Drone(DronePieces::new(ctx)),
            Some(ScriptKind::Tank) => Script::Tank(TankPieces::new(ctx)),
            Some(ScriptKind::StaticAa) => Script::StaticAa(StaticAaPieces::new(ctx)),
            Some(ScriptKind::Building) => Script::Building(BuildingState::new(ctx)),
            Some(ScriptKind::Helipad) => {
                Script::Helipad(Rc::new(RefCell::new(HelipadState::new(ctx))))
            }
            Some(ScriptKind::UfoWreck) | None => Script::Inert,
        })
    }

    pub(crate) fn abduction_float(&mut self, ctx: &UnitCtx) {
        if let Script::Human(state) = &mut self.0 {
            human::abduction_float(ctx, Rc::clone(state));
        }
    }

    pub(crate) fn alien_land(&mut self, ctx: &UnitCtx) {
        if let Script::Human(state) = &mut self.0 {
            human::land(ctx, Rc::clone(state));
        }
    }
}

impl UnitScript for Area17UnitScript {
    fn create(&mut self, ctx: &UnitCtx) {
        match &mut self.0 {
            Script::Ufo(state) => ufo::create(ctx, Rc::clone(state)),
            Script::Human(state) => human::create(ctx, Rc::clone(state)),
            Script::Drone(drone) => {
                ctx.spin(
                    drone.turret,
                    Axis::Z,
                    AngularSpeed::radians_per_second(2.0),
                    AngularSpeed::radians_per_second(0.0),
                );
                ctx.spin(
                    drone.base,
                    Axis::Z,
                    AngularSpeed::radians_per_second(-1.0),
                    AngularSpeed::radians_per_second(0.0),
                );
            }
            Script::Building(building) => {
                for block in building.blocks.iter().copied() {
                    if block != Piece::INVALID {
                        ctx.hide(block);
                    }
                }
            }
            _ => {}
        }
    }

    fn start_moving(&mut self, ctx: &UnitCtx, _reversing: bool) {
        if let Script::Human(state) = &mut self.0 {
            human::start_moving(ctx, Rc::clone(state));
        }
    }

    fn stop_moving(&mut self, ctx: &UnitCtx) {
        if let Script::Human(state) = &mut self.0 {
            human::stop_moving(ctx, Rc::clone(state));
        }
    }

    fn landed(&mut self, ctx: &UnitCtx) {
        if let Script::Human(state) = &mut self.0
            && state.borrow().alien
        {
            human::land(ctx, Rc::clone(state));
        }
    }

    fn query_weapon(&mut self, _ctx: &UnitCtx, weapon: WeaponId) -> Piece {
        match &mut self.0 {
            Script::Ufo(state) => {
                let state = state.borrow();
                if weapon.0 == ufo_slots::SHIELD {
                    state.pieces.independence
                } else if state.current_weapon == Some(Weapon::PulseLaser) {
                    state.pieces.muzzles[state.last_gun]
                } else {
                    state.pieces.muzzles[(weapon.0.max(0) as usize) % 4]
                }
            }
            Script::Drone(drone) => drone.muzzle,
            Script::Tank(tank) => tank.barrels[tank.current_barrel],
            Script::StaticAa(aa) => aa.muzzle,
            _ => Piece::INVALID,
        }
    }

    fn aim_from_weapon(&mut self, _ctx: &UnitCtx, weapon: WeaponId) -> Piece {
        match &mut self.0 {
            Script::Ufo(state) => {
                let state = state.borrow();
                if weapon.0 == ufo_slots::SHIELD {
                    state.pieces.independence
                } else if state.current_weapon == Some(Weapon::PulseLaser) {
                    state.pieces.guns[state.last_gun]
                } else {
                    state.pieces.guns[(weapon.0.max(0) as usize) % 4]
                }
            }
            Script::Drone(drone) => drone.muzzle,
            Script::Tank(tank) => tank.barrels[tank.current_barrel],
            Script::StaticAa(aa) => aa.barrel,
            _ => Piece::INVALID,
        }
    }

    fn aim_weapon(
        &mut self,
        ctx: &UnitCtx,
        weapon: WeaponId,
        heading: f32,
        pitch: f32,
    ) -> Option<bool> {
        match &mut self.0 {
            Script::Ufo(state) => {
                let point_defense = spring::get_unit_rules_param_f32(ctx.unit(), "PDisUnlocked")
                    .unwrap_or(0.0)
                    > 0.0;
                let ready = (ufo_slots::POINT_DEFENSE.contains(&weapon.0) && point_defense)
                    || state
                        .borrow()
                        .current_weapon
                        .is_some_and(|current| ufo_slots::of(current).contains(&weapon.0));
                Some(ready)
            }
            Script::Drone(_) => Some(true),
            Script::Tank(tank) => {
                ctx.signal(SIG_AIM);
                ctx.spawn(TaskDefinition::with_state(
                    "area17-tank-aim",
                    Rc::new(RefCell::new((*tank, weapon, heading, pitch))),
                    props::aim_tank,
                ));
                None
            }
            Script::StaticAa(aa) => {
                ctx.signal(SIG_AIM);
                ctx.spawn(TaskDefinition::with_state(
                    "area17-aa-aim",
                    Rc::new(RefCell::new((*aa, weapon, heading, pitch))),
                    props::aim_aa,
                ));
                None
            }
            _ => None,
        }
    }

    fn shot(&mut self, ctx: &UnitCtx, _weapon: WeaponId) {
        match &mut self.0 {
            Script::Ufo(state) => {
                let mut state = state.borrow_mut();
                if state.current_weapon == Some(Weapon::PulseLaser) {
                    state.last_gun = (state.last_gun + 1) % 4;
                }
            }
            Script::Tank(tank) => {
                tank.current_barrel = (tank.current_barrel + 1) % tank.barrels.len();
                play_sound("sounds/rocket.wav", 70.0, ctx.unit());
            }
            _ => {}
        }
    }

    fn activate(&mut self, ctx: &UnitCtx) {
        if let Script::Helipad(state) = &mut self.0 {
            props::move_helipad(ctx, Rc::clone(state), -15.0, "area17-helipad-activate");
        }
    }

    fn deactivate(&mut self, ctx: &UnitCtx) {
        if let Script::Helipad(state) = &mut self.0 {
            props::move_helipad(ctx, Rc::clone(state), 0.0, "area17-helipad-deactivate");
        }
    }

    fn killed(&mut self, ctx: &UnitCtx, _recent_damage: f32, _max_health: f32) -> WreckLevel {
        let flags = SfxFlags(SfxFlags::FIRE.0 | SfxFlags::SMOKE.0 | SfxFlags::NO_HEATCLOUD.0);
        match &mut self.0 {
            Script::Ufo(state) => ufo::explode_hull(ctx, &state.borrow().pieces, flags),
            Script::Human(state) => {
                for piece in state.borrow().pieces.limbs() {
                    explode(ctx, piece, flags);
                }
            }
            Script::Tank(tank) => {
                for piece in tank.all {
                    explode(ctx, piece, flags);
                }
            }
            Script::StaticAa(aa) => {
                for piece in aa.all {
                    explode(ctx, piece, flags);
                }
            }
            Script::Building(building) => building.shatter(ctx, flags),
            Script::Drone(_) | Script::Helipad(_) | Script::Inert => {}
        }
        WreckLevel::None
    }
}
