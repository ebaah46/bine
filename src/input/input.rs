//! Bine engine
//!
//! Author: BEKs => 25.11.2025
//!
//! This module handles all input devices
//! and their API

use winit::{
    event::{ElementState, KeyEvent, MouseButton, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

use super::{Gamepad, Keyboard, Mouse};

// === Input
pub struct Input {
    keyboard: Option<Keyboard>,
    mouse: Option<Mouse>,
    gamepad: Option<Gamepad>,
}

impl Input {
    // gamepad module will be unvailable for now, but in future
    // must be discoverable by input module.
    // Keyboard & Mouse are default active
    pub fn new() -> Self {
        Self {
            keyboard: Some(Keyboard::new()),
            mouse: Some(Mouse::new()),
            gamepad: None,
        }
    }

    pub fn update(&mut self) {
        if let Some(keyboard) = self.keyboard.as_mut() {
            keyboard.update();
        }

        if let Some(mouse) = self.mouse.as_mut() {
            mouse.update();
        }
    }

    // Query
    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        if let Some(keyboard) = self.keyboard.as_ref() {
            return keyboard.is_key_pressed(key);
        }
        false
    }

    pub fn is_key_released(&self, key: KeyCode) -> bool {
        if let Some(keyboard) = self.keyboard.as_ref() {
            return keyboard.is_key_released(key);
        }
        false
    }

    pub fn is_key_held_down(&self, key: KeyCode) -> bool {
        if let Some(keyboard) = self.keyboard.as_ref() {
            return keyboard.is_key_held_down(key);
        }
        false
    }

    // Event handlers
    pub fn handle_keyboard_event(&mut self, event: &KeyEvent) {
        if let Some(keyboard) = self.keyboard.as_mut() {
            let code: Option<KeyCode> = match event.physical_key {
                PhysicalKey::Code(c) => Some(c),
                PhysicalKey::Unidentified(_) => None,
            };
            if let Some(c) = code {
                keyboard.pressed(c, event.state);
            }
        }
    }

    pub fn handle_mouse_button_event(&mut self, event: MouseButton, state: ElementState) {
        if let Some(mouse) = self.mouse.as_mut() {
            mouse.button_click(event, state);
        }
    }

    pub fn handle_cursor_moved_event(&mut self, x: f64, y: f64) {
        if let Some(mouse) = self.mouse.as_mut() {
            mouse.update_position(x, y);
        }
    }

    pub fn handle_mouse_wheel_event(&mut self, delta: f64) {
        if let Some(mouse) = self.mouse.as_mut() {
            mouse.update_scroll(delta);
        }
    }
}
