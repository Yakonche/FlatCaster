// src/shaders/fov.wgsl

struct FovUniforms {
    player_pos: vec2<f32>,
    offset: vec2<f32>,
    zoom: f32,
    _pad1: f32,
    resolution: vec2<f32>,
    step: i32,
    max_ray: i32,
};

@group(0) @binding(0) var<storage, read> hit_data: array<vec4<f32>>;
@group(0) @binding(1) var<uniform> uniforms: FovUniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

fn world_to_clip(world_pos: vec2<f32>) -> vec4<f32> {
    let screen_pos = world_pos * uniforms.zoom + uniforms.offset;
    var clip_pos = (screen_pos / uniforms.resolution) * 2.0 - 1.0;
    clip_pos.y = -clip_pos.y;
    return vec4<f32>(clip_pos, 0.0, 1.0);
}

fn get_hit_pos(idx: i32) -> vec2<f32> {
    var ray_idx = idx * uniforms.step;
    if (ray_idx >= uniforms.max_ray) {
        ray_idx = uniforms.max_ray - 1;
    }
    return vec2<f32>(hit_data[ray_idx].y, hit_data[ray_idx].z);
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_id: u32) -> VertexOutput {
    var out: VertexOutput;

    // Each triangle: center, hit[i], hit[i+1]
    let tri_idx = vertex_id / 3u;
    let vert_in_tri = vertex_id % 3u;

    var world_pos: vec2<f32>;
    if (vert_in_tri == 0u) {
        world_pos = uniforms.player_pos;
    } else if (vert_in_tri == 1u) {
        world_pos = get_hit_pos(i32(tri_idx));
    } else {
        world_pos = get_hit_pos(i32(tri_idx) + 1);
    }

    out.position = world_to_clip(world_pos);
    return out;
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(0.784, 0.784, 0.784, 0.157);
}
