use bine::core::Engine;

use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    // Demo game to test created modules
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    event_loop.set_control_flow(ControlFlow::Wait);
    let mut engine = Engine::new(
        "Demo Game",
        1980,
        1680,
        bine::renderer::RendererBackends::Metal,
    );
    let _ = event_loop.run_app(&mut engine);
}
