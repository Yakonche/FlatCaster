// src/entities/ranger.rs
use crate::map::MapHandler;
use super::entity_base::EntityBase;

#[derive(Clone, Copy, PartialEq)]
pub enum RangerState { Positioning, Aiming, Firing }

pub fn update_ranger(
    base: &mut EntityBase,
    state: &mut RangerState,
    charge_timer: &mut i32,
    ideal_range: &f32,
    is_firing: &mut bool,
    fire_frame: &mut i32,
    player_pos: (f32, f32),
    map: &mut MapHandler,
    dt: f32,
) {
    let speed = base.speed * dt * 60.0;
    if *is_firing {
        *fire_frame -= 1;
        if *fire_frame <= 0 {
            *is_firing = false;
            *state = RangerState::Positioning;
        }
        return;
    }

    let (px, py) = player_pos;
    let dist_p = base.dist_to(px, py);

    match state {
        RangerState::Positioning => {
            base.color = [0.0, 200.0/255.0, 200.0/255.0];
            if dist_p < *ideal_range - 50.0 {
                let away = (base.y - py).atan2(base.x - px);
                base.angle = away;
                base.move_pos(away.cos() * speed, away.sin() * speed, map);
            } else if dist_p > *ideal_range + 50.0 {
                let to = (py - base.y).atan2(px - base.x);
                base.angle = to;
                base.move_pos(to.cos() * speed, to.sin() * speed, map);
            } else {
                *state = RangerState::Aiming;
                *charge_timer = 60;
            }
        }
        RangerState::Aiming => {
            *charge_timer -= 1;
            base.angle = (py - base.y).atan2(px - base.x);
            if *charge_timer % 10 < 5 {
                base.color = [1.0, 1.0, 1.0];
            } else {
                base.color = [0.0, 200.0/255.0, 200.0/255.0];
            }
            if *charge_timer <= 0 {
                *state = RangerState::Firing;
                *is_firing = true;
                *fire_frame = 5;
                base.color = [1.0, 50.0/255.0, 50.0/255.0];
            }
        }
        RangerState::Firing => {}
    }
}
