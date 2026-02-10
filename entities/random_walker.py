# entities/random_walker.py
import math
import random
from .base_entity import Entity
from colors import ENTITY_BLUE


class RandomWalker(Entity):
    def __init__(self, game, x, y, shape="square"):
        super().__init__(game, x, y, ENTITY_BLUE, shape)
        self.change_dir_timer = 0

    def update(self):
        super().update()
        
        self.change_dir_timer += 1

        dx = math.cos(self.angle) * self.speed
        dy = math.sin(self.angle) * self.speed

        hit_wall = self.check_wall(self.x + dx * 5, self.y + dy * 5)

        if hit_wall or self.change_dir_timer > 120:
            self.angle = random.uniform(0, math.pi * 2)
            self.change_dir_timer = 0
        else:
            self.move(dx, dy)