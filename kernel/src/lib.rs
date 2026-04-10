// Modules
mod app;
mod gpu;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DrawCmd {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub color: u32,
}

use winit::{
    event_loop::EventLoop,
    platform::{
        pump_events::{EventLoopExtPumpEvents, PumpStatus},
        wayland::EventLoopBuilderExtWayland,
    },
};

#[unsafe(no_mangle)]
pub extern "C" fn lambda_run() {
    let Ok(mut event_loop) = EventLoop::builder().with_any_thread(true).build() else {
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
