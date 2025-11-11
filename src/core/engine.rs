//! Bine engine
//!
//! Author: BEKs => 09.11.2025
//!
//! This module orchestrates and manages all other components
//! and their interaction
use pollster;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{Key, NamedKey},
    window::WindowId,
};

use super::super::{
    renderer::{Renderer, RendererBackends},
    window::{Window, WindowConfig},
};

use std::time::Instant;

// === Engine struct
//
pub struct Engine {
    // main resources
    window: Option<Window>,
    renderer: Option<Renderer>,

    // config settings for window
    title: String,
    height: u32,
    width: u32,

    // renderer specific config
    backend: RendererBackends,

    // timing details
    last_update: Option<Instant>,
    accumulator: f32,
}

// === Engine Impl
// Has engine constructor and helper methods for running
// game engine.
//
impl Engine {
    pub fn new(title: &str, width: u32, height: u32, backend: RendererBackends) -> Self {
        Self {
            window: None,
            renderer: None,
            title: title.into(),
            width: width,
            height: height,
            last_update: None,
            accumulator: 0.0,
            backend: backend,
        }
    }

    fn run_game_loop(&mut self) {}

    // update game logic and time changes
    fn update() {}

    // render UI and other sprites in game
    fn render() {}

    // handle window resizing changes using window module

    fn handle_resizing() {}
}

// === winit ApplicationHandler for Engine
impl ApplicationHandler for Engine {
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
                Some(
                    Renderer::new(window.inner(), self.backend.clone())
                        .await
                        .ok()?,
                )
            });
        }

        self.last_update = Some(Instant::now());
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: key,
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => match key {
                Key::Named(NamedKey::Escape) => {
                    println!("Escape key pressed, closing...");
                    event_loop.exit()
                }
                _ => (),
            },
            WindowEvent::RedrawRequested => {
                self.run_game_loop();

                self.window.as_ref().unwrap().request_redraw();
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
