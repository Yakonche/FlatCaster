# raycasting.py
import pygame
import math
import moderngl
import numpy as np
from settings import *
from geometry import WALL_TYPES
from colors import RED_WALL, GREEN_WALL, BLUE_WALL, PURPLE_WALL, ORANGE_WALL, WHITE


def get_shader_source():
    return f"""
    #version 430
    layout(local_size_x = 16, local_size_y = 1) in;

    layout(std430, binding = 0) buffer MapData {{
        float segmentData[];
    }};

    layout(std430, binding = 1) buffer OutputBuffer {{
        float renderData[]; 
    }};

    layout(std430, binding = 2) buffer HitBuffer {{
        vec4 hitData[];
    }};

    uniform vec2 playerTilePos;
    uniform float playerAngle;
    uniform int numSegments;
    uniform int numRays;
    uniform float fov;
    uniform float halfFov;
    uniform float tileSize;
    uniform float screenWidth;
    uniform float scale;

    vec3 getWallColor(int wallType) {{
        if (wallType == 1) return vec3({RED_WALL[0] / 255.0}, {RED_WALL[1] / 255.0}, {RED_WALL[2] / 255.0});
        if (wallType == 2) return vec3({GREEN_WALL[0] / 255.0}, {GREEN_WALL[1] / 255.0}, {GREEN_WALL[2] / 255.0});
        if (wallType == 3) return vec3({BLUE_WALL[0] / 255.0}, {BLUE_WALL[1] / 255.0}, {BLUE_WALL[2] / 255.0});
        if (wallType == 4) return vec3({PURPLE_WALL[0] / 255.0}, {PURPLE_WALL[1] / 255.0}, {PURPLE_WALL[2] / 255.0});
        if (wallType == 5) return vec3({ORANGE_WALL[0] / 255.0}, {ORANGE_WALL[1] / 255.0}, {ORANGE_WALL[2] / 255.0});
        return vec3(1.0);
    }}

    void main() {{
        uint ray = gl_GlobalInvocationID.x;
        if (ray >= numRays) return;

        float deltaAngle = fov / float(numRays);
        float rayAngle = playerAngle - halfFov + float(ray) * deltaAngle;

        vec2 dir = vec2(cos(rayAngle), sin(rayAngle));

        float minDist = 5000.0;
        int hitColorType = 0;
        float hitNormalMod = 1.0;

        for (int i = 0; i < numSegments; i++) {{
            int idx = i * 5;
            vec2 A = vec2(segmentData[idx], segmentData[idx+1]);
            vec2 B = vec2(segmentData[idx+2], segmentData[idx+3]);

            vec2 v1 = playerTilePos - A;
            vec2 v2 = B - A;
            vec2 v3 = vec2(-dir.y, dir.x);

            float dot_v2_v3 = v2.x * v3.x + v2.y * v3.y;

            if (abs(dot_v2_v3) > 1e-6) {{
                float t1 = (v2.x * v1.y - v2.y * v1.x) / dot_v2_v3;
                if (t1 > 0.0 && t1 < minDist) {{
                    float t2 = (v1.x * v3.x + v1.y * v3.y) / dot_v2_v3;
                    if (t2 >= 0.0 && t2 <= 1.0) {{
                        minDist = t1;
                        hitColorType = int(segmentData[idx+4] + 0.1);
                        hitNormalMod = (abs(v2.x) > abs(v2.y)) ? 0.8 : 1.0;
                    }}
                }}
            }}
        }}

        int outIdx = int(ray) * 7;

        if (minDist == 5000.0) {{
            renderData[outIdx] = -10.0; 
            float farX = (playerTilePos.x + 4000.0 * dir.x) * tileSize;
            float farY = (playerTilePos.y + 4000.0 * dir.y) * tileSize;
            hitData[ray] = vec4(5000.0 * tileSize, farX, farY, 1.0); 
            return;
        }}

        float hitX = (playerTilePos.x + minDist * dir.x) * tileSize;
        float hitY = (playerTilePos.y + minDist * dir.y) * tileSize;
        float distCorrected = minDist * cos(playerAngle - rayAngle);
        float zValue = distCorrected * tileSize;
        float distance = minDist * tileSize;

        vec3 color = getWallColor(hitColorType) * hitNormalMod;

        renderData[outIdx + 0] = ((float(ray) * scale) / screenWidth) * 2.0 - 1.0;
        renderData[outIdx + 1] = (scale / screenWidth) * 2.0;
        renderData[outIdx + 2] = 2.0;
        renderData[outIdx + 3] = distance;

        renderData[outIdx + 4] = color.r;
        renderData[outIdx + 5] = color.g;
        renderData[outIdx + 6] = color.b;

        hitData[ray] = vec4(zValue, hitX, hitY, 1.0);
    }}
    """


class RayCasting:
    def __init__(self, game):
        self.game = game
        self.active_segments = []
        self.num_segments = 0
        self.screen_dist = 1000

        self.ctx = game.renderer.ctx
        self.compute_shader = self.ctx.compute_shader(get_shader_source())
        self.output_buffer = None
        self.hit_buffer = None
        self.map_buffer = None
        self.last_chunk = None

        self.update_settings()

    def update_settings(self):
        self.num_rays = self.game.width // SCALE
        if self.num_rays == 0: self.num_rays = 1
        self.delta_angle = FOV / self.num_rays
        self.screen_dist = (self.game.width // 2) / math.tan(HALF_FOV)

        if self.output_buffer: self.output_buffer.release()
        if self.hit_buffer: self.hit_buffer.release()

        self.output_buffer = self.ctx.buffer(reserve=self.num_rays * 7 * 4)
        self.hit_buffer = self.ctx.buffer(reserve=self.num_rays * 4 * 4)

    def update_map_buffer(self, px, py):
        cx = int((px / TILE_SIZE) // 32)
        cy = int((py / TILE_SIZE) // 32)

        zoom = self.game.zoom_level
        max_screen_dim = max(self.game.width, self.game.height)
        tiles_across = max_screen_dim / (TILE_SIZE * zoom)
        chunks_across = tiles_across / 32

        dynamic_radius = int(chunks_across / 2) + 1
        dynamic_radius = max(2, min(dynamic_radius, 10))

        if not hasattr(self, 'last_radius'):
            self.last_radius = -1

        if (cx, cy) != self.last_chunk or dynamic_radius != self.last_radius:
            self.last_chunk = (cx, cy)
            self.last_radius = dynamic_radius

            active_segments = self.game.map_handler.get_active_segments(px, py, radius_chunks=dynamic_radius)
            self.num_segments = len(active_segments)
            if self.num_segments == 0:
                active_segments = [(0, 0, 0, 0, 0)]
                self.num_segments = 0

            flat_data = np.array(active_segments, dtype='f4').flatten()
            req_bytes = max(flat_data.nbytes, 4)

            if not self.map_buffer or self.map_buffer.size < req_bytes:
                if self.map_buffer: self.map_buffer.release()
                self.map_buffer = self.ctx.buffer(reserve=max(req_bytes, 1000000))

            self.map_buffer.write(flat_data.tobytes())
            self.active_segments = active_segments

    def ray_cast_view(self):
        self.update_map_buffer(self.game.player.pos[0], self.game.player.pos[1])

        if self.map_buffer: self.map_buffer.bind_to_storage_buffer(0)
        self.output_buffer.bind_to_storage_buffer(1)
        self.hit_buffer.bind_to_storage_buffer(2)

        self.compute_shader['playerTilePos'].value = (
        self.game.player.pos[0] / TILE_SIZE, self.game.player.pos[1] / TILE_SIZE)
        self.compute_shader['playerAngle'].value = self.game.player.angle
        self.compute_shader['numSegments'].value = self.num_segments
        self.compute_shader['numRays'].value = self.num_rays
        self.compute_shader['fov'].value = FOV
        self.compute_shader['halfFov'].value = HALF_FOV
        self.compute_shader['tileSize'].value = TILE_SIZE
        self.compute_shader['screenWidth'].value = self.game.width
        self.compute_shader['scale'].value = SCALE

        num_groups = (self.num_rays + 15) // 16

        if hasattr(self, 'z_buffer_cache'):
            hit_raw = np.frombuffer(self.hit_buffer.read(), dtype='f4').reshape((self.num_rays, 4))
            self.z_buffer_cache = hit_raw[:, 0].tolist()
        else:
            self.z_buffer_cache = [5000.0] * self.num_rays

        self.compute_shader.run(num_groups, 1, 1)

        return self.output_buffer, self.z_buffer_cache

    def draw_2d_entities(self, surface):
        map_view_height = self.game.height - self.game.strip_height
        clip_rect = pygame.Rect(0, 0, self.game.width, map_view_height)
        surface.set_clip(clip_rect)

        center_x = self.game.width // 2
        center_y = map_view_height // 2
        zoom = self.game.zoom_level

        world_offset_x = center_x - self.game.player.pos[0] * zoom
        world_offset_y = center_y - self.game.player.pos[1] * zoom

        self.game.entity_manager.draw_2d(surface, world_offset_x, world_offset_y, zoom)

        player_screen_radius = max(3, int(PLAYER_SIZE * zoom))
        pygame.draw.circle(surface, WHITE, (center_x, center_y), player_screen_radius)

        surface.set_clip(None)