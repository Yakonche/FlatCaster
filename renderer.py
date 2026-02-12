# renderer.py
import pygame
import moderngl
import numpy as np
from shaders import *
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
        self.quad_verts = np.array([0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0], dtype='f4')
        self.vbo_quad = self.ctx.buffer(self.quad_verts)

        self.prog_2d = self.ctx.program(
            vertex_shader=vertex_shader_2d,
            fragment_shader=fragment_shader_2d
        )
        self.overlay_verts = np.array(
            [-1.0, 1.0, 0.0, 0.0, -1.0, -1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 1.0, -1.0, 1.0, 1.0, ], dtype='f4')
        self.vbo_overlay = self.ctx.buffer(self.overlay_verts)
        self.vao_overlay = self.ctx.vertex_array(self.prog_2d, [(self.vbo_overlay, '2f 2f', 'in_vert', 'in_texcoord')])

        self.prog_map_lines = self.ctx.program(vertex_shader=vertex_shader_map_lines,
                                               fragment_shader=fragment_shader_map_lines)
        self.prog_fov = self.ctx.program(vertex_shader=vertex_shader_fov, fragment_shader=fragment_shader_fov)

        self.vao_map_lines = self.ctx.vertex_array(self.prog_map_lines, [])
        self.vao_fov = self.ctx.vertex_array(self.prog_fov, [])

        self.texture_2d = None
        self.vbo_walls_instances = None
        self.vao_walls = None
        self.vbo_entities_instances = None
        self.vao_entities = None

        self.overlay_update_counter = 0
        self.overlay_update_interval = 2

    def _create_or_update_vao_walls(self, vbo_walls, num_rays):
        if self.vao_walls is None:
            self.vao_walls = self.ctx.vertex_array(
                self.prog_3d,
                [(self.vbo_quad, '2f', 'in_vert'),
                 (vbo_walls, '1f 1f 1f 1f 3f/i', 'in_x_offset', 'in_width', 'in_height', 'in_distance', 'in_color')]
            )
        else:
            self.vao_walls.release()
            self.vao_walls = self.ctx.vertex_array(
                self.prog_3d,
                [(self.vbo_quad, '2f', 'in_vert'),
                 (vbo_walls, '1f 1f 1f 1f 3f/i', 'in_x_offset', 'in_width', 'in_height', 'in_distance', 'in_color')]
            )
        return self.vao_walls

    def render_2d_map_gpu(self, num_segments, num_rays, px, py, zoom, center_x, center_y, resolution):
        world_offset_x = center_x - px * zoom
        world_offset_y = center_y - py * zoom
        tile_size_scaled = TILE_SIZE * zoom

        if num_rays > 0:
            self.prog_fov['u_player_pos'].value = (px, py)
            self.prog_fov['u_offset'].value = (world_offset_x, world_offset_y)
            self.prog_fov['u_zoom'].value = float(zoom)
            self.prog_fov['u_resolution'].value = resolution

            step = max(1, num_rays // 60)
            num_vertices_fov = (num_rays // step) + 2
            self.prog_fov['u_step'].value = step
            self.prog_fov['u_max_ray'].value = num_rays

            self.vao_fov.render(moderngl.TRIANGLE_FAN, vertices=num_vertices_fov)

        if num_segments > 0:
            self.ctx.line_width = 2.0
            self.prog_map_lines['u_offset'].value = (world_offset_x, world_offset_y)
            self.prog_map_lines['u_tile_size'].value = float(tile_size_scaled)
            self.prog_map_lines['u_resolution'].value = resolution
            self.vao_map_lines.render(moderngl.LINES, vertices=num_segments * 2)

    def render(self, vbo_walls, num_rays, entity_data, surface_2d, strip_height, num_segments, player_pos, zoom):
        self.ctx.viewport = (0, 0, self.game.width, self.game.height)
        self.ctx.clear(0.0, 0.0, 0.0)

        self.ctx.viewport = (0, 0, self.game.width, int(strip_height))
        self.prog_3d['u_light_intensity'].value = LIGHT_INTENSITY_FACTOR
        self.prog_3d['u_min_brightness'].value = MIN_BRIGHTNESS

        vao_walls = self._create_or_update_vao_walls(vbo_walls, num_rays)
        vao_walls.render(moderngl.TRIANGLE_STRIP, instances=num_rays)

        if entity_data is not None and len(entity_data) > 0:
            buffer_data = np.array(entity_data, dtype='f4')
            if self.vbo_entities_instances is None or self.vbo_entities_instances.size < buffer_data.nbytes:
                if self.vbo_entities_instances: self.vbo_entities_instances.release()
                self.vbo_entities_instances = self.ctx.buffer(reserve=max(buffer_data.nbytes, 10000))

            self.vbo_entities_instances.write(buffer_data.tobytes())
            if self.vao_entities: self.vao_entities.release()

            self.vao_entities = self.ctx.vertex_array(
                self.prog_3d,
                [(self.vbo_quad, '2f', 'in_vert'), (
                self.vbo_entities_instances, '1f 1f 1f 1f 3f/i', 'in_x_offset', 'in_width', 'in_height', 'in_distance',
                'in_color')]
            )
            self.vao_entities.render(moderngl.TRIANGLE_STRIP, instances=len(buffer_data))

        self.ctx.viewport = (0, 0, self.game.width, self.game.height)

        map_view_height = max(1, self.game.height - int(strip_height))
        self.ctx.scissor = (0, int(strip_height), self.game.width, map_view_height)

        center_x = self.game.width // 2
        center_y = map_view_height // 2

        self.render_2d_map_gpu(num_segments, num_rays, player_pos[0], player_pos[1], zoom, center_x, center_y,
                               (self.game.width, self.game.height))

        self.ctx.scissor = None

        self.ctx.viewport = (0, 0, self.game.width, self.game.height)

        self.overlay_update_counter += 1
        should_update = (
                    self.overlay_update_counter >= self.overlay_update_interval or self.game.freeze_entities or not self.texture_2d)

        if should_update:
            self.overlay_update_counter = 0
            if not self.texture_2d or self.texture_2d.size != surface_2d.get_size():
                if self.texture_2d: self.texture_2d.release()
                self.texture_2d = self.ctx.texture(surface_2d.get_size(), 4)
                self.texture_2d.filter = (moderngl.NEAREST, moderngl.NEAREST)

            self.texture_2d.write(surface_2d.get_view('1'))

        if self.texture_2d:
            self.texture_2d.use(0)
            self.prog_2d['u_texture'].value = 0
            self.vao_overlay.render(moderngl.TRIANGLE_STRIP)