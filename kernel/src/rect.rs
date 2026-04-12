use indexmap::IndexMap;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

static RECTS: OnceLock<Arc<Mutex<Rects>>> = OnceLock::new();

const INITIAL_CAPACITY: usize = 256;

fn rects_lock() -> MutexGuard<'static, Rects> {
    RECTS
        .get()
        .expect("Rects not initialized")
        .lock()
        .unwrap_or_else(|e| e.into_inner())
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
    pos: [f32; 2],          //  0: 8 bytes
    size: [f32; 2],         //  8: 8 bytes
    color: [f32; 4],        // 16: 16 bytes
    radius: f32,            // 32: 4 bytes
    border_width: f32,      // 36: 4 bytes
    border_color: [f32; 4], // 40: 16 bytes
} // = 56 bytes

const SHADER: &str = "
struct Uniform {
    screen: vec2<f32>,
};
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
    @location(2) size: vec2<f32>,
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
    out.pos = vec4<f32>(ndc, 0.0, 1.0);
    out.color = v.inst_color;
    out.local_pos = v.quad_pos * v.inst_size;
    out.size = v.inst_size;
    out.radius = v.inst_radius;
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
    let half = v.size * 0.5;
    let p = v.local_pos - half;
    let r = min(v.radius, min(half.x, half.y));
    let dist = sdf_round_rect(p, half, r);

    // Outer edge anti-aliasing
    let outer_alpha = 1.0 - smoothstep(-0.5, 0.5, dist);
    if outer_alpha < 0.001 { discard; }

    // Border vs fill
    if v.border_width > 0.0 {
        let border_mix = smoothstep(-v.border_width - 0.5, -v.border_width + 0.5, dist);
        let color = mix(v.color, v.border_color, border_mix);
        return vec4<f32>(color.rgb, color.a * outer_alpha);
    }

    return vec4<f32>(v.color.rgb, v.color.a * outer_alpha);
}
";

pub struct Rects {
    slots: IndexMap<u32, Rect>,
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
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let vertex_layouts = [
            // Slot 0: per-vertex quad position
            wgpu::VertexBufferLayout {
                array_stride: 8,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 0,
                }],
            },
            // Slot 1: per-instance data (56 bytes)
            wgpu::VertexBufferLayout {
                array_stride: 56,
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: &[
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x2,
                        offset: 0,
                        shader_location: 1, // inst_pos
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x2,
                        offset: 8,
                        shader_location: 2, // inst_size
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x4,
                        offset: 16,
                        shader_location: 3, // inst_color
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32,
                        offset: 32,
                        shader_location: 4, // inst_radius
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32,
                        offset: 36,
                        shader_location: 5, // inst_border_width
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x4,
                        offset: 40,
                        shader_location: 6, // inst_border_color
                    },
                ],
            },
        ];

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("rect_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &vertex_layouts,
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

        // Unit quad: (0,0) (1,0) (1,1) (0,1)
        let vertices: [[f32; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        let vertex_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rect_verts"),
            size: std::mem::size_of_val(&vertices) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: true,
        });
        vertex_buf
            .slice(..)
            .get_mapped_range_mut()
            .copy_from_slice(bytemuck::cast_slice(&vertices));
        vertex_buf.unmap();

        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
        let index_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rect_idx"),
            size: std::mem::size_of_val(&indices) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: true,
        });
        index_buf
            .slice(..)
            .get_mapped_range_mut()
            .copy_from_slice(bytemuck::cast_slice(&indices));
        index_buf.unmap();

        let instance_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rect_instances"),
            size: (INITIAL_CAPACITY * std::mem::size_of::<RectInstance>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let arc = Arc::new(Mutex::new(Rects {
            slots: IndexMap::new(),
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
        queue.write_buffer(
            &self.uniform_buf,
            0,
            bytemuck::cast_slice(&[width as f32, height as f32]),
        );

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
        if self.instance_count > 0 {
            if count > self.capacity {
                let mut new_cap = self.capacity;
                while new_cap < count {
                    new_cap *= 2;
                }
                self.instance_buf = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("rect_instances"),
                    size: (new_cap * std::mem::size_of::<RectInstance>()) as u64,
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
    rects_lock().slots.shift_remove(&id);
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
        rect.color = [
            ((rgba >> 24) & 0xFF) as f32 / 255.0,
            ((rgba >> 16) & 0xFF) as f32 / 255.0,
            ((rgba >> 8) & 0xFF) as f32 / 255.0,
            (rgba & 0xFF) as f32 / 255.0,
        ];
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
        rect.border_color = [
            ((rgba >> 24) & 0xFF) as f32 / 255.0,
            ((rgba >> 16) & 0xFF) as f32 / 255.0,
            ((rgba >> 8) & 0xFF) as f32 / 255.0,
            (rgba & 0xFF) as f32 / 255.0,
        ];
    }
}
