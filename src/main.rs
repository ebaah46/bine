use bine::core::Engine;

use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    // Demo app to test created modules
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();

    // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
    // dispatched any events. This is ideal for games and similar applications.
    event_loop.set_control_flow(ControlFlow::Poll);

    // ControlFlow::Wait pauses the event loop if no events are available to process.
    // This is ideal for non-game applications that only update in response to user
    // input, and uses significantly less power/CPU time than ControlFlow::Poll.
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut engine = Engine::new(
        "Demo Game",
        1980,
        1680,
        bine::renderer::RendererBackends::Metal,
    );
    let _ = event_loop.run_app(&mut engine);
}
