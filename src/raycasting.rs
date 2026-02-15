// src/raycasting.rs
use wgpu;
use bytemuck::{Pod, Zeroable};
use crate::config::*;
use crate::map::MapHandler;
use crate::map::structures::Segment;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct RaycastParams {
    pub player_tile_pos: [f32; 2],
    pub player_angle: f32,
    pub num_segments: i32,
    pub num_rays: i32,
    pub fov: f32,
    pub half_fov: f32,
    pub tile_size: f32,
    pub screen_width: f32,
    pub scale: f32,
    pub _pad: [f32; 2],
}

pub struct RayCaster {
    pub num_rays: usize,
    pub screen_dist: f32,
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    pub segment_buffer: wgpu::Buffer,
    pub output_buffer: wgpu::Buffer,
    pub hit_buffer: wgpu::Buffer,
    param_buffer: wgpu::Buffer,
    pub readback_buffer: wgpu::Buffer,
    pub hit_readback_buffer: wgpu::Buffer,
    bind_group: Option<wgpu::BindGroup>,
    last_chunk: Option<(i32, i32)>,
    last_radius: i32,
    pub num_segments: usize,
    segment_capacity: usize,
}

impl RayCaster {
    pub fn new(device: &wgpu::Device, screen_width: u32) -> Self {
        let num_rays = (screen_width / SCALE) as usize;
        let num_rays = num_rays.max(1);
        let screen_dist = (screen_width as f32 / 2.0) / HALF_FOV.tan();

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Raycast Compute"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/raycast.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Raycast BGL"),
            entries: &[
                // segment_data
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // render_data
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // hit_data
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // params
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Raycast PL"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Raycast Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "main",
            compilation_options: Default::default(),
            cache: None,
        });

        let initial_seg_cap = 100000;
        let segment_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Segments"),
            size: (initial_seg_cap * 5 * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("RayOutput"),
            size: (num_rays * 7 * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let hit_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("HitData"),
            size: (num_rays * 4 * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let param_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("RayParams"),
            size: std::mem::size_of::<RaycastParams>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let readback_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Readback"),
            size: (num_rays * 7 * 4) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let hit_readback_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("HitReadback"),
            size: (num_rays * 4 * 4) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        Self {
            num_rays,
            screen_dist,
            pipeline,
            bind_group_layout,
            segment_buffer,
            output_buffer,
            hit_buffer,
            param_buffer,
            readback_buffer,
            hit_readback_buffer,
            bind_group: None,
            last_chunk: None,
            last_radius: -1,
            num_segments: 0,
            segment_capacity: initial_seg_cap,
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, screen_width: u32) {
        self.num_rays = (screen_width / SCALE).max(1) as usize;
        self.screen_dist = (screen_width as f32 / 2.0) / HALF_FOV.tan();

        self.output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("RayOutput"),
            size: (self.num_rays * 7 * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        self.hit_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("HitData"),
            size: (self.num_rays * 4 * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        self.readback_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Readback"),
            size: (self.num_rays * 7 * 4) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        self.hit_readback_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("HitReadback"),
            size: (self.num_rays * 4 * 4) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        self.bind_group = None;
        self.last_chunk = None;
    }

    /// Invalide le cache de chunk pour forcer la mise à jour du GPU au prochain frame.
    pub fn invalidate_cache(&mut self) {
        self.last_chunk = None;
    }

    pub fn update_map(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        map: &mut MapHandler,
        extra_segments: &[Segment], // NOUVEAU: Segments dynamiques (ex: entités)
        px: f32, py: f32,
        zoom: f32,
        screen_w: u32, screen_h: u32,
    ) {
        let cx = ((px / TILE_SIZE) / CHUNK_SIZE as f32).floor() as i32;
        let cy = ((py / TILE_SIZE) / CHUNK_SIZE as f32).floor() as i32;

        let max_dim = screen_w.max(screen_h) as f32;
        let tiles_across = max_dim / (TILE_SIZE * zoom);
        let chunks_across = tiles_across / CHUNK_SIZE as f32;
        let dynamic_radius = ((chunks_across / 2.0) as i32 + 1).clamp(2, 10);

        // On force la mise à jour si on a des segments dynamiques car ils bougent à chaque frame
        let must_update = !extra_segments.is_empty();

        if !must_update && self.last_chunk == Some((cx, cy)) && self.last_radius == dynamic_radius && self.bind_group.is_some() {
            return;
        }

        self.last_chunk = Some((cx, cy));
        self.last_radius = dynamic_radius;

        // Combiner les murs de la carte ET les segments dynamiques (entités)
        let map_segments = map.get_active_segments(px, py, dynamic_radius);
        let mut segments: Vec<Segment> = Vec::with_capacity(map_segments.len() + extra_segments.len());
        segments.extend_from_slice(map_segments);
        segments.extend_from_slice(extra_segments);

        self.num_segments = segments.len();

        if self.num_segments == 0 {
            return;
        }

        let flat: Vec<f32> = segments.iter()
            .flat_map(|&(x1, y1, x2, y2, c)| vec![x1, y1, x2, y2, c as f32])
            .collect();

        if self.num_segments > self.segment_capacity {
            self.segment_capacity = self.num_segments * 2;
            self.segment_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Segments"),
                size: (self.segment_capacity * 5 * 4) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        queue.write_buffer(&self.segment_buffer, 0, bytemuck::cast_slice(&flat));

        self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Raycast BG"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: self.segment_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: self.output_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: self.hit_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: self.param_buffer.as_entire_binding() },
            ],
        }));
    }

    pub fn cast(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
        px: f32, py: f32, angle: f32,
        screen_width: u32,
    ) {
        if self.bind_group.is_none() || self.num_segments == 0 { return; }

        let params = RaycastParams {
            player_tile_pos: [px / TILE_SIZE, py / TILE_SIZE],
            player_angle: angle,
            num_segments: self.num_segments as i32,
            num_rays: self.num_rays as i32,
            fov: FOV,
            half_fov: HALF_FOV,
            tile_size: TILE_SIZE,
            screen_width: screen_width as f32,
            scale: SCALE as f32,
            _pad: [0.0; 2],
        };

        queue.write_buffer(&self.param_buffer, 0, bytemuck::bytes_of(&params));

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Raycast"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, self.bind_group.as_ref().unwrap(), &[]);
            let groups = (self.num_rays as u32 + 63) / 64;
            pass.dispatch_workgroups(groups, 1, 1);
        }

        // Copy for readback
        encoder.copy_buffer_to_buffer(
            &self.output_buffer, 0,
            &self.readback_buffer, 0,
            (self.num_rays * 7 * 4) as u64,
        );
        encoder.copy_buffer_to_buffer(
            &self.hit_buffer, 0,
            &self.hit_readback_buffer, 0,
            (self.num_rays * 4 * 4) as u64,
        );
    }

    pub fn read_z_buffer(&self, device: &wgpu::Device) -> Vec<f32> {
        let slice = self.hit_readback_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });
        device.poll(wgpu::Maintain::Wait);
        rx.recv().unwrap().unwrap();

        let data = slice.get_mapped_range();
        let floats: &[f32] = bytemuck::cast_slice(&data);
        let z_buffer: Vec<f32> = (0..self.num_rays)
            .map(|i| floats[i * 4]) // z_value is the first component of vec4
            .collect();
        drop(data);
        self.hit_readback_buffer.unmap();
        z_buffer
    }
}