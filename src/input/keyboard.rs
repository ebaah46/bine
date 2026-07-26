//! Bine engine
//!
//! Author: BEKs => 25.11.2025
//!
//! Handles all that has to do with Keyboard
use winit::{
    event::ElementState,
    keyboard::{self},
};

use crate::input::input::{InputSource, UnifiedInputEvent, UniversalKey};

#[derive(Debug, Default)]
pub struct Keyboard;

impl InputSource for Keyboard {
    fn process_events(&mut self, ctx: &mut super::input::EventContext) {
        let UnifiedInputEvent::Winit(event) = ctx.event else {
            return;
        };
        log::info!("Received Keyboard event:{:?}", &event);

        match &event {
            winit::event::WindowEvent::KeyboardInput { event, .. } => {
                if let keyboard::PhysicalKey::Code(code) = event.physical_key {
                    let key = UniversalKey::Keyboard(code);
                    let is_pressed = event.state == ElementState::Pressed;
                    ctx.buffer.inject_key_event(key, is_pressed);
                }
            }
            _ => (),
        }
    }
}
