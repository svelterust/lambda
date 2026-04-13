use glyphon::{
    Attrs, AttrsList, Buffer, Cache, Color, FamilyOwned, FontSystem, Metrics, Resolution, Shaping,
    SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer, Viewport, Weight,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use crate::Result;

static TEXT: OnceLock<Arc<Mutex<Text>>> = OnceLock::new();

fn text_lock() -> MutexGuard<'static, Text> {
    TEXT.get()
        .expect("Text not initialized")
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

struct TextSlot {
    buffer: Buffer,
    x: f32,
    y: f32,
    bounds: TextBounds,
    color: Color,
    weight: Weight,
    family: FamilyOwned,
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
    ) -> Arc<Mutex<Text>> {
        let font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let cache = Cache::new(device);
        let viewport = Viewport::new(device, &cache);
        let mut atlas = TextAtlas::new(device, queue, &cache, format);
        let renderer =
            TextRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);
        let arc = Arc::new(Mutex::new(Text {
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
        let _ = TEXT.set(Arc::clone(&arc));
        arc
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        let Text {
            font_system, slots, ..
        } = self;
        for slot in slots.values_mut() {
            slot.buffer
                .set_size(font_system, Some(width as f32), Some(height as f32));
            slot.buffer.shape_until_scroll(font_system, false);
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
        // Map each slot to a glyphon TextArea for rendering
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
            .map_err(|e| e.into())
    }

    pub fn render(&self, pass: &mut wgpu::RenderPass<'_>) -> Result<()> {
        self.renderer
            .render(&self.atlas, &self.viewport, pass)
            .map_err(|e| e.into())
    }

    pub fn trim(&mut self) {
        self.atlas.trim();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn lambda_text_create(font_size: f32) -> u32 {
    let mut text = text_lock();
    let id = text.next_id;
    text.next_id += 1;
    let w = text.width;
    let h = text.height;
    let mut buffer = Buffer::new(
        &mut text.font_system,
        Metrics::new(font_size, font_size * 1.4),
    );
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
            color: Color::rgb(0, 0, 0),
            weight: Weight::NORMAL,
            family: FamilyOwned::SansSerif,
        },
    );
    id
}

#[unsafe(no_mangle)]
pub extern "C" fn lambda_text_destroy(id: u32) {
    text_lock().slots.remove(&id);
}

#[unsafe(no_mangle)]
pub extern "C" fn lambda_text_set(id: u32, ptr: *const u8, len: u32) {
    let bytes = unsafe { std::slice::from_raw_parts(ptr, len as usize) };
    let Ok(s) = std::str::from_utf8(bytes) else {
        return;
    };
    let mut text = text_lock();
    let Text {
        font_system, slots, ..
    } = &mut *text;
    if let Some(slot) = slots.get_mut(&id) {
        let attrs = Attrs::new()
            .family(slot.family.as_family())
            .weight(slot.weight);
        slot.buffer
            .set_text(font_system, s, &attrs, Shaping::Advanced, None);
        slot.buffer.shape_until_scroll(font_system, false);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn lambda_text_position(id: u32, x: f32, y: f32) {
    if let Some(slot) = text_lock().slots.get_mut(&id) {
        slot.x = x;
        slot.y = y;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn lambda_text_bounds(id: u32, left: i32, top: i32, right: i32, bottom: i32) {
    if let Some(slot) = text_lock().slots.get_mut(&id) {
        slot.bounds = TextBounds {
            left,
            top,
            right,
            bottom,
        };
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn lambda_text_color(id: u32, rgba: u32) {
    let rgba = if rgba <= 0xFFFFFF {
        (rgba << 8) | 0xFF
    } else {
        rgba
    };
    if let Some(slot) = text_lock().slots.get_mut(&id) {
        let r = ((rgba >> 24) & 0xFF) as u8;
        let g = ((rgba >> 16) & 0xFF) as u8;
        let b = ((rgba >> 8) & 0xFF) as u8;
        let a = (rgba & 0xFF) as u8;
        slot.color = Color::rgba(r, g, b, a);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn lambda_text_metrics(id: u32, font_size: f32, line_height: f32) {
    let mut text = text_lock();
    let Text {
        font_system, slots, ..
    } = &mut *text;
    if let Some(slot) = slots.get_mut(&id) {
        slot.buffer
            .set_metrics(font_system, Metrics::new(font_size, line_height));
        slot.buffer.shape_until_scroll(font_system, false);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn lambda_text_font_size(id: u32, font_size: f32) {
    let mut text = text_lock();
    let Text {
        font_system, slots, ..
    } = &mut *text;
    if let Some(slot) = slots.get_mut(&id) {
        slot.buffer
            .set_metrics(font_system, Metrics::new(font_size, font_size * 1.4));
        slot.buffer.shape_until_scroll(font_system, false);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn lambda_text_weight(id: u32, weight: u32) {
    let mut text = text_lock();
    let Text {
        font_system, slots, ..
    } = &mut *text;
    if let Some(slot) = slots.get_mut(&id) {
        slot.weight = Weight(weight as u16);
        let attrs = Attrs::new()
            .family(slot.family.as_family())
            .weight(slot.weight);
        for line in &mut slot.buffer.lines {
            line.set_attrs_list(AttrsList::new(&attrs));
        }
        slot.buffer.shape_until_scroll(font_system, false);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn lambda_text_family(id: u32, ptr: *const u8, len: u32) {
    let bytes = unsafe { std::slice::from_raw_parts(ptr, len as usize) };
    let Ok(s) = std::str::from_utf8(bytes) else {
        return;
    };
    let mut text = text_lock();
    let Text {
        font_system, slots, ..
    } = &mut *text;
    if let Some(slot) = slots.get_mut(&id) {
        slot.family = FamilyOwned::Name(s.into());
        let attrs = Attrs::new()
            .family(slot.family.as_family())
            .weight(slot.weight);
        for line in &mut slot.buffer.lines {
            line.set_attrs_list(AttrsList::new(&attrs));
        }
        slot.buffer.shape_until_scroll(font_system, false);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn lambda_text_width(id: u32) -> f32 {
    let text = text_lock();
    text.slots.get(&id).map_or(0.0, |slot| {
        slot.buffer
            .layout_runs()
            .map(|run| run.line_w)
            .fold(0.0f32, f32::max)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn lambda_text_height(id: u32) -> f32 {
    let text = text_lock();
    text.slots.get(&id).map_or(0.0, |slot| {
        slot.buffer
            .layout_runs()
            .map(|run| run.line_top + run.line_height)
            .fold(0.0f32, f32::max)
    })
}
