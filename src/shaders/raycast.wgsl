// src/src/shaders/raycast.wgsl

struct Params {
    player_tile_pos: vec2<f32>,
    player_angle: f32,
    num_segments: i32,
    num_rays: i32,
    fov: f32,
    half_fov: f32,
    tile_size: f32,
    screen_width: f32,
    scale: f32,
    _pad: vec2<f32>,
};

@group(0) @binding(0) var<storage, read> segment_data: array<f32>;
@group(0) @binding(1) var<storage, read_write> render_data: array<f32>;
@group(0) @binding(2) var<storage, read_write> hit_data: array<vec4<f32>>;
@group(0) @binding(3) var<uniform> params: Params;

fn get_wall_color(wall_type: i32) -> vec3<f32> {
    if (wall_type == 1) { return vec3<f32>(0.784, 0.196, 0.196); }
    if (wall_type == 2) { return vec3<f32>(0.196, 0.784, 0.196); }
    if (wall_type == 3) { return vec3<f32>(0.196, 0.196, 0.784); }
    if (wall_type == 4) { return vec3<f32>(0.588, 0.196, 0.588); }
    if (wall_type == 5) { return vec3<f32>(0.784, 0.392, 0.196); }
    return vec3<f32>(1.0, 1.0, 1.0);
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let ray = i32(id.x);
    if (ray >= params.num_rays) { return; }

    let delta_angle = params.fov / f32(params.num_rays);
    let ray_angle = params.player_angle - params.half_fov + f32(ray) * delta_angle;

    let dir = vec2<f32>(cos(ray_angle), sin(ray_angle));

    // Deux trackers séparés : murs (pour le cone FOV) et entités (pour l'occlusion sprites)
    var min_dist: f32 = 5000.0;       // distance au mur le plus proche
    var hit_color_type: i32 = 0;
    var hit_normal_mod: f32 = 1.0;

    var entity_min_dist: f32 = 5000.0; // distance à l'entité la plus proche
    var entity_hit_x: f32 = 0.0;
    var entity_hit_y: f32 = 0.0;
    var entity_z_value: f32 = 0.0;

    for (var i: i32 = 0; i < params.num_segments; i = i + 1) {
        let idx = i * 5;
        let a = vec2<f32>(segment_data[idx], segment_data[idx + 1]);
        let b = vec2<f32>(segment_data[idx + 2], segment_data[idx + 3]);

        let v1 = params.player_tile_pos - a;
        let v2 = b - a;
        let v3 = vec2<f32>(-dir.y, dir.x);

        let dot_v2_v3 = v2.x * v3.x + v2.y * v3.y;

        if (abs(dot_v2_v3) > 1e-6) {
            let t1 = (v2.x * v1.y - v2.y * v1.x) / dot_v2_v3;
            let t2 = (v1.x * v3.x + v1.y * v3.y) / dot_v2_v3;
            if (t1 > 1e-4 && t2 >= -1e-4 && t2 <= 1.0 + 1e-4) {
                let color_type = i32(segment_data[idx + 4] + 0.1);
                if (color_type >= 100) {
                    // Segment d'entité : tracker séparé, n'affecte pas min_dist mur
                    if (t1 < entity_min_dist) {
                        entity_min_dist = t1;
                        let ehx = (params.player_tile_pos.x + t1 * dir.x) * params.tile_size;
                        let ehy = (params.player_tile_pos.y + t1 * dir.y) * params.tile_size;
                        entity_hit_x = ehx;
                        entity_hit_y = ehy;
                        let edc = t1 * cos(params.player_angle - ray_angle);
                        entity_z_value = edc * params.tile_size;
                    }
                } else {
                    // Segment de mur
                    if (t1 < min_dist) {
                        min_dist = t1;
                        hit_color_type = color_type;
                        if (abs(v2.x) > abs(v2.y)) {
                            hit_normal_mod = 0.8;
                        } else {
                            hit_normal_mod = 1.0;
                        }
                    }
                }
            }
        }
    }

    let out_idx = ray * 7;

    if (min_dist >= 5000.0) {
        // Aucun mur touché : le cone va au loin
        render_data[out_idx] = -10.0;
        let far_x = (params.player_tile_pos.x + 4000.0 * dir.x) * params.tile_size;
        let far_y = (params.player_tile_pos.y + 4000.0 * dir.y) * params.tile_size;
        // Pour l'occlusion sprites : utiliser l'entité si elle est devant
        if (entity_min_dist < 5000.0) {
            hit_data[ray] = vec4<f32>(entity_z_value, entity_hit_x, entity_hit_y, 2.0);
        } else {
            hit_data[ray] = vec4<f32>(5000.0 * params.tile_size, far_x, far_y, 1.0);
        }
        return;
    }

    let hit_x = (params.player_tile_pos.x + min_dist * dir.x) * params.tile_size;
    let hit_y = (params.player_tile_pos.y + min_dist * dir.y) * params.tile_size;
    let dist_corrected = min_dist * cos(params.player_angle - ray_angle);
    let z_value = dist_corrected * params.tile_size;
    let distance = min_dist * params.tile_size;

    let color = get_wall_color(hit_color_type) * hit_normal_mod;

    render_data[out_idx + 0] = ((f32(ray) * params.scale) / params.screen_width) * 2.0 - 1.0;
    render_data[out_idx + 1] = (params.scale / params.screen_width) * 2.0;
    render_data[out_idx + 2] = 2.0;
    render_data[out_idx + 3] = distance;
    render_data[out_idx + 4] = color.r;
    render_data[out_idx + 5] = color.g;
    render_data[out_idx + 6] = color.b;

    // hit_data pour le cone FOV : si une entité est plus proche qu'un mur,
    // le cône s'arrête sur l'entité (position et z_value de l'entité).
    if (entity_min_dist < min_dist) {
        hit_data[ray] = vec4<f32>(entity_z_value, entity_hit_x, entity_hit_y, 2.0);
    } else {
        hit_data[ray] = vec4<f32>(z_value, hit_x, hit_y, 1.0);
    }
}
