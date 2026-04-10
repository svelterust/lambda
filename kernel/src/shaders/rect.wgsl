@group(0) @binding(0) var<uniform> viewport: vec2<f32>;

struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(
    @builtin(vertex_index) vi: u32,
    @location(0) pos: vec2<f32>,
    @location(1) size: vec2<f32>,
    @location(2) color: u32,
) -> VsOut {
    // Pixel coords to normalized device coordinates (top-left origin, y-down)
    let uv = vec2<f32>(f32(vi & 1u), f32((vi >> 1u) & 1u));
	let pixel = pos + uv * size;
    let ndc = pixel / viewport * vec2(2.0, -2.0) + vec2(-1.0, 1.0);

    // Unpack 0xRRGGBBAA
    let r = f32((color >> 24u) & 0xFFu) / 255.0;
    let g = f32((color >> 16u) & 0xFFu) / 255.0;
    let b = f32((color >> 8u)  & 0xFFu) / 255.0;
    let a = f32( color         & 0xFFu) / 255.0;

    var out: VsOut;
    out.position = vec4(ndc, 0.0, 1.0);
    out.color = vec4(r, g, b, a);
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    return in.color;
}
