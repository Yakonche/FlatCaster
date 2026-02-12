# main.py
import pygame
import sys
import random
import time
import math
from settings import *
from player import Player
from raycasting import RayCasting
from geometry import MapHandler
from colors import *
from entity_manager import EntityManager
from renderer import Renderer
from controls_config import ControlsConfig
from ui_manager import UIManager


class Game:
    def __init__(self):
        pygame.init()
        pygame.joystick.init()

        self.width = res_width
        self.height = res_height

        pygame.display.gl_set_attribute(pygame.GL_SWAP_CONTROL, 0)

        self.screen = pygame.display.set_mode(
            (self.width, self.height),
            pygame.OPENGL | pygame.DOUBLEBUF | pygame.RESIZABLE,
            vsync=0
        )

        pygame.display.set_caption("2D First-Person Viewer")
        self.clock = pygame.time.Clock()

        self.strip_height = DEFAULT_STRIP_HEIGHT
        self.zoom_level = 1.0

        self.controls_config = ControlsConfig()
        self.parsed_controls = {}
        self.update_parsed_controls()

        self.map_handler = MapHandler()
        self.player = Player(self)

        self.renderer = Renderer(self)
        self.raycasting = RayCasting(self)
        self.entity_manager = EntityManager(self)

        self.overlay_surface = pygame.Surface((self.width, self.height), pygame.SRCALPHA)
        self.font_fps = pygame.font.SysFont("Courier", 20, bold=True)

        self.freeze_entities = False
        self.dragging_strip = False
        self.drag_hover = False
        self.gc_timer = 0
        self.current_seed_input = GAME_SEED

        self.ui_manager = UIManager(self)

    def update_parsed_controls(self):
        self.parsed_controls = self.controls_config.get_parsed_binds()

    def reload_world(self, seed_val):
        self.current_seed_input = seed_val
        print(f"Reloading world with seed: {self.current_seed_input}")
        self.map_handler.reset(self.current_seed_input)
        self.entity_manager.reset(self.current_seed_input)
        self.raycasting.last_chunk = None

    def update_dimensions(self, w, h):
        self.width = w
        self.height = h
        self.strip_height = min(self.strip_height, self.height - 50)
        self.raycasting.update_settings()
        self.overlay_surface = pygame.Surface((self.width, self.height), pygame.SRCALPHA)
        self.ui_manager.on_resize(w, h)

    def handle_input(self):
        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                pygame.quit()
                sys.exit()

            if event.type == pygame.KEYDOWN and event.key == pygame.K_ESCAPE:
                if self.freeze_entities:
                    self.ui_manager.handle_escape()
                else:
                    self.freeze_entities = True
                    self.ui_manager.open_main_menu()
                continue

            if self.freeze_entities:
                self.ui_manager.handle_event(event)
            else:
                if event.type == pygame.MOUSEWHEEL:
                    if event.y > 0:
                        self.zoom_level *= 1.1
                    elif event.y < 0:
                        self.zoom_level /= 1.1
                    if self.zoom_level < 0.1: self.zoom_level = 0.1
                elif event.type == pygame.MOUSEBUTTONDOWN and event.button == 1 and self.drag_hover:
                    self.dragging_strip = True
                elif event.type == pygame.MOUSEBUTTONUP and event.button == 1:
                    self.dragging_strip = False
                elif event.type == pygame.MOUSEMOTION:
                    mouse_y = event.pos[1]
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
                if not self.freeze_entities and self.player.joystick.get_numaxes() >= 6:
                    lt_val = self.player.joystick.get_axis(4)
                    rt_val = self.player.joystick.get_axis(5)
                    if lt_val > -0.8: self.zoom_level = max(0.1, self.zoom_level / 1.02)
                    if rt_val > -0.8: self.zoom_level *= 1.02

                start_pressed = False
                for btn in [7, 6, 9]:
                    if self.player.joystick.get_numbuttons() > btn and self.player.joystick.get_button(btn):
                        start_pressed = True
                        break
                if start_pressed and not self.player.start_btn_pressed:
                    if self.freeze_entities:
                        self.ui_manager.handle_escape()
                    else:
                        self.freeze_entities = True
                        self.ui_manager.open_main_menu()
                self.player.start_btn_pressed = start_pressed

                if self.freeze_entities:
                    self.ui_manager.handle_joystick_continuous()

            except pygame.error:
                pass

    def run(self):
        while True:
            self.handle_input()

            if not self.freeze_entities:
                self.player.movement()
                self.entity_manager.update()

                self.gc_timer += 1
                if self.gc_timer > 60:
                    px, py = self.player.pos
                    limit_sq = 2500 * 2500
                    self.entity_manager.entities = [
                        e for e in self.entity_manager.entities
                        if (e.x - px) ** 2 + (e.y - py) ** 2 < limit_sq
                    ]
                    self.gc_timer = 0

            wall_render_buffer, z_buffer = self.raycasting.ray_cast_view()
            entity_render_data = self.entity_manager.get_render_data(z_buffer)

            self.overlay_surface.fill((0, 0, 0, 0))
            self.raycasting.draw_2d_entities(self.overlay_surface)

            split_y = self.height - self.strip_height
            line_color = RED_WALL if (self.drag_hover or self.dragging_strip) else WHITE
            pygame.draw.line(self.overlay_surface, line_color, (0, split_y), (self.width, split_y), 4)

            if self.freeze_entities:
                self.ui_manager.render(self.overlay_surface)

            fps_text = f"{self.clock.get_fps():.0f}"
            fps_surf_shadow = self.font_fps.render(fps_text, False, BLACK)
            self.overlay_surface.blit(fps_surf_shadow, (self.width - 58, 12))
            fps_surf = self.font_fps.render(fps_text, False, WHITE)
            self.overlay_surface.blit(fps_surf, (self.width - 60, 10))

            self.renderer.render(
                wall_render_buffer,
                self.raycasting.num_rays,
                entity_render_data,
                self.overlay_surface,
                self.strip_height,
                self.raycasting.num_segments,
                self.player.pos,
                self.zoom_level
            )

            pygame.display.flip()
            self.clock.tick(0)


if __name__ == "__main__":
    game = Game()
    game.run()