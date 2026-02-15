// src/config.rs
use std::f32::consts::PI;

pub const RES_WIDTH: u32 = 1900;
pub const RES_HEIGHT: u32 = 1000;

pub const DEFAULT_STRIP_HEIGHT: f32 = 50.0;
pub const TILE_SIZE: f32 = 50.0;
pub const FOV: f32 = PI / 3.0;
pub const HALF_FOV: f32 = FOV / 2.0;
pub const SCALE: u32 = 1;
pub const PLAYER_SPEED: f32 = 7.0;
pub const PLAYER_ROT_SPEED: f32 = 0.05;
pub const PLAYER_SIZE: f32 = 10.0;
pub const ENTITY_SIZE: f32 = 15.0;
pub const LIGHT_INTENSITY_FACTOR: f32 = 1500.0;
pub const MIN_BRIGHTNESS: f32 = 0.05;

pub const CHUNK_SIZE: i32 = 32;

// --- Colors Palette (Expanded for variety) ---
pub const WHITE: [f32; 3] = [1.0, 1.0, 1.0];
pub const RED: [f32; 3] = [0.8, 0.2, 0.2];
pub const GREEN: [f32; 3] = [0.2, 0.8, 0.2];
pub const BLUE: [f32; 3] = [0.2, 0.2, 0.8];
pub const PURPLE: [f32; 3] = [0.6, 0.2, 0.6];
pub const ORANGE: [f32; 3] = [0.8, 0.4, 0.1];
pub const CYAN: [f32; 3] = [0.2, 0.8, 0.8];
pub const YELLOW: [f32; 3] = [0.8, 0.8, 0.2];
pub const PINK: [f32; 3] = [0.8, 0.4, 0.6];
pub const LIME: [f32; 3] = [0.6, 0.8, 0.2];
pub const TEAL: [f32; 3] = [0.0, 0.5, 0.5];
pub const MAGENTA: [f32; 3] = [0.9, 0.1, 0.5];

pub const ENTITY_BLUE: [f32; 3] = [0.0, 1.0, 1.0];
pub const ENTITY_RED: [f32; 3] = [1.0, 0.2, 0.0];

pub fn wall_color(wall_type: u32) -> [f32; 3] {
    // Map u32 ID to a specific color
    match wall_type % 12 {
        0 => WHITE,
        1 => RED,
        2 => GREEN,
        3 => BLUE,
        4 => PURPLE,
        5 => ORANGE,
        6 => CYAN,
        7 => YELLOW,
        8 => PINK,
        9 => LIME,
        10 => TEAL,
        11 => MAGENTA,
        _ => WHITE,
    }
}

pub fn generate_random_seed() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let len = rng.gen_range(1..=256);
    let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
        .chars()
        .collect();
    (0..len).map(|_| chars[rng.gen_range(0..chars.len())]).collect()
}