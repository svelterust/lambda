use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use wgpu::util::DeviceExt;

static RECTS: OnceLock<Arc<Mutex<Rects>>> = OnceLock::new();

const SHADER: &str = "
struct Uniform { screen: vec2<f32> };

@group(0) @binding(0) var<uniform> u: Uniform;

struct VsIn {
    @location(0) quad_pos: vec2<f32>,
    @location(1) inst_pos: vec2<f32>,
    @location(2) inst_size: vec2<f32>,
    @location(3) inst_color: vec4<f32>,
    @location(4) inst_radius: f32,
    @location(5) inst_border_width: f32,
    @location(6) inst_border_color: vec4<f32>,
};

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) local_pos: vec2<f32>,
    @location(2) half_size: vec2<f32>,
    @location(3) radius: f32,
    @location(4) border_width: f32,
    @location(5) border_color: vec4<f32>,
};

@vertex
fn vs_main(v: VsIn) -> VsOut {
    var out: VsOut;
    let px = v.quad_pos * v.inst_size + v.inst_pos;
    let ndc = vec2<f32>(
        px.x / u.screen.x * 2.0 - 1.0,
        1.0 - px.y / u.screen.y * 2.0,
    );
    let half = v.inst_size * 0.5;
    out.pos = vec4<f32>(ndc, 0.0, 1.0);
    out.color = v.inst_color;
    out.local_pos = v.quad_pos * v.inst_size;
    out.half_size = half;
    out.radius = min(v.inst_radius, min(half.x, half.y));
    out.border_width = v.inst_border_width;
    out.border_color = v.inst_border_color;
    return out;
}

fn sdf_round_rect(p: vec2<f32>, half: vec2<f32>, r: f32) -> f32 {
    let q = abs(p) - half + r;
    return length(max(q, vec2(0.0))) + min(max(q.x, q.y), 0.0) - r;
}

@fragment
fn fs_main(v: VsOut) -> @location(0) vec4<f32> {
    let p = v.local_pos - v.half_size;
    let dist = sdf_round_rect(p, v.half_size, v.radius);
    let outer_alpha = 1.0 - smoothstep(-0.5, 0.5, dist);
    if outer_alpha < 0.001 { discard; }
    let border_mix = smoothstep(-v.border_width - 0.5, -v.border_width + 0.5, dist) * step(0.001, v.border_width);
    let color = mix(v.color, v.border_color, border_mix);
    return vec4<f32>(color.rgb, color.a * outer_alpha);
}
";

const INITIAL_CAPACITY: usize = 256;
const QUAD_VERTS: [[f32; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
const QUAD_INDICES: [u16; 6] = [0, 1, 2, 0, 2, 3];

fn rects_lock() -> MutexGuard<'static, Rects> {
    RECTS
        .get()
        .expect("Rects not initialized")
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

/// Convert packed #xRRGGBB or #xRRGGBBAA to [0.0..1.0] floats.
fn rgba_to_f32(rgba: u32) -> [f32; 4] {
    let rgba = if rgba <= 0xFFFFFF {
        (rgba << 8) | 0xFF
    } else {
        rgba
    };
    [
        ((rgba >> 24) & 0xFF) as f32 / 255.0,
        ((rgba >> 16) & 0xFF) as f32 / 255.0,
        ((rgba >> 8) & 0xFF) as f32 / 255.0,
        (rgba & 0xFF) as f32 / 255.0,
    ]
}

struct Rect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    color: [f32; 4],
    radius: f32,
    border_width: f32,
    border_color: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct RectInstance {
    pos: [f32; 2],
    size: [f32; 2],
    color: [f32; 4],
    radius: f32,
    border_width: f32,
    border_color: [f32; 4],
}

const INSTANCE_ATTRS: &[wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
    1 => Float32x2, // pos
    2 => Float32x2, // size
    3 => Float32x4, // color
    4 => Float32,   // radius
    5 => Float32,   // border_width
    6 => Float32x4, // border_color
];

pub struct Rects {
    slots: BTreeMap<u32, Rect>,
    next_id: u32,
    pipeline: wgpu::RenderPipeline,
    vertex_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,
    instance_buf: wgpu::Buffer,
    uniform_buf: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    capacity: usize,
    instance_count: u32,
}

impl Rects {
    pub fn init(device: &wgpu::Device, format: wgpu::TextureFormat) -> Arc<Mutex<Rects>> {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("rect_shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });

        let uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rect_uniform"),
            size: 8,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("rect_bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("rect_bg"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buf.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("rect_pl"),
            bind_group_layouts: &[&bind_group_layout],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("rect_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: size_of::<[f32; 2]>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float32x2],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: size_of::<RectInstance>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: INSTANCE_ATTRS,
                    },
                ],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("rect_verts"),
            contents: bytemuck::cast_slice(&QUAD_VERTS),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("rect_idx"),
            contents: bytemuck::cast_slice(&QUAD_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let instance_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rect_instances"),
            size: (INITIAL_CAPACITY * size_of::<RectInstance>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let arc = Arc::new(Mutex::new(Rects {
            slots: BTreeMap::new(),
            next_id: 1,
            pipeline,
            vertex_buf,
            index_buf,
            instance_buf,
            uniform_buf,
            bind_group,
            capacity: INITIAL_CAPACITY,
            instance_count: 0,
        }));
        let _ = RECTS.set(Arc::clone(&arc));
        arc
    }

    pub fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, width: u32, height: u32) {
        // Screen size uniform for pixel -> NDC conversion
        queue.write_buffer(
            &self.uniform_buf,
            0,
            bytemuck::cast_slice(&[width as f32, height as f32]),
        );

        // Collect slot data into GPU instance buffer
        let instances = self
            .slots
            .values()
            .map(|r| RectInstance {
                pos: [r.x, r.y],
                size: [r.width, r.height],
                color: r.color,
                radius: r.radius,
                border_width: r.border_width,
                border_color: r.border_color,
            })
            .collect::<Vec<_>>();

        let count = instances.len();
        self.instance_count = count as u32;
        if count > 0 {
            // Grow buffer with power-of-2 strategy
            if count > self.capacity {
                let mut new_cap = self.capacity;
                while new_cap < count {
                    new_cap *= 2;
                }
                self.instance_buf = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("rect_instances"),
                    size: (new_cap * size_of::<RectInstance>()) as u64,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                self.capacity = new_cap;
            }
            queue.write_buffer(&self.instance_buf, 0, bytemuck::cast_slice(&instances));
        }
    }

    pub fn render<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        if self.instance_count > 0 {
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.set_vertex_buffer(0, self.vertex_buf.slice(..));
            pass.set_vertex_buffer(1, self.instance_buf.slice(..));
            pass.set_index_buffer(self.index_buf.slice(..), wgpu::IndexFormat::Uint16);
            pass.draw_indexed(0..6, 0, 0..self.instance_count);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn lambda_rect_create() -> u32 {
    let mut rects = rects_lock();
    let id = rects.next_id;
    rects.next_id += 1;
    rects.slots.insert(
        id,
        Rect {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            color: [0.0, 0.0, 0.0, 1.0],
            radius: 0.0,
            border_width: 0.0,
            border_color: [0.0, 0.0, 0.0, 0.0],
        },
    );
    id
}

#[unsafe(no_mangle)]
pub extern "C" fn lambda_rect_destroy(id: u32) {
    rects_lock().slots.remove(&id);
}

#[unsafe(no_mangle)]
pub extern "C" fn lambda_rect_position(id: u32, x: f32, y: f32) {
    if let Some(rect) = rects_lock().slots.get_mut(&id) {
        rect.x = x;
        rect.y = y;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn lambda_rect_size(id: u32, w: f32, h: f32) {
    if let Some(rect) = rects_lock().slots.get_mut(&id) {
        rect.width = w;
        rect.height = h;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn lambda_rect_color(id: u32, rgba: u32) {
    if let Some(rect) = rects_lock().slots.get_mut(&id) {
        rect.color = rgba_to_f32(rgba);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn lambda_rect_radius(id: u32, radius: f32) {
    if let Some(rect) = rects_lock().slots.get_mut(&id) {
        rect.radius = radius;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn lambda_rect_border(id: u32, width: f32, rgba: u32) {
    if let Some(rect) = rects_lock().slots.get_mut(&id) {
        rect.border_width = width;
        rect.border_color = rgba_to_f32(rgba);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn lambda_rect_border_width(id: u32, width: f32) {
    if let Some(rect) = rects_lock().slots.get_mut(&id) {
        rect.border_width = width;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn lambda_rect_border_color(id: u32, rgba: u32) {
    if let Some(rect) = rects_lock().slots.get_mut(&id) {
        rect.border_color = rgba_to_f32(rgba);
    }
}
