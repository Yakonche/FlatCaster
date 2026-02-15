// src/app/render/map_data.rs
//
// Construction des données 2D dynamiques : joueur, entités, projectiles, shockwaves,
// et ligne séparatrice entre la vue 3D et la vue 2D.

use crate::config::TILE_SIZE;
use crate::renderer::Renderer;
use crate::app::game_state::GameState;
use crate::entities::types::EntityBehavior;

/// Construit le vecteur de données 2D dynamiques (lignes + cercles) pour cette frame.
pub fn build_dynamic_data(game: &GameState) -> Vec<f32> {
    let mut data = Vec::new();
    let tile = TILE_SIZE;
    let (px, py) = game.player.pos();

    // Joueur
    Renderer::push_circle(&mut data, px / tile, py / tile, 0.4, 0);
    Renderer::push_line(&mut data,
        px / tile, py / tile,
        (px + game.player.angle.cos() * 20.0) / tile,
        (py + game.player.angle.sin() * 20.0) / tile,
        0,
    );

    // Entités - forme distincte par type
    for e in &game.entity_manager.entities {
        let ex = e.base.x / tile;
        let ey = e.base.y / tile;
        let er = e.base.size / tile;
        // Tétard : rendu spécial avec corps ovale + flagelle animée
        if let EntityBehavior::Tadpole { state } = &e.behavior {
            Renderer::push_tadpole(&mut data, ex, ey, er, e.base.angle, state.tail_phase, 2);
        } else {
            let col = if e.base.color[0] > 0.8 { 1 } else { 3 };
            Renderer::push_entity_shape(&mut data, ex, ey, er, col, e.shape_id());
        }
    }

    // Projectiles
    for p in &game.entity_manager.projectiles {
        Renderer::push_circle(&mut data, p.x / tile, p.y / tile, p.radius / tile, 5);
    }

    // Shockwaves
    for s in &game.entity_manager.shockwaves {
        Renderer::push_circle(&mut data, s.x / tile, s.y / tile, s.radius / tile, 5);
    }

    data
}

/// Ajoute la ligne séparatrice entre la vue 3D (bas) et la vue 2D (haut).
pub fn push_separator_line(data: &mut Vec<f32>, game: &GameState) {
    let tile = TILE_SIZE;
    let (px, py) = game.player.pos();
    let strip_px_f = game.strip_height as u32 as f32;
    let map_h_f = (game.height as f32 - strip_px_f).max(1.0);
    let cy_world = map_h_f / 2.0;
    let world_offset_y_sep = cy_world - py * game.zoom_level;
    let sep_screen_y = map_h_f;
    let sep_world_y = (sep_screen_y - world_offset_y_sep) / game.zoom_level / tile;

    let world_offset_x_sep = game.width as f32 / 2.0 - px * game.zoom_level;
    let x_left  = (0.0 - world_offset_x_sep) / game.zoom_level / tile;
    let x_right = (game.width as f32 - world_offset_x_sep) / game.zoom_level / tile;

    let thick = if game.hover_separator || game.is_dragging_separator { 3 } else { 1 };
    let step = 1.0 / game.zoom_level / tile;
    for k in 0..thick {
        let offset = (k as f32 - (thick as f32 - 1.0) * 0.5) * step;
        Renderer::push_line(data,
            x_left,  sep_world_y + offset,
            x_right, sep_world_y + offset,
            0,
        );
    }
}
