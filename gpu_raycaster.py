# gpu_raycaster.py
import moderngl
import numpy as np
import math
from settings import *


class GPURaycaster:
    def __init__(self, ctx):
        self.ctx = ctx
        self.num_rays = res_width // SCALE
        if self.num_rays == 0: self.num_rays = 1

        self.program = self.ctx.program(
            vertex_shader="""
            #version 330
            in float in_ray_index;

            uniform vec2 u_player_pos;
            uniform float u_player_angle;
            uniform float u_fov;
            uniform float u_num_rays;
            uniform float u_scale;
            uniform float u_screen_width;

            uniform sampler2D u_map_tex;
            uniform vec2 u_map_topleft;

            out float out_hit_x;
            out float out_hit_y;
            out float out_x_offset;
            out float out_width;
            out float out_height;
            out float out_distance;
            out vec3 out_color;

            void main() {
                float ray_angle = u_player_angle - u_fov / 2.0 + (in_ray_index / u_num_rays) * u_fov;
                float sin_a = sin(ray_angle);
                float cos_a = cos(ray_angle);

                float delta_dist_x = abs(1.0 / (cos_a + 1e-30));
                float delta_dist_y = abs(1.0 / (sin_a + 1e-30));

                int map_x = int(u_player_pos.x);
                int map_y = int(u_player_pos.y);

                int step_x = cos_a < 0.0 ? -1 : 1;
                int step_y = sin_a < 0.0 ? -1 : 1;

                float side_dist_x = (cos_a < 0.0) ? (u_player_pos.x - float(map_x)) * delta_dist_x : (float(map_x) + 1.0 - u_player_pos.x) * delta_dist_x;
                float side_dist_y = (sin_a < 0.0) ? (u_player_pos.y - float(map_y)) * delta_dist_y : (float(map_y) + 1.0 - u_player_pos.y) * delta_dist_y;

                bool hit = false;
                int side = 0;
                vec4 wall_color = vec4(0.0);

                for (int i = 0; i < 200; i++) {
                    if (side_dist_x < side_dist_y) {
                        side_dist_x += delta_dist_x;
                        map_x += step_x;
                        side = 0;
                    } else {
                        side_dist_y += delta_dist_y;
                        map_y += step_y;
                        side = 1;
                    }

                    vec2 uv = vec2(float(map_x) - u_map_topleft.x + 0.5, float(map_y) - u_map_topleft.y + 0.5) / 256.0;

                    if (uv.x >= 0.0 && uv.x <= 1.0 && uv.y >= 0.0 && uv.y <= 1.0) {
                        wall_color = texture(u_map_tex, uv);
                        if (wall_color.a > 0.1) {
                            hit = true;
                            break;
                        }
                    } else {
                        break; // Sortie de la zone chargée
                    }
                }

                float perp_wall_dist = 5000.0;
                if (hit) {
                    if (side == 0) perp_wall_dist = side_dist_x - delta_dist_x;
                    else           perp_wall_dist = side_dist_y - delta_dist_y;
                }

                out_hit_x = u_player_pos.x + perp_wall_dist * cos_a;
                out_hit_y = u_player_pos.y + perp_wall_dist * sin_a;

                float dist_corrected = perp_wall_dist * cos(u_player_angle - ray_angle);
                out_distance = dist_corrected * 50.0; // TILE_SIZE = 50.0

                vec3 base_color = wall_color.rgb;
                if (side == 1) {
                    base_color *= 0.8;
                }
                out_color = base_color;

                out_x_offset = ((in_ray_index * u_scale) / u_screen_width) * 2.0 - 1.0;
                out_width = (u_scale / u_screen_width) * 2.0;
                out_height = 2.0;
            }
            """,
            varyings=['out_hit_x', 'out_hit_y', 'out_x_offset', 'out_width', 'out_height', 'out_distance', 'out_color']
        )

        self.map_tex = self.ctx.texture((256, 256), 4, dtype='f1')
        self.map_tex.filter = (moderngl.NEAREST, moderngl.NEAREST)
        self.last_chunk = None
        self.map_topleft = (0, 0)

        self.ray_indices = np.arange(self.num_rays, dtype='f4')
        self.vbo_indices = self.ctx.buffer(self.ray_indices)
        self.vao = self.ctx.vertex_array(self.program, [(self.vbo_indices, '1f', 'in_ray_index')])
        self.output_buffer = self.ctx.buffer(reserve=self.num_rays * 9 * 4)

    def update_map(self, map_handler, p_x, p_y):
        chunk_x, chunk_y = int(p_x // 32), int(p_y // 32)
        if (chunk_x, chunk_y) != self.last_chunk:
            self.last_chunk = (chunk_x, chunk_y)

            map_data = np.zeros((256, 256, 4), dtype=np.uint8)
            start_cx = chunk_x - 3
            start_cy = chunk_y - 3

            self.map_topleft = (start_cx * 32, start_cy * 32)

            from geometry import WALL_TYPES
            for cy in range(start_cy, start_cy + 8):
                for cx in range(start_cx, start_cx + 8):
                    if (cx, cy) not in map_handler.world:
                        map_handler.generate_chunk(cx, cy)
                    chunk_dict = map_handler.world[(cx, cy)]
                    for (wx, wy), wtype in chunk_dict.items():
                        local_x = wx - self.map_topleft[0]
                        local_y = wy - self.map_topleft[1]
                        if 0 <= local_x < 256 and 0 <= local_y < 256:
                            if wtype in WALL_TYPES:
                                r, g, b = WALL_TYPES[wtype].color
                                map_data[local_y, local_x] = [r, g, b, 255]

            self.map_tex.write(map_data.tobytes())

    def resize(self, screen_width):
        self.num_rays = screen_width // SCALE
        if self.num_rays == 0: self.num_rays = 1
        self.ray_indices = np.arange(self.num_rays, dtype='f4')
        self.vbo_indices.orphan(self.ray_indices.nbytes)
        self.vbo_indices.write(self.ray_indices.tobytes())
        self.output_buffer.orphan(self.num_rays * 9 * 4)

    def cast(self, px, py, angle, screen_width):
        self.program['u_player_pos'].value = (px, py)
        self.program['u_player_angle'].value = angle
        self.program['u_fov'].value = FOV
        self.program['u_num_rays'].value = self.num_rays
        self.program['u_scale'].value = SCALE
        self.program['u_screen_width'].value = screen_width
        self.program['u_map_topleft'].value = self.map_topleft

        self.map_tex.use(0)
        self.program['u_map_tex'].value = 0

        self.vao.transform(self.output_buffer, mode=moderngl.POINTS, vertices=self.num_rays)

        data = np.frombuffer(self.output_buffer.read(), dtype=np.float32).reshape(self.num_rays, 9)
        return data