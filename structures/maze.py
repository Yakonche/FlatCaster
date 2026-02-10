# structures/maze.py
import random
from .utils import is_area_free


def generate_maze(chunk_data, x, y, w, h, color_id):
    """
    Génère un labyrinthe parfait (solvable) en utilisant l'algorithme 'Recursive Backtracker'.
    x, y : coin haut gauche
    w, h : largeur et hauteur (doivent être impairs de préférence pour un joli rendu)
    """

    # 1. Vérification de l'espace (on a besoin d'une marge de sécurité)
    if not is_area_free(chunk_data, x, y, w, h):
        return False

    # 2. Initialisation : Tout remplir de murs
    # On utilise un set local pour faciliter la logique de l'algo avant de transférer au chunk
    walls = set()
    for i in range(w):
        for j in range(h):
            # On remplit tout le rectangle de murs dans le chunk
            chunk_data[(x + i, y + j)] = color_id
            walls.add((i, j))

    # 3. Algorithme de génération (creusage de chemins)
    # On travaille en coordonnées locales (0 à w, 0 à h)
    # On commence à (1,1) pour laisser une bordure
    start_cell = (1, 1)
    stack = [start_cell]
    visited = {start_cell}

    # On creuse le point de départ
    if (x + 1, y + 1) in chunk_data:
        del chunk_data[(x + 1, y + 1)]

    directions = [(0, 2), (0, -2), (2, 0), (-2, 0)]

    while stack:
        current_x, current_y = stack[-1]
        neighbors = []

        for dx, dy in directions:
            nx, ny = current_x + dx, current_y + dy
            # Vérifier si le voisin est dans les limites (avec une bordure de 1 mur)
            if 0 < nx < w - 1 and 0 < ny < h - 1:
                if (nx, ny) not in visited:
                    neighbors.append((nx, ny, dx, dy))

        if neighbors:
            nx, ny, dx, dy = random.choice(neighbors)

            # On casse le mur entre la cellule actuelle et la voisine (le mur intermédiaire)
            wall_x, wall_y = current_x + dx // 2, current_y + dy // 2

            # Suppression du mur intermédiaire dans le chunk
            if (x + wall_x, y + wall_y) in chunk_data:
                del chunk_data[(x + wall_x, y + wall_y)]

            # Suppression de la cellule cible dans le chunk (devient un chemin)
            if (x + nx, y + ny) in chunk_data:
                del chunk_data[(x + nx, y + ny)]

            visited.add((nx, ny))
            stack.append((nx, ny))
        else:
            stack.pop()

    # 4. Création de l'Entrée et de la Sortie
    # On assure une entrée en haut (1, 0) et une sortie en bas (w-2, h-1)
    # Entrée
    if (x + 1, y) in chunk_data:
        del chunk_data[(x + 1, y)]

    # Sortie (on cherche un point connectable en bas)
    exit_x = w - 2 if (w % 2 != 0) else w - 3
    if (x + exit_x, y + h - 1) in chunk_data:
        del chunk_data[(x + exit_x, y + h - 1)]

    return True