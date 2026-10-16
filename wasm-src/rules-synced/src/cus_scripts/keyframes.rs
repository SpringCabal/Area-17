use super::human::HumanPieces;
use super::*;

pub(super) async fn walk_cycle(ctx: &UnitCtx, p: HumanPieces) {
    walk_frame(ctx, p, 0);
    ctx.sleep(spring::cus::Duration::from_millis(165)).await;
    walk_frame(ctx, p, 5);
    ctx.sleep(spring::cus::Duration::from_millis(165)).await;
    walk_frame(ctx, p, 10);
    ctx.sleep(spring::cus::Duration::from_millis(165)).await;
    walk_frame(ctx, p, 15);
    ctx.sleep(spring::cus::Duration::from_millis(165)).await;
    walk_frame(ctx, p, 20);
    ctx.sleep(spring::cus::Duration::from_millis(165)).await;
    walk_frame(ctx, p, 25);
    ctx.sleep(spring::cus::Duration::from_millis(165)).await;
    walk_frame(ctx, p, 30);
    ctx.sleep(spring::cus::Duration::from_millis(165)).await;
}

fn walk_frame(ctx: &UnitCtx, p: HumanPieces, frame: u32) {
    match frame {
        0 => {
            turn(ctx, p.foot_left, Axis::X, -0.017993, 1.127545);
            turn(ctx, p.foot_left, Axis::Y, 0.0, 0.000004);
            turn(ctx, p.foot_left, Axis::Z, 0.0, 0.0);
            turn(ctx, p.foot_right, Axis::X, 0.327514, 1.310_22);
            turn(ctx, p.foot_right, Axis::Y, 0.0, 0.0);
            turn(ctx, p.foot_right, Axis::Z, 0.0, 0.0);
            turn(ctx, p.leg_left, Axis::X, 0.2798, 0.271893);
            turn(ctx, p.leg_left, Axis::Y, 0.0, 0.0);
            turn(ctx, p.leg_left, Axis::Z, 0.0, 0.000006);
            turn(ctx, p.leg_right, Axis::X, 0.748507, 1.134014);
            turn(ctx, p.leg_right, Axis::Y, 0.0, 0.0);
            turn(ctx, p.leg_right, Axis::Z, 0.0, 0.0);
            turn(ctx, p.thigh_left, Axis::X, 0.188896, 1.605548);
            turn(ctx, p.thigh_left, Axis::Y, 0.0, 0.0);
            turn(ctx, p.thigh_left, Axis::Z, 0.0, 0.0);
            turn(ctx, p.thigh_right, Axis::X, -0.865104, 1.258808);
            turn(ctx, p.thigh_right, Axis::Y, 0.0, 0.0);
            turn(ctx, p.thigh_right, Axis::Z, 0.0, 0.0);
            turn(ctx, p.torso, Axis::X, 0.143826, 0.993616);
            turn(ctx, p.torso, Axis::Y, 0.0, 0.0);
            turn(ctx, p.torso, Axis::Z, 0.0, 0.0);
        }
        5 => {
            pose_with_speeds(
                ctx,
                p.hand_right,
                -0.083862,
                -0.011926,
                0.282358,
                0.251587,
                0.035777,
                0.847075,
            );
            turn(ctx, p.torso, Axis::X, 0.015888, 0.383815);
            turn(ctx, p.leg_right, Axis::X, 1.002721, 1.525281);
            turn(ctx, p.thigh_left, Axis::X, 0.563635, 2.248429);
            turn(ctx, p.thigh_right, Axis::X, -0.763012, 0.612550);
            turn(ctx, p.foot_left, Axis::X, 0.348259, 2.197513);
            pose_with_speeds(
                ctx, p.arm_left, 0.061899, 1.309002, 0.226892, 0.371395, 0.042328, 1.361354,
            );
            pose_with_speeds(
                ctx,
                p.hand_left,
                -0.029538,
                0.001588,
                -0.107445,
                0.355267,
                0.125883,
                1.392_45,
            );
            turn(ctx, p.foot_right, Axis::X, -0.274326, 3.611039);
            pose_with_speeds(
                ctx,
                p.arm_right,
                0.141291,
                -1.313235,
                -0.490947,
                0.395248,
                0.155106,
                1.426826,
            );
        }
        10 => {
            turn(ctx, p.thigh_left, Axis::X, -0.174731, 4.430193);
            pose_with_speeds(
                ctx, p.arm_left, 0.130827, 1.335249, 0.497882, 0.413569, 0.157487, 1.625939,
            );
            turn(ctx, p.foot_right, Axis::X, -0.067415, 1.241466);
            turn(ctx, p.leg_right, Axis::X, -0.175961, 7.072088);
            turn(ctx, p.leg_left, Axis::X, 1.291285, 6.068907);
            turn(ctx, p.leg_left, Axis::Z, 0.000002, 0.000009);
            turn(ctx, p.thigh_right, Axis::X, 0.28405, 6.282371);
            pose_with_speeds(
                ctx,
                p.arm_right,
                0.030269,
                -1.279306,
                -0.100528,
                0.666131,
                0.203577,
                2.342509,
            );
        }
        15 => {
            pose_with_speeds(
                ctx,
                p.hand_right,
                -0.394076,
                -0.136382,
                0.875564,
                1.861282,
                0.746738,
                3.559236,
            );
            pose_with_speeds(
                ctx, p.arm_left, 0.160778, 1.35438, 0.629358, 0.179706, 0.114785, 0.788856,
            );
            turn(ctx, p.thigh_left, Axis::X, -0.921744, 4.482_08);
            pose_with_speeds(
                ctx,
                p.hand_left,
                -0.228648,
                0.126173,
                -1.005911,
                1.194664,
                0.747509,
                5.390_8,
            );
            turn(ctx, p.leg_left, Axis::X, 0.699027, 3.553546);
            turn(ctx, p.thigh_right, Axis::X, 0.567365, 1.699895);
            pose_with_speeds(
                ctx,
                p.arm_right,
                -0.149356,
                -1.034521,
                0.306338,
                1.077749,
                1.468_71,
                2.441201,
            );
            turn(ctx, p.torso, Axis::X, 0.080285, 0.386382);
        }
        20 => {
            pose_with_speeds(
                ctx,
                p.hand_right,
                0.019244,
                -0.012816,
                0.111346,
                2.479919,
                0.741396,
                4.585312,
            );
            pose_with_speeds(
                ctx, p.arm_left, 0.030734, 1.303667, 0.111819, 0.780264, 0.304279, 3.105237,
            );
            turn(ctx, p.thigh_left, Axis::X, -0.633908, 1.727017);
            pose_with_speeds(
                ctx,
                p.hand_left,
                -0.06691,
                0.008264,
                -0.245676,
                0.970432,
                0.707458,
                4.561413,
            );
            turn(ctx, p.foot_right, Axis::X, -0.178527, 0.666673);
            turn(ctx, p.leg_right, Axis::X, -0.139711, 0.217499);
            turn(ctx, p.leg_left, Axis::X, 0.256962, 2.652389);
            turn(ctx, p.thigh_right, Axis::X, 0.648183, 0.484904);
            pose_with_speeds(
                ctx,
                p.arm_right,
                -0.144222,
                -1.227105,
                0.373266,
                0.030804,
                1.155502,
                0.401563,
            );
            turn(ctx, p.torso, Axis::X, -0.00844, 0.532354);
        }
        25 => {
            pose_with_speeds(
                ctx, p.arm_left, -0.038464, 1.304647, -0.140134, 0.415193, 0.005877, 1.511713,
            );
            turn(ctx, p.thigh_left, Axis::X, -0.396715, 1.423158);
            pose_with_speeds(
                ctx,
                p.hand_left,
                -0.120765,
                0.028061,
                -0.45611,
                0.323130,
                0.118783,
                1.262604,
            );
            turn(ctx, p.foot_right, Axis::X, 0.109143, 1.726025);
            turn(ctx, p.leg_right, Axis::X, 1.254807, 8.367105);
            turn(ctx, p.leg_left, Axis::X, 0.801964, 3.270011);
            turn(ctx, p.leg_left, Axis::Z, 0.000001, 0.000002);
            turn(ctx, p.thigh_right, Axis::X, -0.29744, 5.673739);
            pose_with_speeds(
                ctx,
                p.arm_right,
                0.082761,
                -1.208097,
                -0.223086,
                1.361899,
                0.114048,
                3.578108,
            );
            turn(ctx, p.foot_left, Axis::X, -0.205917, 3.325058);
            turn(ctx, p.foot_left, Axis::Y, 0.000001, 0.000004);
            turn(ctx, p.foot_left, Axis::Z, 0.0, 0.000001);
            turn(ctx, p.torso, Axis::X, 0.062122, 0.423377);
        }
        30 => {
            pose_with_speeds(
                ctx,
                p.hand_right,
                0.0,
                0.0,
                0.0,
                0.115463,
                0.076896,
                0.668074,
            );
            pose_with_speeds(
                ctx, p.arm_left, 0.0, 1.301947, 0.0, 0.230786, 0.016198, 0.840801,
            );
            turn(ctx, p.thigh_left, Axis::X, 0.456488, 5.119217);
            pose_with_speeds(
                ctx,
                p.hand_left,
                -0.147960,
                0.043549,
                -0.571595,
                0.163171,
                0.092931,
                0.692909,
            );
            turn(ctx, p.leg_right, Axis::X, 0.559505, 4.171811);
            turn(ctx, p.leg_left, Axis::X, 0.325116, 2.861089);
            turn(ctx, p.leg_left, Axis::Z, 0.000001, 0.000001);
            turn(ctx, p.thigh_right, Axis::X, -0.655302, 2.147172);
            pose_with_speeds(
                ctx,
                p.arm_right,
                0.075416,
                -1.287384,
                -0.253142,
                0.044072,
                0.475726,
                0.180339,
            );
            turn(ctx, p.torso, Axis::X, -0.021776, 0.503391);
        }
        _ => {}
    }
}

fn pose(ctx: &UnitCtx, piece: Piece, x: f32, y: f32, z: f32, speed: f32) {
    turn(ctx, piece, Axis::X, x, speed);
    turn(ctx, piece, Axis::Y, y, speed);
    turn(ctx, piece, Axis::Z, z, speed);
}

fn pose_with_speeds(
    ctx: &UnitCtx,
    piece: Piece,
    x: f32,
    y: f32,
    z: f32,
    speed_x: f32,
    speed_y: f32,
    speed_z: f32,
) {
    turn(ctx, piece, Axis::X, x, speed_x);
    turn(ctx, piece, Axis::Y, y, speed_y);
    turn(ctx, piece, Axis::Z, z, speed_z);
}

pub(super) async fn float_cycle(ctx: &UnitCtx, p: HumanPieces) {
    pose(ctx, p.hand_right, 0.39917, 0.56053, 0.200966, 3.0);
    turn(ctx, p.torso, Axis::X, -0.188606, 3.0);
    pose(ctx, p.arm_left, -0.209676, 0.148263, 0.151346, 3.0);
    pose(ctx, p.hand_left, -0.406562, -0.86927, 0.38091, 3.0);
    pose(ctx, p.arm_right, 0.046031, -0.438708, 0.065776, 3.0);
    pose(ctx, p.leg_right, 0.0, 0.0, 0.0, 3.0);
    pose(ctx, p.foot_left, 0.0, 0.0, 0.0, 3.0);
    pose(ctx, p.foot_right, 0.0, 0.0, 0.0, 3.0);
    turn(ctx, p.thigh_left, Axis::X, -0.19593, 3.0);
    turn(ctx, p.thigh_left, Axis::Y, -0.55965, 3.0);
    turn(ctx, p.thigh_left, Axis::Z, 0.059407, 3.0);
    turn(ctx, p.thigh_right, Axis::X, -0.072063, 3.0);
    turn(ctx, p.thigh_right, Axis::Y, 0.720350, 3.0);
    turn(ctx, p.thigh_right, Axis::Z, -0.102, 3.0);
    pose(ctx, p.leg_left, 0.406139, -0.098387, 0.042185, 3.0);
    pose(ctx, p.head, -0.333966, 0.0, 0.0, 3.0);
    ctx.sleep(spring::cus::Duration::from_millis(165)).await;

    pose(ctx, p.hand_right, 0.356665, 0.421951, 0.22279, 3.0);
    turn(ctx, p.torso, Axis::X, 0.255423, 3.0);
    pose(ctx, p.arm_left, -0.291422, -0.309976, 0.219926, 3.0);
    pose(ctx, p.hand_left, -0.433775, -0.963505, 0.402936, 3.0);
    pose(ctx, p.arm_right, 0.036445, -0.522845, 0.083845, 3.0);
    pose(ctx, p.leg_right, 0.803807, 0.0, 0.0, 3.0);
    pose(ctx, p.foot_left, 0.543768, 0.0, 0.0, 3.0);
    turn(ctx, p.thigh_left, Axis::X, -1.096917, 3.0);
    turn(ctx, p.thigh_left, Axis::Y, -1.093454, 3.0);
    turn(ctx, p.thigh_left, Axis::Z, 0.349626, 3.0);
    turn(ctx, p.thigh_right, Axis::X, 0.084322, 3.0);
    turn(ctx, p.thigh_right, Axis::Y, 0.704352, 3.0);
    turn(ctx, p.thigh_right, Axis::Z, -0.102098, 3.0);
    pose(ctx, p.leg_left, 1.068988, -0.044336, 0.080630, 3.0);
    ctx.sleep(spring::cus::Duration::from_millis(165)).await;

    turn(ctx, p.torso, Axis::X, 0.074989, 3.0);
    pose(ctx, p.arm_left, -0.244215, -1.249632, 0.269414, 3.0);
    pose(ctx, p.hand_left, -0.170446, -0.021515, 0.506482, 3.0);
    pose(ctx, p.arm_right, -0.412074, -0.775589, 0.670168, 3.0);
    pose(ctx, p.leg_left, 1.320951, -1.728395, -2.174936, 3.0);
    turn(ctx, p.thigh_left, Axis::X, -0.852750, 3.0);
    turn(ctx, p.thigh_left, Axis::Y, -0.055582, 3.0);
    turn(ctx, p.thigh_left, Axis::Z, -0.236783, 3.0);
    turn(ctx, p.thigh_right, Axis::X, 0.538137, 3.0);
    turn(ctx, p.thigh_right, Axis::Y, 0.850249, 3.0);
    turn(ctx, p.thigh_right, Axis::Z, 0.354826, 3.0);
    ctx.sleep(spring::cus::Duration::from_millis(165)).await;

    pose(ctx, p.torso, -0.08102, 0.048042, -0.0233, 3.0);
    pose(ctx, p.arm_left, -0.384786, -0.444913, -0.012617, 3.0);
    pose(ctx, p.arm_right, -0.202095, -0.355504, 0.230652, 3.0);
    pose(ctx, p.leg_left, 0.832462, -0.102876, -0.135162, 3.0);
    turn(ctx, p.thigh_left, Axis::X, -0.609309, 3.0);
    turn(ctx, p.thigh_left, Axis::Y, -0.585684, 3.0);
    turn(ctx, p.thigh_left, Axis::Z, -0.347320, 3.0);
    turn(ctx, p.thigh_right, Axis::X, -0.553702, 3.0);
    turn(ctx, p.thigh_right, Axis::Y, 0.676137, 3.0);
    turn(ctx, p.thigh_right, Axis::Z, 0.007015, 3.0);
    ctx.sleep(spring::cus::Duration::from_millis(165)).await;

    pose(ctx, p.torso, -0.171763, -0.053148, 0.014289, 3.0);
    pose(ctx, p.arm_left, -0.419403, 3.255855, -2.436838, 3.0);
    pose(ctx, p.arm_right, -0.516594, -1.16569, 0.837595, 3.0);
    turn(ctx, p.thigh_left, Axis::X, -0.302235, 3.0);
    turn(ctx, p.thigh_left, Axis::Y, 0.024422, 3.0);
    turn(ctx, p.thigh_left, Axis::Z, -0.635688, 3.0);
    turn(ctx, p.thigh_right, Axis::X, -0.647243, 3.0);
    turn(ctx, p.thigh_right, Axis::Y, 0.058210, 3.0);
    turn(ctx, p.thigh_right, Axis::Z, 0.114611, 3.0);
    ctx.sleep(spring::cus::Duration::from_millis(165)).await;
}
