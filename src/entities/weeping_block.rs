// src/entities/weeping_block.rs
use std::f32::consts::PI;
use crate::config::*;
use crate::map::MapHandler;
use super::entity_base::EntityBase;

pub fn update_weeping_block(
    base: &mut EntityBase,
    active: &mut bool,
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
    let looking = angle_to.abs() < HALF_FOV;

    if looking {
        base.color = [100.0/255.0, 100.0/255.0, 100.0/255.0];
        *active = false;
    } else {
        base.color = [220.0/255.0, 220.0/255.0, 200.0/255.0];
        *active = true;
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
