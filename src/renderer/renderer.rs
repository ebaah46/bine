//! Bine renderer
//!
//! Author: BEKs => 08.11.2025
//!
//! This renderer module is tied to wgpu library

use std::vec;

use anyhow::Result;

use wgpu::{
    Backends, Color, DeviceDescriptor, ExperimentalFeatures, Features, Instance,
    InstanceDescriptor, Limits, PowerPreference, RenderPassColorAttachment, RenderPassDescriptor,
    RequestAdapterOptions, SurfaceConfiguration, SurfaceTargetUnsafe, TextureUsages, Trace,
    wgt::{CommandEncoderDescriptor, TextureViewDescriptor},
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

    // Helper state
    size: (u32, u32),
}

impl Renderer {
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
            .await?;

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: None,
                required_features: Features::empty(),
                experimental_features: ExperimentalFeatures::disabled(),
                required_limits: Limits::defaults(),
                memory_hints: Default::default(),
                trace: Trace::Off,
            })
            .await?;

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
        Ok(Self {
            surface: surface,
            instance: instance,
            device: device,
            queue: queue,
            config: config,
            size: (size.width, size.height),
        })
    }

    // Clearing the surface
    // Basic necessity for rendering
    pub fn clear(&self, r: f32, g: f32, b: f32) {
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
            let render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Clear render pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },

                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
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
pub enum RendererBackends {
    OpenGL,
    Metal,
    Vulkan,
    Dx12,
    BrowserWebGL,
}
