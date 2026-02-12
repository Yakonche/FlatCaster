# entities/base_entity.py
import pygame
import math
import random
from settings import *
from colors import *


class Entity:
    def __init__(self, game, x, y, color, shape_type="ciliate"):
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

        self.animation_offset = random.uniform(0, 1000)
        self.wobble_points = [random.uniform(0.8, 1.2) for _ in range(8)]

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

    def _draw_flagellum(self, surface, start_pos, angle, length, color, wave_speed=0.2, amplitude=5):
        time = pygame.time.get_ticks() * 0.01 + self.animation_offset
        points = [start_pos]

        segments = 10
        for i in range(1, segments + 1):
            dist = (length / segments) * i
            wave = math.sin(time - i * 0.5) * (amplitude * (i / segments))

            perp_angle = angle + math.pi / 2

            base_x = start_pos[0] - math.cos(angle) * dist
            base_y = start_pos[1] - math.sin(angle) * dist

            final_x = base_x + math.cos(perp_angle) * wave
            final_y = base_y + math.sin(perp_angle) * wave
            points.append((final_x, final_y))

        if len(points) > 1:
            pygame.draw.lines(surface, color, False, points, 2)

    def _draw_amoeba_body(self, surface, center, radius, color, time):
        points = []
        num_points = 12
        for i in range(num_points):
            theta = (i / num_points) * math.pi * 2
            wobble = math.sin(time * 0.005 + theta * 3) * 0.2 + \
                     math.cos(time * 0.003 + theta * 5) * 0.1
            r = radius * (1 + wobble * 0.3)
            px = center[0] + math.cos(theta) * r
            py = center[1] + math.sin(theta) * r
            points.append((px, py))

        pygame.draw.polygon(surface, color, points)
        darker = (max(0, color[0] - 50), max(0, color[1] - 50), max(0, color[2] - 50))
        pygame.draw.circle(surface, darker, center, int(radius * 0.4))

    def _draw_virus_spikes(self, surface, center, radius, color, angle):
        points = []
        num_spikes = 8
        time = pygame.time.get_ticks() * 0.002

        rot_offset = angle + time

        for i in range(num_spikes * 2):
            theta = (i / (num_spikes * 2)) * math.pi * 2 + rot_offset
            is_spike = i % 2 == 0
            r = radius * (1.6 if is_spike else 0.8)
            px = center[0] + math.cos(theta) * r
            py = center[1] + math.sin(theta) * r
            points.append((px, py))

        pygame.draw.polygon(surface, color, points)
        pygame.draw.polygon(surface, (255, 255, 255), points, 1)

    def _draw_cilia(self, surface, center, radius, color):
        num_cilia = 16
        time = pygame.time.get_ticks() * 0.02
        for i in range(num_cilia):
            theta = (i / num_cilia) * math.pi * 2
            vibration = math.sin(time + i * 10) * 3
            start_x = center[0] + math.cos(theta) * radius
            start_y = center[1] + math.sin(theta) * radius
            end_x = center[0] + math.cos(theta) * (radius + 5 + vibration)
            end_y = center[1] + math.sin(theta) * (radius + 5 + vibration)
            pygame.draw.line(surface, (color[0], color[1], color[2], 100), (start_x, start_y), (end_x, end_y), 1)

    def draw_2d(self, surface, offset_x, offset_y, zoom):
        screen_x = self.x * zoom + offset_x
        screen_y = self.y * zoom + offset_y

        if not (-100 <= screen_x <= self.game.width + 100 and -100 <= screen_y <= self.game.height + 100):
            return

        size = max(4, int(self.size * zoom))
        time_ms = pygame.time.get_ticks()

        if self.shape_type == "flagellate":
            self._draw_flagellum(surface, (screen_x, screen_y), self.angle, size * 3, self.color, amplitude=8)
            pygame.draw.circle(surface, self.color, (screen_x, screen_y), size)
            pygame.draw.circle(surface, (255, 255, 255), (screen_x + 2, screen_y - 2), size // 3)

        elif self.shape_type == "amoeba":
            self._draw_amoeba_body(surface, (screen_x, screen_y), size, self.color, time_ms)

        elif self.shape_type == "virus":
            self._draw_virus_spikes(surface, (screen_x, screen_y), size, self.color, self.angle)
            pulse = abs(math.sin(time_ms * 0.005)) * 0.3 + 0.7
            pygame.draw.circle(surface, (50, 0, 0), (screen_x, screen_y), int(size * 0.5 * pulse))

        elif self.shape_type == "jellyfish":
            for offset in [-0.3, 0, 0.3]:
                self._draw_flagellum(surface, (screen_x, screen_y), self.angle + offset, size * 2.5, self.color,
                                     amplitude=4)

            dome_points = []
            steps = 10
            for i in range(steps + 1):
                angle_offset = (i / steps) * math.pi - math.pi / 2
                total_angle = self.angle + angle_offset
                dx = math.cos(total_angle) * size
                dy = math.sin(total_angle) * size
                dome_points.append((screen_x + dx, screen_y + dy))

            dome_points.append((screen_x, screen_y))
            pygame.draw.polygon(surface, self.color, dome_points)

        elif self.shape_type == "shell":
            active_pulse = 0
            if hasattr(self, 'active') and self.active:
                active_pulse = abs(math.sin(time_ms * 0.02)) * 4

            pygame.draw.circle(surface, self.color, (screen_x, screen_y), size + int(active_pulse))
            pygame.draw.circle(surface, (0, 0, 0), (screen_x, screen_y), size - 4, 2)
            end_spiral_x = screen_x + math.cos(time_ms * 0.001) * (size - 6)
            end_spiral_y = screen_y + math.sin(time_ms * 0.001) * (size - 6)
            pygame.draw.line(surface, (0, 0, 0), (screen_x, screen_y), (end_spiral_x, end_spiral_y), 2)

        else:
            self._draw_cilia(surface, (screen_x, screen_y), size, self.color)
            pygame.draw.circle(surface, self.color, (screen_x, screen_y), size)