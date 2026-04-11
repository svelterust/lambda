use anyhow::{Context, Result};
use glyphon::{
    Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache,
    TextArea, TextAtlas, TextBounds, TextRenderer, Viewport,
};
use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard, OnceLock};

static TEXT: OnceLock<Mutex<Text>> = OnceLock::new();

pub fn text_lock() -> Option<MutexGuard<'static, Text>> {
    TEXT.get()?.lock().ok()
}

struct TextSlot {
    buffer: Buffer,
    x: f32,
    y: f32,
    bounds: TextBounds,
    color: Color,
}

pub struct Text {
    font_system: FontSystem,
    swash_cache: SwashCache,
    viewport: Viewport,
    atlas: TextAtlas,
    renderer: TextRenderer,
    width: u32,
    height: u32,
    slots: HashMap<u32, TextSlot>,
    next_id: u32,
}

impl Text {
    pub fn init(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) {
        let font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let cache = Cache::new(device);
        let viewport = Viewport::new(device, &cache);
        let mut atlas = TextAtlas::new(device, queue, &cache, format);
        let renderer =
            TextRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);
        let _ = TEXT.set(Mutex::new(Text {
            font_system,
            swash_cache,
            viewport,
            atlas,
            renderer,
            width,
            height,
            slots: HashMap::new(),
            next_id: 1,
        }));
    }

    pub fn ffi_create(font_size: f32, line_height: f32) -> u32 {
        match text_lock() {
            Some(mut text) => {
                let id = text.next_id;
                text.next_id += 1;
                let w = text.width;
                let h = text.height;
                let mut buffer =
                    Buffer::new(&mut text.font_system, Metrics::new(font_size, line_height));
                buffer.set_size(&mut text.font_system, Some(w as f32), Some(h as f32));
                text.slots.insert(
                    id,
                    TextSlot {
                        buffer,
                        x: 0.0,
                        y: 0.0,
                        bounds: TextBounds {
                            left: 0,
                            top: 0,
                            right: w as i32,
                            bottom: h as i32,
                        },
                        color: Color::rgb(255, 255, 255),
                    },
                );
                id
            }
            None => 0,
        }
    }

    pub fn ffi_destroy(id: u32) {
        if let Some(mut text) = text_lock() {
            text.slots.remove(&id);
        }
    }

    pub fn ffi_set(id: u32, ptr: *const u8, len: u32) {
        let bytes = unsafe { std::slice::from_raw_parts(ptr, len as usize) };
        let Ok(s) = std::str::from_utf8(bytes) else {
            log::error!("text_set: invalid UTF-8");
            return;
        };
        if let Some(mut text) = text_lock() {
            // safe: splitting borrows between font_system and slots
            let fs = &mut text.font_system as *mut FontSystem;
            if let Some(slot) = text.slots.get_mut(&id) {
                slot.buffer.set_text(
                    unsafe { &mut *fs },
                    s,
                    &Attrs::new().family(Family::Name("JetBrains Mono NL")),
                    Shaping::Advanced,
                    None,
                );
                slot.buffer.shape_until_scroll(unsafe { &mut *fs }, false);
            }
        }
    }

    pub fn ffi_set_position(id: u32, x: f32, y: f32) {
        if let Some(mut text) = text_lock()
            && let Some(slot) = text.slots.get_mut(&id)
        {
            slot.x = x;
            slot.y = y;
        }
    }

    pub fn ffi_set_bounds(id: u32, left: i32, top: i32, right: i32, bottom: i32) {
        if let Some(mut text) = text_lock()
            && let Some(slot) = text.slots.get_mut(&id)
        {
            slot.bounds = TextBounds {
                left,
                top,
                right,
                bottom,
            };
        }
    }

    pub fn ffi_set_color(id: u32, rgba: u32) {
        if let Some(mut text) = text_lock()
            && let Some(slot) = text.slots.get_mut(&id)
        {
            let r = ((rgba >> 24) & 0xFF) as u8;
            let g = ((rgba >> 16) & 0xFF) as u8;
            let b = ((rgba >> 8) & 0xFF) as u8;
            let a = (rgba & 0xFF) as u8;
            slot.color = Color::rgba(r, g, b, a);
        }
    }

    pub fn ffi_set_metrics(id: u32, font_size: f32, line_height: f32) {
        if let Some(mut text) = text_lock() {
            // safe: splitting borrows between font_system and slots
            let fs = &mut text.font_system as *mut FontSystem;
            if let Some(slot) = text.slots.get_mut(&id) {
                slot.buffer
                    .set_metrics(unsafe { &mut *fs }, Metrics::new(font_size, line_height));
                slot.buffer.shape_until_scroll(unsafe { &mut *fs }, false);
            }
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        let fs = &mut self.font_system as *mut FontSystem;
        for slot in self.slots.values_mut() {
            slot.buffer
                .set_size(unsafe { &mut *fs }, Some(width as f32), Some(height as f32));
            slot.buffer.shape_until_scroll(unsafe { &mut *fs }, false);
        }
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) -> Result<()> {
        self.viewport.update(queue, Resolution { width, height });
        let areas = self.slots.values().map(|slot| TextArea {
            buffer: &slot.buffer,
            left: slot.x,
            top: slot.y,
            scale: 1.0,
            bounds: slot.bounds,
            default_color: slot.color,
            custom_glyphs: &[],
        });
        self.renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                areas,
                &mut self.swash_cache,
            )
            .context("Failed to prepare")
    }

    pub fn render(&self, pass: &mut wgpu::RenderPass<'_>) -> Result<()> {
        self.renderer
            .render(&self.atlas, &self.viewport, pass)
            .context("Failed to render")
    }

    pub fn trim(&mut self) {
        self.atlas.trim();
    }
}
