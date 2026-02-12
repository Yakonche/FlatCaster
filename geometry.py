# geometry.py
import random
import math
from colors import *
from settings import GAME_SEED, TILE_SIZE

from structures.house import generate_house
from structures.pillar import generate_circle_pillar
from structures.solid_block import generate_solid_rect
from structures.geometry_shapes import generate_triangle
from structures.maze import generate_maze


class WallDef:
    def __init__(self, color):
        self.color = color


WALL_TYPES = {
    1: WallDef(RED_WALL),
    2: WallDef(GREEN_WALL),
    3: WallDef(BLUE_WALL),
    4: WallDef(PURPLE_WALL),
    5: WallDef(ORANGE_WALL)
}


def dist_to_segment(px, py, x1, y1, x2, y2):
    l2 = (x2 - x1) ** 2 + (y2 - y1) ** 2
    if l2 == 0:
        return math.sqrt((px - x1) ** 2 + (py - y1) ** 2)
    t = max(0, min(1, ((px - x1) * (x2 - x1) + (py - y1) * (y2 - y1)) / l2))
    proj_x = x1 + t * (x2 - x1)
    proj_y = y1 + t * (y2 - y1)
    return math.sqrt((px - proj_x) ** 2 + (py - proj_y) ** 2)


class MapHandler:
    def __init__(self):
        self.world = {}
        self.chunk_size = 32
        self.current_seed = GAME_SEED

    def reset(self, new_seed):
        self.world.clear()
        self.current_seed = new_seed
        if hasattr(self, 'last_cx'):
            del self.last_cx

    def get_active_segments(self, player_x, player_y, radius_chunks=3):
        cx = int((player_x / TILE_SIZE) // self.chunk_size)
        cy = int((player_y / TILE_SIZE) // self.chunk_size)

        if hasattr(self, 'last_cx') and self.last_cx == cx and self.last_cy == cy and self.last_radius == radius_chunks:
            return self.cached_segments

        active = []
        for dy in range(-radius_chunks, radius_chunks + 1):
            for dx in range(-radius_chunks, radius_chunks + 1):
                key = (cx + dx, cy + dy)
                if key not in self.world:
                    self.generate_chunk(*key)
                active.extend(self.world[key]['segments'])

        self.last_cx = cx
        self.last_cy = cy
        self.last_radius = radius_chunks
        self.cached_segments = active
        return active

    def is_position_free(self, x, y, radius):
        tile_x = x / TILE_SIZE
        tile_y = y / TILE_SIZE
        r_tile = radius / TILE_SIZE

        segments = self.get_active_segments(x, y, 1)

        for (x1, y1, x2, y2, c) in segments:
            if dist_to_segment(tile_x, tile_y, x1, y1, x2, y2) < r_tile:
                return False
        return True

    def get_wall(self, tx, ty):
        segments = self.get_active_segments(tx * TILE_SIZE, ty * TILE_SIZE, 1)
        for (x1, y1, x2, y2, c) in segments:
            min_x, max_x = min(x1, x2), max(x1, x2)
            min_y, max_y = min(y1, y2), max(y1, y2)
            if min_x <= tx + 1 and max_x >= tx and min_y <= ty + 1 and max_y >= ty:
                return c
        return 0

    def generate_chunk(self, cx, cy):
        segments = []
        bboxes = []

        random.seed(f"{self.current_seed}_{cx}_{cy}")

        start_x = cx * self.chunk_size
        start_y = cy * self.chunk_size

        attempts = random.randint(15, 30)

        for _ in range(attempts):
            structure_weights = ["house"] * 30 + ["pillar"] * 30 + ["geometry"] * 20 + ["solid_block"] * 15 + [
                "maze"] * 10
            structure_type = random.choice(structure_weights)

            color = random.randint(1, 5)

            px = random.randint(start_x + 2, start_x + self.chunk_size - 15)
            py = random.randint(start_y + 2, start_y + self.chunk_size - 15)

            if structure_type == "house":
                w, h = random.randint(5, 10), random.randint(5, 10)
                generate_house(segments, bboxes, px, py, w, h, color)

            elif structure_type == "pillar":
                r = random.randint(2, 4)
                generate_circle_pillar(segments, bboxes, px, py, r, color, filled=True)

            elif structure_type == "solid_block":
                w, h = random.randint(2, 5), random.randint(2, 5)
                generate_solid_rect(segments, bboxes, px, py, w, h, color, filled=True)

            elif structure_type == "geometry":
                if random.random() > 0.5:
                    generate_triangle(segments, bboxes, px, py, random.randint(4, 8), color)
                else:
                    generate_circle_pillar(segments, bboxes, px, py, random.randint(4, 8), color, filled=False)

            elif structure_type == "maze":
                w = random.randrange(11, 21, 2)
                h = random.randrange(11, 21, 2)
                generate_maze(segments, bboxes, px, py, w, h, color)

        if cx == 0 and cy == 0:
            filtered_segments = []
            for (x1, y1, x2, y2, c) in segments:
                if max(x1, x2) < -5 or min(x1, x2) > 6 or max(y1, y2) < -5 or min(y1, y2) > 6:
                    filtered_segments.append((x1, y1, x2, y2, c))
            segments = filtered_segments

        self.world[(cx, cy)] = {'segments': segments, 'bboxes': bboxes}