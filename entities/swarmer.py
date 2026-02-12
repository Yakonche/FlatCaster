# entities/swarmer.py
import math
import random
from .base_entity import Entity
from settings import *


class Swarmer(Entity):
    def __init__(self, game, x, y):
        super().__init__(game, x, y, (255, 255, 0), "flagellate")
        self.speed = 3.5
        self.size = 8
        self.state = "passive"
        self.wander_timer = 0

    def update(self):
        super().update()

        neighbors = 0
        sentinel_nearby = None

        for e in self.game.entity_manager.entities:
            if e == self: continue
            dist = self.get_distance_to(e.x, e.y)

            if isinstance(e, Swarmer) and dist < 150:
                neighbors += 1

            if e.__class__.__name__ == "Sentinel" and dist < 200:
                sentinel_nearby = e

        if sentinel_nearby:
            self.color = (100, 255, 100)
            angle_away = math.atan2(self.y - sentinel_nearby.y, self.x - sentinel_nearby.x)
            self.angle = angle_away + random.uniform(-0.5, 0.5)
            self.move(math.cos(self.angle) * self.speed * 1.5, math.sin(self.angle) * self.speed * 1.5)
            return

        if neighbors >= 2:
            self.state = "aggressive"
            self.color = (255, 100, 50)
        else:
            self.state = "passive"
            self.color = (200, 200, 50)

        if self.state == "aggressive":
            px, py = self.game.player.pos
            dist_p = self.get_distance_to(px, py)

            if dist_p > 25:
                target_angle = math.atan2(py - self.y, px - self.x)
                diff = target_angle - self.angle
                while diff > math.pi: diff -= 2 * math.pi
                while diff < -math.pi: diff += 2 * math.pi
                self.angle += diff * 0.1

                self.move(math.cos(self.angle) * self.speed, math.sin(self.angle) * self.speed)

        elif self.state == "passive":
            px, py = self.game.player.pos
            dist_p = self.get_distance_to(px, py)

            if dist_p < 200:
                angle_away = math.atan2(self.y - py, self.x - px)
                self.angle = angle_away
                self.move(math.cos(self.angle) * self.speed, math.sin(self.angle) * self.speed)
            else:
                self.wander_timer -= 1
                if self.wander_timer <= 0:
                    self.angle = random.uniform(0, math.pi * 2)
                    self.wander_timer = random.randint(30, 100)

                self.move(math.cos(self.angle) * self.speed * 0.5, math.sin(self.angle) * self.speed * 0.5)