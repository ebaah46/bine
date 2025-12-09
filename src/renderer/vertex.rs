//! Bine engine
//!
//! Author: BEKs => 26.11.2025
//!
//! Vertex struct
//!
use bytemuck::{Pod, Zeroable};
use wgpu::{BufferAddress, VertexBufferLayout};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    const ATTRIB: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

    pub const fn new(position: [f32; 3], color: [f32; 3]) -> Self {
        Self { position, color }
    }

    pub fn desc<'a>() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIB,
        }
    }
}
