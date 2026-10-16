use super::keyframes::{float_cycle, walk_cycle};
use super::*;

#[derive(Clone, Copy)]
pub(super) struct HumanPieces {
    pub(super) arm_left: Piece,
    pub(super) arm_right: Piece,
    pub(super) foot_left: Piece,
    pub(super) foot_right: Piece,
    pub(super) hand_left: Piece,
    pub(super) hand_right: Piece,
    pub(super) head: Piece,
    pub(super) leg_left: Piece,
    pub(super) leg_right: Piece,
    pub(super) thigh_left: Piece,
    pub(super) thigh_right: Piece,
    pub(super) torso: Piece,
    pub(super) parachute: Piece,
}

impl HumanPieces {
    fn new(ctx: &InitCtx<'_>) -> Self {
        Self {
            arm_left: piece(ctx, "armLeft"),
            arm_right: piece(ctx, "armRight"),
            foot_left: piece(ctx, "footLeft"),
            foot_right: piece(ctx, "footRIght"),
            hand_left: piece(ctx, "handLeft"),
            hand_right: piece(ctx, "handRight"),
            head: piece(ctx, "head"),
            leg_left: piece(ctx, "legLeft"),
            leg_right: piece(ctx, "legRight"),
            thigh_left: piece(ctx, "thighLeft"),
            thigh_right: piece(ctx, "thighRight"),
            torso: piece(ctx, "torso"),
            parachute: piece(ctx, "parachute"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Pose {
    Stand,
    Walk,
    Float,
}

pub(super) struct HumanState {
    pub(super) pieces: HumanPieces,
    pub(super) pose: Pose,
    pub(super) floating: bool,
    pub(super) landing_started: bool,
    pub(super) alien: bool,
}

impl HumanPieces {
    pub(super) fn limbs(&self) -> [Piece; 11] {
        [
            self.arm_left,
            self.arm_right,
            self.foot_left,
            self.foot_right,
            self.hand_left,
            self.hand_right,
            self.head,
            self.leg_left,
            self.leg_right,
            self.thigh_left,
            self.thigh_right,
        ]
    }
}

impl HumanState {
    pub(super) fn new(ctx: &InitCtx<'_>, alien: bool) -> Self {
        Self {
            pieces: HumanPieces::new(ctx),
            pose: Pose::Stand,
            floating: false,
            landing_started: false,
            alien,
        }
    }
}

pub(super) fn create(ctx: &UnitCtx, state: Rc<RefCell<HumanState>>) {
    let pieces = state.borrow().pieces;
    stop_pose(ctx, pieces);
    if pieces.parachute != Piece::INVALID {
        ctx.hide(pieces.parachute);
    }
}

fn play(ctx: &UnitCtx, state: Rc<RefCell<HumanState>>, pose: Pose) {
    state.borrow_mut().pose = pose;
    ctx.signal(SIG_HUMAN);
    ctx.spawn(TaskDefinition::with_state(
        "area17-human-controller",
        state,
        controller,
    ));
}

pub(super) fn start_moving(ctx: &UnitCtx, state: Rc<RefCell<HumanState>>) {
    {
        let mut human = state.borrow_mut();
        if human.pose == Pose::Walk {
            return;
        }
        if !human.alien {
            human.floating = false;
        }
    }
    play(ctx, state, Pose::Walk);
}

pub(super) fn stop_moving(ctx: &UnitCtx, state: Rc<RefCell<HumanState>>) {
    {
        let mut human = state.borrow_mut();
        if human.pose == Pose::Stand {
            return;
        }
        if !human.alien {
            human.floating = false;
        }
    }
    play(ctx, state, Pose::Stand);
}

pub(super) fn stop_pose(ctx: &UnitCtx, p: HumanPieces) {
    turn(ctx, p.arm_left, Axis::X, 0.0, 0.0);
    turn(ctx, p.arm_left, Axis::Y, 1.360596, 0.0);
    turn(ctx, p.arm_left, Axis::Z, 0.0, 0.0);
    turn(ctx, p.arm_right, Axis::X, 0.0, 0.0);
    turn(ctx, p.arm_right, Axis::Y, -1.228106, 0.0);
    turn(ctx, p.arm_right, Axis::Z, 0.0, 0.0);

    turn(ctx, p.hand_left, Axis::X, 0.0, 0.0);
    turn(ctx, p.hand_left, Axis::Y, 0.0, 0.0);
    turn(ctx, p.hand_left, Axis::Z, 0.0, 0.0);
    turn(ctx, p.hand_right, Axis::X, 0.285140, 0.0);
    turn(ctx, p.hand_right, Axis::Y, -0.125746, 0.0);
    turn(ctx, p.hand_right, Axis::Z, 0.108168, 0.0);

    for piece in [
        p.foot_left,
        p.foot_right,
        p.head,
        p.leg_left,
        p.leg_right,
        p.thigh_left,
        p.thigh_right,
        p.torso,
    ] {
        for axis in [Axis::X, Axis::Y, Axis::Z] {
            turn(ctx, piece, axis, 0.0, 0.0);
        }
    }

    for piece in [
        p.arm_left,
        p.arm_right,
        p.foot_left,
        p.foot_right,
        p.hand_left,
        p.hand_right,
        p.head,
        p.leg_left,
        p.leg_right,
        p.thigh_left,
        p.thigh_right,
        p.torso,
    ] {
        for axis in [Axis::X, Axis::Y, Axis::Z] {
            move_piece(ctx, piece, axis, 0.0, 0.0);
        }
    }
}

pub(super) async fn controller(state: Rc<RefCell<HumanState>>, ctx: UnitCtx) {
    ctx.set_signal_mask(SIG_HUMAN);
    let (pose, pieces) = {
        let state = state.borrow();
        (state.pose, state.pieces)
    };
    match pose {
        Pose::Stand => stop_pose(&ctx, pieces),
        Pose::Walk => loop {
            walk_cycle(&ctx, pieces).await;
        },
        Pose::Float => {
            while state.borrow().floating {
                float_cycle(&ctx, pieces).await;
            }
        }
    }
}

pub(super) fn abduction_float(ctx: &UnitCtx, state: Rc<RefCell<HumanState>>) {
    {
        let mut human = state.borrow_mut();
        if human.floating || human.landing_started {
            return;
        }
        human.floating = true;
    }
    play_sound("sounds/scream.wav", 0.8, ctx.unit());
    play(ctx, state, Pose::Float);
}

pub(super) fn land(ctx: &UnitCtx, state: Rc<RefCell<HumanState>>) {
    let (is_alien, already_landing) = {
        let mut human = state.borrow_mut();
        let already_landing = human.landing_started;
        if human.alien && human.floating && !already_landing {
            human.landing_started = true;
        }
        (human.alien, already_landing)
    };
    if !is_alien || already_landing {
        return;
    }
    ctx.signal(SIG_HUMAN);
    ctx.spawn(TaskDefinition::with_state(
        "area17-alien-land",
        state,
        parachute_fall,
    ));
}

async fn parachute_fall(state: Rc<RefCell<HumanState>>, ctx: UnitCtx) {
    ctx.set_signal_mask(SIG_HUMAN);
    let parachute = state.borrow().pieces.parachute;
    if parachute != Piece::INVALID {
        ctx.show(parachute);
        move_piece(&ctx, parachute, Axis::Y, 45.0, 15.0);
        move_piece(&ctx, parachute, Axis::Z, -60.0, 20.0);
        ctx.wait_for_move(parachute, Axis::Y).await;
        ctx.wait_for_move(parachute, Axis::Z).await;
        ctx.hide(parachute);
    }
    {
        let mut human = state.borrow_mut();
        human.floating = false;
        human.landing_started = false;
    }
    play(&ctx, state, Pose::Stand);
}
