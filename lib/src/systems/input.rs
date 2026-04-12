use std::ptr::{self, addr_of_mut};
use std::sync::atomic::{AtomicPtr, AtomicU32, Ordering};
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

// Event types
pub const KEY_DOWN: u8 = 1;
pub const KEY_UP: u8 = 2;
pub const MOUSE_MOVE: u8 = 3;
pub const MOUSE_DOWN: u8 = 4;
pub const MOUSE_UP: u8 = 5;
pub const SCROLL: u8 = 6;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct InputEvent {
    pub event_type: u8,
    pub modifiers: u8,
    pub code: u16,
    pub x: f32,
    pub y: f32,
}

impl InputEvent {
    const ZERO: Self = Self {
        event_type: 0,
        modifiers: 0,
        code: 0,
        x: 0.0,
        y: 0.0,
    };
}

// Ring buffer
const CAPACITY: usize = 256;
const MASK: u32 = (CAPACITY as u32) - 1;
static mut INPUT_BUF: [InputEvent; CAPACITY] = [InputEvent::ZERO; CAPACITY];
static INPUT_WRITE: AtomicU32 = AtomicU32::new(0);
static INPUT_READ: AtomicU32 = AtomicU32::new(0);

pub fn push_event(event: InputEvent) {
    let w = INPUT_WRITE.load(Ordering::Relaxed);
    unsafe {
        INPUT_BUF[(w & MASK) as usize] = event;
    }
    INPUT_WRITE.store(w.wrapping_add(1), Ordering::Release);
}

#[unsafe(no_mangle)]
pub extern "C" fn lambda_input_buf_ptr() -> *const InputEvent {
    addr_of_mut!(INPUT_BUF) as *const InputEvent
}

#[unsafe(no_mangle)]
pub extern "C" fn lambda_input_write_index() -> u32 {
    INPUT_WRITE.load(Ordering::Acquire)
}

#[unsafe(no_mangle)]
pub extern "C" fn lambda_input_set_read_index(n: u32) {
    INPUT_READ.store(n, Ordering::Release);
}

// Modifier bits
pub const MOD_SHIFT: u8 = 1;
pub const MOD_CTRL: u8 = 2;
pub const MOD_ALT: u8 = 4;
pub const MOD_SUPER: u8 = 8;

pub fn modifiers_to_u8(mods: &winit::event::Modifiers) -> u8 {
    let state = mods.state();
    let mut m: u8 = 0;
    if state.shift_key() {
        m |= MOD_SHIFT;
    }
    if state.control_key() {
        m |= MOD_CTRL;
    }
    if state.alt_key() {
        m |= MOD_ALT;
    }
    if state.super_key() {
        m |= MOD_SUPER;
    }
    m
}

pub fn keycode_to_u16(key: KeyCode) -> u16 {
    match key {
        // Letters 1..26
        KeyCode::KeyA => 1,
        KeyCode::KeyB => 2,
        KeyCode::KeyC => 3,
        KeyCode::KeyD => 4,
        KeyCode::KeyE => 5,
        KeyCode::KeyF => 6,
        KeyCode::KeyG => 7,
        KeyCode::KeyH => 8,
        KeyCode::KeyI => 9,
        KeyCode::KeyJ => 10,
        KeyCode::KeyK => 11,
        KeyCode::KeyL => 12,
        KeyCode::KeyM => 13,
        KeyCode::KeyN => 14,
        KeyCode::KeyO => 15,
        KeyCode::KeyP => 16,
        KeyCode::KeyQ => 17,
        KeyCode::KeyR => 18,
        KeyCode::KeyS => 19,
        KeyCode::KeyT => 20,
        KeyCode::KeyU => 21,
        KeyCode::KeyV => 22,
        KeyCode::KeyW => 23,
        KeyCode::KeyX => 24,
        KeyCode::KeyY => 25,
        KeyCode::KeyZ => 26,

        // Digits 30..39
        KeyCode::Digit0 => 30,
        KeyCode::Digit1 => 31,
        KeyCode::Digit2 => 32,
        KeyCode::Digit3 => 33,
        KeyCode::Digit4 => 34,
        KeyCode::Digit5 => 35,
        KeyCode::Digit6 => 36,
        KeyCode::Digit7 => 37,
        KeyCode::Digit8 => 38,
        KeyCode::Digit9 => 39,

        // Common keys 40..49
        KeyCode::Space => 40,
        KeyCode::Enter => 41,
        KeyCode::Escape => 42,
        KeyCode::Backspace => 43,
        KeyCode::Tab => 44,
        KeyCode::Delete => 45,
        KeyCode::Insert => 46,
        KeyCode::Home => 47,
        KeyCode::End => 48,
        KeyCode::PageUp => 49,
        KeyCode::PageDown => 50,

        // Punctuation 55..69
        KeyCode::Comma => 55,
        KeyCode::Period => 56,
        KeyCode::Slash => 57,
        KeyCode::Semicolon => 58,
        KeyCode::Quote => 59,
        KeyCode::BracketLeft => 60,
        KeyCode::BracketRight => 61,
        KeyCode::Backslash => 62,
        KeyCode::Minus => 63,
        KeyCode::Equal => 64,
        KeyCode::Backquote => 65,

        // Arrows 80..83
        KeyCode::ArrowUp => 80,
        KeyCode::ArrowDown => 81,
        KeyCode::ArrowLeft => 82,
        KeyCode::ArrowRight => 83,

        // Modifiers 90..97
        KeyCode::ShiftLeft => 90,
        KeyCode::ShiftRight => 91,
        KeyCode::ControlLeft => 92,
        KeyCode::ControlRight => 93,
        KeyCode::AltLeft => 94,
        KeyCode::AltRight => 95,
        KeyCode::SuperLeft => 96,
        KeyCode::SuperRight => 97,

        // F-keys 100..111
        KeyCode::F1 => 100,
        KeyCode::F2 => 101,
        KeyCode::F3 => 102,
        KeyCode::F4 => 103,
        KeyCode::F5 => 104,
        KeyCode::F6 => 105,
        KeyCode::F7 => 106,
        KeyCode::F8 => 107,
        KeyCode::F9 => 108,
        KeyCode::F10 => 109,
        KeyCode::F11 => 110,
        KeyCode::F12 => 111,

        // Misc
        KeyCode::CapsLock => 120,
        KeyCode::NumLock => 121,
        KeyCode::ScrollLock => 122,
        KeyCode::PrintScreen => 123,
        KeyCode::Pause => 124,

        _ => 0,
    }
}

// Input callback
static INPUT_CALLBACK: AtomicPtr<()> = AtomicPtr::new(ptr::null_mut());

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

pub fn mouse_button_to_u16(button: MouseButton) -> u16 {
    match button {
        MouseButton::Left => 1,
        MouseButton::Right => 2,
        MouseButton::Middle => 3,
        MouseButton::Back => 4,
        MouseButton::Forward => 5,
        MouseButton::Other(n) => 10 + n,
    }
}
