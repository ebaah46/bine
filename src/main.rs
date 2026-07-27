use std::sync::{Arc, RwLock};

use bine::{
    core::{Engine, Game},
    input::{Gamepad, InputSystem, Keyboard, Mouse, UniversalAxes},
    renderer::{Camera, Renderer, RendererBackends},
    window::WindowConfig,
};

use cgmath::{self, InnerSpace, Point3, Vector3, num_traits::Float};
use log::info;
use winit::event_loop::{ControlFlow, EventLoop};

const WINDOW_WIDTH: u32 = 1980;
const WINDOW_HEIGHT: u32 = 1680;
const ASPECT_RATIO: f32 = (WINDOW_WIDTH / WINDOW_HEIGHT) as f32;

struct DemoGame {
    camera_controller: CameraController,
    gamepad: Option<Arc<RwLock<Gamepad>>>,
}
impl Game for DemoGame {
    fn on_init(&mut self, renderer: &mut Renderer, input: &mut InputSystem) {
        // setting supported devices
        let source = Arc::new(RwLock::new(Mouse::default()));
        input.add_source(source);
        let source = Arc::new(RwLock::new(Keyboard::default()));
        input.add_source(source);

        let gamepad_raw = Gamepad::default();
        let gamepad = Arc::new(RwLock::new(gamepad_raw));
        self.gamepad = Some(gamepad.clone());
        input.add_source(gamepad);

        let model_paths = vec!["assets/models/cube/cube.obj"];
        renderer
            .set_models_to_load(&model_paths)
            .expect("Failed to load models");

        renderer.set_camera(
            (0.0, 1.0, 2.0).into(),
            (0.0, 0.0, 0.0).into(),
            Vector3::unit_y(),
            ASPECT_RATIO,
            45.0,
            0.1,
            100.0,
        );

        renderer.set_light_properties(&Self::LIGHT_POS, &Self::LIGHT_COLOR);
    }

    fn on_update(&mut self, dt: f32, input: &bine::input::InputSystem) {
        let snapshot = input.input_buffer.generate_snapshot();

        info!("snapshot axes size:{}", snapshot.axes_moved.len());

        // we need a mechanism to determine prefered controller
        // but for now, if gamepad is not connected, we will default to keyboard on mouse
        if let Some(gamepad_arc) = self.gamepad.as_ref() {
            if let Ok(gamepad_lock) = gamepad_arc.read() {
                if let Some((movement, _)) = gamepad_lock.active_gamepad.and_then(|id| {
                    self.camera_controller.sensitivity = Self::GAMEPAD_SENSITIVITY;
                    let x = snapshot.axes_moved(UniversalAxes::GamepadRightStickX(id))?;
                    let y = snapshot.axes_moved(UniversalAxes::GamepadRightStickY(id))?;
                    Some(((x, y), id))
                }) {
                    info!(
                        "controlling camera with gamepad:({},{})",
                        movement.0, movement.1
                    );
                    self.camera_controller.process_gamepad(movement);
                }
            }
        } else {
            if let Some(mouse_pos) = snapshot
                .axes_moved(bine::input::UniversalAxes::MouseMovementX)
                .zip(snapshot.axes_moved(UniversalAxes::MouseMovementY))
            {
                let scroll_delta = {
                    snapshot
                        .axes_moved(UniversalAxes::MouseScrollMovement)
                        .unwrap_or_default()
                };
                self.camera_controller
                    .process_mouse(mouse_pos, Some(scroll_delta));
            }
        }
    }

    fn on_draw(&mut self, renderer: &mut Renderer) {
        renderer.update_camera(&self.camera_controller.to_camera());
        renderer.render(0.0, 0.0, 0.0);
    }
}

impl DemoGame {
    const LIGHT_POS: [f32; 3] = [1.0, 0.0, 1.0];

    const LIGHT_COLOR: [f32; 3] = [1.0, 1.0, 1.0];

    const GAMEPAD_SENSITIVITY: f32 = 0.5;
    const MOUSE_SENSITIVITY: f32 = 0.0001;

    fn new() -> Self {
        let instance = Self {
            camera_controller: CameraController::new(
                Self::MOUSE_SENSITIVITY,
                f32::atan2(0.0, 2.0),
                (1.0 / Vector3::new(0.0, 1.0, 2.0).magnitude()).asin(),
                Vector3::new(0.0, 1.0, 2.0).magnitude(),
                (0.0, 0.0, 0.0),
            ),
            gamepad: None,
        };
        instance
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

    fn process_mouse(&mut self, position_delta: (f32, f32), scroll_delta: Option<f32>) {
        self.yaw += position_delta.0 as f32 * self.sensitivity;

        self.pitch -= position_delta.1 as f32 * self.sensitivity;
        self.pitch = self.pitch.clamp(-Self::PITCH_MAX, Self::PITCH_MAX);
        if let Some(scroll) = scroll_delta {
            self.radius -= scroll * self.sensitivity;
        }
        self.radius = self.radius.clamp(Self::RADIUS_MIN, Self::RADIUS_MAX);
    }

    // This is how the game uses the gamepad rightsick to controll the camera
    fn process_gamepad(&mut self, position_delta: (f32, f32)) {
        self.yaw += position_delta.0 as f32 * self.sensitivity;

        self.pitch -= position_delta.1 as f32 * self.sensitivity;
        self.pitch = self.pitch.clamp(-Self::PITCH_MAX, Self::PITCH_MAX);
        self.radius -= self.sensitivity;
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
