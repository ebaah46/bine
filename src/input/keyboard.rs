//! Bine engine
//!
//! Author: BEKs => 25.11.2025
//!
//! Handles all that has to do with Keyboard
use std::collections::HashSet;

use winit::{event::ElementState, keyboard::KeyCode};

pub struct Keyboard {
    pressed_keys: HashSet<KeyCode>,
    last_pressed_keys: HashSet<KeyCode>,
}

impl Keyboard {
    pub fn new() -> Self {
        Self {
            pressed_keys: HashSet::new(),
            last_pressed_keys: HashSet::new(),
        }
    }

    pub fn update(&mut self) {
        self.last_pressed_keys = self.pressed_keys.clone();
        self.pressed_keys.clear();
    }

    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.pressed_keys.contains(&key) && !self.last_pressed_keys.contains(&key)
    }

    pub fn is_key_released(&self, key: KeyCode) -> bool {
        !self.pressed_keys.contains(&key) && self.last_pressed_keys.contains(&key)
    }

    pub fn is_key_held_down(&self, key: KeyCode) -> bool {
        self.pressed_keys.contains(&key)
    }

    // === Keyboard event handlers
    //
    pub fn pressed(&mut self, key: KeyCode, state: ElementState) {
        match state {
            ElementState::Pressed => self.pressed_keys.insert(key),
            ElementState::Released => self.pressed_keys.remove(&key),
        };
    }
}
