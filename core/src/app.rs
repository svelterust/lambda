use crate::gpu::{DrawCmd, Gpu};
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

pub struct Lambda {
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

impl ApplicationHandler for Lambda {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            // Window
            let attrs = Window::default_attributes().with_title("Lambda");
            let window = match event_loop.create_window(attrs) {
                Ok(window) => Arc::new(window),
                Err(err) => {
                    eprintln!("Failed to create window: {err}");
                    event_loop.exit();
                    return;
                }
            };

            // GPU
            match Gpu::new(&window) {
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
                    gpu.resize(size.width, size.height);
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(gpu) = self.gpu.as_ref() {
                    let test_commands = vec![
                        DrawCmd {
                            x: 50.0,
                            y: 50.0,
                            w: 300.0,
                            h: 200.0,
                            color: 0x2244AAFF,
                        },
                        DrawCmd {
                            x: 200.0,
                            y: 150.0,
                            w: 400.0,
                            h: 250.0,
                            color: 0xCC4422FF,
                        },
                        DrawCmd {
                            x: 100.0,
                            y: 300.0,
                            w: 250.0,
                            h: 150.0,
                            color: 0x22AA44CC,
                        },
                    ];
                    gpu.render(&test_commands);
                }
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}
