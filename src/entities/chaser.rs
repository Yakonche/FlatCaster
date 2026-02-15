// src/entities/chaser.rs
use std::f32::consts::PI;
use rand::Rng;
use crate::map::MapHandler;
use super::entity_base::EntityBase;

#[derive(Clone, Copy, PartialEq)]
pub enum ChaserState { Idle, Moving }

pub fn update_chaser(
    base: &mut EntityBase,
    state: &mut ChaserState,
    timer: &mut i32,
    target_angle: &mut f32,
    player_pos: (f32, f32),
    map: &mut MapHandler,
    dt: f32,
) {
    let speed = base.speed * dt * 60.0;
    *timer -= 1;
    let dist_p = base.dist_to(player_pos.0, player_pos.1);

    if *timer <= 0 {
        match state {
            ChaserState::Idle => {
                *state = ChaserState::Moving;
                *timer = rand::thread_rng().gen_range(60..=200);
                *target_angle = rand::thread_rng().gen_range(0.0..PI * 2.0);
                base.angle = *target_angle;
            }
            ChaserState::Moving => {
                *state = ChaserState::Idle;
                *timer = rand::thread_rng().gen_range(30..=90);
            }
        }
    }

    match state {
        ChaserState::Moving => {
            let dx = base.angle.cos() * speed;
            let dy = base.angle.sin() * speed;
            if dist_p < 30.0 {
                *state = ChaserState::Idle;
                *timer = 60;
                return;
            }
            if base.check_wall(base.x + dx * 5.0, base.y + dy * 5.0, map) {
                *state = ChaserState::Idle;
                *timer = rand::thread_rng().gen_range(10..=30);
            } else {
                base.move_pos(dx, dy, map);
            }
        }
        ChaserState::Idle => {
            base.angle += 0.02;
        }
    }
}
