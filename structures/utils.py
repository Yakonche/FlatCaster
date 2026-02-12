# structures/utils.py

def check_overlap(bboxes, new_box):
    """Vérifie si une bounding box (boîte de collision) chevauche une autre zone déjà occupée."""
    nx, ny, nw, nh = new_box
    for (bx, by, bw, bh) in bboxes:
        if not (nx + nw < bx or nx > bx + bw or ny + nh < by or ny > by + bh):
            return False # Chevauchement !
    return True # Zone libre