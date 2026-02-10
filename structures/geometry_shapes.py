# structures/geometry_shapes.py
from .utils import is_area_free

def generate_triangle(chunk_data, cx, cy, size, color_id):
    """Génère un triangle"""
    if not is_area_free(chunk_data, int(cx - size), int(cy), int(size * 2), int(size)): return False
    for y in range(size):
        for x in range(-y, y + 1):
            chunk_data[(int(cx + x), int(cy + y))] = color_id
    return True