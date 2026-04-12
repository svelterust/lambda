use resvg::usvg::fontdb;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use wgpu::util::DeviceExt;

static IMAGES: OnceLock<Arc<Mutex<Images>>> = OnceLock::new();

const SHADER: &str = "
struct Screen { size: vec2<f32> };
struct Img { pos: vec2<f32>, size: vec2<f32> };

@group(0) @binding(0) var<uniform> screen: Screen;
@group(1) @binding(0) var<uniform> img: Img;
@group(1) @binding(1) var tex: texture_2d<f32>;
@group(1) @binding(2) var samp: sampler;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@location(0) quad_pos: vec2<f32>) -> VsOut {
    var out: VsOut;
    let px = quad_pos * img.size + img.pos;
    let ndc = vec2<f32>(
        px.x / screen.size.x * 2.0 - 1.0,
        1.0 - px.y / screen.size.y * 2.0,
    );
    out.pos = vec4<f32>(ndc, 0.0, 1.0);
    out.uv = quad_pos;
    return out;
}

@fragment
fn fs_main(v: VsOut) -> @location(0) vec4<f32> {
    return textureSample(tex, samp, v.uv);
}
";

const QUAD_VERTS: [[f32; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
const QUAD_INDICES: [u16; 6] = [0, 1, 2, 0, 2, 3];

fn images_lock() -> MutexGuard<'static, Images> {
    IMAGES
        .get()
        .expect("Images not initialized")
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

struct ImageSlot {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    natural_width: u32,
    natural_height: u32,
    pending_pixels: Option<Vec<u8>>,
    gpu: Option<ImageGpu>,
}

struct ImageGpu {
    bind_group: wgpu::BindGroup,
    uniform_buf: wgpu::Buffer,
}

pub struct Images {
    slots: BTreeMap<u32, ImageSlot>,
    next_id: u32,
    pipeline: wgpu::RenderPipeline,
    vertex_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,
    screen_uniform_buf: wgpu::Buffer,
    screen_bind_group: wgpu::BindGroup,
    per_image_bgl: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    fontdb: Arc<fontdb::Database>,
}

impl Images {
    pub fn init(device: &wgpu::Device, format: wgpu::TextureFormat) -> Arc<Mutex<Images>> {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("image_shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });

        // Group 0: screen uniform
        let screen_uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("image_screen_uniform"),
            size: 8,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let screen_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("image_screen_bgl"),
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

        let screen_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("image_screen_bg"),
            layout: &screen_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: screen_uniform_buf.as_entire_binding(),
            }],
        });

        // Group 1: per-image (uniform + texture + sampler)
        let per_image_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("image_per_bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("image_pl"),
            bind_group_layouts: &[&screen_bgl, &per_image_bgl],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("image_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: size_of::<[f32; 2]>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2],
                }],
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
            label: Some("image_verts"),
            contents: bytemuck::cast_slice(&QUAD_VERTS),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("image_idx"),
            contents: bytemuck::cast_slice(&QUAD_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("image_sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let mut fontdb = fontdb::Database::new();
        fontdb.load_system_fonts();
        let fontdb = Arc::new(fontdb);

        let arc = Arc::new(Mutex::new(Images {
            slots: BTreeMap::new(),
            next_id: 1,
            pipeline,
            vertex_buf,
            index_buf,
            screen_uniform_buf,
            screen_bind_group,
            per_image_bgl,
            sampler,
            fontdb,
        }));
        let _ = IMAGES.set(Arc::clone(&arc));
        arc
    }

    pub fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, width: u32, height: u32) {
        queue.write_buffer(
            &self.screen_uniform_buf,
            0,
            bytemuck::cast_slice(&[width as f32, height as f32]),
        );

        for slot in self.slots.values_mut() {
            // Upload pending textures
            if let Some(pixels) = slot.pending_pixels.take() {
                let texture = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("image_tex"),
                    size: wgpu::Extent3d {
                        width: slot.natural_width,
                        height: slot.natural_height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                });

                queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    &pixels,
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(4 * slot.natural_width),
                        rows_per_image: Some(slot.natural_height),
                    },
                    wgpu::Extent3d {
                        width: slot.natural_width,
                        height: slot.natural_height,
                        depth_or_array_layers: 1,
                    },
                );

                let view = texture.create_view(&Default::default());

                let uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("image_uniform"),
                    size: 16, // pos: vec2<f32> + size: vec2<f32>
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });

                let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("image_bg"),
                    layout: &self.per_image_bgl,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: uniform_buf.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Sampler(&self.sampler),
                        },
                    ],
                });

                slot.gpu = Some(ImageGpu {
                    bind_group,
                    uniform_buf,
                });
            }

            // Update per-image uniform
            if let Some(gpu) = &slot.gpu {
                let data: [f32; 4] = [slot.x, slot.y, slot.width, slot.height];
                queue.write_buffer(&gpu.uniform_buf, 0, bytemuck::cast_slice(&data));
            }
        }
    }

    pub fn render<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        let has_any = self.slots.values().any(|s| s.gpu.is_some());
        if has_any {
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.screen_bind_group, &[]);
            pass.set_vertex_buffer(0, self.vertex_buf.slice(..));
            pass.set_index_buffer(self.index_buf.slice(..), wgpu::IndexFormat::Uint16);

            for slot in self.slots.values() {
                if let Some(gpu) = &slot.gpu {
                    pass.set_bind_group(1, &gpu.bind_group, &[]);
                    pass.draw_indexed(0..6, 0, 0..1);
                }
            }
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn lambda_image_create(ptr: *const u8, len: u32) -> u32 {
    let bytes = unsafe { std::slice::from_raw_parts(ptr, len as usize) };
    let Ok(path) = std::str::from_utf8(bytes) else {
        return 0;
    };

    let mut images = images_lock();
    let (w, h, pixels) = if path.ends_with(".svg") {
        let Ok(data) = std::fs::read(path) else {
            return 0;
        };
        let mut opt = resvg::usvg::Options::default();
        opt.fontdb = Arc::clone(&images.fontdb);
        let Ok(tree) = resvg::usvg::Tree::from_data(&data, &opt) else {
            return 0;
        };
        let size = tree.size();
        let w = size.width() as u32;
        let h = size.height() as u32;
        let Some(mut pixmap) = resvg::tiny_skia::Pixmap::new(w, h) else {
            return 0;
        };
        resvg::render(
            &tree,
            resvg::tiny_skia::Transform::default(),
            &mut pixmap.as_mut(),
        );
        (w, h, pixmap.take())
    } else {
        let Ok(img) = image::open(path) else { return 0 };
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();
        (w, h, rgba.into_raw())
    };
    let id = images.next_id;
    images.next_id += 1;
    images.slots.insert(
        id,
        ImageSlot {
            x: 0.0,
            y: 0.0,
            width: w as f32,
            height: h as f32,
            natural_width: w,
            natural_height: h,
            pending_pixels: Some(pixels),
            gpu: None,
        },
    );
    id
}

#[unsafe(no_mangle)]
pub extern "C" fn lambda_image_destroy(id: u32) {
    images_lock().slots.remove(&id);
}

#[unsafe(no_mangle)]
pub extern "C" fn lambda_image_position(id: u32, x: f32, y: f32) {
    if let Some(slot) = images_lock().slots.get_mut(&id) {
        slot.x = x;
        slot.y = y;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn lambda_image_size(id: u32, w: f32, h: f32) {
    if let Some(slot) = images_lock().slots.get_mut(&id) {
        slot.width = w;
        slot.height = h;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn lambda_image_width(id: u32) -> u32 {
    images_lock().slots.get(&id).map_or(0, |s| s.natural_width)
}

#[unsafe(no_mangle)]
pub extern "C" fn lambda_image_height(id: u32) -> u32 {
    images_lock().slots.get(&id).map_or(0, |s| s.natural_height)
}

#[unsafe(no_mangle)]
pub extern "C" fn lambda_image_aspect_ratio(id: u32) -> f32 {
    images_lock()
        .slots
        .get(&id)
        .map_or(0.0, |s| s.natural_width as f32 / s.natural_height as f32)
}
