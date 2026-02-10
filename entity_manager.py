# entity_manager.py
import pygame
import math
import random
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

        for e in self.entities:
            dist = math.sqrt((e.x - px) ** 2 + (e.y - py) ** 2)
            if dist < radius and dist > 0:
                angle = math.atan2(e.y - py, e.x - px)

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

        for proj in self.projectiles:
            proj.update(self.game.map_handler)
            if not proj.marked_for_deletion:
                for entity in self.entities:
                    dist = math.sqrt((proj.x - entity.x) ** 2 + (proj.y - entity.y) ** 2)
                    if dist < entity.size + proj.radius:
                        proj.marked_for_deletion = True
                        if entity in self.entities:
                            self.entities.remove(entity)
                        break
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

            if not self.game.map_handler.get_wall(ex // TILE_SIZE, ey // TILE_SIZE):
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
        for e in self.entities: all_objects.append((e, e.size, e.color))
        for p in self.projectiles: all_objects.append((p, 4, (255, 255, 0)))

        for obj, size, color in all_objects:
            dx = obj.x - px
            dy = obj.y - py
            dist = math.sqrt(dx * dx + dy * dy)
            if dist < 0.1: continue
            entity_angle = math.atan2(dy, dx) - player_angle
            while entity_angle > math.pi: entity_angle -= 2 * math.pi
            while entity_angle < -math.pi: entity_angle += 2 * math.pi
            perp_dist = dist * math.cos(entity_angle)
            if perp_dist < 0.1: perp_dist = 0.1
            if -HALF_FOV - 0.5 < entity_angle < HALF_FOV + 0.5:
                visible_entities.append((perp_dist, size, color, entity_angle))

        visible_entities.sort(key=lambda x: x[0], reverse=True)
        render_data = []
        for perp_dist, size, color, angle in visible_entities:
            width_pixels = int((size / perp_dist) * self.game.raycasting.screen_dist)
            width_rays = max(1, width_pixels // SCALE)
            center_ray = (angle + HALF_FOV) / (FOV / num_rays)
            start_ray = int(center_ray - width_rays // 2)
            end_ray = int(center_ray + width_rays // 2)
            draw_start = max(0, start_ray)
            draw_end = min(num_rays, end_ray)
            for ray in range(draw_start, draw_end):
                if ray < len(z_buffer) and perp_dist < z_buffer[ray]:
                    x_norm = ((ray * SCALE) / self.game.width) * 2 - 1.0
                    render_data.append((x_norm, strip_width_norm, strip_height_norm, perp_dist, color))
        return render_data