# structures/geometry_shapes.py
from .utils import check_overlap


def generate_triangle(segments, bboxes, cx, cy, size, color_id):
    """Génère un triangle parfait à l'aide de 3 segments mathématiques"""
    if not check_overlap(bboxes, (cx - size - 1, cy - 1, size * 2 + 2, size + 2)):
        return False
    bboxes.append((cx - size - 1, cy - 1, size * 2 + 2, size + 2))

    # Trace les 3 côtés du triangle (x1, y1, x2, y2, color)
    segments.append((cx, cy, cx - size, cy + size, color_id))
    segments.append((cx - size, cy + size, cx + size, cy + size, color_id))
    segments.append((cx + size, cy + size, cx, cy, color_id))
    return True