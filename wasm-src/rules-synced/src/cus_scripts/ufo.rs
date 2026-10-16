use super::*;
use spring::ResultExt;

#[derive(Clone, Copy)]
pub(super) struct UfoPieces {
    pub(super) muzzles: [Piece; 4],
    pub(super) guns: [Piece; 4],
    pub(super) bases: [Piece; 4],
    pub(super) railing: [Piece; 4],
    pub(super) turbine: Piece,
    pub(super) inner_hull: Piece,
    pub(super) outer_hull: [Piece; 4],
    pub(super) independence: Piece,
}

pub(super) struct UfoState {
    pub(super) pieces: UfoPieces,
    pub(super) current_weapon: Option<Weapon>,
    pub(super) last_gun: usize,
    pub(super) independence: bool,
    pub(super) gun_heading: Option<[f32; 4]>,
    pub(super) beam: Option<f32>,
    pub(super) slot: Option<i32>,
    pub(super) control: Option<UfoControl>,
}

impl UfoState {
    pub(super) fn new(ctx: &InitCtx<'_>) -> Self {
        Self {
            pieces: UfoPieces {
                muzzles: [
                    piece(ctx, "LowgunMuzzle"),
                    piece(ctx, "LowgunMuzzleSE"),
                    piece(ctx, "LowgunMuzzleNE"),
                    piece(ctx, "LowgunMuzzleNW"),
                ],
                guns: [
                    piece(ctx, "Lowgun"),
                    piece(ctx, "LowgunSE"),
                    piece(ctx, "LowgunNE"),
                    piece(ctx, "LowgunNW"),
                ],
                bases: [
                    piece(ctx, "LowgunBase"),
                    piece(ctx, "LowgunSEBase"),
                    piece(ctx, "LowgunNEBase"),
                    piece(ctx, "LowgunNWBase"),
                ],
                railing: [
                    piece(ctx, "RailingInnerTop"),
                    piece(ctx, "RailingOuterTop"),
                    piece(ctx, "RailingInnerLow"),
                    piece(ctx, "RailingOuterLow"),
                ],
                turbine: piece(ctx, "Turbine"),
                inner_hull: piece(ctx, "InnerHull"),
                outer_hull: [
                    piece(ctx, "OuterHull"),
                    piece(ctx, "OuterHullNE"),
                    piece(ctx, "OuterHullNW"),
                    piece(ctx, "OuterHullSE"),
                ],
                independence: piece(ctx, "independence"),
            },
            current_weapon: None,
            last_gun: 3,
            independence: false,
            gun_heading: None,
            beam: None,
            slot: None,
            control: None,
        }
    }
}

const RAILING_SPIN: [f32; 4] = [-3.0, -5.0, -3.0, -5.0];

pub(super) fn create(ctx: &UnitCtx, state: Rc<RefCell<UfoState>>) {
    let p = state.borrow().pieces;
    ctx.spawn(TaskDefinition::with_state(
        "area17-ufo-initial-heading",
        Rc::clone(&state),
        initial_heading,
    ));
    for (railing, speed) in p.railing.into_iter().zip(RAILING_SPIN) {
        ctx.spin(
            railing,
            Axis::Z,
            AngularSpeed::radians_per_second(speed),
            AngularSpeed::radians_per_second(0.0),
        );
    }
    for hull in p.outer_hull {
        ctx.spin(
            hull,
            Axis::Z,
            AngularSpeed::radians_per_second(1.0),
            AngularSpeed::radians_per_second(0.0),
        );
    }
    ctx.spin(
        p.inner_hull,
        Axis::Z,
        AngularSpeed::radians_per_second(-2.0),
        AngularSpeed::radians_per_second(0.0),
    );
    ctx.spin(
        p.turbine,
        Axis::Z,
        AngularSpeed::radians_per_second(4.0),
        AngularSpeed::radians_per_second(0.0),
    );
    turn(ctx, p.independence, Axis::X, core::f32::consts::PI, 0.0);
    ctx.spawn(TaskDefinition::with_state(
        "area17-ufo-controller",
        state,
        controller,
    ));
}

fn gun_heading(ctx: &UnitCtx, pieces: &UfoPieces, index: usize) -> f32 {
    let position = |piece: Piece| {
        spring::units_pieces::get_unit_piece_position(ctx.unit().0, piece.0 + 1)
            .log_err_once("Area 17 UFO script: get unit piece position")
            .unwrap_or_default()
    };
    let hull = position(pieces.outer_hull[index]);
    let base = position(pieces.bases[index]);
    (base.x - hull.x).atan2(base.z - hull.z)
}

async fn initial_heading(state: Rc<RefCell<UfoState>>, ctx: UnitCtx) {
    let pieces = state.borrow().pieces;
    while gun_heading(&ctx, &pieces, 0) == 0.0 {
        ctx.sleep(spring::cus::Duration::from_millis(33)).await;
    }
    let headings = core::array::from_fn(|index| gun_heading(&ctx, &pieces, index));
    state.borrow_mut().gun_heading = Some(headings);
}

pub(super) fn explode_hull(ctx: &UnitCtx, p: &UfoPieces, flags: SfxFlags) {
    for piece in p.bases {
        explode(ctx, piece, flags);
    }
    for piece in p.outer_hull {
        explode(ctx, piece, SfxFlags(flags.0 | SfxFlags::RECURSIVE.0));
    }
    explode(ctx, p.inner_hull, flags);
}

pub(super) async fn controller(state: Rc<RefCell<UfoState>>, ctx: UnitCtx) {
    loop {
        let control = state.borrow().control;
        let (weapon, beam) = control.map_or((None, BeamState::Idle), |control| {
            (control.weapon, control.beam)
        });
        set_current_weapon(&ctx, Rc::clone(&state), weapon);
        if beam == BeamState::GodGun {
            start_independence(&ctx, Rc::clone(&state));
        } else {
            stop_independence(&ctx, Rc::clone(&state));
            set_beam(&ctx, &state, beam.value());
        }
        ctx.sleep(spring::cus::Duration::from_millis(33)).await;
    }
}

pub(super) fn set_current_weapon(
    ctx: &UnitCtx,
    state: Rc<RefCell<UfoState>>,
    weapon: Option<Weapon>,
) {
    let slot = weapon.map_or(0, |weapon| ufo_slots::of(weapon).start + 1);
    let mut state = state.borrow_mut();
    state.current_weapon = weapon;
    if state.slot.replace(slot) != Some(slot) {
        spring::set_unit_rules_param_f32(ctx.unit(), "active_weapon_slot", slot as f32, PUBLIC)
            .log_err_once("Area 17 UFO script: set unit rules param f32");
    }
}

pub(super) fn start_independence(ctx: &UnitCtx, state: Rc<RefCell<UfoState>>) {
    if core::mem::replace(&mut state.borrow_mut().independence, true) {
        return;
    }
    set_beam(ctx, &state, -1.0);
    ctx.spawn(TaskDefinition::with_state(
        "area17-ufo-independence",
        state,
        independence,
    ));
}

pub(super) fn stop_independence(ctx: &UnitCtx, state: Rc<RefCell<UfoState>>) {
    if !core::mem::replace(&mut state.borrow_mut().independence, false) {
        return;
    }
    ctx.signal(SIG_INDEPENDENCE);
    set_beam(ctx, &state, 0.0);
}

async fn independence(state: Rc<RefCell<UfoState>>, ctx: UnitCtx) {
    ctx.set_signal_mask(SIG_INDEPENDENCE);
    loop {
        let piece = state.borrow().pieces.independence;
        if piece != Piece::INVALID {
            ctx.emit_sfx(piece, SFX_FIRE_WEAPON + ufo_slots::GOD_GUN);
        }
        ctx.sleep(spring::cus::Duration::from_millis(33)).await;
    }
}

pub(super) fn set_beam(ctx: &UnitCtx, state: &RefCell<UfoState>, beam: f32) {
    if state.borrow().beam == Some(beam) {
        return;
    }
    state.borrow_mut().beam = Some(beam);
    spring::set_unit_rules_param_f32(ctx.unit(), "beam_enabled", beam, PUBLIC)
        .log_err_once("Area 17 UFO script: set unit rules param f32");
}

pub(super) fn aim(ctx: &UnitCtx, state: &RefCell<UfoState>) {
    let (pieces, control, initial) = {
        let state = state.borrow();
        (state.pieces, state.control, state.gun_heading)
    };
    if let (Some(control), Some(initial)) = (control, initial) {
        let aim = control.aim;
        aim_weapons(ctx, &pieces, &initial, aim.x, aim.y, aim.z);
    }
}

pub(super) fn aim_weapons(
    ctx: &UnitCtx,
    pieces: &UfoPieces,
    initial: &[f32; 4],
    tx: f32,
    ty: f32,
    tz: f32,
) {
    for (index, &start) in initial.iter().enumerate() {
        let base = pieces.bases[index];
        let gun = pieces.guns[index];
        let Some(base_position) =
            spring::units_pieces::get_unit_piece_pos_dir(ctx.unit().0, base.0 + 1).ok()
        else {
            continue;
        };
        let dx = tx - base_position.position.x;
        let dy = ty - base_position.position.y;
        let dz = tz - base_position.position.z;
        let pitch = dy.atan2((dx * dx + dz * dz).sqrt());
        let heading = dx.atan2(dz) + gun_heading(ctx, pieces, index) - start;
        turn(ctx, gun, Axis::X, -pitch, 0.0);
        turn(ctx, base, Axis::Z, heading, 0.0);
    }
}
