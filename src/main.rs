use bine::{
    core::{Engine, Game},
    renderer::{Renderer, RendererBackends, Vertex},
    window::WindowConfig,
};

use cgmath;
use winit::event_loop::{ControlFlow, EventLoop};

const WINDOW_WIDTH: u32 = 1980;
const WINDOW_HEIGHT: u32 = 1680;
const ASPECT_RATIO: f32 = (WINDOW_WIDTH / WINDOW_HEIGHT) as f32;

struct DemoGame {}
impl Game for DemoGame {
    fn on_init(&mut self, renderer: &mut Renderer) {
        renderer.set_geometry(Self::VERTICES, Self::INDICES);

        let bytes = include_bytes!("../assets/textures/happy-tree.png");
        renderer.load_texture(bytes, "happy-tree");

        renderer.set_camera(
            (0.0, 1.0, 2.0).into(),
            (0.0, 0.0, 0.0).into(),
            cgmath::Vector3::unit_y(),
            ASPECT_RATIO,
            45.0,
            0.1,
            100.0,
        );
    }

    fn on_update(&mut self, dt: f32, input: &bine::input::Input) {}

    fn on_draw(&mut self, renderer: &mut Renderer) {
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
        Self {}
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
