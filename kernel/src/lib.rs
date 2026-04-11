mod app;
mod gpu;

use gpu::DrawCmd;

use std::ptr::addr_of_mut;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;
use winit::{
    event_loop::EventLoop,
    platform::{
        pump_events::{EventLoopExtPumpEvents, PumpStatus},
        wayland::EventLoopBuilderExtWayland,
    },
};

const CAPACITY: usize = 1024;

static mut DRAW_BUF: [DrawCmd; CAPACITY] = [DrawCmd {
    x: 0.0,
    y: 0.0,
    w: 0.0,
    h: 0.0,
    color: 0,
}; CAPACITY];

static DRAW_COUNT: AtomicU32 = AtomicU32::new(0);

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
    let Ok(mut event_loop) = EventLoop::builder().with_any_thread(true).build() else {
        return;
    };
    let mut app = app::Lambda::new();

    loop {
        if let PumpStatus::Exit(_) =
            event_loop.pump_app_events(Some(Duration::from_millis(16)), &mut app)
        {
            break;
        }
    }
}

pub fn read_commands() -> &'static [DrawCmd] {
    let count = DRAW_COUNT.load(Ordering::Acquire) as usize;
    unsafe { std::slice::from_raw_parts(addr_of_mut!(DRAW_BUF) as *const DrawCmd, count) }
}
