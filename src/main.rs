use bine::{
    core::{Engine, Game},
    renderer::{Camera, Renderer, RendererBackends, Vertex},
    window::WindowConfig,
};

use cgmath::{self, InnerSpace, Point3, Rad, Vector3, num_traits::Float};
use winit::event_loop::{ControlFlow, EventLoop};

const WINDOW_WIDTH: u32 = 1980;
const WINDOW_HEIGHT: u32 = 1680;
const ASPECT_RATIO: f32 = (WINDOW_WIDTH / WINDOW_HEIGHT) as f32;

struct DemoGame {
    camera_controller: CameraController,
}
impl Game for DemoGame {
    fn on_init(&mut self, renderer: &mut Renderer) {
        renderer.set_geometry(Self::VERTICES, Self::INDICES);

        let bytes = include_bytes!("../assets/textures/happy-tree.png");
        renderer.load_texture(bytes, "happy-tree");

        renderer.set_camera(
            (0.0, 1.0, 2.0).into(),
            (0.0, 0.0, 0.0).into(),
            Vector3::unit_y(),
            ASPECT_RATIO,
            45.0,
            0.1,
            100.0,
        );
    }

    fn on_update(&mut self, dt: f32, input: &bine::input::Input) {
        if let Some((position_delta, scroll_delta)) =
            input.mouse_position_delta().zip(input.mouse_scroll_delta())
        {
            self.camera_controller
                .process_mouse(position_delta, scroll_delta);
        }
    }

    fn on_draw(&mut self, renderer: &mut Renderer) {
        renderer.update_camera(&self.camera_controller.to_camera());
        renderer.render(120.0, 250.0, 88.0);
    }
}

impl DemoGame {
    const VERTICES: &[Vertex] = &[
        Vertex::new([-0.0868241, 0.49240386, 0.0], [0.4131759, 1.0 - 0.99240386]),
        Vertex::new(
            [-0.49513406, 0.06958647, 0.0],
            [0.0048659444, 1.0 - 0.56958647],
        ),
        Vertex::new(
            [-0.21918549, -0.44939706, 0.0],
            [0.28081453, 1.0 - 0.05060294],
        ),
        Vertex::new([0.35966998, -0.3473291, 0.0], [0.85967, 1.0 - 0.1526709]),
        Vertex::new([0.44147372, 0.2347359, 0.0], [0.9414737, 1.0 - 0.7347359]),
    ];

    const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];

    fn new() -> Self {
        Self {
            camera_controller: CameraController::new(
                0.005,
                f32::atan2(0.0, 2.0),
                (1.0 / Vector3::new(0.0, 1.0, 2.0).magnitude()).asin(),
                Vector3::new(0.0, 1.0, 2.0).magnitude(),
                (0.0, 0.0, 0.0),
            ),
        }
    }
}

// Orbit camera : https://learnopengl.com/Getting-started/Camera
struct CameraController {
    sensitivity: f32,    // mouse sensitivity in this case
    yaw: f32,            // current horizontal angle
    pitch: f32,          // current vertical angle
    radius: f32,         // distance from target
    target: Point3<f32>, // the fixed point we orbit around
}
impl CameraController {
    const RADIUS_MAX: f32 = 20.0;
    const RADIUS_MIN: f32 = 1.0;
    const PITCH_MAX: f32 = 89.0;

    fn new(sensitivity: f32, yaw: f32, pitch: f32, radius: f32, target: (f32, f32, f32)) -> Self {
        Self {
            sensitivity: sensitivity, // default sensitivity
            yaw: yaw,
            pitch: pitch,
            radius: radius,
            target: target.into(), // initially object at the centre
        }
    }

    fn process_mouse(&mut self, position_delta: (f64, f64), scroll_delta: f32) {
        self.yaw += position_delta.0 as f32 * self.sensitivity;

        self.pitch -= position_delta.1 as f32 * self.sensitivity;
        self.pitch = self.pitch.clamp(-Self::PITCH_MAX, Self::PITCH_MAX);

        self.radius -= scroll_delta * self.sensitivity;
        self.radius = self.radius.clamp(Self::RADIUS_MIN, Self::RADIUS_MAX);
    }

    fn process_keys() {}

    fn to_camera(&self) -> Camera {
        let eye = (
            self.target.x + self.radius * self.pitch.cos() * self.yaw.sin(),
            self.target.y + self.radius * self.pitch.sin(),
            self.target.z + self.radius * self.pitch.cos() * self.yaw.cos(),
        );
        Camera::new(
            eye.into(),
            self.target.clone(),
            Vector3::unit_y(),
            ASPECT_RATIO,
            45.0,
            0.1,
            100.0,
        )
    }
}

fn main() {
    // Demo game to test created modules
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    event_loop.set_control_flow(ControlFlow::Wait);

    let config = WindowConfig {
        title: "Demo Game".into(),
        width: WINDOW_WIDTH,
        height: WINDOW_HEIGHT,
        resizable: true,
        vsync: true,
        fullscreen: false,
    };
    let game = DemoGame::new();
    let mut engine = Engine::new(config, RendererBackends::Metal, game);
    let _ = event_loop.run_app(&mut engine);
}
