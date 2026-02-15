// src/renderer/pipeline_3d.rs
use wgpu;
use wgpu::util::DeviceExt;
use bytemuck;
use crate::config::*;
use super::types::{Render3dUniforms, QuadVertex, WallInstance};

pub struct Pipeline3dBundle {
    pub pipeline: wgpu::RenderPipeline,
    pub uniform_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub quad_vbo: wgpu::Buffer,
}

pub fn create_pipeline_3d(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Pipeline3dBundle {
    let quad_verts = [
        QuadVertex { pos: [0.0, 0.0] },
        QuadVertex { pos: [1.0, 0.0] },
        QuadVertex { pos: [0.0, 1.0] },
        QuadVertex { pos: [1.0, 1.0] },
    ];
    let quad_vbo = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("QuadVBO"),
        contents: bytemuck::cast_slice(&quad_verts),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Render3D"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/render3d.wgsl").into()),
    });

    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Render3D Uniforms"),
        contents: bytemuck::bytes_of(&Render3dUniforms {
            light_intensity: LIGHT_INTENSITY_FACTOR,
            min_brightness: MIN_BRIGHTNESS,
            _pad: [0.0; 2],
        }),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Render3D BGL"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Render3D BG"),
        layout: &bgl,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
    });

    let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render3D PL"),
        bind_group_layouts: &[&bgl],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render3D Pipeline"),
        layout: Some(&pl),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            compilation_options: Default::default(),
            buffers: &[
                wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<QuadVertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x2,
                        offset: 0,
                        shader_location: 0,
                    }],
                },
                wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<WallInstance>() as u64,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &[
                        wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32,   offset: 0,  shader_location: 1 },
                        wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32,   offset: 4,  shader_location: 2 },
                        wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32,   offset: 8,  shader_location: 3 },
                        wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32,   offset: 12, shader_location: 4 },
                        wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x3, offset: 16, shader_location: 5 },
                    ],
                },
            ],
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
            topology: wgpu::PrimitiveTopology::TriangleStrip,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });

    Pipeline3dBundle { pipeline, uniform_buffer, bind_group, quad_vbo }
}
