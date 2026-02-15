// src/entities/projectile.rs
use crate::config::*;
use crate::map::MapHandler;

pub struct Projectile {
    pub x: f32,
    pub y: f32,
    pub angle: f32,
    pub speed: f32,
    pub radius: f32,
    pub life_time: i32,
    pub marked_for_deletion: bool,
}

impl Projectile {
    pub fn new(x: f32, y: f32, angle: f32) -> Self {
        Self {
            x, y, angle,
            speed: 15.0,
            radius: 4.0,
            life_time: 100,
            marked_for_deletion: false,
        }
    }

    pub fn update(&mut self, map: &mut MapHandler) {
        self.life_time -= 1;
        if self.life_time <= 0 {
            self.marked_for_deletion = true;
            return;
        }

        let dx = self.angle.cos() * self.speed;
        let dy = self.angle.sin() * self.speed;

        if map.get_wall(
            ((self.x + dx) / TILE_SIZE) as i32,
            ((self.y + dy) / TILE_SIZE) as i32,
        ) != 0 {
            self.marked_for_deletion = true;
        } else {
            self.x += dx;
            self.y += dy;
        }
    }
}
