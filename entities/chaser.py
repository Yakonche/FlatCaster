# entities/chaser.py
import math
import random
from .base_entity import Entity
from colors import ENTITY_RED


class Chaser(Entity):
    def __init__(self, game, x, y, shape="triangle"):
        super().__init__(game, x, y, ENTITY_RED, shape)
        self.speed = 1.5
        self.state = "idle"
        self.timer = 0
        self.target_angle = 0

    def update(self):
        super().update()
        self.timer -= 1

        if self.timer <= 0:
            if self.state == "idle":
                self.state = "moving"
                self.timer = random.randint(60, 200)
                self.target_angle = random.uniform(0, math.pi * 2)
                self.angle = self.target_angle
            else:
                self.state = "idle"
                self.timer = random.randint(30, 90)

        if self.state == "moving":
            dx = math.cos(self.angle) * self.speed
            dy = math.sin(self.angle) * self.speed

            px, py = self.game.player.pos
            dist_p = self.get_distance_to(px, py)

            if dist_p < 30:
                self.state = "idle"
                self.timer = 60
                return

            hit_wall = self.check_wall(self.x + dx * 5, self.y + dy * 5)
            if hit_wall:
                self.state = "idle"
                self.timer = random.randint(10, 30)
            else:
                self.move(dx, dy)
        else:
            self.angle += 0.02