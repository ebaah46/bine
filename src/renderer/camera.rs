//! Bine renderer
//!
//! Author: BEKs => 22.12.2025
//!
//! This camera module for handling all camera related details

use std::ops::Deref;
use bytemuck::{Pod, Zeroable};
use cgmath::*;

// === Camera struct

pub struct Camera {
    eye: Point3<f32>,
    target: Point3<f32>,
    up: Vector3<f32>,
    projection: Projection,
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::from_cols(
    Vector4::new(1.0, 0.0, 0.0, 0.0),
    Vector4::new(0.0, 1.0, 0.0, 0.0),
    Vector4::new(0.0, 0.0, 0.5, 0.0),
    Vector4::new(0.0, 0.0, 0.5, 1.0),
);

impl Camera {
    pub fn new_perspective(
        eye: Point3<f32>,
        target: Point3<f32>,
        up: Vector3<f32>,
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
    ) -> Self {
        let projection = Projection::Perspective(aspect, fovy, znear, zfar);
        Self {
            eye,
            target,
            up,
            projection,
        }
    }

    pub fn new_orthographic(position: Point2<f32>, left:f32, right:f32, bottom:f32, top:f32, near:f32, far:f32) -> Self {
        let projection = Projection::Orthographic(left, right, bottom, top, near, far);
        Self{
            eye: Point3::new(position.x, position.y, 10.0),// arbitrary distance for
            target: Point3::new(position.x, position.y, 0.0), // strait ahead
            up: Vector3::new(0.0, 1.0, 0.0),
            projection,
        }
    }

    fn build_view_projection_matrix(&self, projection: &Projection) -> Matrix4<f32> {
        let view = Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = match *projection {
            Projection::Perspective(a, f, n, c) => perspective(Deg(f), a, n, c),
            Projection::Orthographic(l, r, b, t, n, f) => ortho(l, r, b, t, n, f),
        };

        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }
}

// === CameraUniform Struct

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub(crate) fn new() -> Self {
        Self {
            view_proj: Matrix4::identity().into(),
        }
    }

    pub(crate) fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix(&camera.projection).into();
    }
}


pub enum Projection {
    Perspective(f32, f32, f32, f32),
    Orthographic(f32, f32, f32, f32, f32, f32),
}