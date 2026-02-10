# settings.py
import math
import random
import string

res_width = 1900
res_height = 1000

DEFAULT_STRIP_HEIGHT = 50
TILE_SIZE = 50
FPS = 120
FOV = math.pi / 3
HALF_FOV = FOV / 2
SCALE = 2
PLAYER_SPEED = 7
PLAYER_ROT_SPEED = 0.05
PLAYER_SIZE = 10
ENTITY_SIZE = 15
LIGHT_INTENSITY_FACTOR = 1500.0
MIN_BRIGHTNESS = 0.05

MANUAL_SEED = "" # Vous pouvez écrire votre seed personnalisée ici

def generate_random_seed():
    chars = string.ascii_letters + string.digits
    length = random.randint(8, 32)
    return "".join(random.choice(chars) for _ in range(length))

GAME_SEED = MANUAL_SEED if MANUAL_SEED else generate_random_seed()
print(f"--- SEED DU MONDE : {GAME_SEED} ---")