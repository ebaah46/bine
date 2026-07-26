//! Bine engine
//!
//! Author: BEKs => 25.11.2025
//!
//! This module handles all input devices
//! and their API

use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};

// === Input
pub struct InputSystem {
    pub input_buffer: InputBuffer,
    pub devices: Vec<Arc<RwLock<dyn InputSource>>>,
}

impl InputSystem {
    // gamepad module will be unvailable for now, but in future
    // must be discoverable by input module.
    // Keyboard & Mouse are default active
    pub fn new() -> Self {
        //
        Self {
            input_buffer: InputBuffer::default(),
            devices: vec![],
        }
    }

    // Add extra input device source.
    // Keyboard and Mouse are default devices connected
    pub fn add_source(&mut self, source: Arc<RwLock<dyn InputSource>>) {
        self.devices.push(source);
    }

    pub fn handle_winit_event(&mut self, winit_event: &winit::event::WindowEvent) {
        let mut ctx = EventContext {
            buffer: &mut self.input_buffer,
            event: UnifiedInputEvent::Winit(winit_event),
        };
        log::info!(
            "Trying to process:{} mouse device events",
            self.devices.len()
        );
        for source in &self.devices {
            if let Ok(mut dev) = source.write() {
                dev.process_events(&mut ctx);
            }
        }
    }

    pub fn update_frame_tick(&mut self) {
        let mut ctx = EventContext {
            buffer: &mut self.input_buffer,
            event: UnifiedInputEvent::FrameTick,
        };
        for source in &self.devices {
            if let Ok(mut dev) = source.write() {
                dev.process_events(&mut ctx);
            }
        }
    }
}

// A universal key enumeration for all kinds of keys supported in this engine
//
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum UniversalKey {
    Keyboard(winit::keyboard::KeyCode),
    Mouse(winit::event::MouseButton),
    Gamepad {
        id: gilrs::GamepadId,
        button: gilrs::Button,
    },
}

/// A universal axes movement for all kinds of axes supported in this engine
/// Currently, axes are supported for only mouse and gamepad axes.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum UniversalAxes {
    MouseMovementX,
    MouseMovementY,
    GamepadLeftStickX(gilrs::GamepadId),
    GamepadLeftStickY(gilrs::GamepadId),
    GamepadRightStickX(gilrs::GamepadId),
    GamepadRightStickY(gilrs::GamepadId),
    GamepadLeftTrigger(gilrs::GamepadId),
    GamepadRightTrigger(gilrs::GamepadId),
}

/// A unified buffer that tracks all input events received from all input devices
/// connected in any game built by this engine.
/// It tracks input events from previous frame unto the current frame
#[derive(Debug, Default)]
pub struct InputBuffer {
    previous_frame_buffer: HashSet<UniversalKey>,
    current_frame_buffer: HashSet<UniversalKey>,

    // axes and trigger continous value store
    current_frame_axes: HashMap<UniversalAxes, f32>,
}

impl InputBuffer {
    // Inject key into input buffer
    pub fn inject_key_event(&mut self, key: UniversalKey, pressed: bool) {
        if pressed {
            self.current_frame_buffer.insert(key);
        } else {
            self.current_frame_buffer.remove(&key);
        }
    }

    pub fn inject_axes_event(&mut self, key: UniversalAxes, value: f32) {
        self.current_frame_axes.insert(key, value);
    }
    // Update to be done after delta time changes
    pub fn post_update(&mut self) {
        self.previous_frame_buffer = self.current_frame_buffer.clone();
    }

    // Generate snapshot that will be used by game to process player actions during each
    // frame.
    pub fn generate_snapshot(&self) -> InputSnapshot {
        let keys_pressed = self
            .current_frame_buffer
            .difference(&self.previous_frame_buffer)
            .cloned()
            .collect();

        let keys_released = self
            .previous_frame_buffer
            .difference(&self.current_frame_buffer)
            .cloned()
            .collect();

        InputSnapshot {
            keys_held: self.current_frame_buffer.clone(),
            keys_pressed: keys_pressed,
            keys_released: keys_released,
            axes_moved: self.current_frame_axes.clone(),
        }
    }
}

// A unified input event that can be received from
// connected gamepad or a keyboard/mouse event from winit
#[derive(Debug, Clone, PartialEq)]
pub enum UnifiedInputEvent<'a> {
    // And event reveived from winit
    Winit(&'a winit::event::WindowEvent),
    // A event to notify the begining of a new frame
    // Will be useful for notifying gamepad to check its underlying library for
    // input events
    FrameTick,
}

// The event context that is passed arround
pub struct EventContext<'a> {
    // Unified input buffer that is written to by all events
    pub buffer: &'a mut InputBuffer,
    // Specific event that is being processed
    pub event: UnifiedInputEvent<'a>,
}

// This trait provides an interface for the each input event that
// is received from the underlying systems that handle inputs
// to the global input buffer
pub trait InputSource {
    // process input events received
    fn process_events(&mut self, ctx: &mut EventContext);
}

// A snapshot that game code can call to know which keys have been issued by player
// controller during the begining of a frame
#[derive(Debug, Clone, Default)]
pub struct InputSnapshot {
    pub keys_held: HashSet<UniversalKey>,
    pub keys_pressed: HashSet<UniversalKey>,
    pub keys_released: HashSet<UniversalKey>,
    pub axes_moved: HashMap<UniversalAxes, f32>,
}

impl InputSnapshot {
    pub fn is_held(&self, key: UniversalKey) -> bool {
        self.keys_held.contains(&key)
    }

    pub fn just_pressed(&self, key: UniversalKey) -> bool {
        self.keys_pressed.contains(&key)
    }

    pub fn just_released(&self, key: UniversalKey) -> bool {
        self.keys_released.contains(&key)
    }

    pub fn axes_moved(&self, key: UniversalAxes) -> Option<f32> {
        self.axes_moved.get(&key).copied()
    }
}
