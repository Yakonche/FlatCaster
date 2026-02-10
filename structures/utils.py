# structures/utils.py

def is_area_free(chunk_data, x, y, w, h):
    """Vérifie si une zone rectangulaire est libre de murs dans le chunk data donné."""
    for i in range(x, x + w):
        for j in range(y, y + h):
            if (i, j) in chunk_data:
                return False
    return True