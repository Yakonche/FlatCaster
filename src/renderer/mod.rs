// src/renderer/mod.rs
pub mod types;
pub mod pipeline_3d;
pub mod pipeline_map;
pub mod pipeline_overlay;
pub mod draw_2d;
pub mod text_renderer;

pub use types::{WallInstance, MapLineUniforms, FovUniforms};
pub use text_renderer::{HudTextRenderer, TextEntry};

use pipeline_3d::create_pipeline_3d;
use pipeline_map::{create_pipeline_map_lines, create_pipeline_fov};
use pipeline_overlay::create_pipeline_overlay;

pub struct Renderer {
    // 3D wall/entity rendering
    pub render3d_pipeline: wgpu::RenderPipeline,
    pub render3d_uniform_buffer: wgpu::Buffer,
    pub render3d_bind_group: wgpu::BindGroup,
    pub quad_vbo: wgpu::Buffer,

    // Overlay (2D texture blit)
    pub overlay_pipeline: wgpu::RenderPipeline,
    pub overlay_bind_group_layout: wgpu::BindGroupLayout,
    pub overlay_sampler: wgpu::Sampler,
    pub overlay_texture: Option<wgpu::Texture>,
    pub overlay_bind_group: Option<wgpu::BindGroup>,

    // Map lines
    pub map_lines_pipeline: wgpu::RenderPipeline,
    pub map_lines_bind_group_layout: wgpu::BindGroupLayout,
    pub map_lines_uniform_buffer: wgpu::Buffer,
    pub dynamic_lines_buffer: wgpu::Buffer,
    pub dynamic_lines_capacity: usize,
    pub dynamic_bind_group: Option<wgpu::BindGroup>,

    // FOV cone
    pub fov_pipeline: wgpu::RenderPipeline,
    pub fov_bind_group_layout: wgpu::BindGroupLayout,
    pub fov_uniform_buffer: wgpu::Buffer,

    // Instance buffers
    pub wall_instance_buffer: Option<wgpu::Buffer>,
    pub entity_instance_buffer: Option<wgpu::Buffer>,
    wall_instance_capacity: usize,
    entity_instance_capacity: usize,

    // HUD text (TTF via glyphon)
    pub hud_text: HudTextRenderer,
}

impl Renderer {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, surface_format: wgpu::TextureFormat) -> Self {
        let p3d     = create_pipeline_3d(device, surface_format);
        let pmap    = create_pipeline_map_lines(device, surface_format);
        let pfov    = create_pipeline_fov(device, surface_format);
        let poverlay = create_pipeline_overlay(device, surface_format);

        let font_bytes = include_bytes!("../../assets/Jersey10-Regular.ttf");
        let hud_text = HudTextRenderer::new(device, queue, surface_format, font_bytes);

        Self {
            render3d_pipeline: p3d.pipeline,
            render3d_uniform_buffer: p3d.uniform_buffer,
            render3d_bind_group: p3d.bind_group,
            quad_vbo: p3d.quad_vbo,

            overlay_pipeline: poverlay.pipeline,
            overlay_bind_group_layout: poverlay.bind_group_layout,
            overlay_sampler: poverlay.sampler,
            overlay_texture: None,
            overlay_bind_group: None,

            map_lines_pipeline: pmap.pipeline,
            map_lines_bind_group_layout: pmap.bind_group_layout,
            map_lines_uniform_buffer: pmap.uniform_buffer,
            dynamic_lines_buffer: pmap.dynamic_lines_buffer,
            dynamic_lines_capacity: pmap.dynamic_lines_capacity,
            dynamic_bind_group: None,

            fov_pipeline: pfov.pipeline,
            fov_bind_group_layout: pfov.bind_group_layout,
            fov_uniform_buffer: pfov.uniform_buffer,

            wall_instance_buffer: None,
            entity_instance_buffer: None,
            wall_instance_capacity: 0,
            entity_instance_capacity: 0,

            hud_text,
        }
    }

    pub fn ensure_wall_buffer(&mut self, device: &wgpu::Device, count: usize) {
        if count > self.wall_instance_capacity {
            self.wall_instance_capacity = count.max(2048);
            self.wall_instance_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("WallInstances"),
                size: (self.wall_instance_capacity * std::mem::size_of::<WallInstance>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }
    }

    pub fn ensure_entity_buffer(&mut self, device: &wgpu::Device, count: usize) {
        if count > self.entity_instance_capacity {
            self.entity_instance_capacity = count.max(1024);
            self.entity_instance_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("EntityInstances"),
                size: (self.entity_instance_capacity * std::mem::size_of::<WallInstance>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }
    }

    pub fn update_overlay_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32, height: u32,
        rgba_data: &[u8],
    ) {
        let needs_recreate = match &self.overlay_texture {
            Some(t) => t.size().width != width || t.size().height != height,
            None => true,
        };

        if needs_recreate {
            self.overlay_texture = Some(device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Overlay Tex"),
                size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            }));

            let view = self.overlay_texture.as_ref().unwrap().create_view(&Default::default());
            self.overlay_bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Overlay BG"),
                layout: &self.overlay_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&self.overlay_sampler) },
                ],
            }));
        }

        if let Some(tex) = &self.overlay_texture {
            queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: tex,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                rgba_data,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * width),
                    rows_per_image: Some(height),
                },
                wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            );
        }
    }

    pub fn update_dynamic_lines(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, flat_data: &[f32]) {
        let count = flat_data.len() / 5;
        if count == 0 { return; }

        if count > self.dynamic_lines_capacity {
            self.dynamic_lines_capacity = (count * 2).max(4096);
            self.dynamic_lines_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Dynamic Lines"),
                size: (self.dynamic_lines_capacity * 5 * 4) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        queue.write_buffer(&self.dynamic_lines_buffer, 0, bytemuck::cast_slice(flat_data));

        self.dynamic_bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Dynamic Lines BG"),
            layout: &self.map_lines_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.dynamic_lines_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.map_lines_uniform_buffer.as_entire_binding(),
                },
            ],
        }));
    }

    // --- Délégations vers draw_2d ---

    pub fn push_line(data: &mut Vec<f32>, x1: f32, y1: f32, x2: f32, y2: f32, color: u32) {
        draw_2d::push_line(data, x1, y1, x2, y2, color);
    }

    pub fn push_rect(data: &mut Vec<f32>, x: f32, y: f32, w: f32, h: f32, color: u32) {
        draw_2d::push_rect(data, x, y, w, h, color);
    }

    pub fn push_circle(data: &mut Vec<f32>, cx: f32, cy: f32, radius: f32, color: u32) {
        draw_2d::push_circle(data, cx, cy, radius, color);
    }

    pub fn push_entity_shape(data: &mut Vec<f32>, cx: f32, cy: f32, radius: f32, color: u32, shape_id: u8) {
        draw_2d::push_entity_shape(data, cx, cy, radius, color, shape_id);
    }

    pub fn push_tadpole(data: &mut Vec<f32>, cx: f32, cy: f32, radius: f32, angle: f32, tail_phase: f32, color: u32) {
        draw_2d::push_tadpole(data, cx, cy, radius, angle, tail_phase, color);
    }

}
