// src/entities/mod.rs
pub mod entity_base;
pub mod random_walker;
pub mod stalker;
pub mod swarmer;
pub mod sentinel;
pub mod weeping_block;
pub mod ranger;
pub mod chaser;
pub mod tadpole;
pub mod types;
pub mod projectile;
pub mod shockwave;

use crate::config::*;
use crate::map::MapHandler;
use types::EntityInstance;
use projectile::Projectile;
use shockwave::Shockwave;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

pub struct EntityManager {
    pub entities: Vec<EntityInstance>,
    pub projectiles: Vec<Projectile>,
    pub shockwaves: Vec<Shockwave>,
    rng: ChaCha8Rng,
}

impl EntityManager {
    pub fn new(seed: &str) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        seed.hash(&mut hasher);
        let s = hasher.finish();

        Self {
            entities: Vec::new(),
            projectiles: Vec::new(),
            shockwaves: Vec::new(),
            rng: ChaCha8Rng::seed_from_u64(s),
        }
    }

    pub fn add_projectile(&mut self, x: f32, y: f32, angle: f32) {
        self.projectiles.push(Projectile::new(x, y, angle));
    }

    pub fn apply_shockwave(&mut self, x: f32, y: f32) {
        self.shockwaves.push(Shockwave::new(x, y));
    }

    pub fn update(&mut self, player_pos: (f32, f32), player_angle: f32, map: &mut MapHandler, time_ms: u32, dt: f32) {
        // Update projectiles
        for p in &mut self.projectiles {
            p.update(map);
        }
        self.projectiles.retain(|p| !p.marked_for_deletion);

        // Update shockwaves
        for s in &mut self.shockwaves {
            s.update(dt);
        }
        self.shockwaves.retain(|s| !s.marked_for_deletion);

        // Spawn logic (simple density check)
        if self.entities.len() < 50 && self.rng.gen_bool(0.05) {
            let angle = self.rng.gen_range(0.0..std::f32::consts::PI * 2.0);
            let dist = self.rng.gen_range(300.0..800.0);
            let ex = player_pos.0 + angle.cos() * dist;
            let ey = player_pos.1 + angle.sin() * dist;
            if map.is_position_free(ex, ey, 10.0) {
                // Randomly choose type
                let roll = self.rng.gen_range(0..6);
                let ent = match roll {
                    0 => EntityInstance::new_random_walker(ex, ey),
                    1 => EntityInstance::new_stalker(ex, ey),
                    2 => EntityInstance::new_swarmer(ex, ey),
                    3 => EntityInstance::new_sentinel(ex, ey),
                    4 => EntityInstance::new_ranger(ex, ey),
                    _ => EntityInstance::new_chaser(ex, ey),
                };
                self.entities.push(ent);
            }
        }

        // --- Collision Logic ---
        let mut indices_to_remove = std::collections::HashSet::new();

        for i in 0..self.entities.len() {
            let (px, py) = (self.entities[i].base.x, self.entities[i].base.y);
            let size = self.entities[i].base.size;

            // Projectile collisions -> KILL
            for proj in &mut self.projectiles {
                if proj.marked_for_deletion { continue; }
                let dx = px - proj.x;
                let dy = py - proj.y;
                let dist = (dx*dx + dy*dy).sqrt();

                if dist < size + proj.radius {
                    proj.marked_for_deletion = true;
                    indices_to_remove.insert(i);
                }
            }

            // Shockwave collisions -> PUSH
            for wave in &self.shockwaves {
                let dx = px - wave.x;
                let dy = py - wave.y;
                let dist = (dx*dx + dy*dy).sqrt();
                if (dist - wave.radius).abs() < 20.0 {
                    let force = 15.0;
                    let angle = dy.atan2(dx);
                    self.entities[i].base.vel_x += angle.cos() * force;
                    self.entities[i].base.vel_y += angle.sin() * force;
                }
            }
        }

        // Remove dead entities
        if !indices_to_remove.is_empty() {
            let mut new_entities = Vec::new();
            for (i, e) in self.entities.drain(..).enumerate() {
                if !indices_to_remove.contains(&i) {
                    new_entities.push(e);
                }
            }
            self.entities = new_entities;
        }

        // Update remaining entities
        for e in &mut self.entities {
            e.update(player_pos, player_angle, map, time_ms, dt);
        }

        // --- Entity-entity separation ---
        // Collect positions and sizes first to avoid borrow issues
        let n = self.entities.len();
        for i in 0..n {
            for j in (i + 1)..n {
                let ax = self.entities[i].base.x;
                let ay = self.entities[i].base.y;
                let ar = self.entities[i].base.size;
                let bx = self.entities[j].base.x;
                let by = self.entities[j].base.y;
                let br = self.entities[j].base.size;

                let dx = bx - ax;
                let dy = by - ay;
                let dist_sq = dx * dx + dy * dy;
                let min_dist = ar + br;

                if dist_sq < min_dist * min_dist && dist_sq > 0.0001 {
                    let dist = dist_sq.sqrt();
                    let overlap = (min_dist - dist) * 0.5;
                    let nx = dx / dist;
                    let ny = dy / dist;

                    let new_ax = ax - nx * overlap;
                    let new_ay = ay - ny * overlap;
                    let new_bx = bx + nx * overlap;
                    let new_by = by + ny * overlap;

                    // Only apply if new position is free
                    if map.is_position_free(new_ax, new_ay, ar) {
                        self.entities[i].base.x = new_ax;
                        self.entities[i].base.y = new_ay;
                    }
                    if map.is_position_free(new_bx, new_by, br) {
                        self.entities[j].base.x = new_bx;
                        self.entities[j].base.y = new_by;
                    }
                }
            }
        }
    }

    pub fn gc_far_entities(&mut self, px: f32, py: f32) {
        self.entities.retain(|e| {
            let dx = e.base.x - px;
            let dy = e.base.y - py;
            (dx*dx + dy*dy) < 2000.0 * 2000.0 // Keep entities within 2000 units
        });
    }

    /// Returns segments representing entities for the raycaster (color_type=100 = entity, invisible as wall).
    pub fn get_ray_segments(&self, player_pos: (f32, f32)) -> Vec<crate::map::structures::Segment> {
        let (_px, _py) = player_pos;
        let mut segments = Vec::new();
        let ts = crate::config::TILE_SIZE;

        let push = |segs: &mut Vec<_>, x0, y0, x1, y1| {
            segs.push((x0, y0, x1, y1, 100u32));
        };

        for e in &self.entities {
            let r = e.base.size / ts;
            let cx = e.base.x / ts;
            let cy = e.base.y / ts;

            match e.shape_id() {
                1 => {
                    // Triangle (Stalker)
                    use std::f32::consts::PI;
                    let a0 = -PI / 2.0;
                    let a1 = a0 + 2.0 * PI / 3.0;
                    let a2 = a1 + 2.0 * PI / 3.0;
                    let (x0, y0) = (cx + a0.cos() * r, cy + a0.sin() * r);
                    let (x1, y1) = (cx + a1.cos() * r, cy + a1.sin() * r);
                    let (x2, y2) = (cx + a2.cos() * r, cy + a2.sin() * r);
                    push(&mut segments, x0, y0, x1, y1);
                    push(&mut segments, x1, y1, x2, y2);
                    push(&mut segments, x2, y2, x0, y0);
                }
                2 => {
                    // Carré (Sentinel / WeepingBlock)
                    let h = r * 0.8;
                    push(&mut segments, cx - h, cy - h, cx + h, cy - h);
                    push(&mut segments, cx + h, cy - h, cx + h, cy + h);
                    push(&mut segments, cx + h, cy + h, cx - h, cy + h);
                    push(&mut segments, cx - h, cy + h, cx - h, cy - h);
                }
                3 => {
                    // Losange (Ranger)
                    push(&mut segments, cx,     cy - r, cx + r, cy    );
                    push(&mut segments, cx + r, cy,     cx,     cy + r);
                    push(&mut segments, cx,     cy + r, cx - r, cy    );
                    push(&mut segments, cx - r, cy,     cx,     cy - r);
                }
                6 => {
                    // Tétard : ellipse du corps + segments de la flagelle
                    if let types::EntityBehavior::Tadpole { state } = &e.behavior {
                        // Corps : ellipse orientée
                        let rx = r;
                        let ry = r * 0.62;
                        let cos_a = e.base.angle.cos();
                        let sin_a = e.base.angle.sin();
                        const N: usize = 12;
                        for i in 0..N {
                            let t0 = (i as f32 / N as f32) * std::f32::consts::TAU;
                            let t1 = ((i + 1) as f32 / N as f32) * std::f32::consts::TAU;
                            let lx0 = rx * t0.cos(); let ly0 = ry * t0.sin();
                            let lx1 = rx * t1.cos(); let ly1 = ry * t1.sin();
                            let wx0 = cx + lx0 * cos_a - ly0 * sin_a;
                            let wy0 = cy + lx0 * sin_a + ly0 * cos_a;
                            let wx1 = cx + lx1 * cos_a - ly1 * sin_a;
                            let wy1 = cy + lx1 * sin_a + ly1 * cos_a;
                            push(&mut segments, wx0, wy0, wx1, wy1);
                        }
                        // Flagelle : segments de la colonne centrale + bords coniques
                        let spine = tadpole::tail_spine(
                            e.base.x, e.base.y,
                            e.base.angle,
                            state.tail_phase,
                            e.base.size,
                        );
                        for i in 1..spine.len() {
                            let (ax, ay, ah) = spine[i - 1];
                            let (bx, by, bh) = spine[i];
                            // axe central
                            push(&mut segments, ax / ts, ay / ts, bx / ts, by / ts);
                            // bords (seulement si épaisseur non nulle)
                            if ah > 0.5 || bh > 0.5 {
                                let cos_a = e.base.angle.cos();
                                let sin_a = e.base.angle.sin();
                                push(&mut segments,
                                    ax / ts - sin_a * ah / ts, ay / ts + cos_a * ah / ts,
                                    bx / ts - sin_a * bh / ts, by / ts + cos_a * bh / ts);
                                push(&mut segments,
                                    ax / ts + sin_a * ah / ts, ay / ts - cos_a * ah / ts,
                                    bx / ts + sin_a * bh / ts, by / ts - cos_a * bh / ts);
                            }
                        }
                    }
                }
                _ => {
                    // Cercle (RandomWalker, Swarmer, projectiles) — polygone à 12 côtés
                    const N: usize = 12;
                    for i in 0..N {
                        let a0 = (i as f32) * std::f32::consts::TAU / N as f32;
                        let a1 = (i as f32 + 1.0) * std::f32::consts::TAU / N as f32;
                        push(&mut segments, cx + r * a0.cos(), cy + r * a0.sin(),
                                            cx + r * a1.cos(), cy + r * a1.sin());
                    }
                }
            }
        }

        for p in &self.projectiles {
            let r = p.radius / ts;
            let cx = p.x / ts;
            let cy = p.y / ts;
            const N: usize = 12;
            for i in 0..N {
                let a0 = (i as f32) * std::f32::consts::TAU / N as f32;
                let a1 = (i as f32 + 1.0) * std::f32::consts::TAU / N as f32;
                push(&mut segments, cx + r * a0.cos(), cy + r * a0.sin(),
                                    cx + r * a1.cos(), cy + r * a1.sin());
            }
        }

        segments
    }

    pub fn get_render_data(
        &self,
        z_buffer: &[f32],
        player_pos: (f32, f32),
        player_angle: f32,
        num_rays: usize,
        screen_width: u32,
        screen_dist: f32
    ) -> Vec<[f32; 7]> {
        let mut data = Vec::new();
        let (px, py) = player_pos;

        // Collect all renderable objects
        let mut render_list = Vec::new();
        for e in &self.entities {
            render_list.push((e.base.x, e.base.y, e.base.color, e.base.size));
        }
        // Projectiles
        for p in &self.projectiles {
            render_list.push((p.x, p.y, [1.0, 1.0, 0.5], p.radius));
        }

        // Sort by distance (far to near)
        render_list.sort_by(|a, b| {
            let da = (a.0 - px).powi(2) + (a.1 - py).powi(2);
            let db = (b.0 - px).powi(2) + (b.1 - py).powi(2);
            db.partial_cmp(&da).unwrap()
        });

        for (ex, ey, color, size) in render_list {
            let dx = ex - px;
            let dy = ey - py;
            let dist = (dx*dx + dy*dy).sqrt();

            let sprite_angle = dy.atan2(dx) - player_angle;
            let mut angle_diff = sprite_angle;
            while angle_diff > std::f32::consts::PI { angle_diff -= 2.0 * std::f32::consts::PI; }
            while angle_diff < -std::f32::consts::PI { angle_diff += 2.0 * std::f32::consts::PI; }

            // Check if in front
            if angle_diff.abs() > HALF_FOV + 0.5 { continue; }

            let delta_angle = FOV / num_rays as f32;
            let ray_idx = (angle_diff + HALF_FOV) / delta_angle;

            let screen_x_pos = ray_idx * (screen_width as f32 / num_rays as f32);

            let perp_dist = dist * angle_diff.cos();
            if perp_dist < 0.1 { continue; }

            // CORRECTION: Pour la "vue 1D", on veut que les entités ressemblent à des lignes/murs.
            // Au lieu d'avoir une hauteur perspective qui en fait des carrés, on force la hauteur au max.
            // Ainsi, elles apparaîtront comme des colonnes verticales colorées.
            let sprite_width_persp = (size / perp_dist) * screen_dist * 2.0;

            // Hauteur forcée à 2.0 (toute la hauteur NDC) pour faire "ligne"
            let gl_h = 2.0;

            // Largeur toujours en perspective
            let gl_w = (sprite_width_persp / screen_width as f32) * 2.0;

            // Convert x to Normalized Device Coordinates (-1.0 to 1.0)
            let gl_x = (screen_x_pos / screen_width as f32) * 2.0 - 1.0;

            // Simple occlusion check
            let center_ray = ray_idx as usize;
            if center_ray < num_rays {
                let wall_dist = z_buffer[center_ray];
                // Si l'entité est derrière un mur, on ne l'affiche pas
                // Note: comme les entités sont maintenant ajoutées au raycaster,
                // elles devraient s'auto-occulter si elles sont traitées comme des murs,
                // mais ce z_buffer check reste utile pour les murs statiques vs sprites.
                if wall_dist < perp_dist {
                    continue;
                }
            }

            data.push([
                gl_x - gl_w / 2.0,
                gl_w,
                gl_h,
                perp_dist,
                color[0], color[1], color[2]
            ]);
        }

        data
    }
}