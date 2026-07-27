//! Bine engine
//!
//! Author: BEKs => 25.11.2025
//!
//! Handles all that has to do with gamepad usage

use gilrs::{
    EventType::{ButtonPressed, ButtonReleased, Connected, Disconnected},
    GamepadId, Gilrs,
};

use crate::input::input::{InputSource, UnifiedInputEvent, UniversalAxes};
#[derive(Debug)]
pub struct Gamepad {
    pub active_gamepad: Option<GamepadId>,
    lib: Option<Gilrs>,
    // This is to avoid stick drift causing bad player experience
    deadzone: f32,
}

impl Default for Gamepad {
    fn default() -> Self {
        let mut lib = None;
        if let Ok(l) = Gilrs::new() {
            lib = Some(l);
        }

        // default deadzone value value
        Self {
            active_gamepad: Default::default(),
            lib: lib,
            deadzone: 0.15,
        }
    }
}

impl Gamepad {
    pub fn new() -> Self {
        let mut instance: Gamepad = Default::default();
        instance.discover_gamepads();
        instance
    }

    fn discover_gamepads(&mut self) {
        let Some(lib) = &self.lib else {
            return;
        };

        if let Some((id, _)) = lib.gamepads().next() {
            self.active_gamepad = Some(id);
        }
    }

    pub fn update_deadzone(&mut self, deadzone: f32) {
        self.deadzone = deadzone
    }
}

impl InputSource for Gamepad {
    fn process_events(&mut self, ctx: &mut super::input::EventContext) {
        // ignore all other events but frametick
        let UnifiedInputEvent::FrameTick = ctx.event else {
            return;
        };

        let Some(ref mut lib) = self.lib else {
            log::error!("Gamepad library not properly intialized");
            return;
        };

        while let Some(event) = lib.next_event() {
            lib.update(&event);
            match event.event {
                Connected => {
                    if self.active_gamepad.is_none() {
                        self.active_gamepad = Some(event.id);
                    }
                }
                Disconnected => {
                    if self.active_gamepad.is_some() {
                        self.active_gamepad = None;
                    }
                }
                ButtonPressed(button, _) => {
                    let key = super::input::UniversalKey::Gamepad {
                        id: event.id,
                        button,
                    };

                    ctx.buffer.inject_key_event(key, true);
                }
                ButtonReleased(button, _) => {
                    let key = super::input::UniversalKey::Gamepad {
                        id: event.id,
                        button,
                    };

                    ctx.buffer.inject_key_event(key, false);
                }
                _ => {}
            }
        }
        // collect axes information from gamepad
        if let Some(id) = self.active_gamepad {
            let Some(gamepad) = lib.connected_gamepad(id) else {
                lib.inc();
                return;
            };

            let left_x: f32 = {
                let raw_x = gamepad.value(gilrs::Axis::LeftStickX);
                if raw_x > self.deadzone {
                    raw_x
                } else {
                    0.0 as f32
                }
            };
            ctx.buffer
                .inject_axes_event(UniversalAxes::GamepadLeftStickX(id), left_x);
            let left_y: f32 = {
                let raw_y = gamepad.value(gilrs::Axis::LeftStickY);
                if raw_y > self.deadzone {
                    raw_y
                } else {
                    0.0 as f32
                }
            };
            ctx.buffer
                .inject_axes_event(UniversalAxes::GamepadLeftStickY(id), left_y);
            let right_x: f32 = {
                let raw_x = gamepad.value(gilrs::Axis::RightStickX);
                if raw_x > self.deadzone {
                    raw_x
                } else {
                    0.0 as f32
                }
            };
            ctx.buffer
                .inject_axes_event(UniversalAxes::GamepadRightStickX(id), right_x);
            let right_y: f32 = {
                let raw_y = gamepad.value(gilrs::Axis::RightStickY);
                if raw_y > self.deadzone {
                    raw_y
                } else {
                    0.0 as f32
                }
            };
            ctx.buffer
                .inject_axes_event(UniversalAxes::GamepadRightStickY(id), right_y);
        }
        lib.inc();
    }
}
