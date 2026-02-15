// src/entities/shockwave.rs

pub struct Shockwave {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub max_radius: f32,
    pub speed: f32,
    pub alpha: f32,
    pub marked_for_deletion: bool,
}

impl Shockwave {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x, y,
            radius: 10.0,
            max_radius: 400.0,
            speed: 30.0,
            alpha: 255.0,
            marked_for_deletion: false,
        }
    }

    pub fn update(&mut self, dt: f32) {
        let t = dt * 60.0; // normalise à 60fps
        self.radius += self.speed * t;
        self.speed *= 0.92f32.powf(t);
        self.alpha -= 8.0 * t;

        if self.alpha <= 0.0 || self.speed < 0.5 {
            self.marked_for_deletion = true;
        }
    }
}
