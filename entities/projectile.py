# entities/projectile.py
import math
import pygame
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
        pygame.draw.circle(surface, (255, 255, 0), (sx, sy), int(self.radius * zoom))