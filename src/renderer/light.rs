//! Bine renderer light module.
//!
//! Author: BEKs => 08.06.2026
//!Light definition

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    pub position: [f32; 3],
    _padding: u32, // padding for 16-byte uniform requirement
    pub color: [f32; 3],
    _padding_2: u32, // padding for 16-byte uniform requirement
}

impl LightUniform {
    pub fn new(position: &[f32; 3], color: &[f32; 3]) -> Self {
        Self {
            position: *position,
            color: *color,
            _padding: 0,
            _padding_2: 0,
        }
    }
}
