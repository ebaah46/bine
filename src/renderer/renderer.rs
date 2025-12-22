//! Bine renderer
//!
//! Author: BEKs => 08.11.2025
//!
//! This renderer module is tied to wgpu library

use std::vec;

use anyhow::{Context, Result};

use cgmath::Vector3;
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
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    diffuse_bind_group: wgpu::BindGroup,
    diffuse_texture: Texture,

    // camera
    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    // Helper state
    size: (u32, u32),
    num_indices: u32,
}

impl Renderer {
    const VERTICES: &[Vertex] = &[
        Vertex::new([-0.0868241, 0.49240386, 0.0], [0.4131759, 1.0 - 0.99240386]),
        Vertex::new(
            [-0.49513406, 0.06958647, 0.0],
            [0.0048659444, 1.0 - 0.56958647],
        ),
        Vertex::new(
            [-0.21918549, -0.44939706, 0.0],
            [0.28081453, 1.0 - 0.05060294],
        ),
        Vertex::new([0.35966998, -0.3473291, 0.0], [0.85967, 1.0 - 0.1526709]),
        Vertex::new([0.44147372, 0.2347359, 0.0], [0.9414737, 1.0 - 0.7347359]),
    ];

    const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];

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

        let diffuse_bytes = include_bytes!("../../assets/textures/happy-tree.png");
        let diffuse_texture =
            Texture::from_bytes(&device, &queue, diffuse_bytes, "happy-tree.png").unwrap();

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

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
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

        let camera = Camera::new(
            (0.0, 1.0, 2.0).into(),
            (0.0, 0.0, 0.0).into(),
            Vector3::unit_y(),
            config.width as f32 / config.height as f32,
            45.0,
            0.1,
            100.0,
        );

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
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

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera_bind_group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
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

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&Self::VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&Self::INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let num_indices = Self::INDICES.len() as u32;
        Ok(Self {
            surface: surface,
            instance: instance,
            device: device,
            queue: queue,
            config: config,
            size: (size.width, size.height),
            pipeline: render_pipeline,
            vertex_buffer: vertex_buffer,
            index_buffer: index_buffer,
            num_indices: num_indices,
            diffuse_bind_group: bind_group,
            diffuse_texture: diffuse_texture,
            camera: camera,
            camera_uniform: camera_uniform,
            camera_buffer: camera_buffer,
            camera_bind_group: camera_bind_group,
        })
    }

    // Clearing the surface
    // Basic necessity for rendering
    pub fn clear(&self, r: f64, g: f64, b: f64) {
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
                            a: 0.5,
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
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
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
