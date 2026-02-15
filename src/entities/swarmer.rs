// src/entities/swarmer.rs
use std::f32::consts::PI;
use rand::Rng;
use crate::map::MapHandler;
use super::entity_base::EntityBase;

#[derive(Clone, Copy, PartialEq)]
pub enum SwarmerState { Passive, Aggressive }

pub fn update_swarmer(
    base: &mut EntityBase,
    state: &mut SwarmerState,
    wander_timer: &mut i32,
    player_pos: (f32, f32),
    map: &mut MapHandler,
    dt: f32,
) {
    let speed = base.speed * dt * 60.0;
    let (px, py) = player_pos;
    let dist_p = base.dist_to(px, py);
    match state {
        SwarmerState::Aggressive => {
            base.color = [1.0, 100.0/255.0, 50.0/255.0];
            if dist_p > 25.0 {
                let target = (py - base.y).atan2(px - base.x);
                let mut diff = target - base.angle;
                while diff > PI { diff -= 2.0 * PI; }
                while diff < -PI { diff += 2.0 * PI; }
                base.angle += diff * 0.1;
                base.move_pos(
                    base.angle.cos() * speed,
                    base.angle.sin() * speed,
                    map,
                );
            }
        }
        SwarmerState::Passive => {
            base.color = [200.0/255.0, 200.0/255.0, 50.0/255.0];
            if dist_p < 200.0 {
                let away = (base.y - py).atan2(base.x - px);
                base.angle = away;
                base.move_pos(
                    base.angle.cos() * speed,
                    base.angle.sin() * speed,
                    map,
                );
            } else {
                *wander_timer -= 1;
                if *wander_timer <= 0 {
                    base.angle = rand::thread_rng().gen_range(0.0..PI * 2.0);
                    *wander_timer = rand::thread_rng().gen_range(30..=100);
                }
                base.move_pos(
                    base.angle.cos() * speed * 0.5,
                    base.angle.sin() * speed * 0.5,
                    map,
                );
            }
        }
    }
}
