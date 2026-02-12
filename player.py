# player.py
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

    def is_action_pressed(self, action, keys, mouse_btns, joystick):

        b_type, b_val = self.game.parsed_controls['Bindings'].get(action, (None, None))
        if b_type == 'key' and b_val != -1 and keys[b_val]:
            return True
        if b_type == 'mouse' and b_val != -1:
            idx = b_val - 1
            if 0 <= idx < len(mouse_btns) and mouse_btns[idx]:
                return True

        b_type_pad, b_val_pad = self.game.parsed_controls['Gamepad'].get(action, (None, None))
        if b_type_pad == 'btn' and b_val_pad != -1 and joystick and joystick.get_init():
            if joystick.get_numbuttons() > b_val_pad and joystick.get_button(b_val_pad):
                return True

        return False

    def movement(self):
        dx = 0
        dy = 0
        current_speed = PLAYER_SPEED

        keys = pygame.key.get_pressed()
        mouse_btns = pygame.mouse.get_pressed()

        if self.shoot_cooldown > 0: self.shoot_cooldown -= 1
        if self.shockwave_cooldown > 0: self.shockwave_cooldown -= 1

        if self.is_action_pressed('sprint', keys, mouse_btns, self.joystick):
            current_speed *= 2.0
        if self.is_action_pressed('slow', keys, mouse_btns, self.joystick):
            current_speed *= 0.5

        if self.is_action_pressed('forward', keys, mouse_btns, self.joystick): dy -= current_speed
        if self.is_action_pressed('backward', keys, mouse_btns, self.joystick): dy += current_speed
        if self.is_action_pressed('left', keys, mouse_btns, self.joystick): dx -= current_speed
        if self.is_action_pressed('right', keys, mouse_btns, self.joystick): dx += current_speed

        # Rotation
        if self.is_action_pressed('rot_left', keys, mouse_btns, self.joystick): self.angle -= PLAYER_ROT_SPEED
        if self.is_action_pressed('rot_right', keys, mouse_btns, self.joystick): self.angle += PLAYER_ROT_SPEED

        # Actions
        if self.is_action_pressed('shoot', keys, mouse_btns, self.joystick): self.shoot()
        if self.is_action_pressed('shockwave', keys, mouse_btns, self.joystick): self.cast_shockwave()

        if self.joystick and self.joystick.get_init():
            try:
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

    def check_collision(self, dx, dy):
        scale = PLAYER_SIZE * 0.5

        if dx != 0:
            new_x = self.x + dx
            if self.game.map_handler.is_position_free(new_x, self.y, scale):
                self.x += dx

        if dy != 0:
            new_y = self.y + dy
            if self.game.map_handler.is_position_free(self.x, new_y, scale):
                self.y += dy

    @property
    def pos(self):
        return (self.x, self.y)

    @property
    def map_pos(self):
        return (int(self.x // TILE_SIZE), int(self.y // TILE_SIZE))