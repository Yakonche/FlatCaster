# entities/shockwave.py
import pygame
from settings import *


class Shockwave:
    def __init__(self, x, y):
        self.x = x
        self.y = y
        self.radius = 10
        self.max_radius = 400
        self.speed = 30.0
        self.alpha = 255
        self.width = 15
        self.marked_for_deletion = False

    def update(self):
        self.radius += self.speed

        self.speed *= 0.92

        self.alpha -= 8
        self.width = max(1, self.width * 0.95)

        if self.alpha <= 0 or self.speed < 0.5:
            self.marked_for_deletion = True

    def draw_2d(self, surface, offset_x, offset_y, zoom):
        sx = self.x * zoom + offset_x
        sy = self.y * zoom + offset_y
        current_radius = int(self.radius * zoom)
        current_width = max(1, int(self.width * zoom))

        if current_radius > 0 and self.alpha > 0:
            diam = current_radius * 2 + 10
            temp_surf = pygame.Surface((diam, diam), pygame.SRCALPHA)

            color = (0, 255, 255, int(self.alpha))

            center = (diam // 2, diam // 2)
            pygame.draw.circle(temp_surf, color, center, current_radius, current_width)

            surface.blit(temp_surf, (sx - center[0], sy - center[1]))