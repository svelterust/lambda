// Modules
mod app;
mod gpu;
mod input;
mod logger;
mod rect;
mod text;

use winit::{
    event_loop::EventLoop,
    platform::{
        pump_events::{EventLoopExtPumpEvents, PumpStatus},
        wayland::EventLoopBuilderExtWayland,
    },
};

#[unsafe(no_mangle)]
pub extern "C" fn lambda_run() {
    logger::init();
    if let Ok(mut event_loop) = EventLoop::builder().with_any_thread(true).build() {
        let mut app = app::Lambda::default();
        loop {
            if let PumpStatus::Exit(_) = event_loop.pump_app_events(None, &mut app) {
                break;
            }
        }
    };
}
