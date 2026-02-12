# structures/house.py
import random
from .utils import check_overlap


def generate_house(segments, bboxes, x, y, w, h, color_id):
    """Crée une maison avec des murs d'une épaisseur absolue (segments mathématiques) et une porte"""
    if not check_overlap(bboxes, (x - 1, y - 1, w + 2, h + 2)):
        return False
    bboxes.append((x - 1, y - 1, w + 2, h + 2))

    door = random.choice(['top', 'bottom', 'left', 'right'])
    door_size = 1.0  # Taille précise de l'ouverture

    # Mur du haut
    if door == 'top':
        segments.append((x, y, x + w / 2 - door_size / 2, y, color_id))
        segments.append((x + w / 2 + door_size / 2, y, x + w, y, color_id))
    else:
        segments.append((x, y, x + w, y, color_id))

    # Mur du bas
    if door == 'bottom':
        segments.append((x, y + h, x + w / 2 - door_size / 2, y + h, color_id))
        segments.append((x + w / 2 + door_size / 2, y + h, x + w, y + h, color_id))
    else:
        segments.append((x, y + h, x + w, y + h, color_id))

    # Mur de gauche
    if door == 'left':
        segments.append((x, y, x, y + h / 2 - door_size / 2, color_id))
        segments.append((x, y + h / 2 + door_size / 2, x, y + h, color_id))
    else:
        segments.append((x, y, x, y + h, color_id))

    # Mur de droite
    if door == 'right':
        segments.append((x + w, y, x + w, y + h / 2 - door_size / 2, color_id))
        segments.append((x + w, y + h / 2 + door_size / 2, x + w, y + h, color_id))
    else:
        segments.append((x + w, y, x + w, y + h, color_id))

    return True