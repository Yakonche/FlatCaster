# structures/solid_block.py
from .utils import is_area_free

def generate_solid_rect(chunk_data, x, y, w, h, color_id, filled=True):
    """Génère un bloc rectangulaire simple"""
    if not is_area_free(chunk_data, x, y, w, h): return False
    for i in range(x, x + w):
        for j in range(y, y + h):
            if filled or (i == x or i == x + w - 1 or j == y or j == y + h - 1):
                chunk_data[(i, j)] = color_id
    return True