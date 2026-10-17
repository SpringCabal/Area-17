use core::cell::Cell;

use spring::gl;
use spring_addons::ui::Widget;

use crate::state::Global;

const CONE_SEGMENTS: usize = 25;
const CONE_TOP_RADIUS: f32 = 10.0;
const MAX_UFO_HEIGHT: f32 = 350.0;

#[derive(Clone, Copy, Default)]
struct ConeParams {
    x: f32,
    y: f32,
    z: f32,
    height: f32,
    radius: f32,
    top_radius: f32,
    color: [f32; 4],
}

#[derive(Default)]
pub struct GfxVolumesWidget {
    seconds: Cell<f32>,
}

impl GfxVolumesWidget {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Widget<Global> for GfxVolumesWidget {
    fn name(&self) -> &'static str {
        "GfxVolumes"
    }

    fn update(&self, _ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>, dt: f32) {
        self.seconds.set(self.seconds.get() + dt.max(0.0));
    }

    fn draw_world(&self, ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>) {
        let spring = ctx.spring();
        let (ufo_id, abduction_radius) = {
            let simulation = ctx.global().simulation.access();
            (simulation.ufo_id, simulation.abduction_radius)
        };
        if ufo_id < 0 || abduction_radius <= 0.0 {
            return;
        }
        let beam = spring
            .rules_params()
            .get_unit_rules_param_f32(ufo_id, "beam_enabled")
            .unwrap_or(0.0);
        if beam < 0.0 {
            return;
        }
        let Ok(position) = spring
            .unit_rendering()
            .get_unit_view_position(ufo_id, false)
        else {
            return;
        };
        let ground = spring
            .terrain()
            .get_ground_height(position.x, position.z)
            .unwrap_or(0.0)
            .max(0.0);
        let height = position.y - ground;
        if height <= 0.0 {
            return;
        }

        let active = beam > 0.5;
        let (radius_oscillation, period, base_alpha, alpha_oscillation) = if active {
            (0.15, 3.0, 1.0, -0.2)
        } else {
            (0.05, 15.0, 0.6, 0.0)
        };
        let pulse = (self.seconds.get() * 30.0 / period).sin();
        let normal =
            (1.0 + radius_oscillation * pulse) * abduction_radius * height / MAX_UFO_HEIGHT;
        let alpha = base_alpha + alpha_oscillation * pulse;
        draw_cone(
            spring,
            position.x,
            ground,
            position.z,
            height,
            normal,
            CONE_TOP_RADIUS,
            [1.0, 0.9, 0.0, alpha],
        );
    }
}

fn draw_cone(
    spring: &spring::Spring<'_, spring::Ui>,
    x: f32,
    y: f32,
    z: f32,
    height: f32,
    radius: f32,
    top_radius: f32,
    color: [f32; 4],
) {
    let cone = ConeParams {
        x,
        y,
        z,
        height,
        radius: radius.max(1.0),
        top_radius: top_radius.max(0.0),
        color,
    };

    let _ = spring.gfx().alpha_test(true, gl::GREATER, 0.0);
    let _ = spring.gfx().depth_test(spring::gfx::GfxDepthTestOptions {
        enable: true,
        set_func: false,
        func: 0,
    });
    let _ = spring.gfx().culling(true);
    let _ = spring.gfx().begin_end(gl::TRIANGLE_STRIP, |spring| {
        emit_cone_vertices(spring, &cone);
    });

    let _ = spring.gfx().alpha_test(false, gl::GREATER, 0.0);
    let _ = spring.gfx().depth_test(spring::gfx::GfxDepthTestOptions {
        enable: false,
        set_func: false,
        func: 0,
    });
    let _ = spring.gfx().culling(false);
}

fn emit_cone_vertices(spring: &spring::Spring<'_, spring::Ui>, cone: &ConeParams) {
    for index in 0..=CONE_SEGMENTS {
        let angle = core::f32::consts::TAU * index as f32 / CONE_SEGMENTS as f32;
        let sin = angle.sin();
        let cos = angle.cos();
        emit_vertex(
            spring,
            cone.x + cone.top_radius * sin,
            cone.y + cone.height,
            cone.z + cone.top_radius * cos,
            cone.color,
            1.0,
        );
        emit_vertex(
            spring,
            cone.x + cone.radius * sin,
            cone.y,
            cone.z + cone.radius * cos,
            cone.color,
            0.0,
        );
    }
}

fn emit_vertex(
    spring: &spring::Spring<'_, spring::Ui>,
    x: f32,
    y: f32,
    z: f32,
    color: [f32; 4],
    alpha: f32,
) {
    let _ = spring
        .gfx()
        .color(color[0], color[1], color[2], color[3] * alpha);
    let _ = spring.gfx().vertex(x, y, z, 1.0, 3);
}
