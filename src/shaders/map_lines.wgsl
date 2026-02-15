// src/shaders/map_lines.wgsl

struct MapUniforms {
    offset: vec2<f32>,
    tile_size: f32,
    _pad1: f32,
    resolution: vec2<f32>,
    _pad2: vec2<f32>,
};

@group(0) @binding(0) var<storage, read> segment_data: array<f32>;
@group(0) @binding(1) var<uniform> uniforms: MapUniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

fn get_wall_color(wall_type: i32) -> vec3<f32> {
    if (wall_type == 1) { return vec3<f32>(0.784, 0.196, 0.196); }
    if (wall_type == 2) { return vec3<f32>(0.196, 0.784, 0.196); }
    if (wall_type == 3) { return vec3<f32>(0.196, 0.196, 0.784); }
    if (wall_type == 4) { return vec3<f32>(0.588, 0.196, 0.588); }
    if (wall_type == 5) { return vec3<f32>(0.784, 0.392, 0.196); }
    return vec3<f32>(1.0, 1.0, 1.0);
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_id: u32) -> VertexOutput {
    var out: VertexOutput;
    let segment_idx = vertex_id / 2u;
    let is_end = vertex_id % 2u;
    let base_idx = segment_idx * 5u;

    var x: f32;
    var y: f32;
    if (is_end == 0u) {
        x = segment_data[base_idx];
        y = segment_data[base_idx + 1u];
    } else {
        x = segment_data[base_idx + 2u];
        y = segment_data[base_idx + 3u];
    }
    let c = i32(segment_data[base_idx + 4u] + 0.1);

    let screen_pos = vec2<f32>(x, y) * uniforms.tile_size + uniforms.offset;
    var clip_pos = (screen_pos / uniforms.resolution) * 2.0 - 1.0;
    clip_pos.y = -clip_pos.y;

    // color_type >= 100 = entity segment, not a wall: discard by placing outside clip space
    if (c >= 100) {
        out.position = vec4<f32>(2.0, 2.0, 0.0, 1.0);
        out.color = vec3<f32>(0.0);
        return out;
    }

    out.position = vec4<f32>(clip_pos, 0.0, 1.0);
    out.color = get_wall_color(c);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
