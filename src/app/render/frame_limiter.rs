// src/app/render/frame_limiter.rs
//
// Throttle FPS : limite le nombre d'images par seconde.
// Utilise un sleep grossier puis un spinlock pour la précision finale.

use std::time::{Duration, Instant};
use crate::app::pause_menu::fps_millis_to_frame_duration;
use crate::app::game_state::GameState;

/// Applique le throttle FPS si activé. Bloque jusqu'à ce que le budget de frame soit écoulé.
pub fn throttle_fps(game: &mut GameState) {
    if game.fps_millis > 0 && game.fps_enabled {
        if let Some(frame_budget) = fps_millis_to_frame_duration(game.fps_millis) {
            let elapsed = game.last_throttle_time.elapsed();
            if elapsed < frame_budget {
                let remaining = frame_budget - elapsed;
                if remaining > Duration::from_millis(2) {
                    std::thread::sleep(remaining - Duration::from_millis(1));
                }
                while game.last_throttle_time.elapsed() < frame_budget {}
            }
        }
    }
    game.last_throttle_time = Instant::now();
}
