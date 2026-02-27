//! Bine window type.
//!
//! Author: BEKs => 03.11.2025
//!
//! Current windowing system of this engine is coupled with the winit system
//! for simplicity

//! Common types
//!
//! Types of events that the window should handle
//!
use anyhow::Ok;
use winit::{
    dpi::PhysicalSize,
    event_loop::ActiveEventLoop,
    window::{Fullscreen, Window as WinWindow},
};

use super::errors::WindowError;

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
    pub fn new(title: &str, width: u32, height: u32, resizable: bool) -> Self {
        WindowConfig {
            title: title.into(),
            width: width,
            height: height,
            resizable: resizable,
            vsync: false,
            fullscreen: false,
        }
    }
}

// Main window struct
//
#[derive(Debug)]
pub struct Window {
    // internal type that is hidden outsiders
    pub(crate) inner: WinWindow,

    // Public API
    height: u32,
    width: u32,
    title: String,
    fullscreen: bool,
    resizable: bool,
    vsync: bool,
}

impl Window {
    pub fn create(config: &WindowConfig, event_loop: &ActiveEventLoop) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let win_attributes = WinWindow::default_attributes()
            .with_title(config.title.as_str())
            .with_inner_size(PhysicalSize::new(config.width, config.height));
        let window = event_loop.create_window(win_attributes)?;

        Ok(Self {
            inner: window,
            height: config.height,
            width: config.width,
            title: config.title.clone(),
            fullscreen: config.fullscreen,
            vsync: config.vsync,
            resizable: config.resizable,
        })
    }

    pub fn set_title(&mut self, title: String) {
        self.title = title
    }

    pub fn set_fullscreen(&mut self, status: bool) {
        self.fullscreen = status;
        if status {
            self.inner
                .set_fullscreen(Some(Fullscreen::Borderless(None)))
        } else {
            todo!("to be investigated further, it is not yet clear from the docs how to do this.")
        }
    }

    pub(crate) fn inner(&self) -> &WinWindow {
        &self.inner
    }

    pub fn request_redraw(&self) {
        self.inner.request_redraw();
    }

    // Key event handlers component is next
}
