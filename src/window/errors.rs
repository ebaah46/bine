//! Bine window module erros.
//!
//! Author: BEKs => 04.11.2025
//!Error cases and explanations

use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum WindowError {
    CreationFailed(String),
    BackendError(String),
    InvalidConfig(String),
}

// Implement Display
impl fmt::Display for WindowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WindowError::CreationFailed(msg) => write!(f, "Window creation failed: {}", msg),
            WindowError::BackendError(msg) => write!(f, "Backend error: {}", msg),
            WindowError::InvalidConfig(msg) => write!(f, "Invalid configuration: {}", msg),
        }
    }
}

// Implement std::error::Error
impl Error for WindowError {}
