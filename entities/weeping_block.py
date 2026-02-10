# entities/weeping_block.py
import math
from .base_entity import Entity
from settings import *


class WeepingBlock(Entity):
    def __init__(self, game, x, y):
        super().__init__(game, x, y, (120, 120, 120), "circle")
        self.speed = 6.0
        self.active = False

    def update(self):
        super().update()

        px, py = self.game.player.pos
        p_angle = self.game.player.angle

        dx = self.x - px
        dy = self.y - py
        dist_to_player = math.sqrt(dx * dx + dy * dy)

        angle_to_entity = math.atan2(dy, dx) - p_angle
        while angle_to_entity > math.pi: angle_to_entity -= 2 * math.pi
        while angle_to_entity < -math.pi: angle_to_entity += 2 * math.pi

        player_is_looking = -HALF_FOV < angle_to_entity < HALF_FOV

        if player_is_looking:
            self.color = (120, 120, 120)
            self.active = False
        else:
            self.color = (200, 200, 200)
            self.active = True

            if dist_to_player > 30:
                target_angle = math.atan2(py - self.y, px - self.x)
                self.angle = target_angle
                self.move(math.cos(self.angle) * self.speed, math.sin(self.angle) * self.speed)