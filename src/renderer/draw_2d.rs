// src/renderer/draw_2d.rs

pub fn push_line(data: &mut Vec<f32>, x1: f32, y1: f32, x2: f32, y2: f32, color: u32) {
    data.extend_from_slice(&[x1, y1, x2, y2, color as f32]);
}

pub fn push_rect(data: &mut Vec<f32>, x: f32, y: f32, w: f32, h: f32, color: u32) {
    let step = 1.0;
    let mut cy = y;
    while cy < y + h {
        push_line(data, x, cy, x + w, cy, color);
        cy += step;
    }
}

pub fn push_circle(data: &mut Vec<f32>, cx: f32, cy: f32, radius: f32, color: u32) {
    let segs = 12;
    for i in 0..segs {
        let a1 = (i as f32 / segs as f32) * std::f32::consts::PI * 2.0;
        let a2 = ((i + 1) as f32 / segs as f32) * std::f32::consts::PI * 2.0;
        push_line(data,
            cx + a1.cos() * radius, cy + a1.sin() * radius,
            cx + a2.cos() * radius, cy + a2.sin() * radius,
            color,
        );
    }
}

/// Triangle pointant vers le haut (Stalker)
pub fn push_triangle(data: &mut Vec<f32>, cx: f32, cy: f32, radius: f32, color: u32) {
    use std::f32::consts::PI;
    let a0 = -PI / 2.0;
    let a1 = a0 + 2.0 * PI / 3.0;
    let a2 = a1 + 2.0 * PI / 3.0;
    let (x0, y0) = (cx + a0.cos() * radius, cy + a0.sin() * radius);
    let (x1, y1) = (cx + a1.cos() * radius, cy + a1.sin() * radius);
    let (x2, y2) = (cx + a2.cos() * radius, cy + a2.sin() * radius);
    push_line(data, x0, y0, x1, y1, color);
    push_line(data, x1, y1, x2, y2, color);
    push_line(data, x2, y2, x0, y0, color);
}

/// Carré centré (Sentinel / WeepingBlock)
pub fn push_square(data: &mut Vec<f32>, cx: f32, cy: f32, radius: f32, color: u32) {
    let h = radius * 0.8;
    push_line(data, cx - h, cy - h, cx + h, cy - h, color);
    push_line(data, cx + h, cy - h, cx + h, cy + h, color);
    push_line(data, cx + h, cy + h, cx - h, cy + h, color);
    push_line(data, cx - h, cy + h, cx - h, cy - h, color);
}

/// Losange (Ranger)
pub fn push_diamond(data: &mut Vec<f32>, cx: f32, cy: f32, radius: f32, color: u32) {
    push_line(data, cx, cy - radius, cx + radius, cy, color);
    push_line(data, cx + radius, cy, cx, cy + radius, color);
    push_line(data, cx, cy + radius, cx - radius, cy, color);
    push_line(data, cx - radius, cy, cx, cy - radius, color);
}

/// Croix / plus (Chaser)
pub fn push_cross(data: &mut Vec<f32>, cx: f32, cy: f32, radius: f32, color: u32) {
    push_line(data, cx - radius, cy, cx + radius, cy, color);
    push_line(data, cx, cy - radius, cx, cy + radius, color);
    // Diagonales pour faire un X
    let d = radius * 0.6;
    push_line(data, cx - d, cy - d, cx + d, cy + d, color);
    push_line(data, cx + d, cy - d, cx - d, cy + d, color);
}

/// Convertit des coordonnées locales (axe du tétard) en coordonnées monde
#[inline]
fn tadpole_local_to_world(cx: f32, cy: f32, cos_a: f32, sin_a: f32, lx: f32, ly: f32) -> (f32, f32) {
    (cx + lx * cos_a - ly * sin_a,
     cy + lx * sin_a + ly * cos_a)
}

/// Tétard : corps ovale avec détails internes + longue flagelle conique ondulante
pub fn push_tadpole(data: &mut Vec<f32>, cx: f32, cy: f32, radius: f32, angle: f32, tail_phase: f32, _color: u32) {
    use std::f32::consts::PI;
    let cos_a = angle.cos();
    let sin_a = angle.sin();
    let w = |data: &mut Vec<f32>, lx0: f32, ly0: f32, lx1: f32, ly1: f32, col: u32| {
        let (wx0, wy0) = tadpole_local_to_world(cx, cy, cos_a, sin_a, lx0, ly0);
        let (wx1, wy1) = tadpole_local_to_world(cx, cy, cos_a, sin_a, lx1, ly1);
        push_line(data, wx0, wy0, wx1, wy1, col);
    };

    let rx = radius;
    let ry = radius * 0.62;

    // ═══════════════════════════════════════
    // CORPS : contour elliptique
    // ═══════════════════════════════════════
    let body_segs = 20;
    for i in 0..body_segs {
        let t0 = (i as f32 / body_segs as f32) * PI * 2.0;
        let t1 = ((i + 1) as f32 / body_segs as f32) * PI * 2.0;
        w(data, rx * t0.cos(), ry * t0.sin(), rx * t1.cos(), ry * t1.sin(), 2);
    }

    // Notochorde (axe central)
    w(data, -rx * 0.85, 0.0, rx * 0.85, 0.0, 0);

    // Somites musculaires (chevrons)
    let somites = 7;
    for i in 0..somites {
        let t = -0.75 + (i as f32 / (somites - 1) as f32) * 1.5;
        let lx = t * rx;
        let chevron_h = ry * (1.0 - (t * 1.1).powi(2)).max(0.0).sqrt() * 0.85;
        w(data, lx, -chevron_h, lx, chevron_h, 0);
        if i > 0 {
            let lx_prev = (-0.75 + ((i - 1) as f32 / (somites - 1) as f32) * 1.5) * rx;
            let h_prev = ry * (1.0 - ((-0.75 + ((i-1) as f32/(somites-1) as f32)*1.5) * 1.1).powi(2)).max(0.0).sqrt() * 0.85;
            w(data, lx_prev, -h_prev * 0.7, lx, -chevron_h * 0.7, 0);
            w(data, lx_prev,  h_prev * 0.7, lx,  chevron_h * 0.7, 0);
        }
    }

    // Vésicule céphalique (tête)
    let head_segs = 8;
    let hx = rx * 0.55;
    let hr = radius * 0.28;
    for i in 0..head_segs {
        let t0 = (i as f32 / head_segs as f32) * PI * 2.0;
        let t1 = ((i + 1) as f32 / head_segs as f32) * PI * 2.0;
        w(data, hx + hr * t0.cos(), hr * 0.7 * t0.sin(),
                hx + hr * t1.cos(), hr * 0.7 * t1.sin(), 2);
    }

    // Œil
    let eye_x = hx + hr * 0.3;
    let eye_y = -hr * 0.55;
    let eye_r = radius * 0.08;
    let eye_segs = 5;
    for i in 0..eye_segs {
        let t0 = (i as f32 / eye_segs as f32) * PI * 2.0;
        let t1 = ((i + 1) as f32 / eye_segs as f32) * PI * 2.0;
        w(data, eye_x + eye_r * t0.cos(), eye_y + eye_r * t0.sin(),
                eye_x + eye_r * t1.cos(), eye_y + eye_r * t1.sin(), 0);
    }
    w(data, eye_x - eye_r * 0.4, eye_y, eye_x + eye_r * 0.4, eye_y, 0);
    w(data, eye_x, eye_y - eye_r * 0.4, eye_x, eye_y + eye_r * 0.4, 0);

    // Tube digestif
    let gut_segs = 10;
    let mut gx0 = rx * 0.3;
    let mut gy0 = ry * 0.15;
    for i in 1..=gut_segs {
        let t = i as f32 / gut_segs as f32;
        let gx1 = (0.3 - t * 1.0) * rx;
        let gy1 = ((t * PI * 2.5).sin() * 0.22) * ry;
        w(data, gx0, gy0, gx1, gy1, 4);
        gx0 = gx1; gy0 = gy1;
    }

    // Vésicule vitelline
    let yolk_segs = 10;
    let yx = -rx * 0.05;
    let yrx = rx * 0.38;
    let yry = ry * 0.30;
    for i in 0..yolk_segs {
        let t0 = (i as f32 / yolk_segs as f32) * PI * 2.0;
        let t1 = ((i + 1) as f32 / yolk_segs as f32) * PI * 2.0;
        w(data, yx + yrx * t0.cos(), ry * 0.35 + yry * t0.sin(),
                yx + yrx * t1.cos(), ry * 0.35 + yry * t1.sin(), 5);
    }

    // ═══════════════════════════════════════
    // FLAGELLE : long cône fin à ondulation douce
    // ═══════════════════════════════════════
    //
    // Principe : à chaque pas on trace 3 segments transversaux (haut, centre, bas)
    // qui forment la "section" du cône à cette position. Leur épaisseur décroît
    // linéairement vers la pointe → silhouette conique.
    // Le centre de chaque section ondule latéralement → illusion de souplesse.

    let tail_len  = radius * 6.0;   // longueur totale
    let base_half = ry * 0.30;      // demi-épaisseur à la racine
    let tail_segs = 48;             // résolution du filament central
    let freq      = 1.4;            // 1.4 longueurs d'onde sur toute la flagelle

    // Racine de la flagelle : bord arrière du corps
    let root_lx = -rx;

    // Centre de section précédent
    let mut prev_cx = root_lx;
    let mut prev_cy_mid = 0.0_f32;

    for i in 1..=tail_segs {
        let t = i as f32 / tail_segs as f32;

        // Position le long de l'axe (de la racine vers la pointe)
        let lx = root_lx - t * tail_len;

        // Demi-épaisseur conique : pleine à la racine, nulle à la pointe
        let half = base_half * (1.0 - t);

        // Ondulation du centre : amplitude croissante linéairement, large et visible
        let wave_amp = radius * 1.2 * t;
        let phase    = tail_phase - t * freq * PI * 2.0;
        let ly_mid   = phase.sin() * wave_amp;

        // Bords haut et bas de la section (épaisseur conique)
        let ly_top = ly_mid - half;
        let ly_bot = ly_mid + half;

        // Segment central (axe du cône)
        w(data, prev_cx, prev_cy_mid, lx, ly_mid, 2);

        // Bords supérieur et inférieur (donnent la forme conique)
        if half > 0.003 {
            w(data, prev_cx, prev_cy_mid - base_half * (1.0 - (i-1) as f32 / tail_segs as f32),
                     lx, ly_top, 2);
            w(data, prev_cx, prev_cy_mid + base_half * (1.0 - (i-1) as f32 / tail_segs as f32),
                     lx, ly_bot, 2);
        }

        prev_cx    = lx;
        prev_cy_mid = ly_mid;
    }
}

/// Dispatch par shape_id
pub fn push_entity_shape(data: &mut Vec<f32>, cx: f32, cy: f32, radius: f32, color: u32, shape_id: u8) {
    match shape_id {
        1 => push_triangle(data, cx, cy, radius, color),
        2 => push_square(data, cx, cy, radius, color),
        3 => push_diamond(data, cx, cy, radius, color),
        4 => push_cross(data, cx, cy, radius, color),
        _ => push_circle(data, cx, cy, radius, color), // 0 et 5
    }
}
