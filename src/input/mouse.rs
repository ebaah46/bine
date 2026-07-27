//! Bine engine
//!
//! Author: BEKs => 25.11.2025
//!
//! Handles all that has to do with mouse usage
//!
use winit::event::{ElementState, MouseScrollDelta};

use crate::input::input::{InputSource, UnifiedInputEvent, UniversalAxes};

// === Mouse
#[derive(Debug, Default)]
pub struct Mouse {
    // Position
    position: (f64, f64),
    scroll_delta: f64,
}

impl Mouse {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn position(&self) -> (f64, f64) {
        self.position
    }

    pub fn scroll_delta(&self) -> f32 {
        self.scroll_delta as f32
    }
}

impl InputSource for Mouse {
    fn process_events(&mut self, ctx: &mut super::input::EventContext) {
        let UnifiedInputEvent::Winit(event) = ctx.event else {
            return;
        };
        log::info!("Received Mouse event:{:?}", &event);

        match event {
            winit::event::WindowEvent::CursorMoved { position, .. } => {
                self.position = (position.x, position.y);
                ctx.buffer
                    .inject_axes_event(UniversalAxes::MouseMovementX, position.x as f32);
                ctx.buffer
                    .inject_axes_event(UniversalAxes::MouseMovementY, position.y as f32);
            }
            winit::event::WindowEvent::CursorEntered { .. } => {}
            winit::event::WindowEvent::CursorLeft { .. } => {}
            winit::event::WindowEvent::MouseWheel { delta, .. } => {
                let d = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => *y as f64,
                    MouseScrollDelta::PixelDelta(pos) => pos.y / 100.0,
                    _ => 0.0,
                };
                self.scroll_delta += d;
                ctx.buffer.inject_axes_event(
                    UniversalAxes::MouseScrollMovement,
                    self.scroll_delta as f32,
                );
            }
            winit::event::WindowEvent::MouseInput { state, button, .. } => {
                let key = super::input::UniversalKey::Mouse(*button);
                let is_pressed = *state == ElementState::Pressed;
                ctx.buffer.inject_key_event(key, is_pressed);
            }
            _ => (),
        }
    }
}
