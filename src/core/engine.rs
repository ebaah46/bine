//! Bine engine
//!
//! Author: BEKs => 09.11.2025
//!
//! This module orchestrates and manages all other components
//! and their interaction

use winit::application::ApplicationHandler;

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

    // timing details
    last_update: Option<Instant>,
    accumulator: f32,
}

// === Engine Impl
// Has engine constructor and helper methods for running
// game engine.
//
impl Engine {
    pub fn new(title: &str, width: u32, height: u32) -> Self {
        Self {
            window: None,
            renderer: None,
            title: title.into(),
            width: width,
            height: height,
            last_update: None,
            accumulator: 0.0,
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
        todo!()
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        todo!()
    }
}
