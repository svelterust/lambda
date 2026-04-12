use crate::systems::{image::Images, rect::Rects, text::Text};
use crate::Result;
use std::sync::{Arc, Mutex};
use winit::window::Window;

pub struct Gpu {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    rects: Arc<Mutex<Rects>>,
    images: Arc<Mutex<Images>>,
    text: Arc<Mutex<Text>>,
}

impl Gpu {
    pub fn new(window: &Arc<Window>) -> Result<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            ..Default::default()
        });
        let surface = instance.create_surface(window.clone())?;
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }))?;
        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
                label: Some("Lambda"),
                ..Default::default()
            }))?;
        let size = window.inner_size();
        let caps = surface.get_capabilities(&adapter);
        let &format = caps.formats.first().ok_or("No surface format")?;
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

        let rects = Rects::init(&device, format);
        let images = Images::init(&device, format);
        let text = Text::init(&device, &queue, format, config.width, config.height);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            rects,
            images,
            text,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.text
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .resize(width, height);
    }

    pub fn render(&mut self) {
        if let Some(frame) = self.acquire_frame() {
            let view = frame.texture.create_view(&Default::default());
            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            let (width, height) = (self.config.width, self.config.height);

            {
                // Get subsystems
                let mut rects = self.rects.lock().unwrap_or_else(|e| e.into_inner());
                let mut images = self.images.lock().unwrap_or_else(|e| e.into_inner());
                let mut text = self.text.lock().unwrap_or_else(|e| e.into_inner());

                // Prepare on GPU
                rects.prepare(&self.device, &self.queue, width, height);
                images.prepare(&self.device, &self.queue, width, height);
                let _ = text.prepare(&self.device, &self.queue, width, height);

                {
                    // Render to GPU
                    let mut pass = begin_pass(&mut encoder, &view);
                    rects.render(&mut pass);
                    images.render(&mut pass);
                    let _ = text.render(&mut pass);
                }
                text.trim();
            }

            self.queue.submit(Some(encoder.finish()));
            frame.present();
        };
    }

    fn acquire_frame(&mut self) -> Option<wgpu::SurfaceTexture> {
        match self.surface.get_current_texture() {
            Ok(tex) => Some(tex),
            Err(wgpu::SurfaceError::Outdated) => {
                self.surface.configure(&self.device, &self.config);
                self.surface.get_current_texture().ok()
            }
            Err(_) => None,
        }
    }
}

fn begin_pass<'a>(
    encoder: &'a mut wgpu::CommandEncoder,
    view: &'a wgpu::TextureView,
) -> wgpu::RenderPass<'a> {
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view,
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
    })
}
