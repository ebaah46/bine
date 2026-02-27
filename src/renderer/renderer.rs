//! Bine renderer
//!
//! Author: BEKs => 08.11.2025
//!
//! This renderer module is tied to wgpu library

use std::{fs, num, path::Path, vec};

use anyhow::{Context, Result};

use cgmath::{Point3, Vector3};
use wgpu::{
    Backends, Color, DeviceDescriptor, ExperimentalFeatures, Features, FragmentState, Instance,
    InstanceDescriptor, Limits, PipelineLayoutDescriptor, PowerPreference,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipelineDescriptor,
    RequestAdapterOptions, SurfaceConfiguration, SurfaceTargetUnsafe, TextureUsages, Trace,
    VertexState, include_wgsl,
    util::DeviceExt,
    wgt::{CommandEncoderDescriptor, TextureViewDescriptor},
};

use super::{Texture, Vertex};
use crate::renderer::{Camera, CameraUniform};
use winit::window::Window;

// === Renderer Struct
pub struct Renderer {
    // wgpu specific internals
    instance: wgpu::Instance,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,

    // pipeline internals
    pipeline: wgpu::RenderPipeline,
    size: (u32, u32),

    texture_bind_group_layout: wgpu::BindGroupLayout,
    camera_bind_group_layout: wgpu::BindGroupLayout,

    // camera
    camera: Option<Camera>,
    camera_uniform: Option<CameraUniform>,
    camera_buffer: Option<wgpu::Buffer>,
    camera_bind_group: Option<wgpu::BindGroup>,

    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    diffuse_texture: Option<Texture>,
    diffuse_bind_group: Option<wgpu::BindGroup>,
    num_indices: Option<u32>,
}

impl Renderer {
    //TODO: Builder pattern could be used to make this Renderer construction
    //      more idiomatic. But I do not have time for that now.
    pub async fn new(window: &Window, backend: RendererBackends) -> Result<Self> {
        let size = window.inner_size();
        let bd = match backend {
            RendererBackends::OpenGL => Backends::GL,
            RendererBackends::Dx12 => Backends::DX12,
            RendererBackends::Metal => Backends::METAL,
            RendererBackends::Vulkan => Backends::VULKAN,
            RendererBackends::BrowserWebGL => Backends::BROWSER_WEBGPU,
            _ => Backends::NOOP,
        };

        let instance = Instance::new(&InstanceDescriptor {
            backends: bd,
            ..Default::default()
        });

        // ===
        // This hack is to be investigated later
        let surface = unsafe {
            let target =
                SurfaceTargetUnsafe::from_window(window).expect("Failed to create unsafe surface");
            instance
                .create_surface_unsafe(target)
                .expect("failed to create unsafe surface with unsafe target")
        };

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .context("Failed to create adapter")?;

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: None,
                required_features: Features::ADDRESS_MODE_CLAMP_TO_BORDER,
                experimental_features: ExperimentalFeatures::disabled(),
                required_limits: Limits::defaults(),
                memory_hints: Default::default(),
                trace: Trace::Off,
            })
            .await
            .context("Failed to create device")?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let shader = device.create_shader_module(include_wgsl!("../../shaders/basic.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Basic Layout"),
            bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Shape Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[Vertex::desc()],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: 0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
            cache: None,
        });

        Ok(Self {
            surface: surface,
            instance: instance,
            device: device,
            queue: queue,
            config: config,
            size: (size.width, size.height),
            pipeline: render_pipeline,
            vertex_buffer: None,
            index_buffer: None,
            num_indices: None,
            diffuse_bind_group: None,
            diffuse_texture: None,
            camera: None,
            camera_uniform: None,
            camera_buffer: None,
            camera_bind_group: None,
            texture_bind_group_layout: texture_bind_group_layout,
            camera_bind_group_layout: camera_bind_group_layout,
        })
    }

    // Clearing the surface
    // Basic necessity for rendering
    pub fn render(&self, r: f64, g: f64, b: f64) {
        let frame = self
            .surface
            .get_current_texture()
            .expect("failed to retrieve frame");

        let view = frame.texture.create_view(&TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Clear Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Clear render pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(Color {
                            r: r,
                            g: g,
                            b: b,
                            a: 0.5, // default at this point
                        }),
                        store: wgpu::StoreOp::Store,
                    },

                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            render_pass.set_pipeline(&self.pipeline);
            if let Some(diffuse_bind_group) = &self.diffuse_bind_group {
                render_pass.set_bind_group(0, diffuse_bind_group, &[]);
            }
            if let Some(camera_bind_group) = &self.camera_bind_group {
                render_pass.set_bind_group(1, camera_bind_group, &[]);
            }
            if let Some(vertex_buffer) = &self.vertex_buffer {
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            }
            if let Some(index_buffer) = &self.index_buffer {
                render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            }
            if let Some(num_indices) = self.num_indices {
                render_pass.draw_indexed(0..num_indices, 0, 0..1);
            }
        } // drop render_pass so we can use encoder again

        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }

    // React to changes in window size
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.size = (width, height);
            self.surface.configure(&self.device, &self.config);
        }
    }

    // Provides access for game to register data to be used in
    // renderer.
    pub fn load_texture(&mut self, bytes: &[u8], file_name: &str) {
        let diffuse_texture = Texture::from_bytes(&self.device, &self.queue, bytes, file_name)
            .expect("Failed to load texture from bytes");

        let texture_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });
        self.diffuse_texture = Some(diffuse_texture);
        self.diffuse_bind_group = Some(texture_bind_group);
    }

    // Provides access for the game to register the indices and vertices for
    // the textures that it provides
    pub fn set_geometry(&mut self, vertices: &[Vertex], indices: &[u16]) {
        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        let num_indices = indices.len() as u32;

        self.num_indices = Some(num_indices);
        self.index_buffer = Some(index_buffer);
        self.vertex_buffer = Some(vertex_buffer);
    }

    // Provides access for the game to set the position of the camera
    pub fn set_camera(
        &mut self,
        eye: Point3<f32>,
        target: Point3<f32>,
        up: Vector3<f32>,
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
    ) {
        let camera = Camera::new(eye, target, up, aspect, fovy, znear, zfar);

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let camera_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera_bind_group"),
            layout: &self.camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        self.camera = Some(camera);
        self.camera_bind_group = Some(camera_bind_group);
    }
}

// === Enumeration for different backends to use
// this is used during renderer instantiation
#[derive(Debug, Clone)]
pub enum RendererBackends {
    OpenGL,
    Metal,
    Vulkan,
    Dx12,
    BrowserWebGL,
}
