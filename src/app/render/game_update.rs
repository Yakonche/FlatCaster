// src/app/render/game_update.rs
//
// Mise à jour de la logique de jeu par frame :
// déplacement joueur, projectiles, shockwaves, entités, GC.

use crate::app::game_state::GameState;
use crate::input::InputState;

/// Met à jour toute la logique de jeu pour une frame.
/// Ne doit être appelé que si le jeu n'est PAS en pause.
pub fn update_game(game: &mut GameState, input: &mut InputState, dt: f32) {
    // Calcul de l'angle cible vers la souris (vue 2D du dessus)
    // La vue 2D occupe la partie haute de l'écran (hauteur = game.height - strip_height)
    let map_h = game.height as f32 - game.strip_height;
    let center_x = game.width as f32 / 2.0;
    let center_y = map_h / 2.0;
    let mouse_dx = input.mouse_x as f32 - center_x;
    let mouse_dy = input.mouse_y as f32 - center_y;
    let aim_angle = mouse_dy.atan2(mouse_dx);
    input.aim_angle = Some(aim_angle);

    let (wants_shoot, wants_shockwave) = game.player.update(input, &mut game.map, dt);

    // Push player out of entities
    let (mut px, mut py) = (game.player.x, game.player.y);
    let pr = crate::config::PLAYER_SIZE;
    for e in &game.entity_manager.entities {
        let dx = px - e.base.x;
        let dy = py - e.base.y;
        let dist_sq = dx * dx + dy * dy;
        let min_dist = pr + e.base.size;
        if dist_sq < min_dist * min_dist && dist_sq > 0.0001 {
            let dist = dist_sq.sqrt();
            let overlap = min_dist - dist;
            let nx = dx / dist;
            let ny = dy / dist;
            let new_px = px + nx * overlap;
            let new_py = py + ny * overlap;
            if game.map.is_position_free(new_px, py, pr) { px = new_px; }
            if game.map.is_position_free(px, new_py, pr) { py = new_py; }
        }
    }
    game.player.x = px;
    game.player.y = py;

    if wants_shoot {
        game.entity_manager.add_projectile(
            game.player.x, game.player.y, game.player.angle,
        );
    }
    if wants_shockwave {
        game.entity_manager.apply_shockwave(game.player.x, game.player.y);
    }

    game.entity_manager.update(
        game.player.pos(),
        game.player.angle,
        &mut game.map,
        game.time_ms,
        dt,
    );

    game.gc_timer += 1;
    if game.gc_timer > 60 {
        let (px, py) = game.player.pos();
        game.entity_manager.gc_far_entities(px, py);
        game.gc_timer = 0;
    }
}
