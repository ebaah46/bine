pub mod input;
pub use input::{InputSnapshot, InputSystem, UniversalAxes, UniversalKey};

pub mod gamepad;
pub mod keyboard;
pub mod mouse;

pub use gamepad::Gamepad;
pub use keyboard::Keyboard;
pub use mouse::Mouse;
