//! Bine renderer
//!
//! Author: BEKs => 08.11.2025
//!
//! This renderer module is tied to wgpu library

use anyhow::{Context, Ok, Result};

use cgmath::{Point3, Vector3, prelude::*};
use wgpu::{
    Backends, Color, DeviceDescriptor, ExperimentalFeatures, Features, FragmentState, Instance,
    InstanceDescriptor, Limits, PipelineLayoutDescriptor, PowerPreference,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipelineDescriptor,
    RequestAdapterOptions, SurfaceConfiguration, SurfaceTargetUnsafe, TextureUsages, Trace,
    VertexState, include_wgsl,
    util::DeviceExt,
    wgt::{CommandEncoderDescriptor, TextureViewDescriptor},
};

use super::{DrawModel, Model, Vertex};
use crate::{
    core::resources,
    renderer::{
        Camera, CameraUniform, Instance as RendererInstance, InstanceRaw, LightUniform,
        ModelVertex, Texture as RendererTexture, model::DrawLight,
    },
};
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
    light_pipeline: wgpu::RenderPipeline,
    size: (u32, u32),

    // texture
    texture_bind_group_layout: wgpu::BindGroupLayout,
    camera_bind_group_layout: wgpu::BindGroupLayout,
    depth_texture: RendererTexture,

    // light
    light_uniform: Option<LightUniform>,
    light_buffer: Option<wgpu::Buffer>,
    light_bind_group: Option<wgpu::BindGroup>,
    light_bind_group_layout: wgpu::BindGroupLayout,

    // camera
    camera: Option<Camera>,
    camera_uniform: Option<CameraUniform>,
    camera_buffer: Option<wgpu::Buffer>,
    camera_bind_group: Option<wgpu::BindGroup>,

    models: Vec<Model>,
    // instances
    instances: Vec<RendererInstance>,
    instance_buffer: wgpu::Buffer,
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

        let light_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("light_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        // source: sotrh.github.io/learn-wgpu/beginner/tutorial7-instancing/#the-instance-buffer
        const NUM_INSTANCES_PER_ROW: i32 = 1;
        const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(
            NUM_INSTANCES_PER_ROW as f32 * 0.5,
            0.0,
            NUM_INSTANCES_PER_ROW as f32 * 0.5,
        );
        const SPACE_BETWEEN: f32 = 8.0;
        let instances = (0..NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {
                (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                    let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                    let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                    let position = cgmath::Vector3 {
                        x: x as f32,
                        y: 0.0,
                        z: z as f32,
                    } - INSTANCE_DISPLACEMENT;

                    let rotation = if position.is_zero() {
                        cgmath::Quaternion::from_axis_angle(
                            cgmath::Vector3::unit_z(),
                            cgmath::Deg(0.0),
                        )
                    } else {
                        cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
                    };

                    RendererInstance::new(position, rotation)
                })
            })
            .collect::<Vec<_>>();
        let instance_data = instances
            .iter()
            .map(RendererInstance::to_raw)
            .collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let depth_texture =
            RendererTexture::create_depth_texture(&device, &config, "depth_texture");

        let shader = device.create_shader_module(include_wgsl!("../../shaders/basic.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Basic Render Pipeline Layout"),
            bind_group_layouts: &[
                &texture_bind_group_layout,
                &camera_bind_group_layout,
                &light_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let render_pipeline = Self::create_render_pipeline(
            &device,
            &pipeline_layout,
            config.format,
            None,
            &[ModelVertex::desc(), InstanceRaw::desc()],
            shader,
        );

        let light_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Basic light render pipeline layout"),
            bind_group_layouts: &[&camera_bind_group_layout, &light_bind_group_layout],
            push_constant_ranges: &[],
        });

        let light_shader = device.create_shader_module(include_wgsl!("../../shaders/light.wgsl"));
        let light_render_pipeline = Self::create_render_pipeline(
            &device,
            &light_pipeline_layout,
            config.format,
            Some(RendererTexture::DEPTH_FORMAT),
            &[ModelVertex::desc()],
            light_shader,
        );

        Ok(Self {
            surface: surface,
            instance: instance,
            device: device,
            queue: queue,
            config: config,
            size: (size.width, size.height),
            pipeline: render_pipeline,
            depth_texture: depth_texture,
            instances: instances,
            instance_buffer: instance_buffer,
            light_pipeline: light_render_pipeline,
            light_buffer: None,
            light_uniform: None,
            light_bind_group: None,
            light_bind_group_layout: light_bind_group_layout,
            camera: None,
            camera_uniform: None,
            camera_buffer: None,
            camera_bind_group: None,
            texture_bind_group_layout: texture_bind_group_layout,
            camera_bind_group_layout: camera_bind_group_layout,
            models: vec![],
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
                            a: 0.8, // default at this point
                        }),
                        store: wgpu::StoreOp::Store,
                    },

                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

            if let (Some(camera_bind_group), Some(light_bind_group)) =
                (&self.camera_bind_group, &self.light_bind_group)
            {
                render_pass.set_pipeline(&self.light_pipeline);
                self.models.iter().for_each(|m| {
                    render_pass.draw_light_model(m, camera_bind_group, light_bind_group);
                });

                render_pass.set_pipeline(&self.pipeline);
                self.models.iter().for_each(|m| {
                    render_pass.draw_model_instanced(
                        m,
                        0..self.instances.len() as u32,
                        camera_bind_group,
                        light_bind_group,
                    );
                });
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
        self.depth_texture =
            RendererTexture::create_depth_texture(&self.device, &self.config, "depth_texture");
    }

    // Provides access for the game to define some key properties of the lighting module
    pub fn set_light_properties(&mut self, position: &[f32; 3], color: &[f32; 3]) {
        let light_uniform = LightUniform::new(position, color);

        let light_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Light Buffer"),
                contents: bytemuck::cast_slice(&[light_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let light_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("light bind group"),
            layout: &self.light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
        });

        self.light_uniform = Some(light_uniform);
        self.light_buffer = Some(light_buffer);
        self.light_bind_group = Some(light_bind_group);
    }

    // Provide the ability for game to load relevant models
    // This could be threaded as load could take a while
    pub fn set_models_to_load(&mut self, model_paths: &[&str]) -> anyhow::Result<()> {
        for &path in model_paths {
            let model = pollster::block_on(resources::load_model(
                path,
                &self.device,
                &self.queue,
                &self.texture_bind_group_layout,
            ))?;
            self.models.push(model);
        }
        Ok(())
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
        self.camera_buffer = Some(camera_buffer);
        self.camera_uniform = Some(camera_uniform);
        self.camera_bind_group = Some(camera_bind_group);
    }

    pub fn update_camera(&mut self, camera: &Camera) {
        if let Some(camera_uniform) = &mut self.camera_uniform {
            camera_uniform.update_view_proj(camera);
        }

        if let Some((camera_buffer, camera_uniform)) =
            self.camera_buffer.as_ref().zip(self.camera_uniform)
        {
            self.queue
                .write_buffer(camera_buffer, 0, bytemuck::cast_slice(&[camera_uniform]));
        }

        // TODO: This part must be removed, this is for testing purposes,
        // should be moved to separate renderer method.
        // Update the light
        //
        if let (Some(light_uniform), Some(light_buffer)) =
            (self.light_uniform.as_mut(), &self.light_buffer)
        {
            let old_position: cgmath::Vector3<_> = light_uniform.position.clone().into();
            light_uniform.position =
                (cgmath::Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), cgmath::Deg(1.0))
                    * old_position)
                    .into();

            self.queue
                .write_buffer(&light_buffer, 0, bytemuck::cast_slice(&[*light_uniform]));
        }
    }

    fn create_render_pipeline(
        device: &wgpu::Device,
        layout: &wgpu::PipelineLayout,
        color_format: wgpu::TextureFormat,
        depth_format: Option<wgpu::TextureFormat>,
        vertex_layouts: &[wgpu::VertexBufferLayout],
        shader: wgpu::ShaderModule,
    ) -> wgpu::RenderPipeline {
        device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: vertex_layouts,
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
            depth_stencil: Some(wgpu::DepthStencilState {
                format: RendererTexture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
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
                    format: color_format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
            cache: None,
        })
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
