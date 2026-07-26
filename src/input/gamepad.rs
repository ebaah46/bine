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
    active_gamepad: Option<GamepadId>,
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
                let raw_x = gamepad.value(gilrs::Axis::LeftStickX).abs();
                if raw_x > self.deadzone {
                    raw_x
                } else {
                    0.0 as f32
                }
            };
            ctx.buffer
                .inject_axes_event(UniversalAxes::GamepadLeftStickX(id), left_x);
            let left_y: f32 = {
                let raw_y = gamepad.value(gilrs::Axis::LeftStickY).abs();
                if raw_y > self.deadzone {
                    raw_y
                } else {
                    0.0 as f32
                }
            };
            ctx.buffer
                .inject_axes_event(UniversalAxes::GamepadLeftStickY(id), left_y);
            let right_x: f32 = {
                let raw_x = gamepad.value(gilrs::Axis::RightStickX).abs();
                if raw_x > self.deadzone {
                    raw_x
                } else {
                    0.0 as f32
                }
            };
            ctx.buffer
                .inject_axes_event(UniversalAxes::GamepadRightStickX(id), right_x);
            let right_y: f32 = {
                let raw_y = gamepad.value(gilrs::Axis::RightStickY).abs();
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

// ButtonState struct stores the current state of a button
// in a given gamepad snapshot
// #[derive(Debug, Default)]
// pub struct ButtonState {
//     pub is_held: bool,
//     pub is_pressed: bool,
//     pub is_released: bool,
// }

// // Tracks the snapshot of the gamepad before the start of
// // each frame. So that input actions may not change during
// // gameplay script execution.
// #[derive(Debug)]
// pub struct GamepadSnapshot {
//     // connection info
//     pub gamepad_id: u32,
//     pub is_connected: bool,

//     // Analog Axes
//     // these are buttons that provide continous values like the
//     // left stick and right stick.
//     // L2 and R2 also provide continous values.
//     pub left_stick: (f32, f32),
//     pub right_stick: (f32, f32),
//     pub left_trigger: f32,  // L2
//     pub right_trigger: f32, // R2

//     // Standard Face Buttons
//     // These are the standardized buttons
//     // A, B, X, Y in XBox and Triangle, Square, X, Circle in Playstation
//     pub button_1: ButtonState, // A
//     pub button_2: ButtonState, // B
//     pub button_3: ButtonState, // X
//     pub button_4: ButtonState, // Y

//     // Direction Pad (D-Pad)
//     // Mostly used for navigation
//     pub d_pad_up: ButtonState,
//     pub d_pad_down: ButtonState,
//     pub d_pad_left: ButtonState,
//     pub d_pad_right: ButtonState,

//     // Bumpers & Stick Buttons
//     // Handles the R1 and L1 buttons with
//     // the clicks of the analog sticks
//     pub left_bumper: ButtonState,
//     pub right_bumper: ButtonState,
//     pub left_stick_click: ButtonState,
//     pub right_stick_click: ButtonState,

//     // Utility & System Buttons
//     pub start_button: ButtonState,
//     pub select_button: ButtonState,
// }
