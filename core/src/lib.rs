use std::time::Duration;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    platform::pump_events::{EventLoopExtPumpEvents, PumpStatus},
    window::{Window, WindowId},
};

#[derive(Default)]
struct Lambda {
    window: Option<Window>,
}

impl ApplicationHandler for Lambda {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let attrs = Window::default_attributes()
                .with_title("Lambda")
                .with_inner_size(winit::dpi::LogicalSize::new(800, 600));
            match event_loop.create_window(attrs) {
                Ok(window) => self.window = Some(window),
                Err(err) => {
                    eprintln!("Failed to create window: {err}");
                    event_loop.exit();
                }
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if let WindowEvent::CloseRequested = event {
            event_loop.exit();
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn lambda_run() {
    match EventLoop::new() {
        Ok(mut event_loop) => {
            let mut lambda = Lambda::default();
            let timeout = Some(Duration::from_millis(16));
            loop {
                if let PumpStatus::Exit(_) = event_loop.pump_app_events(timeout, &mut lambda) {
                    break;
                }
            }
        }
        Err(err) => eprintln!("Failed to create event loop: {err:?}"),
    }
}
