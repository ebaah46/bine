//! Bine engine
//!
//! Author: BEKs => 09.11.2025
//!
//! This module orchestrates and manages all other components
//! and their interaction
use pollster;

use winit::{
    application::ApplicationHandler,
    event::{MouseScrollDelta, WindowEvent},
    event_loop::ActiveEventLoop,
    window::WindowId,
};

use crate::input::Input;

use super::super::{
    renderer::{Renderer, RendererBackends},
    window::{Window, WindowConfig},
};

use std::time::Instant;

// === Game Trait
// This is the user facing trait that every new game has to
// satisfy. It exposes on_init(), on_update() and on_draw()
pub trait Game {
    /**
     * This is where the game will load textures and other initial setup
     * for the game. This will always be called after window and renderer
     * are created.
     */
    fn on_init(&mut self, renderer: &mut Renderer);

    /**
     * This receives the delta time and a reference to input so the game will
     * read the input keys and update the game state. All game state processing is
     * done here.
     */
    fn on_update(&mut self, dt: f32, input: &Input);

    /**
     * This is called on each frame with a reference to the renderer so the game
     * can issue draw calls.
     */
    fn on_draw(&mut self, renderer: &mut Renderer);
}

// === Engine struct
//
pub struct Engine<G: Game> {
    // main resources
    window: Option<Window>,
    renderer: Option<Renderer>,
    game: G, // game object

    // config settings for window
    title: String,
    height: u32,
    width: u32,

    // renderer specific config
    backend: RendererBackends,

    // input device
    input: Input,

    // timing details
    last_update: Option<Instant>,
    accumulator: f32,
}

// === Engine Impl
// Has engine constructor and helper methods for running
// game engine.
//
impl<G: Game> Engine<G> {
    const FRAME_TIME_CAP: f32 = 0.25; //
    const TIME_STEP: f32 = 1.0 / 60.0; // how often should I update game logic

    pub fn new(title: &str, width: u32, height: u32, backend: RendererBackends, game: G) -> Self {
        Self {
            window: None,
            renderer: None,
            game: game,
            title: title.into(),
            width: width,
            height: height,
            last_update: None,
            accumulator: 0.0,
            backend: backend,
            input: Input::new(),
        }
    }

    fn run_game_loop(&mut self) {
        // calculate delta time as time since last frame processing began
        let dt = Instant::now();
        let frame_time = if let Some(last_update) = self.last_update {
            (dt - last_update).as_secs_f32().min(Self::FRAME_TIME_CAP)
        } else {
            0.0
        };
        self.accumulator += frame_time;

        // update game logic since last update
        let mut iterations = 0;
        while self.accumulator >= Self::TIME_STEP {
            self.update(Self::TIME_STEP);
            self.accumulator -= Self::TIME_STEP;

            iterations += 1;
            if iterations > 10 {
                self.accumulator = 0.0;
                break;
            }
        }

        self.render();

        self.last_update = Some(dt);

        self.input.update();
    }

    // update game logic and time changes
    fn update(&mut self, delta_time: f32) {
        self.game.on_update(delta_time, &self.input);
    }

    // render UI and other sprites in game
    fn render(&mut self) {
        if let Some(renderer) = self.renderer.as_mut() {
            self.game.on_draw(renderer);
            // renderer.clear(120.0, 250.0, 88.0);
        }
    }

    // handle window resizing changes using window module

    fn handle_resizing() {}
}

// === winit ApplicationHandler for Engine
impl<G: Game> ApplicationHandler for Engine<G> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        // create window
        let config = WindowConfig {
            title: self.title.clone(),
            width: self.width,
            height: self.height,
            resizable: true,
            vsync: true,
            fullscreen: false,
        };
        match Window::create(config, event_loop) {
            Ok(window) => {
                println!("Window created successfully!");
                self.window = Some(window);
            }
            Err(e) => {
                eprintln!("Failed to create window: {:?}", e);
                event_loop.exit();
            }
        }
        // create renderer
        if let Some(window) = &self.window {
            self.renderer = pollster::block_on(async {
                if let Ok(r) = Renderer::new(window.inner(), self.backend.clone()).await {
                    Some(r)
                } else {
                    println!("Failed to create valid renderer");
                    None
                }
            });
        }

        self.last_update = Some(Instant::now());

        // initialize game after renderer is created
        if let Some(renderer) = self.renderer.as_mut() {
            self.game.on_init(renderer);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::KeyboardInput {
                event: key_event, ..
            } => self.input.handle_keyboard_event(&key_event),

            WindowEvent::RedrawRequested => {
                self.run_game_loop();

                self.window.as_ref().unwrap().request_redraw();
            }

            WindowEvent::CursorMoved { position, .. } => {
                self.input.handle_cursor_moved_event(position.x, position.y)
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let d = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 100.0,
                    _ => 0.0,
                };
                self.input.handle_mouse_wheel_event(d as f64)
            }

            WindowEvent::MouseInput { state, button, .. } => {
                self.input.handle_mouse_button_event(button, state);
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
}
