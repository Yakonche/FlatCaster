# geometry.py
import random
from colors import *
from settings import GAME_SEED

from structures import (
    generate_house,
    generate_circle_pillar,
    generate_solid_rect,
    generate_triangle,
    generate_maze
)


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


class MapHandler:
    def __init__(self):
        self.world = {}
        self.chunk_size = 32
        self.current_seed = GAME_SEED

    def reset(self, new_seed):
        self.world.clear()
        self.current_seed = new_seed

    def get_wall(self, x, y):
        chunk_x = x // self.chunk_size
        chunk_y = y // self.chunk_size
        if (chunk_x, chunk_y) not in self.world:
            self.generate_chunk(chunk_x, chunk_y)
        return self.world[(chunk_x, chunk_y)].get((x, y), 0)

    def generate_chunk(self, cx, cy):
        chunk_data = {}

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
                generate_house(chunk_data, px, py, w, h, color)

            elif structure_type == "pillar":
                r = random.randint(2, 4)
                generate_circle_pillar(chunk_data, px, py, r, color, filled=True)

            elif structure_type == "solid_block":
                w, h = random.randint(2, 5), random.randint(2, 5)
                generate_solid_rect(chunk_data, px, py, w, h, color, filled=True)

            elif structure_type == "geometry":
                if random.random() > 0.5:
                    generate_triangle(chunk_data, px, py, random.randint(4, 8), color)
                else:
                    generate_circle_pillar(chunk_data, px, py, random.randint(4, 8), color, filled=False)

            elif structure_type == "maze":
                w = random.randrange(11, 21, 2)
                h = random.randrange(11, 21, 2)
                generate_maze(chunk_data, px, py, w, h, color)

        if cx == 0 and cy == 0:
            for y in range(-5, 6):
                for x in range(-5, 6):
                    if (x, y) in chunk_data:
                        del chunk_data[(x, y)]

        self.world[(cx, cy)] = chunk_data