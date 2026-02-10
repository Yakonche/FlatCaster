# structures/house.py
import random
from .utils import is_area_free

def generate_house(chunk_data, x, y, w, h, color_id):
    """Crée une maison creuse avec une porte"""
    if not is_area_free(chunk_data, x - 1, y - 1, w + 2, h + 2): return False

    # Murs
    for i in range(x, x + w):
        for j in range(y, y + h):
            if i == x or i == x + w - 1 or j == y or j == y + h - 1:
                chunk_data[(i, j)] = color_id

    # Porte
    side = random.choice(['top', 'bottom', 'left', 'right'])
    if side == 'top':
        chunk_data.pop((x + w // 2, y), None)
    elif side == 'bottom':
        chunk_data.pop((x + w // 2, y + h - 1), None)
    elif side == 'left':
        chunk_data.pop((x, y + h // 2), None)
    elif side == 'right':
        chunk_data.pop((x + w - 1, y + h // 2), None)
    return True