mod app;
mod gpu;
mod input;
mod logger;
mod text;

use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};
use winit::{
    event_loop::EventLoop,
    platform::{
        pump_events::{EventLoopExtPumpEvents, PumpStatus},
        wayland::EventLoopBuilderExtWayland,
    },
};
use app::Lambda;

// Start Lambda
#[unsafe(no_mangle)]
pub extern "C" fn lambda_run() {
    logger::init();
    if let Ok(mut event_loop) = EventLoop::builder().with_any_thread(true).build() {
        let mut app = Lambda::default();
        loop {
            if let PumpStatus::Exit(_) = event_loop.pump_app_events(None, &mut app) {
                break;
            }
        }
    };
}

// Notify Lisp callback on input
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
