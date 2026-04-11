mod app;
mod gpu;

use gpu::DrawCmd;
use std::ptr::addr_of_mut;
use std::sync::atomic::{AtomicU32, Ordering};
use winit::{
    event_loop::EventLoop,
    platform::{
        pump_events::{EventLoopExtPumpEvents, PumpStatus},
        wayland::EventLoopBuilderExtWayland,
    },
};

// Shared draw command buffer (written by Lisp, read by renderer)
const CAPACITY: usize = 1024;
static DRAW_COUNT: AtomicU32 = AtomicU32::new(0);
static mut DRAW_BUF: [DrawCmd; CAPACITY] = [DrawCmd::ZERO_CMD; CAPACITY];

#[unsafe(no_mangle)]
pub extern "C" fn lambda_buf_ptr() -> *mut DrawCmd {
    addr_of_mut!(DRAW_BUF) as *mut DrawCmd
}

#[unsafe(no_mangle)]
pub extern "C" fn lambda_buf_set_count(n: u32) {
    DRAW_COUNT.store(n.min(CAPACITY as u32), Ordering::Release);
}

#[unsafe(no_mangle)]
pub extern "C" fn lambda_run() {
    if let Ok(mut event_loop) = EventLoop::builder().with_any_thread(true).build() {
        let mut app = app::Lambda::default();
        loop {
            if let PumpStatus::Exit(_) = event_loop.pump_app_events(None, &mut app) {
                break;
            }
        }
    }
}

pub fn read_commands() -> &'static [DrawCmd] {
    let count = DRAW_COUNT.load(Ordering::Acquire) as usize;
    unsafe { std::slice::from_raw_parts(addr_of_mut!(DRAW_BUF) as *const DrawCmd, count) }
}
