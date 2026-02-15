// src/renderer/pipeline_map.rs
use wgpu;
use wgpu::util::DeviceExt;
use bytemuck;
use crate::config::*;
use super::types::{MapLineUniforms, FovUniforms};

pub struct PipelineMapBundle {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub uniform_buffer: wgpu::Buffer,
    pub dynamic_lines_buffer: wgpu::Buffer,
    pub dynamic_lines_capacity: usize,
}

pub struct PipelineFovBundle {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub uniform_buffer: wgpu::Buffer,
}

pub fn create_pipeline_map_lines(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> PipelineMapBundle {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("MapLines"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/map_lines.wgsl").into()),
    });

    let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("MapLines BGL"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });

    let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("MapLines PL"),
        bind_group_layouts: &[&bgl],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("MapLines Pipeline"),
        layout: Some(&pl),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            compilation_options: Default::default(),
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::LineList,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });

    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("MapLines Uniforms"),
        contents: bytemuck::bytes_of(&MapLineUniforms {
            offset: [0.0; 2],
            tile_size: TILE_SIZE,
            _pad1: 0.0,
            resolution: [RES_WIDTH as f32, RES_HEIGHT as f32],
            _pad2: [0.0; 2],
        }),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let dynamic_lines_capacity = 2048;
    let dynamic_lines_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Dynamic Lines"),
        size: (dynamic_lines_capacity * 5 * 4) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    PipelineMapBundle { pipeline, bind_group_layout: bgl, uniform_buffer, dynamic_lines_buffer, dynamic_lines_capacity }
}

pub fn create_pipeline_fov(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> PipelineFovBundle {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("FOV"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/fov.wgsl").into()),
    });

    let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("FOV BGL"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });

    let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("FOV PL"),
        bind_group_layouts: &[&bgl],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("FOV Pipeline"),
        layout: Some(&pl),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            compilation_options: Default::default(),
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });

    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("FOV Uniforms"),
        contents: bytemuck::bytes_of(&FovUniforms {
            player_pos: [0.0; 2],
            offset: [0.0; 2],
            zoom: 1.0,
            _pad1: 0.0,
            resolution: [RES_WIDTH as f32, RES_HEIGHT as f32],
            step: 1,
            max_ray: 1,
        }),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    PipelineFovBundle { pipeline, bind_group_layout: bgl, uniform_buffer }
}
