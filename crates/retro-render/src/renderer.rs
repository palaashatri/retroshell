use crate::{primitives::DrawCommand, Color, Result};
use parking_lot::Mutex;
use wgpu::PowerPreference;
use wgpu::{Device, Instance, Queue, Surface as WgpuSurface, SurfaceConfiguration};

pub struct Renderer {
    pub instance: Instance,
    pub adapter: wgpu::Adapter,
    pub device: Device,
    pub queue: Queue,
    pub config: SurfaceConfiguration,
    draw_commands: Mutex<Vec<DrawCommand>>,
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

        // Dynamic HDR selection: prefer wide color formats like float16 or 10-bit formats
        // FIXME: Unconditionally selecting an HDR format (e.g. Rgba16Float) when supported without doing proper SDR-to-HDR
        // tonemapping/color scaling. SDR app colors (which are standard sRGB 8-bit) may appear washed out or incorrectly mapped.
        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(|&f| {
                f == wgpu::TextureFormat::Rgba16Float || f == wgpu::TextureFormat::Rgb10a2Unorm
            })
            .unwrap_or(capabilities.formats[0]);

        // Dynamic VRR (Variable Refresh Rate) and low-latency modes selection:
        // Prefer AutoVsync (freesync/g-sync adaptive sync) or Mailbox (low latency VSync-free)
        // FIXME: Falling back directly to Mailbox or Fifo might cause visual tearing or frame pacing stutter
        // if the client rendering loop cannot maintain display refresh rate constraints.
        let present_mode = if capabilities
            .present_modes
            .contains(&wgpu::PresentMode::AutoVsync)
        {
            wgpu::PresentMode::AutoVsync
        } else if capabilities
            .present_modes
            .contains(&wgpu::PresentMode::Mailbox)
        {
            wgpu::PresentMode::Mailbox
        } else {
            wgpu::PresentMode::Fifo
        };

        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode,
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
            draw_commands: Mutex::new(Vec::new()),
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

    pub fn enqueue(&self, command: DrawCommand) {
        self.draw_commands.lock().push(command);
    }

    pub fn drain_commands(&self) -> Vec<DrawCommand> {
        self.draw_commands.lock().drain(..).collect()
    }

    pub fn queued_command_count(&self) -> usize {
        self.draw_commands.lock().len()
    }

    pub fn device(&self) -> &Device {
        &self.device
    }
    pub fn queue(&self) -> &Queue {
        &self.queue
    }
}
