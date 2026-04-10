mod app;
mod gpu;

pub use gpu::DrawCmd;

use winit::{
    event_loop::EventLoop,
    platform::pump_events::{EventLoopExtPumpEvents, PumpStatus},
};

#[unsafe(no_mangle)]
pub extern "C" fn lambda_run() {
    let Ok(mut event_loop) = EventLoop::new() else {
        eprintln!("Failed to create event loop");
        return;
    };
    let mut app = app::Lambda::default();

    loop {
        if let PumpStatus::Exit(_) = event_loop.pump_app_events(None, &mut app) {
            break;
        }
    }
}
