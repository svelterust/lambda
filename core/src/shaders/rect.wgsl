struct Viewport {
    size: vec2<f32>,
}
@group(0) @binding(0) var<uniform> viewport: Viewport;

struct Instance {
    @location(0) pos: vec2<f32>,
    @location(1) size: vec2<f32>,
    @location(2) color: u32,
}

struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(
    @builtin(vertex_index) vi: u32,
    instance: Instance,
) -> VsOut {
    // Unit quad from vertex index: 0=(0,0) 1=(1,0) 2=(0,1) 3=(1,1)
    let uv = vec2<f32>(
        f32(vi & 1u),
        f32((vi >> 1u) & 1u),
    );

    // Pixel position
    let pixel = instance.pos + uv * instance.size;

    // Pixel → NDC (top-left origin, y-down)
    let ndc = vec2<f32>(
        (pixel.x / viewport.size.x) * 2.0 - 1.0,
        1.0 - (pixel.y / viewport.size.y) * 2.0,
    );

    // Unpack color from u32 (0xRRGGBBAA)
    let r = f32((instance.color >> 24u) & 0xFFu) / 255.0;
    let g = f32((instance.color >> 16u) & 0xFFu) / 255.0;
    let b = f32((instance.color >> 8u) & 0xFFu) / 255.0;
    let a = f32(instance.color & 0xFFu) / 255.0;

    var out: VsOut;
    out.position = vec4<f32>(ndc, 0.0, 1.0);
    out.color = vec4<f32>(r, g, b, a);
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    return in.color;
}
