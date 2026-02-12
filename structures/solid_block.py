# structures/solid_block.py
from .utils import check_overlap

def generate_solid_rect(segments, bboxes, x, y, w, h, color_id, filled=True):
    """Un cube fermé par 4 lignes vectorielles"""
    if not check_overlap(bboxes, (x - 1, y - 1, w + 2, h + 2)):
        return False
    bboxes.append((x - 1, y - 1, w + 2, h + 2))
    segments.append((x, y, x + w, y, color_id))
    segments.append((x + w, y, x + w, y + h, color_id))
    segments.append((x + w, y + h, x, y + h, color_id))
    segments.append((x, y + h, x, y, color_id))
    return True