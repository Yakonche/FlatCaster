# renderer.py
import pygame
import moderngl
import numpy as np
from shaders import vertex_shader_3d, fragment_shader_3d, vertex_shader_2d, fragment_shader_2d
from settings import *


class Renderer:
    def __init__(self, game):
        self.game = game
        self.ctx = moderngl.create_context()
        self.ctx.enable(moderngl.BLEND)
        self.ctx.blend_func = moderngl.SRC_ALPHA, moderngl.ONE_MINUS_SRC_ALPHA

        self.prog_3d = self.ctx.program(
            vertex_shader=vertex_shader_3d,
            fragment_shader=fragment_shader_3d
        )

        self.quad_verts = np.array([
            0.0, 0.0,
            1.0, 0.0,
            0.0, 1.0,
            1.0, 1.0
        ], dtype='f4')

        self.vbo_quad = self.ctx.buffer(self.quad_verts)

        self.prog_2d = self.ctx.program(
            vertex_shader=vertex_shader_2d,
            fragment_shader=fragment_shader_2d
        )

        self.overlay_verts = np.array([
            -1.0, 1.0, 0.0, 0.0,
            -1.0, -1.0, 0.0, 1.0,
            1.0, 1.0, 1.0, 0.0,
            1.0, -1.0, 1.0, 1.0,
        ], dtype='f4')

        self.vbo_overlay = self.ctx.buffer(self.overlay_verts)
        self.vao_overlay = self.ctx.vertex_array(
            self.prog_2d,
            [
                (self.vbo_overlay, '2f 2f', 'in_vert', 'in_texcoord')
            ]
        )

        self.texture_2d = None

    def render(self, wall_data, entity_data, surface_2d, strip_height):
        self.ctx.viewport = (0, 0, self.game.width, self.game.height)
        self.ctx.clear(0.0, 0.0, 0.0)

        self.ctx.viewport = (0, 0, self.game.width, int(strip_height))

        combined_data = []
        if wall_data:
            for w in wall_data:
                r, g, b = w[4]
                combined_data.extend([
                    w[0], w[1], w[2], w[3],
                    r / 255.0, g / 255.0, b / 255.0
                ])

        if entity_data:
            for e in entity_data:
                r, g, b = e[4]
                combined_data.extend([
                    e[0], e[1], e[2], e[3],
                    r / 255.0, g / 255.0, b / 255.0
                ])

        if combined_data:
            buffer_data = np.array(combined_data, dtype='f4')
            vbo_instance = self.ctx.buffer(buffer_data)

            vao_3d = self.ctx.vertex_array(
                self.prog_3d,
                [
                    (self.vbo_quad, '2f', 'in_vert'),
                    (vbo_instance, '1f 1f 1f 1f 3f/i',
                     'in_x_offset', 'in_width', 'in_height',
                     'in_distance', 'in_color')
                ]
            )

            self.prog_3d['u_light_intensity'].value = LIGHT_INTENSITY_FACTOR
            self.prog_3d['u_min_brightness'].value = MIN_BRIGHTNESS

            vao_3d.render(moderngl.TRIANGLE_STRIP, instances=len(combined_data) // 7)

            vbo_instance.release()
            vao_3d.release()

        self.ctx.viewport = (0, 0, self.game.width, self.game.height)

        if self.texture_2d:
            self.texture_2d.release()

        texture_data = pygame.image.tostring(surface_2d, "RGBA")
        self.texture_2d = self.ctx.texture(surface_2d.get_size(), 4, texture_data)
        self.texture_2d.use(0)
        self.prog_2d['u_texture'].value = 0

        self.vao_overlay.render(moderngl.TRIANGLE_STRIP)