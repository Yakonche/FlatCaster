# ui_manager.py
import pygame
import sys
import string
import math
import time
from settings import *
from colors import *


class UIManager:
    def __init__(self, game):
        self.game = game
        self.pause_overlay = None
        self.pause_needs_update = True

        self.pause_menu_state = "MAIN"
        self.waiting_for_bind = None

        self.ui_elements = []
        self.ui_focus_idx = 0
        self.menu_cooldown = 0

        self.current_seed_input = GAME_SEED
        self.seed_input_active = False
        self.font_ui = pygame.font.SysFont("Arial", 24)

    def on_resize(self, w, h):
        self.pause_overlay = None
        self.pause_needs_update = True

    def open_main_menu(self):
        self.pause_menu_state = "MAIN"
        self.ui_focus_idx = 0
        self.pause_needs_update = True

    def handle_escape(self):
        if self.waiting_for_bind:
            self.waiting_for_bind = None
            self.pause_needs_update = True
        elif self.pause_menu_state == "CONTROLS":
            self.pause_menu_state = "MAIN"
            self.ui_focus_idx = 1
            self.pause_needs_update = True
        else:
            self.game.freeze_entities = False
            self.pause_needs_update = True

    def draw_text_with_outline(self, surface, text, font, color, pos, outline_color=BLACK, center=False):
        offsets = [(-2, -2), (-2, 2), (2, -2), (2, 2)]
        x, y = pos
        if center:
            surf_size = font.size(text)
            x -= surf_size[0] // 2
            y -= surf_size[1] // 2
        for dx, dy in offsets:
            surf = font.render(text, True, outline_color)
            surface.blit(surf, (x + dx, y + dy))
        surf = font.render(text, True, color)
        surface.blit(surf, (x, y))

    def draw_pixel_art_title(self, surface, text, center_x, center_y, scale=8):
        font_small = pygame.font.SysFont("Arial", 12, bold=True)
        small_surf = font_small.render(text, False, WHITE)

        w = small_surf.get_width() * scale
        h = small_surf.get_height() * scale
        big_surf = pygame.transform.scale(small_surf, (w, h))

        rect = big_surf.get_rect(center=(center_x, center_y))

        shadow_surf = font_small.render(text, False, BLACK)
        shadow_big = pygame.transform.scale(shadow_surf, (w, h))
        shadow_rect = shadow_big.get_rect(center=(center_x + 6, center_y + 6))
        surface.blit(shadow_big, shadow_rect)
        surface.blit(big_surf, rect)

    def draw_main_pause_window(self, surface, center_x, center_y):
        win_width = 500
        win_height = 420
        win_x = center_x - win_width // 2
        win_y = center_y - win_height // 2

        bg_surface = pygame.Surface((win_width, win_height), pygame.SRCALPHA)
        pygame.draw.rect(bg_surface, (25, 30, 45, 230), bg_surface.get_rect(), border_radius=15)
        pygame.draw.rect(bg_surface, (80, 100, 150, 255), bg_surface.get_rect(), width=3, border_radius=15)
        surface.blit(bg_surface, (win_x, win_y))

        self.ui_elements = []
        font_btn = pygame.font.SysFont("Arial", 26, bold=True)

        def draw_btn(text, y_offset, action_id, base_color=(45, 55, 75)):
            rect = pygame.Rect(center_x - 120, win_y + y_offset, 240, 50)
            is_focused = (len(self.ui_elements) == self.ui_focus_idx)

            color = (min(255, base_color[0] + 40), min(255, base_color[1] + 50),
                     min(255, base_color[2] + 70)) if is_focused else base_color
            border_col = (255, 200, 50) if is_focused else (80, 100, 150)

            pygame.draw.rect(surface, color, rect, border_radius=8)
            pygame.draw.rect(surface, border_col, rect, width=3 if is_focused else 2, border_radius=8)

            surf = font_btn.render(text, True, WHITE)
            surface.blit(surf, surf.get_rect(center=rect.center))
            self.ui_elements.append({'id': action_id, 'rect': rect})

        draw_btn("Reprendre", 60, "resume", (40, 90, 50))
        draw_btn("Contrôles", 130, "controls")
        draw_btn("Quitter", 200, "quit", (130, 40, 40))

        seed_y = win_y + 310
        pygame.draw.line(surface, (80, 100, 150), (win_x + 30, seed_y - 25),
                         (win_x + win_width - 30, seed_y - 25), 2)
        font_item = pygame.font.SysFont("Arial", 22)
        surface.blit(font_item.render("Seed :", True, (200, 220, 255)), (win_x + 40, seed_y + 5))

        input_rect = pygame.Rect(win_x + 110, seed_y, 230, 40)
        is_input_focused = (len(self.ui_elements) == self.ui_focus_idx)
        self.ui_elements.append({'id': 'seed_input', 'rect': input_rect})
        self.seed_input_active = is_input_focused

        color_input = (40, 50, 70)
        border_input = (255, 200, 50) if is_input_focused else (80, 100, 150)
        pygame.draw.rect(surface, color_input, input_rect, border_radius=6)
        pygame.draw.rect(surface, border_input, input_rect, width=3 if is_input_focused else 2, border_radius=6)

        cursor = "_" if is_input_focused and time.time() % 1 > 0.5 else ""
        text_surf = self.font_ui.render(self.current_seed_input + cursor, True, WHITE)
        surface.blit(text_surf, (win_x + 120, seed_y + 7), pygame.Rect(0, 0, 210, 35))

        gen_rect = pygame.Rect(win_x + 355, seed_y, 110, 40)
        is_gen_focused = (len(self.ui_elements) == self.ui_focus_idx)
        self.ui_elements.append({'id': 'generate', 'rect': gen_rect})

        gen_col = (60, 130, 70) if is_gen_focused else (40, 90, 50)
        gen_bord = (255, 200, 50) if is_gen_focused else (80, 100, 150)
        pygame.draw.rect(surface, gen_col, gen_rect, border_radius=6)
        pygame.draw.rect(surface, gen_bord, gen_rect, width=3 if is_gen_focused else 2, border_radius=6)
        btn_text = font_item.render("Générer", True, WHITE)
        surface.blit(btn_text, btn_text.get_rect(center=gen_rect.center))

    def draw_settings_window(self, surface, center_x, center_y):
        win_width = 860
        win_height = 680
        win_x = center_x - win_width // 2
        win_y = center_y - win_height // 2

        bg_surface = pygame.Surface((win_width, win_height), pygame.SRCALPHA)
        pygame.draw.rect(bg_surface, (25, 30, 45, 240), bg_surface.get_rect(), border_radius=15)
        pygame.draw.rect(bg_surface, (80, 100, 150, 255), bg_surface.get_rect(), width=3, border_radius=15)
        surface.blit(bg_surface, (win_x, win_y))

        font_header = pygame.font.SysFont("Arial", 36, bold=True)
        self.draw_text_with_outline(surface, "CONTRÔLES", font_header, WHITE, (center_x, win_y + 40), BLACK,
                                    center=True)

        if self.waiting_for_bind:
            msg = "Appuyez sur une touche ou un bouton... (Echap pour annuler)"
            font_msg = pygame.font.SysFont("Arial", 22, italic=True)
            self.draw_text_with_outline(surface, msg, font_msg, (255, 220, 50), (center_x, win_y + 85), BLACK,
                                        center=True)

        font_item = pygame.font.SysFont("Arial", 22)

        row_y = win_y + 120
        surface.blit(font_item.render("Action", True, (150, 180, 220)), (win_x + 60, row_y))
        surface.blit(font_item.render("Clavier / Souris", True, (150, 180, 220)), (win_x + 320, row_y))
        surface.blit(font_item.render("Manette", True, (150, 180, 220)), (win_x + 580, row_y))
        pygame.draw.line(surface, (80, 100, 150), (win_x + 30, row_y + 35), (win_x + win_width - 30, row_y + 35), 2)

        row_y += 50
        self.ui_elements = []

        actions = [
            ("Avancer", "forward"), ("Reculer", "backward"),
            ("Gauche", "left"), ("Droite", "right"),
            ("Rotation G.", "rot_left"), ("Rotation D.", "rot_right"),
            ("Tirer (Kill)", "shoot"), ("Onde de choc", "shockwave"),
            ("Sprint", "sprint"), ("Ralentir", "slow")
        ]

        def format_bind(b_str):
            if not b_str: return "Non assigné"
            parts = b_str.split(':')
            if len(parts) != 2: return "Non assigné"
            if parts[0] == 'key': return f"Touche: {parts[1].upper()}"
            if parts[0] == 'mouse': return f"Clic {parts[1]}"
            if parts[0] == 'btn': return f"Bouton {parts[1]}"
            return b_str

        for label, action_key in actions:
            surface.blit(font_item.render(label, True, WHITE), (win_x + 60, row_y))

            rect_prim = pygame.Rect(win_x + 280, row_y - 8, 240, 36)
            is_focused_prim = (len(self.ui_elements) == self.ui_focus_idx)
            self.ui_elements.append({'id': f'bind:Bindings:{action_key}', 'rect': rect_prim})
            is_waiting_prim = (self.waiting_for_bind == ('Bindings', action_key))

            bg_prim = (180, 130, 30) if is_waiting_prim else ((60, 80, 120) if is_focused_prim else (40, 50, 70))
            border_prim = (255, 200, 50) if is_focused_prim else (80, 100, 150)
            pygame.draw.rect(surface, bg_prim, rect_prim, border_radius=6)
            pygame.draw.rect(surface, border_prim, rect_prim, width=2 if not is_focused_prim else 3, border_radius=6)

            p_bind = self.game.controls_config.get_bind('Bindings', action_key)
            surf_p = font_item.render(format_bind(p_bind), True, WHITE)
            surface.blit(surf_p, surf_p.get_rect(center=rect_prim.center))

            rect_pad = pygame.Rect(win_x + 550, row_y - 8, 240, 36)
            is_focused_pad = (len(self.ui_elements) == self.ui_focus_idx)
            self.ui_elements.append({'id': f'bind:Gamepad:{action_key}', 'rect': rect_pad})
            is_waiting_pad = (self.waiting_for_bind == ('Gamepad', action_key))

            bg_pad = (180, 130, 30) if is_waiting_pad else ((60, 80, 120) if is_focused_pad else (40, 50, 70))
            border_pad = (255, 200, 50) if is_focused_pad else (80, 100, 150)
            pygame.draw.rect(surface, bg_pad, rect_pad, border_radius=6)
            pygame.draw.rect(surface, border_pad, rect_pad, width=2 if not is_focused_pad else 3, border_radius=6)

            g_bind = self.game.controls_config.get_bind('Gamepad', action_key)
            surf_g = font_item.render(format_bind(g_bind), True, WHITE)
            surface.blit(surf_g, surf_g.get_rect(center=rect_pad.center))

            row_y += 45

        btn_back_rect = pygame.Rect(center_x - 100, win_y + win_height - 70, 200, 45)
        is_back_focused = (len(self.ui_elements) == self.ui_focus_idx)
        self.ui_elements.append({'id': 'back', 'rect': btn_back_rect})

        bg_back = (160, 60, 60) if is_back_focused else (120, 40, 40)
        border_back = (255, 200, 50) if is_back_focused else (180, 80, 80)
        pygame.draw.rect(surface, bg_back, btn_back_rect, border_radius=8)
        pygame.draw.rect(surface, border_back, btn_back_rect, width=3 if is_back_focused else 2, border_radius=8)

        font_btn = pygame.font.SysFont("Arial", 26, bold=True)
        surf_back = font_btn.render("Retour", True, WHITE)
        surface.blit(surf_back, surf_back.get_rect(center=btn_back_rect.center))

    def update_pause_overlay(self):
        self.pause_overlay = pygame.Surface((self.game.width, self.game.height), pygame.SRCALPHA)
        center_x = self.game.width // 2
        center_y = self.game.height // 2

        if self.pause_menu_state == "MAIN":
            self.draw_pixel_art_title(self.pause_overlay, "PAUSE", center_x, center_y - 260, scale=8)
            self.draw_main_pause_window(self.pause_overlay, center_x, center_y)
        elif self.pause_menu_state == "CONTROLS":
            self.draw_settings_window(self.pause_overlay, center_x, center_y)

        self.pause_needs_update = False

    def navigate_menu(self, dx, dy):
        if not self.ui_elements: return
        if self.ui_focus_idx >= len(self.ui_elements): self.ui_focus_idx = 0

        current = self.ui_elements[self.ui_focus_idx]['rect']
        best_idx = self.ui_focus_idx
        min_dist = float('inf')

        for i, el in enumerate(self.ui_elements):
            if i == self.ui_focus_idx: continue
            r = el['rect']

            valid = False
            angle = math.degrees(math.atan2(r.centery - current.centery, r.centerx - current.centerx))

            if dx > 0 and -45 <= angle <= 45:
                valid = True
            elif dx < 0 and (angle >= 135 or angle <= -135):
                valid = True
            elif dy > 0 and 45 <= angle <= 135:
                valid = True
            elif dy < 0 and -135 <= angle <= -45:
                valid = True

            if valid:
                dist = math.hypot(r.centerx - current.centerx, r.centery - current.centery)
                if dx != 0: dist += abs(r.centery - current.centery) * 3
                if dy != 0: dist += abs(r.centerx - current.centerx) * 3

                if dist < min_dist:
                    min_dist = dist
                    best_idx = i

        if best_idx != self.ui_focus_idx:
            self.ui_focus_idx = best_idx
            self.pause_needs_update = True

    def activate_focused_ui(self):
        if not self.ui_elements or self.ui_focus_idx >= len(self.ui_elements): return
        action = self.ui_elements[self.ui_focus_idx]['id']

        if action == 'resume':
            self.game.freeze_entities = False
            self.pause_needs_update = True
        elif action == 'controls':
            self.pause_menu_state = "CONTROLS"
            self.ui_focus_idx = 0
            self.pause_needs_update = True
        elif action == 'quit':
            pygame.quit()
            sys.exit()
        elif action == 'seed_input':
            pass
        elif action == 'generate':
            self.game.reload_world(self.current_seed_input)
        elif action == 'back':
            self.pause_menu_state = "MAIN"
            self.ui_focus_idx = 1
            self.pause_needs_update = True
        elif action.startswith('bind:'):
            _, section, act = action.split(':')
            self.waiting_for_bind = (section, act)
            self.pause_needs_update = True

    def handle_event(self, event):
        if self.waiting_for_bind:
            if event.type == pygame.KEYDOWN and event.key != pygame.K_ESCAPE:
                section, action = self.waiting_for_bind
                if section == 'Bindings':
                    self.game.controls_config.set_bind(section, action, f"key:{pygame.key.name(event.key)}")
                    self.game.update_parsed_controls()
                self.waiting_for_bind = None
                self.pause_needs_update = True
            elif event.type == pygame.MOUSEBUTTONDOWN and event.button in (1, 2, 3, 4, 5):
                section, action = self.waiting_for_bind
                if section == 'Bindings':
                    self.game.controls_config.set_bind(section, action, f"mouse:{event.button}")
                    self.game.update_parsed_controls()
                self.waiting_for_bind = None
                self.pause_needs_update = True
            elif event.type == pygame.JOYBUTTONDOWN:
                section, action = self.waiting_for_bind
                if section == 'Gamepad':
                    self.game.controls_config.set_bind(section, action, f"btn:{event.button}")
                    self.game.update_parsed_controls()
                self.waiting_for_bind = None
                self.pause_needs_update = True
            return

        if event.type == pygame.KEYDOWN:
            if event.key == pygame.K_UP:
                self.navigate_menu(0, -1)
            elif event.key == pygame.K_DOWN:
                self.navigate_menu(0, 1)
            elif event.key == pygame.K_LEFT:
                self.navigate_menu(-1, 0)
            elif event.key == pygame.K_RIGHT:
                self.navigate_menu(1, 0)
            elif event.key == pygame.K_RETURN:
                self.activate_focused_ui()
            elif self.seed_input_active and self.pause_menu_state == "MAIN":
                if event.key == pygame.K_BACKSPACE:
                    self.current_seed_input = self.current_seed_input[:-1]
                    self.pause_needs_update = True
                elif event.unicode in string.ascii_letters + string.digits and len(self.current_seed_input) < 25:
                    self.current_seed_input += event.unicode
                    self.pause_needs_update = True

        elif event.type == pygame.JOYHATMOTION:
            hx, hy = event.value
            if hy == 1:
                self.navigate_menu(0, -1)
            elif hy == -1:
                self.navigate_menu(0, 1)
            if hx == -1:
                self.navigate_menu(-1, 0)
            elif hx == 1:
                self.navigate_menu(1, 0)

        elif event.type == pygame.JOYBUTTONDOWN:
            if event.button == 0:
                self.activate_focused_ui()
            elif event.button == 1:
                if self.pause_menu_state == "CONTROLS":
                    self.pause_menu_state = "MAIN"
                    self.ui_focus_idx = 1
                    self.pause_needs_update = True
                elif self.pause_menu_state == "MAIN":
                    self.game.freeze_entities = False
                    self.pause_needs_update = True

        elif event.type == pygame.MOUSEMOTION:
            for i, el in enumerate(self.ui_elements):
                if el['rect'].collidepoint(event.pos):
                    if self.ui_focus_idx != i:
                        self.ui_focus_idx = i
                        self.pause_needs_update = True
                    break

        elif event.type == pygame.MOUSEBUTTONDOWN and event.button == 1:
            for i, el in enumerate(self.ui_elements):
                if el['rect'].collidepoint(event.pos):
                    self.ui_focus_idx = i
                    self.activate_focused_ui()
                    break

    def handle_joystick_continuous(self):
        if self.waiting_for_bind: return
        if self.menu_cooldown > 0: self.menu_cooldown -= 1
        if self.menu_cooldown == 0:
            try:
                y_axis = self.game.player.joystick.get_axis(1)
                x_axis = self.game.player.joystick.get_axis(0)
                moved = False
                if y_axis < -0.6:
                    self.navigate_menu(0, -1);
                    moved = True
                elif y_axis > 0.6:
                    self.navigate_menu(0, 1);
                    moved = True
                elif x_axis < -0.6:
                    self.navigate_menu(-1, 0);
                    moved = True
                elif x_axis > 0.6:
                    self.navigate_menu(1, 0);
                    moved = True
                if moved: self.menu_cooldown = 15
            except pygame.error:
                pass

    def render(self, target_surface):
        if self.seed_input_active:
            self.pause_needs_update = True
        if self.pause_needs_update or self.pause_overlay is None:
            self.update_pause_overlay()
        target_surface.blit(self.pause_overlay, (0, 0))