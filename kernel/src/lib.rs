mod app;
mod gpu;
mod input;
mod logger;

use gpu::DrawCmd;
use std::ptr;
use std::ptr::addr_of_mut;
use std::sync::atomic::{AtomicPtr, AtomicU32, Ordering};
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
    logger::init();
    log::info!("Lambda started");
    match EventLoop::builder().with_any_thread(true).build() {
        Ok(mut event_loop) => {
            log::info!("Event loop created");
            let mut app = app::Lambda::default();
            panic!("Something terrible has happenned...");
            loop {
                if let PumpStatus::Exit(_) = event_loop.pump_app_events(None, &mut app) {
                    break;
                }
            }
        }
        Err(e) => {
            log::error!("Failed to build event loop: {e:?}");
        }
    }
    log::info!("Lambda exiting");
}

// Input callback (called from Rust on input events, runs Lisp code)
pub static INPUT_CALLBACK: AtomicPtr<()> = AtomicPtr::new(ptr::null_mut());

#[unsafe(no_mangle)]
pub extern "C" fn lambda_set_input_callback(cb: Option<extern "C" fn()>) {
    let ptr = cb.map_or(ptr::null_mut(), |f| f as *mut ());
    INPUT_CALLBACK.store(ptr, Ordering::Release);
}

pub fn call_input_callback() {
    let cb = INPUT_CALLBACK.load(Ordering::Acquire);
    if !cb.is_null() {
        let f: extern "C" fn() = unsafe { std::mem::transmute(cb) };
        f();
    }
}

// Frame callback (called once per vsync frame before render)
pub static FRAME_CALLBACK: AtomicPtr<()> = AtomicPtr::new(ptr::null_mut());

#[unsafe(no_mangle)]
pub extern "C" fn lambda_set_frame_callback(cb: Option<extern "C" fn()>) {
    let ptr = cb.map_or(ptr::null_mut(), |f| f as *mut ());
    FRAME_CALLBACK.store(ptr, Ordering::Release);
}

pub fn call_frame_callback() {
    let cb = FRAME_CALLBACK.load(Ordering::Acquire);
    if !cb.is_null() {
        let f: extern "C" fn() = unsafe { std::mem::transmute(cb) };
        f();
    }
}

pub fn read_commands() -> &'static [DrawCmd] {
    let count = DRAW_COUNT.load(Ordering::Acquire) as usize;
    unsafe { std::slice::from_raw_parts(addr_of_mut!(DRAW_BUF) as *const DrawCmd, count) }
}
