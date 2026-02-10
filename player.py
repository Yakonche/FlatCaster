import math
import pygame
from settings import *


class Player:
    def __init__(self, game):
        self.game = game
        self.x = TILE_SIZE * 2
        self.y = TILE_SIZE * 2
        self.angle = -math.pi / 2

        if not pygame.joystick.get_init():
            pygame.joystick.init()

        self.joystick = None
        self.start_btn_pressed = False

        self.shoot_cooldown = 0
        self.shockwave_cooldown = 0

        self.connect_joystick()

    def connect_joystick(self):
        if pygame.joystick.get_count() > 0:
            try:
                self.joystick = pygame.joystick.Joystick(0)
                self.joystick.init()
                print(f"Joystick connecté : {self.joystick.get_name()}")
            except Exception as e:
                print(f"Erreur connexion joystick : {e}")
                self.joystick = None
        else:
            print("Aucun joystick détecté pour le moment.")
            self.joystick = None

    def disconnect_joystick(self):
        if self.joystick:
            print(f"Déconnexion du joystick : {self.joystick.get_name()}")
            self.joystick.quit()
            self.joystick = None

    def movement(self):
        dx = 0
        dy = 0
        current_speed = PLAYER_SPEED

        keys = pygame.key.get_pressed()

        if self.shoot_cooldown > 0: self.shoot_cooldown -= 1
        if self.shockwave_cooldown > 0: self.shockwave_cooldown -= 1

        if keys[pygame.K_z] or keys[pygame.K_w]: dy -= current_speed
        if keys[pygame.K_s]: dy += current_speed
        if keys[pygame.K_q] or keys[pygame.K_a]: dx -= current_speed
        if keys[pygame.K_d]: dx += current_speed

        if keys[pygame.K_UP]:
            dx += math.cos(self.angle) * current_speed
            dy += math.sin(self.angle) * current_speed
        if keys[pygame.K_DOWN]:
            dx -= math.cos(self.angle) * current_speed
            dy -= math.sin(self.angle) * current_speed

        if keys[pygame.K_LEFT]: self.angle -= PLAYER_ROT_SPEED
        if keys[pygame.K_RIGHT]: self.angle += PLAYER_ROT_SPEED

        if keys[pygame.K_SPACE]: self.shoot()
        if keys[pygame.K_LSHIFT]: self.cast_shockwave()

        mouse_btns = pygame.mouse.get_pressed()
        if mouse_btns[0]: self.shoot()
        if mouse_btns[2]: self.cast_shockwave()

        if self.joystick and self.joystick.get_init():
            try:
                if self.joystick.get_numbuttons() > 0 and self.joystick.get_button(0):
                    current_speed *= 2.0
                if self.joystick.get_numbuttons() > 1 and self.joystick.get_button(1):
                    current_speed *= 0.5

                if self.joystick.get_numbuttons() >= 6:
                    if self.joystick.get_button(5):
                        self.shoot()
                    if self.joystick.get_button(4):
                        self.cast_shockwave()

                ax0 = self.joystick.get_axis(0)
                ax1 = self.joystick.get_axis(1)

                deadzone = 0.15

                if abs(ax0) > deadzone:
                    dx += ax0 * current_speed
                if abs(ax1) > deadzone:
                    dy += ax1 * current_speed

                rs_x, rs_y = 0, 0
                if self.joystick.get_numaxes() >= 4:
                    rs_x = self.joystick.get_axis(2)
                    rs_y = self.joystick.get_axis(3)

                if math.sqrt(rs_x ** 2 + rs_y ** 2) > 0.5:
                    target_angle = math.atan2(rs_y, rs_x)
                    self.angle = target_angle


            except pygame.error:
                self.disconnect_joystick()

        self.check_collision(dx, dy)

    def shoot(self):
        if self.shoot_cooldown <= 0:
            self.game.entity_manager.add_projectile(self.x, self.y, self.angle)
            self.shoot_cooldown = 15

    def cast_shockwave(self):
        if self.shockwave_cooldown <= 0:
            self.game.entity_manager.apply_shockwave(self.x, self.y)
            self.shockwave_cooldown = 60

    def check_wall(self, x, y):
        return self.game.map_handler.get_wall(int(x // TILE_SIZE), int(y // TILE_SIZE))

    def check_collision(self, dx, dy):
        scale = PLAYER_SIZE * 0.5

        if dx != 0:
            new_x = self.x + dx
            wall_hit = (self.check_wall(new_x + scale, self.y + scale) or
                        self.check_wall(new_x + scale, self.y - scale) or
                        self.check_wall(new_x - scale, self.y + scale) or
                        self.check_wall(new_x - scale, self.y - scale))

            if not wall_hit:
                self.x += dx

        if dy != 0:
            new_y = self.y + dy
            wall_hit = (self.check_wall(self.x + scale, new_y + scale) or
                        self.check_wall(self.x + scale, new_y - scale) or
                        self.check_wall(self.x - scale, new_y + scale) or
                        self.check_wall(self.x - scale, new_y - scale))

            if not wall_hit:
                self.y += dy

    @property
    def pos(self):
        return (self.x, self.y)

    @property
    def map_pos(self):
        return (int(self.x // TILE_SIZE), int(self.y // TILE_SIZE))