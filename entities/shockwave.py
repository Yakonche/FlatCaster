# entities/shockwave.py
import pygame
import math
import random
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

        self.points_offset = [random.uniform(0.8, 1.2) for _ in range(16)]

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
        current_radius = self.radius * zoom

        if current_radius > 0 and self.alpha > 0:
            points = []
            num_points = len(self.points_offset)
            angle_step = (math.pi * 2) / num_points

            for i in range(num_points):
                angle = i * angle_step + pygame.time.get_ticks() * 0.005
                r = current_radius * self.points_offset[i]

                px = sx + math.cos(angle) * r
                py = sy + math.sin(angle) * r
                points.append((px, py))

            color = (0, 255, 255)
            if len(points) > 2:
                pygame.draw.lines(surface, color, True, points, max(1, int(self.width * zoom)))