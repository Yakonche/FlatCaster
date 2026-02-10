# entities/sentinel.py
import math
import random
from .base_entity import Entity
from settings import *


class Sentinel(Entity):
    def __init__(self, game, x, y):
        super().__init__(game, x, y, (50, 50, 180), "square")
        self.speed = 1.2
        self.size = 20
        self.state = "patrol"
        self.last_known_pos = None
        self.search_timer = 0
        self.patrol_target = (x, y)
        self.find_new_patrol_point()

    def find_new_patrol_point(self):
        for _ in range(10):
            tx = self.x + random.randint(-200, 200)
            ty = self.y + random.randint(-200, 200)
            if not self.check_wall(tx, ty):
                self.patrol_target = (tx, ty)
                break

    def update(self):
        super().update()

        px, py = self.game.player.pos
        dist_p = self.get_distance_to(px, py)

        can_see_player = dist_p < 400

        if can_see_player:
            self.state = "alert"
            self.last_known_pos = (px, py)
            self.color = (255, 50, 50)
        elif self.state == "alert":
            self.state = "search"
            self.search_timer = 180
            self.color = (255, 165, 0)

        target_x, target_y = self.x, self.y

        if self.state == "alert":
            target_x, target_y = px, py
            if dist_p < 40:
                return

        elif self.state == "search":
            if self.last_known_pos:
                target_x, target_y = self.last_known_pos

            dist_target = self.get_distance_to(target_x, target_y)
            if dist_target < 20:
                self.search_timer -= 1
                self.angle += 0.1
                if self.search_timer <= 0:
                    self.state = "patrol"
                    self.color = (50, 50, 180)
                    self.find_new_patrol_point()
                return

        elif self.state == "patrol":
            target_x, target_y = self.patrol_target
            if self.get_distance_to(target_x, target_y) < 10:
                self.find_new_patrol_point()

        angle = math.atan2(target_y - self.y, target_x - self.x)
        self.angle = angle
        self.move(math.cos(angle) * self.speed, math.sin(angle) * self.speed)