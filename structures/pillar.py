# structures/pillar.py
import math
from .utils import check_overlap


def generate_circle_pillar(segments, bboxes, cx, cy, radius, color_id, filled=True):
    """Un cercle parfait constitué de multiples micro-segments (fini l'escalier carré)"""
    if not check_overlap(bboxes, (cx - radius - 1, cy - radius - 1, radius * 2 + 2, radius * 2 + 2)):
        return False
    bboxes.append((cx - radius - 1, cy - radius - 1, radius * 2 + 2, radius * 2 + 2))

    # Résolution adaptative selon la taille (un grand cercle aura plus de faces)
    num_segs = max(8, int(radius * 4))
    for i in range(num_segs):
        a1 = (i / num_segs) * 2 * math.pi
        a2 = ((i + 1) / num_segs) * 2 * math.pi
        segments.append((
            cx + math.cos(a1) * radius, cy + math.sin(a1) * radius,
            cx + math.cos(a2) * radius, cy + math.sin(a2) * radius,
            color_id
        ))
    return True