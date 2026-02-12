# entities/projectile.py
import math
import pygame
import random
from settings import *


class Projectile:
    def __init__(self, x, y, angle):
        self.x = x
        self.y = y
        self.angle = angle
        self.speed = 15
        self.radius = 4
        self.life_time = 100
        self.marked_for_deletion = False
        self.anim_offset = random.uniform(0, 10)

    def update(self, game_map):
        self.life_time -= 1
        if self.life_time <= 0:
            self.marked_for_deletion = True
            return

        dx = math.cos(self.angle) * self.speed
        dy = math.sin(self.angle) * self.speed

        if game_map.get_wall(int((self.x + dx) // TILE_SIZE), int((self.y + dy) // TILE_SIZE)):
            self.marked_for_deletion = True
        else:
            self.x += dx
            self.y += dy

    def draw_2d(self, surface, offset_x, offset_y, zoom):
        sx = self.x * zoom + offset_x
        sy = self.y * zoom + offset_y

        pulse = math.sin(pygame.time.get_ticks() * 0.05 + self.anim_offset) * 2
        size = int((self.radius + pulse) * zoom)
        size = max(2, size)

        color_core = (200, 255, 0)
        color_glow = (100, 200, 0, 100)

        tail_len = 20 * zoom
        ex = sx - math.cos(self.angle) * tail_len
        ey = sy - math.sin(self.angle) * tail_len
        pygame.draw.line(surface, color_glow, (sx, sy), (ex, ey), int(2 * zoom))

        pygame.draw.circle(surface, color_core, (sx, sy), size)