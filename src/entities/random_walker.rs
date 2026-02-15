// src/entities/random_walker.rs
use std::f32::consts::PI;
use rand::Rng;
use crate::map::MapHandler;
use super::entity_base::EntityBase;

pub fn update_random_walker(
    base: &mut EntityBase,
    change_dir_timer: &mut i32,
    map: &mut MapHandler,
    dt: f32,
) {
    *change_dir_timer += 1;
    let speed = base.speed * dt * 60.0;
    let dx = base.angle.cos() * speed;
    let dy = base.angle.sin() * speed;
    let hit = base.check_wall(base.x + dx * 5.0, base.y + dy * 5.0, map);
    if hit || *change_dir_timer > 120 {
        base.angle = rand::thread_rng().gen_range(0.0..PI * 2.0);
        *change_dir_timer = 0;
    } else {
        base.move_pos(dx, dy, map);
    }
}
