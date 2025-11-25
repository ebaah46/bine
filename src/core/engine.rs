//! Bine engine
//!
//! Author: BEKs => 09.11.2025
//!
//! This module orchestrates and manages all other components
//! and their interaction
use pollster;
use wgpu::Color;
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

use std::time::{Duration, Instant};

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
    const FRAME_TIME_CAP: f32 = 0.25; //
    const TIME_STEP: f32 = 1.0 / 60.0; // how often should I update game logic

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
    }

    // update game logic and time changes
    fn update(&mut self, delta_time: f32) {}

    // render UI and other sprites in game
    fn render(&mut self) {
        let renderer = self.renderer.as_mut().unwrap();
        renderer.clear(120.0, 250.0, 88.0);
    }

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

            WindowEvent::CursorMoved {
                device_id,
                position,
            } => {
                if let Some(_) = self.renderer.as_ref() {
                    dbg!(position);
                };
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
