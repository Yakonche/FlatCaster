# entities/base_entity.py
import pygame
import math
import random
from settings import *
from colors import *


class Entity:
    def __init__(self, game, x, y, color, shape_type="circle"):
        self.game = game
        self.x = x
        self.y = y
        self.color = color
        self.angle = random.uniform(0, math.pi * 2)
        self.speed = random.uniform(1.0, 3.0)
        self.size = ENTITY_SIZE
        self.shape_type = shape_type

        self.vel_x = 0
        self.vel_y = 0
        self.friction = 0.85

    def get_distance_to(self, target_x, target_y):
        return math.sqrt((self.x - target_x) ** 2 + (self.y - target_y) ** 2)

    def check_wall(self, x, y):
        return self.game.map_handler.get_wall(int(x // TILE_SIZE), int(y // TILE_SIZE))

    def move(self, dx, dy):
        if not self.check_wall(self.x + dx * 2, self.y):
            self.x += dx
        if not self.check_wall(self.x, self.y + dy * 2):
            self.y += dy

    def apply_force(self, fx, fy):
        self.vel_x += fx
        self.vel_y += fy

    def apply_separation(self, separation_dist=20):
        for other in self.game.entity_manager.entities:
            if other != self:
                dist = self.get_distance_to(other.x, other.y)
                if dist < separation_dist and dist > 0:
                    dx = self.x - other.x
                    dy = self.y - other.y
                    dx /= dist
                    dy /= dist
                    self.move(dx * 1.0, dy * 1.0)

    def update(self):
        if abs(self.vel_x) > 0.1 or abs(self.vel_y) > 0.1:
            self.move(self.vel_x, self.vel_y)
            self.vel_x *= self.friction
            self.vel_y *= self.friction
        else:
            self.vel_x = 0
            self.vel_y = 0

        self.apply_separation()

    def draw_2d(self, surface, offset_x, offset_y, zoom):
        screen_x = self.x * zoom + offset_x
        screen_y = self.y * zoom + offset_y

        if -50 <= screen_x <= self.game.width + 50 and -50 <= screen_y <= self.game.height + 50:
            size = max(4, int(self.size * zoom))

            if self.shape_type == "circle":
                pygame.draw.circle(surface, self.color, (screen_x, screen_y), size)
            elif self.shape_type == "square":
                rect = pygame.Rect(screen_x - size, screen_y - size, size * 2, size * 2)
                pygame.draw.rect(surface, self.color, rect)
            elif self.shape_type == "triangle":
                points = [
                    (screen_x, screen_y - size),
                    (screen_x - size, screen_y + size),
                    (screen_x + size, screen_y + size)
                ]
                pygame.draw.polygon(surface, self.color, points)
            elif self.shape_type == "pentagon":
                points = []
                for i in range(5):
                    angle_deg = 72 * i - 18
                    rad = math.radians(angle_deg)
                    points.append((screen_x + math.cos(rad) * size, screen_y + math.sin(rad) * size))
                pygame.draw.polygon(surface, self.color, points)
            elif self.shape_type == "hexagon":
                points = []
                for i in range(6):
                    angle_deg = 60 * i
                    rad = math.radians(angle_deg)
                    points.append((screen_x + math.cos(rad) * size, screen_y + math.sin(rad) * size))
                pygame.draw.polygon(surface, self.color, points)
            elif self.shape_type == "diamond":
                points = [
                    (screen_x, screen_y - size * 1.5),
                    (screen_x + size, screen_y),
                    (screen_x, screen_y + size * 1.5),
                    (screen_x - size, screen_y)
                ]
                pygame.draw.polygon(surface, self.color, points)

            end_x = screen_x + math.cos(self.angle) * size * 1.5
            end_y = screen_y + math.sin(self.angle) * size * 1.5
            pygame.draw.line(surface, WHITE, (screen_x, screen_y), (end_x, end_y), 1)