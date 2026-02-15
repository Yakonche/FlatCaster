// src/app/render/arrow.rs
//
// Rastérisation CPU de la flèche animée du menu pause dans le buffer RGBA de l'overlay.

/// Rastérise la flèche (tronc + triangle animé) dans le buffer RGBA de l'overlay.
///
/// Flèche pointant à droite (→) :
///   Tronc  : rectangle horizontal
///   Triangle : pointe vers la droite, seule la hauteur s'anime
///
/// `tip_x`, `tip_y` : position de la POINTE (à droite, juste avant le texte)
/// `size`            : hauteur totale souhaitée (triangle inclus)
/// `phase`           : 1.0 = triangle plein, 0.0 = triangle aplati (juste un trait)
pub fn draw_arrow(
    rgba: &mut [u8],
    tex_w: u32,
    tex_h: u32,
    tip_x: f32,
    tip_y: f32,
    size: f32,
    phase: f32,
) {
    let total_h = size;
    let tri_half_h = (total_h * 0.5 * phase).max(0.5);
    let tri_w      = total_h * 0.86;
    let trunk_h    = (total_h * 0.29 * phase).max(0.5);
    let trunk_w    = total_h * 0.87;

    let tri_base_x = tip_x - tri_w;
    let tri_top_y  = tip_y - tri_half_h;
    let tri_bot_y  = tip_y + tri_half_h;

    let trunk_x1 = tri_base_x - trunk_w;
    let trunk_x2 = tri_base_x;
    let trunk_y1 = tip_y - trunk_h * 0.5;
    let trunk_y2 = tip_y + trunk_h * 0.5;

    // Couleur jaune doré (identique à col_selected)
    let r: u8 = 255;
    let g: u8 = 220;
    let b: u8 = 60;
    let a: u8 = 255;

    let w = tex_w as f32;
    let h = tex_h as f32;

    // ── Tronc (rectangle) ─────────────────────────────────────────────────
    {
        let x0 = trunk_x1.max(0.0) as i32;
        let y0 = trunk_y1.max(0.0) as i32;
        let x1 = trunk_x2.min(w - 1.0) as i32;
        let y1 = trunk_y2.min(h - 1.0) as i32;
        for row in y0..=y1 {
            for col in x0..=x1 {
                set_pixel(rgba, tex_w, col as u32, row as u32, r, g, b, a);
            }
        }
    }

    // ── Triangle (scan-line de gauche vers droite) ─────────────────────────
    {
        let x_start = tri_base_x.max(0.0) as i32;
        let x_end   = tip_x.min(w - 1.0) as i32;

        for col in x_start..=x_end {
            let cx_f = col as f32 + 0.5;
            let t = if (tip_x - tri_base_x).abs() > 0.001 {
                (cx_f - tri_base_x) / (tip_x - tri_base_x)
            } else {
                1.0
            };
            let y_top_f = tri_top_y + (tip_y - tri_top_y) * t;
            let y_bot_f = tri_bot_y + (tip_y - tri_bot_y) * t;

            let row_top = y_top_f.max(0.0) as i32;
            let row_bot = y_bot_f.min(h - 1.0) as i32;
            for row in row_top..=row_bot {
                set_pixel(rgba, tex_w, col as u32, row as u32, r, g, b, a);
            }
        }
    }
}

#[inline(always)]
pub fn set_pixel(rgba: &mut [u8], tex_w: u32, x: u32, y: u32, r: u8, g: u8, b: u8, a: u8) {
    let idx = ((y * tex_w + x) * 4) as usize;
    if idx + 3 < rgba.len() {
        rgba[idx]     = r;
        rgba[idx + 1] = g;
        rgba[idx + 2] = b;
        rgba[idx + 3] = a;
    }
}
