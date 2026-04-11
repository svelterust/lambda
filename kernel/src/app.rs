use crate::input::{self, InputEvent, KEY_DOWN, KEY_UP, MOUSE_DOWN, MOUSE_MOVE, MOUSE_UP, SCROLL};
use crate::{gpu::Gpu, read_commands};
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, MouseScrollDelta, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::PhysicalKey,
    window::{Window, WindowId},
};

#[derive(Default)]
pub struct Lambda {
    window: Option<Arc<Window>>,
    gpu: Option<Gpu>,
    modifiers: u8,
    cursor_x: f32,
    cursor_y: f32,
}

impl ApplicationHandler for Lambda {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let attrs = Window::default_attributes().with_title("Lambda");
            match event_loop.create_window(attrs) {
                Ok(window) => {
                    let window = Arc::new(window);
                    match Gpu::new(&window) {
                        Ok(gpu) => {
                            self.gpu = Some(gpu);
                            window.request_redraw();
                            self.window = Some(window);
                        }
                        Err(_) => event_loop.exit(),
                    }
                }
                Err(_) => event_loop.exit(),
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(gpu) = self.gpu.as_mut() {
                    gpu.resize(size.width, size.height);
                }
            }
            WindowEvent::ModifiersChanged(mods) => {
                self.modifiers = input::modifiers_to_u8(&mods);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(keycode) = event.physical_key {
                    let code = input::keycode_to_u16(keycode);
                    if code != 0 {
                        let event_type = match event.state {
                            ElementState::Pressed => KEY_DOWN,
                            ElementState::Released => KEY_UP,
                        };
                        input::push_event(InputEvent {
                            event_type,
                            modifiers: self.modifiers,
                            code,
                            x: 0.0,
                            y: 0.0,
                        });
                        crate::call_input_callback();
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_x = position.x as f32;
                self.cursor_y = position.y as f32;
                input::push_event(InputEvent {
                    event_type: MOUSE_MOVE,
                    modifiers: self.modifiers,
                    code: 0,
                    x: self.cursor_x,
                    y: self.cursor_y,
                });
                crate::call_input_callback();
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let event_type = match state {
                    ElementState::Pressed => MOUSE_DOWN,
                    ElementState::Released => MOUSE_UP,
                };
                input::push_event(InputEvent {
                    event_type,
                    modifiers: self.modifiers,
                    code: input::mouse_button_to_u16(button),
                    x: self.cursor_x,
                    y: self.cursor_y,
                });
                crate::call_input_callback();
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let (dx, dy) = match delta {
                    MouseScrollDelta::LineDelta(x, y) => (x, y),
                    MouseScrollDelta::PixelDelta(pos) => (pos.x as f32, pos.y as f32),
                };
                input::push_event(InputEvent {
                    event_type: SCROLL,
                    modifiers: self.modifiers,
                    code: 0,
                    x: dx,
                    y: dy,
                });
                crate::call_input_callback();
            }
            WindowEvent::RedrawRequested => {
                crate::call_frame_callback();
                if let Some(gpu) = self.gpu.as_mut() {
                    gpu.render(read_commands());
                }
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}
