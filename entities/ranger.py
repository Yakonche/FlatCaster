# entities/ranger.py
import math
import pygame
import random
from .base_entity import Entity
from settings import *


class Ranger(Entity):
    def __init__(self, game, x, y):
        super().__init__(game, x, y, (0, 255, 255), "diamond")
        self.speed = 2.0
        self.state = "positioning"
        self.charge_timer = 0
        self.ideal_range = 300

        self.is_firing = False
        self.fire_frame = 0

    def update(self):
        super().update()

        if self.is_firing:
            self.fire_frame -= 1
            if self.fire_frame <= 0:
                self.is_firing = False
                self.state = "positioning"
            return

        px, py = self.game.player.pos
        dist_p = self.get_distance_to(px, py)

        if self.state == "positioning":
            self.color = (0, 255, 255)

            if dist_p < self.ideal_range - 50:
                angle_away = math.atan2(self.y - py, self.x - px)
                self.angle = angle_away
                self.move(math.cos(self.angle) * self.speed, math.sin(self.angle) * self.speed)

            elif dist_p > self.ideal_range + 50:
                angle_to = math.atan2(py - self.y, px - self.x)
                self.angle = angle_to
                self.move(math.cos(self.angle) * self.speed, math.sin(self.angle) * self.speed)

            else:
                self.state = "aiming"
                self.charge_timer = 60

        elif self.state == "aiming":
            self.color = (255, 255, 255)
            self.charge_timer -= 1

            self.angle = math.atan2(py - self.y, px - self.x)

            if self.charge_timer <= 0:
                self.state = "firing"
                self.fire_shot()

    def fire_shot(self):
        self.is_firing = True
        self.fire_frame = 5
        self.color = (255, 0, 0)
        # Ici on pourrait réduire la vie du joueur
        print("Ranger fired!")

    def draw_2d(self, surface, offset_x, offset_y, zoom):
        super().draw_2d(surface, offset_x, offset_y, zoom)

        if self.is_firing:
            sx = self.x * zoom + offset_x
            sy = self.y * zoom + offset_y

            range_len = 600 * zoom
            ex = sx + math.cos(self.angle) * range_len
            ey = sy + math.sin(self.angle) * range_len

            pygame.draw.line(surface, (255, 255, 0), (sx, sy), (ex, ey), 2)