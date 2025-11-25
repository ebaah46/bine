//! Bine engine
//!
//! Author: BEKs => 25.11.2025
//!
//! Handles all that has to do with mouse usage
//!
use std::collections::HashSet;
use winit::event::{ElementState, MouseButton};

// === Mouse
pub struct Mouse {
    // Position
    position: (f64, f64),
    last_position: (f64, f64),

    // Buttons
    pressed_buttons: HashSet<MouseButton>,
    last_pressed_buttons: HashSet<MouseButton>,

    scroll_delta: f64,
}

impl Mouse {
    pub fn new() -> Self {
        let position = (0.0, 0.0);
        let scroll_delta = 0.0;

        Self {
            position: position,
            last_position: position,
            pressed_buttons: HashSet::new(),
            last_pressed_buttons: HashSet::new(),
            scroll_delta: scroll_delta,
        }
    }

    pub fn update(&mut self) {
        self.last_position = self.position.clone();
        self.position = (0.0, 0.0);

        self.last_pressed_buttons = self.pressed_buttons.clone();
        self.pressed_buttons.clear();
        self.scroll_delta = 0.0;
    }

    pub fn position(&self) -> (f64, f64) {
        self.position
    }

    pub fn position_delta(&self) -> (f64, f64) {
        (
            self.position.0 - self.last_position.0,
            self.position.1 - self.last_position.1,
        )
    }

    pub fn is_button_pressed(&self, button: MouseButton) -> bool {
        self.pressed_buttons.contains(&button)
    }

    pub fn scroll_delta(&self) -> f32 {
        0.0
    }

    // === Mouse event handlers
    //
    pub fn update_position(&mut self, x: f64, y: f64) {
        self.position = (x, y)
    }

    pub fn update_scroll(&mut self, delta: f64) {
        self.scroll_delta += delta;
    }

    pub fn button_click(&mut self, button: MouseButton, state: ElementState) {
        match state {
            ElementState::Pressed => self.pressed_buttons.insert(button),
            ElementState::Released => self.pressed_buttons.remove(&button),
        };
    }
}
