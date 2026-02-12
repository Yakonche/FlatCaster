# structures/geometry_shapes.py
from .utils import check_overlap


def generate_triangle(segments, bboxes, cx, cy, size, color_id):
    if not check_overlap(bboxes, (cx - size - 1, cy - 1, size * 2 + 2, size + 2)):
        return False
    bboxes.append((cx - size - 1, cy - 1, size * 2 + 2, size + 2))

    segments.append((cx, cy, cx - size, cy + size, color_id))
    segments.append((cx - size, cy + size, cx + size, cy + size, color_id))
    segments.append((cx + size, cy + size, cx, cy, color_id))
    return True