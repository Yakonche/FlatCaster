// src/entities/sentinel.rs
use rand::Rng;
use crate::map::MapHandler;
use super::entity_base::EntityBase;

#[derive(Clone, Copy, PartialEq)]
pub enum SentinelState { Patrol, Alert, Search }

pub fn update_sentinel(
    base: &mut EntityBase,
    state: &mut SentinelState,
    last_known_pos: &mut Option<(f32, f32)>,
    search_timer: &mut i32,
    patrol_target: &mut (f32, f32),
    player_pos: (f32, f32),
    map: &mut MapHandler,
    dt: f32,
) {
    let speed = base.speed * dt * 60.0;
    let (px, py) = player_pos;
    let dist_p = base.dist_to(px, py);
    let can_see = dist_p < 400.0;

    if can_see {
        *state = SentinelState::Alert;
        *last_known_pos = Some((px, py));
        base.color = [220.0/255.0, 50.0/255.0, 50.0/255.0];
    } else if *state == SentinelState::Alert {
        *state = SentinelState::Search;
        *search_timer = 180;
        base.color = [200.0/255.0, 140.0/255.0, 20.0/255.0];
    }

    let (tx, ty) = match state {
        SentinelState::Alert => {
            if dist_p < 40.0 { return; }
            (px, py)
        }
        SentinelState::Search => {
            if let Some(lk) = last_known_pos {
                let d = base.dist_to(lk.0, lk.1);
                if d < 20.0 {
                    *search_timer -= 1;
                    base.angle += 0.05;
                    if *search_timer <= 0 {
                        *state = SentinelState::Patrol;
                        base.color = [50.0/255.0, 50.0/255.0, 180.0/255.0];
                        let mut rng = rand::thread_rng();
                        *patrol_target = (
                            base.x + rng.gen_range(-200.0..200.0),
                            base.y + rng.gen_range(-200.0..200.0),
                        );
                    }
                    return;
                }
                (lk.0, lk.1)
            } else {
                return;
            }
        }
        SentinelState::Patrol => {
            if base.dist_to(patrol_target.0, patrol_target.1) < 10.0 {
                let mut rng = rand::thread_rng();
                *patrol_target = (
                    base.x + rng.gen_range(-200.0..200.0),
                    base.y + rng.gen_range(-200.0..200.0),
                );
            }
            (patrol_target.0, patrol_target.1)
        }
    };

    let angle = (ty - base.y).atan2(tx - base.x);
    base.angle = angle;
    base.move_pos(angle.cos() * speed, angle.sin() * speed, map);
}
