use crate::text::{text_lock, Text};
use anyhow::{Context, Result};
use std::sync::Arc;
use winit::window::Window;

pub struct Gpu {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
}

impl Gpu {
    pub fn new(window: &Arc<Window>) -> Result<Self> {
        log::info!("Creating wgpu instance (Vulkan)");
        let mut desc = wgpu::InstanceDescriptor::new_without_display_handle();
        desc.backends = wgpu::Backends::VULKAN;
        let instance = wgpu::Instance::new(desc);
        log::info!("Creating surface");
        let surface = instance.create_surface(window.clone())?;
        log::info!("Requesting adapter");
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }))?;
        log::info!("Requesting device");
        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
                label: Some("lambda"),
                ..Default::default()
            }))?;
        let size = window.inner_size();
        let caps = surface.get_capabilities(&adapter);
        let &format = caps.formats.first().context("No surface format")?;
        log::info!(
            "Configuring surface: {}x{} {:?}",
            size.width,
            size.height,
            format
        );
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: caps.alpha_modes.first().copied().unwrap_or_default(),
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);
        log::info!("Initializing text rendering");
        Text::init(&device, &queue, format, config.width, config.height);
        log::info!("GPU initialization complete");
        Ok(Self {
            surface,
            device,
            queue,
            config,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        log::info!("Resize: {}x{}", width, height);
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        if let Some(mut text) = text_lock() {
            text.resize(width, height);
        }
    }

    fn acquire_frame(&mut self) -> Option<wgpu::SurfaceTexture> {
        match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(tex)
            | wgpu::CurrentSurfaceTexture::Suboptimal(tex) => Some(tex),
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                match self.surface.get_current_texture() {
                    wgpu::CurrentSurfaceTexture::Success(tex)
                    | wgpu::CurrentSurfaceTexture::Suboptimal(tex) => Some(tex),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    pub fn render(&mut self) {
        if let Some(frame) = self.acquire_frame() {
            let view = frame.texture.create_view(&Default::default());
            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            if let Some(mut text) = text_lock() {
                if let Err(err) = text.prepare(
                    &self.device,
                    &self.queue,
                    self.config.width,
                    self.config.height,
                ) {
                    log::error!("Text prepare failed: {err}");
                }
                {
                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 1.0,
                                    g: 1.0,
                                    b: 1.0,
                                    a: 1.0,
                                }),
                                store: wgpu::StoreOp::Store,
                            },
                            depth_slice: None,
                        })],
                        ..Default::default()
                    });
                    if let Err(err) = text.render(&mut pass) {
                        log::error!("Text render failed: {err}");
                    }
                }
                text.trim();
            }
            self.queue.submit(Some(encoder.finish()));
            frame.present();
        };
    }
}
