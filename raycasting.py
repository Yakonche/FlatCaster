# raycasting.py
import pygame
import math
import numpy as np
from settings import *
from geometry import WALL_TYPES
from colors import *

FOV_CONE_COLOR = (200, 200, 200, 40)


class RayCasting:
    def __init__(self, game):
        self.game = game
        self.ray_hits_for_2d = []
        self.screen_dist = 1000
        self.update_settings()

    def update_settings(self):
        self.num_rays = self.game.width // SCALE
        if self.num_rays == 0: self.num_rays = 1
        self.delta_angle = FOV / self.num_rays
        self.screen_dist = (self.game.width // 2) / math.tan(HALF_FOV)

    def ray_cast_view(self):
        self.ray_hits_for_2d = []

        render_data = []

        ox, oy = self.game.player.pos
        map_x, map_y = int(ox // TILE_SIZE), int(oy // TILE_SIZE)

        ray_angle = self.game.player.angle - HALF_FOV + 0.0001

        z_buffer = []

        strip_width_norm = (SCALE / self.game.width) * 2

        strip_height_norm = 2.0

        for ray in range(self.num_rays):
            sin_a = math.sin(ray_angle)
            cos_a = math.cos(ray_angle)

            delta_dist_x = abs(1 / (cos_a + 1e-30))
            delta_dist_y = abs(1 / (sin_a + 1e-30))

            cur_map_x, cur_map_y = map_x, map_y

            if cos_a < 0:
                step_x = -1
                side_dist_x = (ox / TILE_SIZE - cur_map_x) * delta_dist_x
            else:
                step_x = 1
                side_dist_x = (cur_map_x + 1.0 - ox / TILE_SIZE) * delta_dist_x

            if sin_a < 0:
                step_y = -1
                side_dist_y = (oy / TILE_SIZE - cur_map_y) * delta_dist_y
            else:
                step_y = 1
                side_dist_y = (cur_map_y + 1.0 - oy / TILE_SIZE) * delta_dist_y

            hit = False
            side = 0
            wall_type = 0

            for _ in range(500):
                if side_dist_x < side_dist_y:
                    side_dist_x += delta_dist_x
                    cur_map_x += step_x
                    side = 0
                else:
                    side_dist_y += delta_dist_y
                    cur_map_y += step_y
                    side = 1

                wall = self.game.map_handler.get_wall(cur_map_x, cur_map_y)
                if wall:
                    wall_type = wall
                    hit = True
                    break

            hit_world_x, hit_world_y = 0, 0
            perp_wall_dist = 0

            if hit:
                if side == 0:
                    perp_wall_dist = side_dist_x - delta_dist_x
                else:
                    perp_wall_dist = side_dist_y - delta_dist_y

                hit_world_x = ox + (perp_wall_dist * TILE_SIZE) * cos_a
                hit_world_y = oy + (perp_wall_dist * TILE_SIZE) * sin_a
                self.ray_hits_for_2d.append((hit_world_x, hit_world_y))
            else:
                perp_wall_dist = 5000 / TILE_SIZE
                hit_world_x = ox + 5000 * cos_a
                hit_world_y = oy + 5000 * sin_a
                self.ray_hits_for_2d.append((hit_world_x, hit_world_y))

            dist_corrected = perp_wall_dist * math.cos(self.game.player.angle - ray_angle)
            z_buffer.append(dist_corrected * TILE_SIZE)

            if wall_type in WALL_TYPES:
                base_color = WALL_TYPES[wall_type].color
                if side == 1:
                    base_color = (
                        int(base_color[0] * 0.8),
                        int(base_color[1] * 0.8),
                        int(base_color[2] * 0.8)
                    )

                x_norm = ((ray * SCALE) / self.game.width) * 2 - 1.0

                distance_pixels = perp_wall_dist * TILE_SIZE

                render_data.append((
                    x_norm,
                    strip_width_norm,
                    strip_height_norm,
                    distance_pixels,
                    base_color
                ))

            ray_angle += self.delta_angle

        return render_data, z_buffer

    def draw_fov_cone(self, surface, offset_x, offset_y, zoom):
        if not self.ray_hits_for_2d:
            return

        temp_surface = pygame.Surface((self.game.width, self.game.height), pygame.SRCALPHA)
        center_x = self.game.width // 2
        center_y = (self.game.height - self.game.strip_height) // 2

        points = [(center_x, center_y)]

        step = 2
        for i in range(0, len(self.ray_hits_for_2d), step):
            wx, wy = self.ray_hits_for_2d[i]
            sx = wx * zoom + offset_x
            sy = wy * zoom + offset_y
            points.append((sx, sy))

        pygame.draw.polygon(temp_surface, FOV_CONE_COLOR, points)
        surface.blit(temp_surface, (0, 0))

    def draw_2d_map(self, surface):
        map_view_height = self.game.height - self.game.strip_height

        clip_rect = pygame.Rect(0, 0, self.game.width, map_view_height)
        surface.set_clip(clip_rect)

        p_x, p_y = self.game.player.map_pos
        current_tile_size = TILE_SIZE * self.game.zoom_level

        tiles_in_width = (self.game.width // current_tile_size) // 2 + 2
        tiles_in_height = (map_view_height // current_tile_size) // 2 + 2

        center_x = self.game.width // 2
        center_y = map_view_height // 2

        offset_x = center_x - self.game.player.x * self.game.zoom_level
        offset_y = center_y - self.game.player.y * self.game.zoom_level

        for y in range(int(p_y - tiles_in_height), int(p_y + tiles_in_height)):
            for x in range(int(p_x - tiles_in_width), int(p_x + tiles_in_width)):
                val = self.game.map_handler.get_wall(x, y)
                if val in WALL_TYPES:
                    color = WALL_TYPES[val].color
                    draw_x = x * TILE_SIZE * self.game.zoom_level + offset_x
                    draw_y = y * TILE_SIZE * self.game.zoom_level + offset_y

                    if -current_tile_size < draw_x < self.game.width and -current_tile_size < draw_y < map_view_height:
                        pygame.draw.rect(surface, color,
                                         (draw_x, draw_y, current_tile_size + 1, current_tile_size + 1))

        self.draw_fov_cone(surface, offset_x, offset_y, self.game.zoom_level)
        self.game.entity_manager.draw_2d(surface, offset_x, offset_y, self.game.zoom_level)

        player_screen_radius = max(3, int(PLAYER_SIZE * self.game.zoom_level))
        pygame.draw.circle(surface, WHITE, (center_x, center_y), player_screen_radius)

        surface.set_clip(None)