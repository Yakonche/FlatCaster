# structures/maze.py
import random
from .utils import check_overlap


def generate_maze(segments, bboxes, start_x, start_y, w, h, color_id):
    """Génère un labyrinthe dont les murs sont des lignes mathématiques fines, sans effet d'escalier."""
    if not check_overlap(bboxes, (start_x - 1, start_y - 1, w + 2, h + 2)):
        return False
    bboxes.append((start_x - 1, start_y - 1, w + 2, h + 2))

    horiz = [[True] * w for _ in range(h + 1)]
    vert = [[True] * (w + 1) for _ in range(h)]

    visited = set([(0, 0)])
    stack = [(0, 0)]

    while stack:
        cx, cy = stack[-1]
        neighbors = []
        for dx, dy in [(0, 1), (1, 0), (0, -1), (-1, 0)]:
            nx, ny = cx + dx, cy + dy
            if 0 <= nx < w and 0 <= ny < h and (nx, ny) not in visited:
                neighbors.append((nx, ny, dx, dy))

        if neighbors:
            nx, ny, dx, dy = random.choice(neighbors)
            if dx == 1:
                vert[cy][nx] = False
            elif dx == -1:
                vert[cy][cx] = False
            elif dy == 1:
                horiz[ny][cx] = False
            elif dy == -1:
                horiz[cy][cx] = False
            visited.add((nx, ny))
            stack.append((nx, ny))
        else:
            stack.pop()

    # Entrée / Sortie
    horiz[0][0] = False
    horiz[h][w - 1] = False

    # Conversion en lignes pour la carte graphique
    for y in range(h + 1):
        for x in range(w):
            if horiz[y][x]:
                segments.append((start_x + x, start_y + y, start_x + x + 1, start_y + y, color_id))

    for y in range(h):
        for x in range(w + 1):
            if vert[y][x]:
                segments.append((start_x + x, start_y + y, start_x + x, start_y + y + 1, color_id))

    return True