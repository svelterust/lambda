use anyhow::{Context, Result};
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    platform::pump_events::{EventLoopExtPumpEvents, PumpStatus},
    window::{Window, WindowId},
};

struct Gpu {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
}

struct Lambda {
    window: Option<Arc<Window>>,
    gpu: Option<Gpu>,
}

impl Default for Lambda {
    fn default() -> Self {
        Self {
            window: None,
            gpu: None,
        }
    }
}

fn init_gpu(window: &Arc<Window>) -> Result<Gpu> {
    // Instance
    let mut desc = wgpu::InstanceDescriptor::new_without_display_handle();
    desc.backends = wgpu::Backends::VULKAN;
    let instance = wgpu::Instance::new(desc);

    // Device
    let surface = instance.create_surface(window.clone())?;
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        compatible_surface: Some(&surface),
        ..Default::default()
    }))?;
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("lambda"),
        ..Default::default()
    }))?;

    // Surface configuration
    let size = window.inner_size();
    let caps = surface.get_capabilities(&adapter);
    let &format = caps.formats.first().context("No surface format")?;
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

    Ok(Gpu {
        surface,
        device,
        queue,
        config,
    })
}

fn render(gpu: &Gpu) {
    // Acquire framebuffer
    let frame = match gpu.surface.get_current_texture() {
        wgpu::CurrentSurfaceTexture::Success(tex)
        | wgpu::CurrentSurfaceTexture::Suboptimal(tex) => tex,
        _ => return,
    };

    // Encode commands
    let view = frame.texture.create_view(&Default::default());
    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    // Clear to white
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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

    // Submit + present
    gpu.queue.submit(Some(encoder.finish()));
    frame.present();
}

impl ApplicationHandler for Lambda {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            // Window
            let attrs = Window::default_attributes()
                .with_title("Lambda")
                .with_maximized(true);

            let window = match event_loop.create_window(attrs) {
                Ok(w) => Arc::new(w),
                Err(err) => {
                    eprintln!("Failed to create window: {err}");
                    event_loop.exit();
                    return;
                }
            };

            // GPU
            match init_gpu(&window) {
                Ok(gpu) => {
                    self.gpu = Some(gpu);
                    window.request_redraw();
                    self.window = Some(window);
                }
                Err(err) => {
                    eprintln!("Failed to initialize GPU: {err}");
                    event_loop.exit();
                }
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(gpu) = self.gpu.as_mut() {
                    gpu.config.width = size.width;
                    gpu.config.height = size.height;
                    gpu.surface.configure(&gpu.device, &gpu.config);
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(gpu) = self.gpu.as_ref() {
                    render(gpu);
                }
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn lambda_run() {
    // Create event loop
    let Ok(mut event_loop) = EventLoop::new() else {
        eprintln!("Failed to create event loop");
        return;
    };
    let mut app = Lambda::default();

    // Run event loop until exit
    loop {
        if let PumpStatus::Exit(_) = event_loop.pump_app_events(None, &mut app) {
            break;
        }
    }
}
