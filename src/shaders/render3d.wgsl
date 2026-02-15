// src/shaders/render3d.wgsl

struct Uniforms {
    light_intensity: f32,
    min_brightness: f32,
};

struct VertexInput {
    @location(0) vert: vec2<f32>,
    @location(1) x_offset: f32,
    @location(2) width: f32,
    @location(3) height: f32,
    @location(4) distance: f32,
    @location(5) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) distance: f32,
    @location(1) color: vec3<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let y = (in.vert.y - 0.5) * in.height;
    let x = in.x_offset + (in.vert.x * in.width);
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.distance = in.distance;
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let dist = max(in.distance, 1.0);
    let decay = dist / uniforms.light_intensity;
    var brightness = 1.0 / (1.0 + decay * decay);
    brightness = max(brightness, uniforms.min_brightness);

    let hdr_color = in.color * brightness;
    return vec4<f32>(hdr_color, 1.0);
}
