// src/entities/entity_base.rs
use crate::config::*;
use crate::map::MapHandler;

#[derive(Clone)]
pub struct EntityBase {
    pub x: f32,
    pub y: f32,
    pub angle: f32,
    pub speed: f32,
    pub size: f32,
    pub color: [f32; 3],
    pub vel_x: f32,
    pub vel_y: f32,
    pub friction: f32,
}

impl EntityBase {
    pub fn new(x: f32, y: f32, color: [f32; 3], speed: f32, size: f32) -> Self {
        let mut rng = rand::thread_rng();
        use rand::Rng;
        use std::f32::consts::PI;
        Self {
            x, y,
            angle: rng.gen_range(0.0..PI * 2.0),
            speed,
            size,
            color,
            vel_x: 0.0,
            vel_y: 0.0,
            friction: 0.85,
        }
    }

    pub fn dist_to(&self, tx: f32, ty: f32) -> f32 {
        ((self.x - tx).powi(2) + (self.y - ty).powi(2)).sqrt()
    }

    pub fn check_wall(&self, x: f32, y: f32, map: &mut MapHandler) -> bool {
        !map.is_position_free(x, y, self.size)
    }

    pub fn move_pos(&mut self, dx: f32, dy: f32, map: &mut MapHandler) {
        let new_x = self.x + dx;
        if map.is_position_free(new_x, self.y, self.size) {
            self.x = new_x;
        }
        let new_y = self.y + dy;
        if map.is_position_free(self.x, new_y, self.size) {
            self.y = new_y;
        }
    }

    pub fn apply_velocity(&mut self, map: &mut MapHandler, dt: f32) {
        if self.vel_x.abs() > 0.1 || self.vel_y.abs() > 0.1 {
            self.move_pos(self.vel_x * dt * 60.0, self.vel_y * dt * 60.0, map);
            let friction_dt = self.friction.powf(dt * 60.0);
            self.vel_x *= friction_dt;
            self.vel_y *= friction_dt;
        } else {
            self.vel_x = 0.0;
            self.vel_y = 0.0;
        }
    }
}
