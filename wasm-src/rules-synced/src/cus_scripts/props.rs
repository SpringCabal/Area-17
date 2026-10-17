use super::*;

#[derive(Clone, Copy)]
pub(super) struct DronePieces {
    pub(super) turret: Piece,
    pub(super) base: Piece,
    pub(super) muzzle: Piece,
}

#[derive(Clone, Copy)]
pub(super) struct TankPieces {
    pub(super) barrels: [Piece; 2],
    pub(super) current_barrel: usize,
    pub(super) turret: Piece,
    pub(super) tubes: [Piece; 2],
    pub(super) all: [Piece; 4],
}

#[derive(Clone, Copy)]
pub(super) struct StaticAaPieces {
    pub(super) barrel: Piece,
    pub(super) muzzle: Piece,
    pub(super) stalk: Piece,
    pub(super) turret: Piece,
    pub(super) all: [Piece; 4],
}

pub(super) struct BuildingState {
    pub(super) base: Piece,
    pub(super) blocks: Vec<Piece>,
}

#[derive(Clone, Copy)]
pub(super) struct HelipadPieces {
    pub(super) button: Piece,
    pub(super) middle: Piece,
}

pub(super) struct HelipadState {
    pub(super) pieces: HelipadPieces,
    pub(super) target: f32,
}

const BUILDING_BLOCKS: [&str; 9] = [
    "Cube_cell",
    "Cube_cell_001",
    "Cube_cell_002",
    "Cube_cell_004",
    "Cube_cell_005",
    "Cube_cell_006",
    "Cube_cell_007",
    "Cube_cell_008",
    "Cube_cell_009",
];

impl DronePieces {
    pub(super) fn new(ctx: &InitCtx<'_>) -> Self {
        Self {
            turret: piece(ctx, "turret"),
            base: piece(ctx, "base"),
            muzzle: piece(ctx, "muzzle"),
        }
    }
}

impl TankPieces {
    pub(super) fn new(ctx: &InitCtx<'_>) -> Self {
        Self {
            barrels: [piece(ctx, "muzzleLeft"), piece(ctx, "muzzleRight")],
            current_barrel: 1,
            turret: piece(ctx, "turret"),
            tubes: [piece(ctx, "tubeLeft"), piece(ctx, "tubeRight")],
            all: [
                piece(ctx, "tubeLeft"),
                piece(ctx, "tubeRight"),
                piece(ctx, "turret"),
                piece(ctx, "base"),
            ],
        }
    }
}

impl StaticAaPieces {
    pub(super) fn new(ctx: &InitCtx<'_>) -> Self {
        Self {
            barrel: piece(ctx, "barrel"),
            muzzle: piece(ctx, "muzzle"),
            stalk: piece(ctx, "stalk"),
            turret: piece(ctx, "turret"),
            all: [
                piece(ctx, "barrel"),
                piece(ctx, "turret"),
                piece(ctx, "stalk"),
                piece(ctx, "base"),
            ],
        }
    }
}

impl BuildingState {
    pub(super) fn new(ctx: &InitCtx<'_>) -> Self {
        Self {
            base: piece(ctx, "Cube"),
            blocks: BUILDING_BLOCKS
                .iter()
                .map(|name| piece(ctx, name))
                .collect(),
        }
    }
}

impl BuildingState {
    pub(super) fn shatter(&self, ctx: &UnitCtx, flags: SfxFlags) {
        ctx.hide(self.base);
        for block in self.blocks.iter().copied() {
            if block != Piece::INVALID {
                ctx.show(block);
                explode(ctx, block, flags);
            }
        }
    }
}

impl HelipadState {
    pub(super) fn new(ctx: &InitCtx<'_>) -> Self {
        Self {
            pieces: HelipadPieces {
                button: piece(ctx, "button"),
                middle: piece(ctx, "middle"),
            },
            target: 0.0,
        }
    }
}

pub(super) fn move_helipad(
    ctx: &UnitCtx,
    state: Rc<RefCell<HelipadState>>,
    target: f32,
    task: &'static str,
) {
    state.borrow_mut().target = target;
    ctx.signal(SIG_HELIPAD);
    ctx.spawn(TaskDefinition::with_state(task, state, move_pad));
}

async fn move_pad(state: Rc<RefCell<HelipadState>>, ctx: UnitCtx) {
    ctx.set_signal_mask(SIG_HELIPAD);
    let (pieces, target) = {
        let state = state.borrow();
        (state.pieces, state.target)
    };
    play_sound(&ctx, "sounds/click.ogg", 1.0);
    move_piece(&ctx, pieces.button, Axis::Z, target, 150.0);
    move_piece(
        &ctx,
        pieces.middle,
        Axis::Z,
        if target == 0.0 { 0.0 } else { -5.0 },
        50.0,
    );
    ctx.wait_for_move(pieces.button, Axis::Z).await;
    ctx.wait_for_move(pieces.middle, Axis::Z).await;
}

pub(super) async fn aim_tank(state: Rc<RefCell<(TankPieces, WeaponId, f32, f32)>>, ctx: UnitCtx) {
    ctx.set_signal_mask(SIG_AIM);
    let (tank, weapon, heading, pitch) = *state.borrow();
    turn(
        &ctx,
        tank.turret,
        Axis::Z,
        heading,
        AngularSpeed::degrees_per_second(100.0).0,
    );
    for tube in tank.tubes {
        turn(
            &ctx,
            tube,
            Axis::X,
            -pitch,
            AngularSpeed::degrees_per_second(50.0).0,
        );
    }
    ctx.wait_for_turn(tank.turret, Axis::Z).await;
    for tube in tank.tubes {
        ctx.wait_for_turn(tube, Axis::X).await;
    }
    ctx.set_aim_ready(weapon, true);
}

pub(super) async fn aim_aa(state: Rc<RefCell<(StaticAaPieces, WeaponId, f32, f32)>>, ctx: UnitCtx) {
    ctx.set_signal_mask(SIG_AIM);
    let (aa, weapon, heading, pitch) = *state.borrow();
    turn(
        &ctx,
        aa.stalk,
        Axis::Z,
        heading,
        AngularSpeed::degrees_per_second(100.0).0,
    );
    turn(
        &ctx,
        aa.turret,
        Axis::X,
        -pitch,
        AngularSpeed::degrees_per_second(50.0).0,
    );
    ctx.wait_for_turn(aa.stalk, Axis::Z).await;
    ctx.wait_for_turn(aa.turret, Axis::X).await;
    ctx.set_aim_ready(weapon, true);
}
