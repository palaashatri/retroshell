use crate::{Color, Result};
use wgpu::PowerPreference;
use wgpu::{Device, Instance, Queue, Surface as WgpuSurface, SurfaceConfiguration};

pub struct Renderer {
    pub instance: Instance,
    pub adapter: wgpu::Adapter,
    pub device: Device,
    pub queue: Queue,
    pub config: SurfaceConfiguration,
}

impl Renderer {
    pub async fn new(surface: &WgpuSurface<'static>, width: u32, height: u32) -> Result<Self> {
        let instance = Instance::new(Default::default());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: Some(surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or(crate::RenderError::Surface("no adapter found".into()))?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("RetroRender Device"),
                    required_features: wgpu::Features::default(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await?;

        let capabilities = surface.get_capabilities(&adapter);
        let format = capabilities.formats[0];

        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            config,
        })
    }

    pub fn resize(&mut self, surface: &WgpuSurface<'static>, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        surface.configure(&self.device, &self.config);
    }

    pub fn begin_frame(&self, surface: &WgpuSurface<'static>) -> Option<wgpu::SurfaceTexture> {
        surface.get_current_texture().ok()
    }

    pub fn clear(&self, view: &wgpu::TextureView, color: Color) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("clear encoder"),
            });
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("clear pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: color.r as f64,
                        g: color.g as f64,
                        b: color.b as f64,
                        a: color.a as f64,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });
        self.queue.submit(Some(encoder.finish()));
    }

    pub fn present(frame: wgpu::SurfaceTexture) {
        frame.present();
    }

    pub fn device(&self) -> &Device {
        &self.device
    }
    pub fn queue(&self) -> &Queue {
        &self.queue
    }
}
