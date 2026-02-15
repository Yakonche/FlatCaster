// src/renderer/types.rs
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Render3dUniforms {
    pub light_intensity: f32,
    pub min_brightness: f32,
    pub _pad: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct QuadVertex {
    pub pos: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct WallInstance {
    pub x_offset: f32,
    pub width: f32,
    pub height: f32,
    pub distance: f32,
    pub color: [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct MapLineUniforms {
    pub offset: [f32; 2],
    pub tile_size: f32,
    pub _pad1: f32,
    pub resolution: [f32; 2],
    pub _pad2: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct FovUniforms {
    pub player_pos: [f32; 2],
    pub offset: [f32; 2],
    pub zoom: f32,
    pub _pad1: f32,
    pub resolution: [f32; 2],
    pub step: i32,
    pub max_ray: i32,
}
