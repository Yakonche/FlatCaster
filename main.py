# main.py
import pygame
import sys
import string
import random
from settings import *
from player import Player
from raycasting import RayCasting
from geometry import MapHandler
from colors import *
from entity_manager import EntityManager
from renderer import Renderer


class Game:
    def __init__(self):
        pygame.init()
        pygame.joystick.init()

        self.width = res_width
        self.height = res_height

        self.screen = pygame.display.set_mode(
            (self.width, self.height),
            pygame.OPENGL | pygame.DOUBLEBUF | pygame.RESIZABLE
        )

        pygame.display.set_caption("2D First-Person Viewer")
        self.clock = pygame.time.Clock()

        self.strip_height = DEFAULT_STRIP_HEIGHT
        self.zoom_level = 1.0

        self.map_handler = MapHandler()
        self.player = Player(self)
        self.raycasting = RayCasting(self)
        self.entity_manager = EntityManager(self)

        self.renderer = Renderer(self)

        self.overlay_surface = pygame.Surface((self.width, self.height), pygame.SRCALPHA)

        self.dragging_strip = False
        self.drag_hover = False

        self.freeze_entities = False

        # --- UI Seed ---
        self.current_seed_input = GAME_SEED
        self.seed_input_active = False
        self.font_ui = pygame.font.SysFont("Arial", 24)

    def reload_world(self):
        """Recharge tout avec la nouvelle seed"""
        print(f"Reloading world with seed: {self.current_seed_input}")
        self.map_handler.reset(self.current_seed_input)
        self.entity_manager.reset(self.current_seed_input)

    def update_dimensions(self, w, h):
        self.width = w
        self.height = h
        self.strip_height = min(self.strip_height, self.height - 50)
        self.raycasting.update_settings()
        self.overlay_surface = pygame.Surface((self.width, self.height), pygame.SRCALPHA)

    def draw_text_with_outline(self, text, font, color, pos, outline_color=BLACK, center=False):
        offsets = [(-2, -2), (-2, 2), (2, -2), (2, 2)]
        x, y = pos
        if center:
            surf_size = font.size(text)
            x -= surf_size[0] // 2
            y -= surf_size[1] // 2
        for dx, dy in offsets:
            surf = font.render(text, True, outline_color)
            self.overlay_surface.blit(surf, (x + dx, y + dy))
        surf = font.render(text, True, color)
        self.overlay_surface.blit(surf, (x, y))

    def draw_pixel_art_title(self, text, center_x, center_y, scale=8):
        font_small = pygame.font.SysFont("Arial", 12, bold=True)
        small_surf = font_small.render(text, False, WHITE)

        w = small_surf.get_width() * scale
        h = small_surf.get_height() * scale
        big_surf = pygame.transform.scale(small_surf, (w, h))

        rect = big_surf.get_rect(center=(center_x, center_y))

        shadow_surf = font_small.render(text, False, BLACK)
        shadow_big = pygame.transform.scale(shadow_surf, (w, h))
        shadow_rect = shadow_big.get_rect(center=(center_x + 6, center_y + 6))
        self.overlay_surface.blit(shadow_big, shadow_rect)

        self.overlay_surface.blit(big_surf, rect)

    def draw_controls_window(self, center_x, center_y):
        controls = [
            ("Déplacement", "ZQSD", "L-Stick"),
            ("Rotation", "Flèches / Souris", "R-Stick"),
            ("Tir (Kill)", "Clic Gauche / Espace", "RB"),
            ("Onde de Choc", "Clic Droit / Shift", "LB"),
            ("Zoom", "Molette", "Gâchettes LT/RT"),
            ("Pause", "Aucune", "Start"),
            ("Quitter", "Echap", "B")
        ]

        win_width = 900
        win_height = 520
        win_x = center_x - win_width // 2
        win_y = center_y - win_height // 2

        bg_surface = pygame.Surface((win_width, win_height))
        bg_surface.set_alpha(240)
        bg_surface.fill(BLACK)
        self.overlay_surface.blit(bg_surface, (win_x, win_y))
        pygame.draw.rect(self.overlay_surface, WHITE, (win_x, win_y, win_width, win_height), 4)

        font_header = pygame.font.SysFont("Arial", 32, bold=True)
        self.draw_text_with_outline("CONTRÔLES", font_header, WHITE, (win_x + win_width // 2, win_y + 40), BLACK,
                                    center=True)

        row_y = win_y + 100
        font_item = pygame.font.SysFont("Arial", 26)
        col_x = [win_x + 50, win_x + 350, win_x + 650]

        for action, key, pad in controls:
            self.overlay_surface.blit(font_item.render(action, True, WHITE), (col_x[0], row_y))
            self.overlay_surface.blit(font_item.render(key, True, LIGHTGREY), (col_x[1], row_y))
            self.overlay_surface.blit(font_item.render(pad, True, (0, 255, 255)), (col_x[2], row_y))
            row_y += 40

        seed_y = row_y + 40
        pygame.draw.line(self.overlay_surface, WHITE, (win_x + 20, seed_y), (win_x + win_width - 20, seed_y), 2)

        seed_y += 30
        self.overlay_surface.blit(font_item.render("Seed : ", True, WHITE), (win_x + 50, seed_y))

        input_rect = pygame.Rect(win_x + 150, seed_y - 5, 470, 40)
        color_input = WHITE if self.seed_input_active else LIGHTGREY
        pygame.draw.rect(self.overlay_surface, DARKGREY, input_rect)
        pygame.draw.rect(self.overlay_surface, color_input, input_rect, 2)

        text_surf = self.font_ui.render(self.current_seed_input, True, WHITE)
        area = pygame.Rect(0, 0, 460, 40)
        self.overlay_surface.blit(text_surf, (win_x + 160, seed_y), area)

        btn_rect = pygame.Rect(win_x + 640, seed_y - 5, 150, 40)
        pygame.draw.rect(self.overlay_surface, (0, 100, 0), btn_rect)
        pygame.draw.rect(self.overlay_surface, WHITE, btn_rect, 2)
        btn_text = self.font_ui.render("GÉNÉRER", True, WHITE)
        text_rect = btn_text.get_rect(center=btn_rect.center)
        self.overlay_surface.blit(btn_text, text_rect)

        self.ui_input_rect = input_rect
        self.ui_gen_btn_rect = btn_rect

    def handle_input(self):
        keys = pygame.key.get_pressed()
        if keys[pygame.K_ESCAPE]:
            pygame.quit()
            sys.exit()

        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                pygame.quit()
                sys.exit()

            if self.freeze_entities:
                if event.type == pygame.MOUSEBUTTONDOWN:
                    if hasattr(self, 'ui_input_rect') and self.ui_input_rect.collidepoint(event.pos):
                        self.seed_input_active = True
                    else:
                        self.seed_input_active = False

                    if hasattr(self, 'ui_gen_btn_rect') and self.ui_gen_btn_rect.collidepoint(event.pos):
                        self.reload_world()

                if event.type == pygame.KEYDOWN and self.seed_input_active:
                    if event.key == pygame.K_RETURN:
                        self.reload_world()
                    elif event.key == pygame.K_BACKSPACE:
                        self.current_seed_input = self.current_seed_input[:-1]
                    else:
                        if len(self.current_seed_input) < 25 and event.unicode in string.ascii_letters + string.digits:
                            self.current_seed_input += event.unicode

            if not self.freeze_entities:
                if event.type == pygame.MOUSEWHEEL:
                    if event.y > 0:
                        self.zoom_level *= 1.1
                    elif event.y < 0:
                        self.zoom_level /= 1.1
                    if self.zoom_level < 0.1: self.zoom_level = 0.1
                elif event.type == pygame.MOUSEBUTTONDOWN:
                    if event.button == 1 and self.drag_hover: self.dragging_strip = True
                elif event.type == pygame.MOUSEBUTTONUP:
                    if event.button == 1: self.dragging_strip = False
                elif event.type == pygame.MOUSEMOTION:
                    mouse_x, mouse_y = event.pos
                    limit_y = self.height - self.strip_height
                    if limit_y - 10 < mouse_y < limit_y + 10:
                        self.drag_hover = True
                        pygame.mouse.set_cursor(pygame.SYSTEM_CURSOR_SIZENS)
                    else:
                        if not self.dragging_strip:
                            self.drag_hover = False
                            pygame.mouse.set_cursor(pygame.SYSTEM_CURSOR_ARROW)
                    if self.dragging_strip:
                        new_h = self.height - mouse_y
                        if 50 < new_h < self.height - 50: self.strip_height = new_h

            if event.type == pygame.VIDEORESIZE:
                self.update_dimensions(event.w, event.h)
            elif event.type == pygame.JOYDEVICEADDED:
                self.player.connect_joystick()
            elif event.type == pygame.JOYDEVICEREMOVED:
                if self.player.joystick and self.player.joystick.get_instance_id() == event.instance_id:
                    self.player.disconnect_joystick()

        if self.player.joystick and self.player.joystick.get_init():
            try:
                if self.player.joystick.get_numaxes() >= 6:
                    lt_val = self.player.joystick.get_axis(4)
                    rt_val = self.player.joystick.get_axis(5)
                    if lt_val > -0.8:
                        self.zoom_level /= 1.02
                        if self.zoom_level < 0.1: self.zoom_level = 0.1
                    if rt_val > -0.8:
                        self.zoom_level *= 1.02

                start_pressed = False
                start_buttons = [7, 6, 9]
                for btn in start_buttons:
                    if self.player.joystick.get_numbuttons() > btn and self.player.joystick.get_button(btn):
                        start_pressed = True
                        break
                if start_pressed and not self.player.start_btn_pressed:
                    self.freeze_entities = not self.freeze_entities
                self.player.start_btn_pressed = start_pressed
            except pygame.error:
                pass

    def run(self):
        while True:
            self.handle_input()

            if not self.freeze_entities:
                self.player.movement()
                self.entity_manager.update()

            wall_render_data, z_buffer = self.raycasting.ray_cast_view()
            entity_render_data = self.entity_manager.get_render_data(z_buffer)

            self.overlay_surface.fill((0, 0, 0, 0))
            self.raycasting.draw_2d_map(self.overlay_surface)

            split_y = self.height - self.strip_height
            line_color = RED_WALL if (self.drag_hover or self.dragging_strip) else WHITE
            pygame.draw.line(self.overlay_surface, line_color, (0, split_y), (self.width, split_y), 4)

            if self.freeze_entities:
                center_x = self.width // 2
                center_y = self.height // 2

                self.draw_pixel_art_title("PAUSE", center_x, center_y - 380, scale=8)

                self.draw_controls_window(center_x, center_y)

            self.renderer.render(wall_render_data, entity_render_data, self.overlay_surface, self.strip_height)

            pygame.display.flip()
            self.clock.tick(FPS)


if __name__ == "__main__":
    game = Game()
    game.run()