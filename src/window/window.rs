//! Bine window type.
//!
//! Author: BEKs => 03.11.2025
//!
//! Current abstraction will be tied to the winit API.
//! This is to be abstracted in the future when other windowing
//! systems will be supported.

//! Common types
//!
//! Types of events that the window should handle
//!

use super::errors::WindowError;

#[derive(Clone, Debug)]
pub enum WindowEvent {
    Resized(u32, u32),
    Closed,
    KeyPressed(Key),
    KeyReleased(Key),
    MouseMoved(f64, f64),
    MouseButtonPressed(MouseButton),
}

// Keys that can be pressed on the window
//
#[derive(Debug, Clone, Copy)]
pub enum Key {
    A,
    B,
    C,
    D,
    E,
    Escape,
    Return,
    Space,
}

// Mouse buttons that can be pressed
#[derive(Debug, Clone)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

// Window Configuration builder
#[derive(Debug, Clone)]
pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub resizable: bool,
    pub vsync: bool,
    pub fullscreen: bool,
}

impl WindowConfig {
    pub fn new(title: impl Into<String>, width: u32, height: u32) -> Self {
        WindowConfig {
            title: title.into(),
            width,
            height,
            resizable: true,
            vsync: true,
            fullscreen: false,
        }
    }

    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    pub fn vsync(mut self, vsync: bool) -> Self {
        self.vsync = vsync;
        self
    }
}

// Main window trait
pub trait Window {
    fn create(config: WindowConfig) -> anyhow::Result<Self>
    where
        Self: Sized;

    fn poll_events(&mut self) -> Vec<WindowEvent>;

    fn set_title(&mut self, title: &str);
    fn get_title(&self) -> &str;

    fn set_size(&mut self, width: u32, height: u32);
    fn get_size(&self) -> (u32, u32);

    fn set_position(&mut self, x: i32, y: i32);
    fn get_position(&self) -> (i32, i32);

    fn set_fullscreen(&mut self, fullscreen: bool);
    fn is_fullscreen(&self) -> bool;

    fn set_visible(&mut self, visible: bool);
    fn is_visible(&self) -> bool;

    fn is_open(&self) -> bool;
    fn close(&mut self);

    fn request_redraw(&mut self);
}
