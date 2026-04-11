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

#[unsafe(no_mangle)]
pub extern "C" fn lambda_run() {
    logger::init();
    log::info!("Lambda started");
    match EventLoop::builder().with_any_thread(true).build() {
        Ok(mut event_loop) => {
            log::info!("Event loop created");
            let mut app = app::Lambda::default();
            loop {
                if let PumpStatus::Exit(_) = event_loop.pump_app_events(None, &mut app) {
                    break;
                }
            }
        }
        Err(err) => {
            log::error!("Failed to build event loop: {err:?}");
        }
    }
    log::info!("Lambda exiting");
}

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

#[unsafe(no_mangle)]
pub extern "C" fn lambda_text_create(font_size: f32, line_height: f32) -> u32 {
    text::Text::ffi_create(font_size, line_height)
}

#[unsafe(no_mangle)]
pub extern "C" fn lambda_text_destroy(id: u32) {
    text::Text::ffi_destroy(id);
}

#[unsafe(no_mangle)]
pub extern "C" fn lambda_text_set(id: u32, ptr: *const u8, len: u32) {
    text::Text::ffi_set(id, ptr, len);
}

#[unsafe(no_mangle)]
pub extern "C" fn lambda_text_position(id: u32, x: f32, y: f32) {
    text::Text::ffi_set_position(id, x, y);
}

#[unsafe(no_mangle)]
pub extern "C" fn lambda_text_bounds(id: u32, left: i32, top: i32, right: i32, bottom: i32) {
    text::Text::ffi_set_bounds(id, left, top, right, bottom);
}

#[unsafe(no_mangle)]
pub extern "C" fn lambda_text_color(id: u32, rgba: u32) {
    text::Text::ffi_set_color(id, rgba);
}

#[unsafe(no_mangle)]
pub extern "C" fn lambda_text_metrics(id: u32, font_size: f32, line_height: f32) {
    text::Text::ffi_set_metrics(id, font_size, line_height);
}
