mod app;
mod gpu;
mod systems;

use winit::{
    event_loop::EventLoop,
    platform::{
        pump_events::{EventLoopExtPumpEvents, PumpStatus},
        wayland::EventLoopBuilderExtWayland,
    },
};

type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

#[unsafe(no_mangle)]
pub extern "C" fn lambda_run() {
    std::panic::set_hook(Box::new(|info| {
        let bt = std::backtrace::Backtrace::force_capture();
        let _ = std::fs::write("/tmp/lambda.log", format!("{info}\n{bt}"));
    }));
    let mut event_loop = EventLoop::builder()
        .with_any_thread(true)
        .build()
        .expect("Failed to create event loop");
    let mut app = app::Lambda::default();
    while let PumpStatus::Continue = event_loop.pump_app_events(None, &mut app) {}
}
