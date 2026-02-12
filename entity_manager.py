# entity_manager.py
import pygame
import math
import random
import numpy as np
from settings import *
from colors import *

from entities.base_entity import Entity
from entities.random_walker import RandomWalker
from entities.chaser import Chaser
from entities.stalker import Stalker
from entities.swarmer import Swarmer
from entities.sentinel import Sentinel
from entities.weeping_block import WeepingBlock
from entities.ranger import Ranger
from entities.projectile import Projectile
from entities.shockwave import Shockwave


class EntityManager:
    def __init__(self, game):
        self.game = game
        self.entities = []
        self.projectiles = []
        self.shockwaves = []
        self.chunk_loaded = set()
        self.current_seed = GAME_SEED

    def reset(self, new_seed):
        self.entities.clear()
        self.projectiles.clear()
        self.shockwaves.clear()
        self.chunk_loaded.clear()
        self.current_seed = new_seed

    def add_projectile(self, x, y, angle):
        self.projectiles.append(Projectile(x, y, angle))

    def apply_shockwave(self, px, py, radius=300, strength=25):
        self.shockwaves.append(Shockwave(px, py))
        radius_sq = radius * radius
        for e in self.entities:
            dx = e.x - px
            dy = e.y - py
            dist_sq = dx * dx + dy * dy
            if dist_sq < radius_sq and dist_sq > 0:
                dist = math.sqrt(dist_sq)
                angle = math.atan2(dy, dx)
                force_magnitude = strength * (1.2 - (dist / radius) * 0.5)
                fx = math.cos(angle) * force_magnitude
                fy = math.sin(angle) * force_magnitude
                e.apply_force(fx, fy)

    def update(self):
        px, py = self.game.player.map_pos
        center_chunk_x = px // 32
        center_chunk_y = py // 32

        for dy in range(-2, 3):
            for dx in range(-2, 3):
                chunk_coords = (center_chunk_x + dx, center_chunk_y + dy)
                if chunk_coords not in self.chunk_loaded:
                    self.spawn_entities_in_chunk(chunk_coords)
                    self.chunk_loaded.add(chunk_coords)

        for entity in self.entities:
            entity.update()

        for sw in self.shockwaves:
            sw.update()
        self.shockwaves = [s for s in self.shockwaves if not s.marked_for_deletion]

        entities_to_remove = set()

        for proj in self.projectiles:
            proj.update(self.game.map_handler)
            if not proj.marked_for_deletion:
                for entity in self.entities:
                    if entity in entities_to_remove:
                        continue
                    dx = proj.x - entity.x
                    dy = proj.y - entity.y
                    dist_sq = dx * dx + dy * dy
                    threshold_sq = (entity.size + proj.radius) ** 2
                    if dist_sq < threshold_sq:
                        proj.marked_for_deletion = True
                        entities_to_remove.add(entity)
                        break

        if entities_to_remove:
            self.entities = [e for e in self.entities if e not in entities_to_remove]

        self.projectiles = [p for p in self.projectiles if not p.marked_for_deletion]

    def spawn_entities_in_chunk(self, chunk_coords):
        cx, cy = chunk_coords
        if cx == 0 and cy == 0: return

        start_x = cx * 32 * TILE_SIZE
        start_y = cy * 32 * TILE_SIZE

        random.seed(f"{self.current_seed}_entities_{cx}_{cy}")

        count = random.randint(3, 8)
        for _ in range(count):
            ex = random.randint(start_x, start_x + 32 * TILE_SIZE)
            ey = random.randint(start_y, start_y + 32 * TILE_SIZE)

            if self.game.map_handler.is_position_free(ex, ey, ENTITY_SIZE):
                roll = random.random()
                if roll < 0.25:
                    self.entities.append(Swarmer(self.game, ex, ey))
                    self.entities.append(Swarmer(self.game, ex + 20, ey + 20))
                elif roll < 0.45:
                    self.entities.append(RandomWalker(self.game, ex, ey))
                elif roll < 0.60:
                    self.entities.append(Stalker(self.game, ex, ey))
                elif roll < 0.75:
                    self.entities.append(Ranger(self.game, ex, ey))
                elif roll < 0.90:
                    self.entities.append(Sentinel(self.game, ex, ey))
                else:
                    self.entities.append(WeepingBlock(self.game, ex, ey))

    def draw_2d(self, surface, offset_x, offset_y, zoom):
        for sw in self.shockwaves:
            sw.draw_2d(surface, offset_x, offset_y, zoom)
        for entity in self.entities:
            entity.draw_2d(surface, offset_x, offset_y, zoom)
        for proj in self.projectiles:
            proj.draw_2d(surface, offset_x, offset_y, zoom)

    def get_render_data(self, z_buffer):
        px, py = self.game.player.pos
        player_angle = self.game.player.angle
        num_rays = self.game.raycasting.num_rays
        visible_entities = []
        strip_width_norm = (SCALE / self.game.width) * 2
        strip_height_norm = 2.0

        all_objects = []
        for e in self.entities:
            all_objects.append((e, e.size, e.color))
        for p in self.projectiles:
            all_objects.append((p, 4, (255, 255, 0)))

        max_dist_sq = 5000 * 5000
        filtered_objects = []

        for obj, size, color in all_objects:
            dx = obj.x - px
            dy = obj.y - py
            dist_sq = dx * dx + dy * dy
            if dist_sq > max_dist_sq:
                continue
            dist = math.sqrt(dist_sq)
            if dist < 0.1:
                continue
            entity_angle = math.atan2(dy, dx) - player_angle
            while entity_angle > math.pi:
                entity_angle -= 2 * math.pi
            while entity_angle < -math.pi:
                entity_angle += 2 * math.pi
            perp_dist = dist * math.cos(entity_angle)
            if perp_dist < 0.1:
                perp_dist = 0.1
            if -HALF_FOV - 0.5 < entity_angle < HALF_FOV + 0.5:
                filtered_objects.append((perp_dist, size, color, entity_angle))

        filtered_objects.sort(key=lambda x: x[0], reverse=True)

        render_data = []
        for perp_dist, size, color, angle in filtered_objects:
            width_pixels = int((size / perp_dist) * self.game.raycasting.screen_dist)
            width_rays = max(1, width_pixels // SCALE)
            center_ray = (angle + HALF_FOV) / (FOV / num_rays)
            start_ray = int(center_ray - width_rays // 2)
            end_ray = int(center_ray + width_rays // 2)
            draw_start = max(0, start_ray)
            draw_end = min(num_rays, end_ray)

            for ray in range(draw_start, draw_end):
                if ray < len(z_buffer):
                    if perp_dist < z_buffer[ray] * 1.2:
                        x_norm = ((ray * SCALE) / self.game.width) * 2 - 1.0
                        r, g, b = color[:3]
                        render_data.append(
                            [x_norm, strip_width_norm, strip_height_norm, perp_dist,
                             r / 255.0, g / 255.0, b / 255.0])

        if not render_data:
            return np.empty((0, 7), dtype='f4')
        return np.array(render_data, dtype='f4')