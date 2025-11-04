//! Bine implementation of winit window.
//!
//! Author: BEKs => 04.11.2025
//!

// === Winit window implementing the window trait
use super::window::{Window, WindowConfig, WindowEvent};
use anyhow::{Ok, Result};

pub struct WinitWindow {}

impl Window for WinitWindow {
    fn create(config: WindowConfig) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(WinitWindow {})
    }

    fn poll_events(&mut self) -> Vec<WindowEvent> {
        let v: Vec<WindowEvent> = Vec::new();
        v
    }

    fn set_title(&mut self, title: &str) {}

    fn get_title(&self) -> &str {
        "title"
    }

    fn set_size(&mut self, width: u32, height: u32) {}

    fn get_size(&self) -> (u32, u32) {
        (640, 480)
    }

    fn set_position(&mut self, x: i32, y: i32) {}

    fn get_position(&self) -> (i32, i32) {
        (640, 480)
    }

    fn set_fullscreen(&mut self, fullscreen: bool) {}

    fn is_fullscreen(&self) -> bool {
        false
    }

    fn set_visible(&mut self, visible: bool) {}

    fn is_visible(&self) -> bool {
        false
    }

    fn is_open(&self) -> bool {
        false
    }

    fn close(&mut self) {}

    fn request_redraw(&mut self) {}
}
