# entities/stalker.py
import math
from .base_entity import Entity
from settings import *


class Stalker(Entity):
    def __init__(self, game, x, y):
        super().__init__(game, x, y, (40, 0, 40), "hexagon")
        self.speed = 2.5
        self.flee_mode = False

    def update(self):
        super().update()

        px, py = self.game.player.pos
        p_angle = self.game.player.angle

        dx = self.x - px
        dy = self.y - py
        dist_to_player = math.sqrt(dx * dx + dy * dy)

        angle_to_me = math.atan2(dy, dx) - p_angle
        while angle_to_me > math.pi: angle_to_me -= 2 * math.pi
        while angle_to_me < -math.pi: angle_to_me += 2 * math.pi

        is_seen = -HALF_FOV < angle_to_me < HALF_FOV and dist_to_player < 600

        if is_seen:
            self.color = (60, 0, 60)

            if dist_to_player < 200:
                self.move(math.cos(angle_to_me + p_angle) * 1.0, math.sin(angle_to_me + p_angle) * 1.0)

        else:
            self.color = (30, 0, 30)

            if dist_to_player > 30:
                target_angle = math.atan2(py - self.y, px - self.x)
                self.angle = target_angle

                mx = math.cos(self.angle) * self.speed
                my = math.sin(self.angle) * self.speed
                self.move(mx, my)