// src/entities/stalker.rs
use std::f32::consts::PI;
use crate::config::*;
use crate::map::MapHandler;
use super::entity_base::EntityBase;

pub fn update_stalker(
    base: &mut EntityBase,
    player_pos: (f32, f32),
    player_angle: f32,
    map: &mut MapHandler,
    dt: f32,
) {
    let speed = base.speed * dt * 60.0;
    let (px, py) = player_pos;
    let dx = base.x - px;
    let dy = base.y - py;
    let dist = (dx * dx + dy * dy).sqrt();
    let mut angle_to = dy.atan2(dx) - player_angle;
    while angle_to > PI { angle_to -= 2.0 * PI; }
    while angle_to < -PI { angle_to += 2.0 * PI; }
    let is_seen = angle_to.abs() < HALF_FOV && dist < 600.0;

    if is_seen {
        base.color = [120.0/255.0, 50.0/255.0, 120.0/255.0];
        if dist < 200.0 {
            base.move_pos(
                (angle_to + player_angle).cos() * speed,
                (angle_to + player_angle).sin() * speed,
                map,
            );
        }
    } else {
        base.color = [80.0/255.0, 0.0, 80.0/255.0];
        if dist > 30.0 {
            let target = (py - base.y).atan2(px - base.x);
            base.angle = target;
            base.move_pos(
                base.angle.cos() * speed,
                base.angle.sin() * speed,
                map,
            );
        }
    }
}
