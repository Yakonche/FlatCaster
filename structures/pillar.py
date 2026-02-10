# structures/pillar.py
import math
from .utils import is_area_free

def generate_circle_pillar(chunk_data, cx, cy, radius, color_id, filled=True):
    """Génère un pilier circulaire"""
    if not is_area_free(chunk_data, int(cx - radius), int(cy - radius), int(radius * 2) + 1,
                             int(radius * 2) + 1): return False

    for x in range(int(cx - radius), int(cx + radius + 1)):
        for y in range(int(cy - radius), int(cy + radius + 1)):
            dist = math.sqrt((x - cx) ** 2 + (y - cy) ** 2)
            if filled:
                if dist <= radius:
                    chunk_data[(x, y)] = color_id
            else:
                if radius - 1.5 <= dist <= radius:
                    chunk_data[(x, y)] = color_id
    return True