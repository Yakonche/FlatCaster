// src/map/mod.rs
pub mod structures;

use std::collections::HashMap;
use rand::SeedableRng;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use crate::config::*;
use structures::*;

pub struct ChunkData {
    pub segments: Vec<Segment>,
    pub bboxes: Vec<BBox>,
    pub chunk_center: (f32, f32), // Pour les animations locales si besoin
    pub maze_type: u32, // 0=Static, 1=RotatingCircle
}

pub struct MapHandler {
    pub world: HashMap<(i32, i32), ChunkData>,
    pub current_seed: String,
    cached_segments: Vec<Segment>,
    last_cx: Option<i32>,
    last_cy: Option<i32>,
    last_radius: i32,
    pub global_time: f32, // Pour animation
}

impl MapHandler {
    pub fn new(seed: &str) -> Self {
        Self {
            world: HashMap::new(),
            current_seed: seed.to_string(),
            cached_segments: Vec::new(),
            last_cx: None,
            last_cy: None,
            last_radius: -1,
            global_time: 0.0,
        }
    }

    pub fn reset(&mut self, new_seed: &str) {
        self.world.clear();
        self.current_seed = new_seed.to_string();
        self.last_cx = None;
        self.last_cy = None;
        self.last_radius = -1;
        self.cached_segments.clear();
        self.global_time = 0.0;
    }

    fn seed_for_chunk(&self, cx: i32, cy: i32) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        format!("{}_{}_{}", self.current_seed, cx, cy).hash(&mut hasher);
        hasher.finish()
    }

    pub fn generate_chunk(&mut self, cx: i32, cy: i32) {
        if self.world.contains_key(&(cx, cy)) {
            return;
        }

        let mut segments = Vec::new();
        let mut bboxes = Vec::new();

        let seed = self.seed_for_chunk(cx, cy);
        let mut rng = ChaCha8Rng::seed_from_u64(seed);

        let start_x = (cx * CHUNK_SIZE) as f32;
        let start_y = (cy * CHUNK_SIZE) as f32;
        let chunk_center_x = start_x + CHUNK_SIZE as f32 / 2.0;
        let chunk_center_y = start_y + CHUNK_SIZE as f32 / 2.0;

        let mut maze_type = 0; // Default static

        // Generate 1 big maze structure per chunk or multiple small ones
        let structure_type = rng.gen_range(0..10);
        let color_id = rng.gen_range(1..12);

        if structure_type < 4 {
            // --- RECTANGULAR MAZE ---
            let w = rng.gen_range(10..=25);
            let h = rng.gen_range(10..=25);
            let algo_choice = rng.gen_range(0..3);
            let algo = match algo_choice {
                0 => MazeAlgo::RecursiveBacktracker,
                1 => MazeAlgo::Prim,
                _ => MazeAlgo::Kruskal,
            };
            generate_rectangular_maze(
                &mut segments, &mut bboxes,
                start_x + 2.0, start_y + 2.0,
                w, h, color_id, algo, &mut rng
            );
        } else if structure_type < 7 {
            // --- CIRCULAR MAZE (MOVING!) ---
            let num_rings = rng.gen_range(3..8);
            let sectors = rng.gen_range(8..20);
            generate_circular_maze(
                &mut segments, &mut bboxes,
                chunk_center_x, chunk_center_y,
                num_rings, sectors, 2.0, // radius step
                color_id, &mut rng
            );
            maze_type = 1; // Mark for animation
        } else if structure_type < 9 {
            // --- HEXAGONAL MAZE ---
            let radius = rng.gen_range(3..7);
            generate_hex_maze(
                &mut segments, &mut bboxes,
                chunk_center_x, chunk_center_y,
                radius, 1.5,
                color_id, &mut rng
            );
        } else {
            // --- CHAOS (Scatter blocks) ---
            for _ in 0..10 {
                let px = rng.gen_range((start_x + 2.0)..(start_x + CHUNK_SIZE as f32 - 5.0));
                let py = rng.gen_range((start_y + 2.0)..(start_y + CHUNK_SIZE as f32 - 5.0));
                let w = rng.gen_range(2.0..5.0);
                let h = rng.gen_range(2.0..5.0);
                generate_solid_rect(&mut segments, &mut bboxes, px, py, w, h, color_id);
            }
        }

        // Clear spawn area for chunk (0,0)
        if cx == 0 && cy == 0 {
            segments.retain(|&(x1, y1, x2, y2, _)| {
                x1.max(x2) < -5.0 || x1.min(x2) > 6.0 || y1.max(y2) < -5.0 || y1.min(y2) > 6.0
            });
        }

        self.world.insert((cx, cy), ChunkData {
            segments,
            bboxes,
            chunk_center: (chunk_center_x, chunk_center_y),
            maze_type
        });
    }

    /// Appelle cette fonction à chaque frame avec le temps écoulé (en secondes)
    /// pour animer les murs qui doivent bouger.
    pub fn update_animations(&mut self, dt: f32) {
        self.global_time += dt;
        // Optimization: In a real engine, we would use a vertex shader for this.
        // Here we just invalidate the cache to force a rebuild of active segments if we are near a moving maze.
        // Since we modify segments on the fly in `get_active_segments`, we just need to update time.
    }

    pub fn get_active_segments(&mut self, player_x: f32, player_y: f32, radius_chunks: i32) -> &[Segment] {
        let cx = ((player_x / TILE_SIZE) / CHUNK_SIZE as f32).floor() as i32;
        let cy = ((player_y / TILE_SIZE) / CHUNK_SIZE as f32).floor() as i32;

        // Force regenerate if we have moving parts (simplified check: always regen active list if time passes)
        // Ideally we check if active chunks contain type 1.

        let mut active = Vec::new();
        for dy in -radius_chunks..=radius_chunks {
            for dx in -radius_chunks..=radius_chunks {
                let key = (cx + dx, cy + dy);
                self.generate_chunk(key.0, key.1);

                if let Some(chunk) = self.world.get(&key) {
                    if chunk.maze_type == 1 {
                        // Apply rotation on the fly for circular mazes
                        let (ccx, ccy) = chunk.chunk_center;
                        let angle = self.global_time * 0.5; // Rotate speed
                        let cos_a = angle.cos();
                        let sin_a = angle.sin();

                        for &(x1, y1, x2, y2, c) in &chunk.segments {
                            // Rotate around chunk center
                            let rx1 = x1 - ccx; let ry1 = y1 - ccy;
                            let rx2 = x2 - ccx; let ry2 = y2 - ccy;

                            let nx1 = rx1 * cos_a - ry1 * sin_a + ccx;
                            let ny1 = rx1 * sin_a + ry1 * cos_a + ccy;
                            let nx2 = rx2 * cos_a - ry2 * sin_a + ccx;
                            let ny2 = rx2 * sin_a + ry2 * cos_a + ccy;

                            active.push((nx1, ny1, nx2, ny2, c));
                        }
                    } else {
                        active.extend_from_slice(&chunk.segments);
                    }
                }
            }
        }

        self.last_cx = Some(cx);
        self.last_cy = Some(cy);
        self.last_radius = radius_chunks;
        self.cached_segments = active;
        &self.cached_segments
    }

    pub fn is_position_free(&mut self, x: f32, y: f32, radius: f32) -> bool {
        let tile_x = x / TILE_SIZE;
        let tile_y = y / TILE_SIZE;
        let r_tile = radius / TILE_SIZE;

        // Note: Collision is checked against ANIMATED segments because we call get_active_segments inside
        // However, standard get_active_segments might be cached.
        // For collision, we need to ensure we use the latest animated positions.
        let segments = self.cached_segments.clone(); // Using the cached (and animated) frame segments

        for (x1, y1, x2, y2, _) in segments {
            if dist_to_segment(tile_x, tile_y, x1, y1, x2, y2) < r_tile {
                return false;
            }
        }
        true
    }

    pub fn get_wall(&mut self, _tx: i32, _ty: i32) -> u32 {
        // Simplified picking
        0
    }
}

fn dist_to_segment(px: f32, py: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let l2 = (x2 - x1) * (x2 - x1) + (y2 - y1) * (y2 - y1);
    if l2 == 0.0 {
        return ((px - x1) * (px - x1) + (py - y1) * (py - y1)).sqrt();
    }
    let t = (((px - x1) * (x2 - x1) + (py - y1) * (y2 - y1)) / l2).clamp(0.0, 1.0);
    let proj_x = x1 + t * (x2 - x1);
    let proj_y = y1 + t * (y2 - y1);
    ((px - proj_x) * (px - proj_x) + (py - proj_y) * (py - proj_y)).sqrt()
}