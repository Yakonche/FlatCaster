// src/entities/tadpole.rs
//
// Comportement "tétard" : se propulse par coups de flagelle,
// dérive entre les battements, tourne progressivement vers le joueur.

use std::f32::consts::PI;
use crate::map::MapHandler;
use super::entity_base::EntityBase;

#[derive(Clone)]
pub struct TadpoleState {
    /// Phase de l'ondulation de la flagelle (avance en continu)
    pub tail_phase: f32,
    /// Temps restant avant le prochain coup de flagelle
    pub stroke_cooldown: f32,
    /// Amplitude actuelle du coup (décroît entre les battements)
    pub stroke_power: f32,
    /// Direction cible (vers le joueur, mise à jour progressivement)
    pub target_angle: f32,
}

impl TadpoleState {
    pub fn new() -> Self {
        Self {
            tail_phase: 0.0,
            stroke_cooldown: 0.0,
            stroke_power: 0.0,
            target_angle: 0.0,
        }
    }
}

pub fn update_tadpole(
    base: &mut EntityBase,
    state: &mut TadpoleState,
    player_pos: (f32, f32),
    map: &mut MapHandler,
    dt: f32,
) {
    let dist_p = base.dist_to(player_pos.0, player_pos.1);

    // --- Orientation progressive vers le joueur ---
    let angle_to_player = (player_pos.1 - base.y).atan2(player_pos.0 - base.x);
    state.target_angle = angle_to_player;

    // Rotation douce vers la cible (vitesse proportionnelle à la distance)
    let turn_speed = if dist_p > 150.0 { 1.5 } else { 0.8 };
    let angle_diff = angle_diff(state.target_angle, base.angle);
    base.angle += angle_diff.clamp(-turn_speed * dt, turn_speed * dt);

    // --- Avancement de la phase de flagelle ---
    // Fréquence d'ondulation : ~1.1 cycles/s
    state.tail_phase += dt * 1.1 * PI * 2.0;
    if state.tail_phase > PI * 2.0 {
        state.tail_phase -= PI * 2.0;
    }

    // --- Déclenchement des coups de flagelle ---
    state.stroke_cooldown -= dt;
    if state.stroke_cooldown <= 0.0 {
        // Déclenche un coup : puissance proportionnelle à la distance au joueur
        let power = if dist_p > 80.0 { 1.0 } else { 0.4 };
        state.stroke_power = base.speed * power;
        // Intervalle entre coups : ~0.45s (irrégulier comme un vrai tétard)
        state.stroke_cooldown = 0.35 + (state.tail_phase.sin().abs() * 0.15);
    }

    // --- Propulsion : impulsion dans la direction du corps ---
    // La poussée suit le rythme de la flagelle (max à la phase 0 et PI)
    let thrust_mod = state.tail_phase.sin().abs();
    let thrust = state.stroke_power * thrust_mod * dt * 60.0;

    let dx = base.angle.cos() * thrust;
    let dy = base.angle.sin() * thrust;

    // Glissement latéral léger (ondulation)
    let side = (state.tail_phase * 2.0).sin() * 0.15 * thrust;
    let perp_x = -base.angle.sin() * side;
    let perp_y =  base.angle.cos() * side;

    base.move_pos(dx + perp_x, dy + perp_y, map);

    // Décroissance naturelle de la puissance entre les coups
    state.stroke_power *= 0.92f32.powf(dt * 60.0);
}

/// Retourne les points centraux de la flagelle en coordonnées monde,
/// avec la demi-épaisseur conique à chaque point.
/// Utilisé à la fois par le rendu et par get_ray_segments.
pub fn tail_spine(
    bx: f32, by: f32,   // position du corps
    angle: f32,
    tail_phase: f32,
    size: f32,          // rayon du corps (en pixels monde)
) -> Vec<(f32, f32, f32)> {  // (x, y, half_width)
    const TAIL_SEGS: usize = 20;  // résolution (moins que le rendu pour perf)
    let tail_len  = size * 6.0;
    let base_half = size * 0.62 * 0.30; // ry * 0.30
    let freq      = 1.4;
    let root_lx   = -size;               // bord arrière du corps en local

    let cos_a = angle.cos();
    let sin_a = angle.sin();

    let mut pts = Vec::with_capacity(TAIL_SEGS + 1);
    // point de racine
    let (rx0, ry0) = (bx + root_lx * cos_a, by + root_lx * sin_a);
    pts.push((rx0, ry0, base_half));

    for i in 1..=TAIL_SEGS {
        let t       = i as f32 / TAIL_SEGS as f32;
        let lx      = root_lx - t * tail_len;
        let half    = base_half * (1.0 - t);
        let wave_amp = size * 1.2 * t; // synchronisé avec draw_2d.rs
        let phase   = tail_phase - t * freq * PI * 2.0;
        let ly_mid  = phase.sin() * wave_amp;
        // local → monde
        let wx = bx + lx * cos_a - ly_mid * sin_a;
        let wy = by + lx * sin_a + ly_mid * cos_a;
        pts.push((wx, wy, half));
    }
    pts
}

/// Différence angulaire signée entre deux angles (résultat dans [-PI, PI])
fn angle_diff(target: f32, current: f32) -> f32 {
    let mut diff = target - current;
    while diff > PI  { diff -= 2.0 * PI; }
    while diff < -PI { diff += 2.0 * PI; }
    diff
}
