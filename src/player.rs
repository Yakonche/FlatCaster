// src/player.rs
use crate::config::*;
use crate::map::MapHandler;
use crate::input::InputState;

pub struct Player {
    pub x: f32,
    pub y: f32,
    pub angle: f32,
    pub shoot_cooldown: i32,
    pub shockwave_cooldown: i32,
}

impl Player {
    pub fn new() -> Self {
        Self {
            x: TILE_SIZE * 2.0,
            y: TILE_SIZE * 2.0,
            angle: -std::f32::consts::FRAC_PI_2,
            shoot_cooldown: 0,
            shockwave_cooldown: 0,
        }
    }

    pub fn pos(&self) -> (f32, f32) {
        (self.x, self.y)
    }

    pub fn map_pos(&self) -> (i32, i32) {
        ((self.x / TILE_SIZE) as i32, (self.y / TILE_SIZE) as i32)
    }

    pub fn update(&mut self, input: &InputState, map: &mut MapHandler, dt: f32) -> (bool, bool) {
        let mut dx = 0.0f32;
        let mut dy = 0.0f32;
        // Vitesse en pixels/seconde, normalisée à 60fps pour conserver le feeling
        let mut current_speed = PLAYER_SPEED * dt * 60.0;
        let mut wants_shoot = false;
        let mut wants_shockwave = false;

        if self.shoot_cooldown > 0 { self.shoot_cooldown -= 1; }
        if self.shockwave_cooldown > 0 { self.shockwave_cooldown -= 1; }

        if input.sprint { current_speed *= 2.0; }
        if input.slow { current_speed *= 0.5; }

        if input.forward { dy -= current_speed; }
        if input.backward { dy += current_speed; }
        if input.left { dx -= current_speed; }
        if input.right { dx += current_speed; }

        if input.rot_left { self.angle -= PLAYER_ROT_SPEED * dt * 60.0; }
        if input.rot_right { self.angle += PLAYER_ROT_SPEED * dt * 60.0; }

        // Click gauche : orienter le joueur vers la souris et avancer dans cette direction
        if input.mouse_left_down {
            if let Some(aim) = input.aim_angle {
                self.angle = aim;
                dx += aim.cos() * current_speed;
                dy += aim.sin() * current_speed;
            }
        }

        if input.shoot {
            wants_shoot = self.try_shoot();
        }
        if input.shockwave {
            wants_shockwave = self.try_shockwave();
        }

        // Gamepad analog sticks
        if input.move_x.abs() > 0.15 {
            dx += input.move_x * current_speed;
        }
        if input.move_y.abs() > 0.15 {
            dy += input.move_y * current_speed;
        }

        // Right stick aim
        let rs_mag = (input.look_x * input.look_x + input.look_y * input.look_y).sqrt();
        if rs_mag > 0.5 {
            self.angle = input.look_y.atan2(input.look_x);
        }

        // Collision check
        self.check_collision(dx, dy, map);

        (wants_shoot, wants_shockwave)
    }

    fn try_shoot(&mut self) -> bool {
        if self.shoot_cooldown <= 0 {
            self.shoot_cooldown = 45;
            true
        } else {
            false
        }
    }

    fn try_shockwave(&mut self) -> bool {
        if self.shockwave_cooldown <= 0 {
            self.shockwave_cooldown = 60;
            true
        } else {
            false
        }
    }

    fn check_collision(&mut self, dx: f32, dy: f32, map: &mut MapHandler) {
        let scale = PLAYER_SIZE * 0.5;

        if dx != 0.0 {
            let new_x = self.x + dx;
            if map.is_position_free(new_x, self.y, scale) {
                self.x = new_x;
            }
        }

        if dy != 0.0 {
            let new_y = self.y + dy;
            if map.is_position_free(self.x, new_y, scale) {
                self.y = new_y;
            }
        }
    }
}
