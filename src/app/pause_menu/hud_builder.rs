// src/app/pause_menu/hud_builder.rs
//
// Construction du HUD du menu pause (textes, positions, couleurs).

use glyphon::{Attrs, Buffer, Family, Metrics, Shaping};
use super::types::*;

/// Mesure la largeur réelle en pixels d'un texte Jersey10 à une taille donnée.
pub(super) fn measure_text(text: &str, size_px: f32, font_system: &mut glyphon::FontSystem) -> f32 {
    let mut buf = Buffer::new(font_system, Metrics::new(size_px, size_px * 1.2));
    buf.set_size(font_system, None, None);
    buf.set_text(font_system, text, Attrs::new().family(Family::Name("Jersey 10")), Shaping::Basic);
    buf.shape_until_scroll(font_system, false);
    buf.layout_runs().map(|r| r.line_w).fold(0.0_f32, f32::max)
}

impl PauseMenu {
    pub fn build_hud(
        &mut self,
        screen_w: f32,
        screen_h: f32,
        _current_fps: f32,
        _time_ms: u32,
        font_system: &mut glyphon::FontSystem,
    ) -> Vec<(String, f32, f32, f32, [u8; 4])> {
        let mut entries: Vec<(String, f32, f32, f32, [u8; 4])> = Vec::new();

        self.seed_field_rect = None;

        let (box_x, box_y, box_w, box_h) = self.box_rect(screen_w, screen_h);

        let s = self.ui_scale;
        let small  = 22.0_f32 * s;
        let medium = 28.0_f32 * s;
        let large  = 36.0_f32 * s;

        let col_normal   = [200u8, 200, 200, 255];
        let col_selected = [255u8, 220,  60, 255];
        let col_title    = [255u8, 255, 255, 255];
        let col_dim      = [140u8, 140, 140, 255];
        let col_awaiting = [255u8,  80,  80, 255];
        let col_typing   = [100u8, 220, 255, 255];
        // Couleurs Xbox (réservées pour extension future)
        let col_xbox_a   = [ 80u8, 200,  80, 255];
        let col_xbox_b   = [220u8,  60,  60, 255];
        let col_xbox_x   = [ 80u8, 140, 220, 255];
        let col_xbox_y   = [220u8, 200,  60, 255];

        let margin    = 28.0 * s;
        let content_x = box_x + margin;
        let content_w = box_w - margin * 2.0;
        let line_h    = small + 10.0 * s;

        match self.active_screen {

            // ══════════════════════════════════════════
            // Menu principal
            // ══════════════════════════════════════════
            ActiveScreen::Main => {
                let title = "PAUSE";
                let title_size = large * 1.6;
                let title_w = measure_text(title, title_size, font_system);
                let title_x = (screen_w - title_w) * 0.5;
                let title_y = box_y + 6.0 * s;
                // Ombre décalée (orange foncé)
                entries.push((title.into(), title_x + 2.0, title_y + 2.0, title_size, [200u8, 80, 0, 200]));
                // Texte principal (jaune vif)
                entries.push((title.into(), title_x, title_y, title_size, [255u8, 200, 30, 255]));

                let start_y = box_y + title_size + 18.0 * s;
                let item_h  = medium + 20.0 * s;

                let arrow_size = medium * 0.75;
                let arrow_gap  = medium * 0.35;

                self.arrow_screen_pos = None;
                self.clickable_rects.retain(|r| r.4 != 0);

                for (i, &item) in MainMenuItem::ALL.iter().enumerate() {
                    let iy = start_y + i as f32 * item_h;
                    let selected = i == self.main_row;
                    let color = if selected { col_selected } else { col_normal };
                    let label = item.label();
                    let label_w = measure_text(label, medium, font_system);
                    let lx = (screen_w - label_w) * 0.5;
                    entries.push((label.into(), lx, iy, medium, color));
                    if selected {
                        let tip_x = lx - arrow_gap;
                        let tip_y = iy + medium * 0.35;
                        self.arrow_screen_pos = Some((tip_x, tip_y, arrow_size));
                    }
                    self.clickable_rects.push((
                        lx - arrow_gap - arrow_size,
                        iy - 4.0 * s,
                        label_w + arrow_gap + arrow_size + 4.0 * s,
                        medium + 8.0 * s,
                        0u32,
                        i,
                    ));
                }
            }

            // ══════════════════════════════════════════
            // Contrôles
            // ══════════════════════════════════════════
            ActiveScreen::Controls => {
                let arrow_size = medium * 0.75;
                let arrow_gap  = medium * 0.35;
                let arrow_total_w = arrow_size * 1.73;
                self.arrow_screen_pos = None;

                let title_c = "CONTROLES";
                let title_c_w = measure_text(title_c, large, font_system);
                entries.push((title_c.into(), (screen_w - title_c_w) * 0.5, box_y + 8.0 * s, large, col_title));

                let margin_ctrl = arrow_gap + arrow_total_w + 6.0 * s;
                let content_ctrl_x = box_x + margin_ctrl;
                let content_ctrl_w = box_w - margin_ctrl * 2.0;

                let col_key_x = content_ctrl_x + content_ctrl_w * 0.47;
                let col_pad_x = content_ctrl_x + content_ctrl_w * 0.74;
                let header_y  = box_y + large + 14.0 * s;
                entries.push(("Clavier / Souris".into(), col_key_x, header_y, small * 0.85, col_dim));
                entries.push(("Manette".into(),           col_pad_x, header_y, small * 0.85, col_dim));

                let start_y  = header_y + line_h * 0.9;
                let row_h2   = small + 8.0 * s;

                let bottom_margin = 12.0 * s;
                let available_h   = (box_y + box_h - start_y - bottom_margin).max(row_h2);
                let max_visible   = (available_h / row_h2).floor() as usize;
                let max_visible   = max_visible.max(1);
                let n_total       = self.bindings.len();

                if self.selected_row < self.controls_scroll {
                    self.controls_scroll = self.selected_row;
                } else if self.selected_row >= self.controls_scroll + max_visible {
                    self.controls_scroll = self.selected_row + 1 - max_visible;
                }
                if self.controls_scroll + max_visible > n_total {
                    self.controls_scroll = n_total.saturating_sub(max_visible);
                }

                let scroll = self.controls_scroll;
                let vis_end = (scroll + max_visible).min(n_total);

                let selected_row = self.selected_row;
                let control_col  = self.control_col;
                let awaiting     = self.awaiting_rebind_key;

                let visible_rows: Vec<(String, String, String)> = (scroll..vis_end).map(|i| {
                    let (action, binding) = &self.bindings[i];
                    let action_lbl = action.label().to_string();
                    let key_str = match &binding.key {
                        Some(kb) => kb.label(),
                        None     => "-".into(),
                    };
                    let pad_str = binding.gamepad.label().to_string();
                    (action_lbl, key_str, pad_str)
                }).collect();

                for (idx, (action_lbl, key_str, pad_str)) in visible_rows.into_iter().enumerate() {
                    let i     = scroll + idx;
                    let row_y = start_y + idx as f32 * row_h2;
                    let is_row = i == selected_row;
                    let act_col = if is_row { col_selected } else { col_normal };
                    if is_row {
                        self.arrow_screen_pos = Some((content_ctrl_x - arrow_gap, row_y + small * 0.35, arrow_size));
                    }
                    entries.push((action_lbl, content_ctrl_x, row_y, small, act_col));

                    let is_await_key = awaiting == Some((i, ControlCol::Key));
                    let key_col = if is_await_key { col_awaiting }
                        else if is_row && control_col == ControlCol::Key { col_selected }
                        else { col_normal };
                    let key_disp = if is_await_key { "[ appuyez... ]".into() } else { key_str };
                    entries.push((key_disp, col_key_x, row_y, small, key_col));

                    let is_await_pad = awaiting == Some((i, ControlCol::Pad));
                    let pad_col = if is_await_pad { col_awaiting }
                        else if is_row && control_col == ControlCol::Pad { col_selected }
                        else { col_normal };
                    let pad_disp = if is_await_pad { "[ manette... ]".into() } else { pad_str };
                    entries.push((pad_disp, col_pad_x, row_y, small, pad_col));
                }

                if scroll > 0 {
                    entries.push(("▲".into(), box_x + box_w - margin_ctrl * 0.8, start_y, small, col_dim));
                }
                if vis_end < n_total {
                    let last_row_y = start_y + (max_visible - 1) as f32 * row_h2;
                    entries.push(("▼".into(), box_x + box_w - margin_ctrl * 0.8, last_row_y, small, col_dim));
                }
            }

            // ══════════════════════════════════════════
            // Graphismes
            // ══════════════════════════════════════════
            ActiveScreen::Graphics => {
                let arrow_size = medium * 0.75;
                let arrow_gap  = medium * 0.35;

                let title_g = "GRAPHISMES";
                let title_g_w = measure_text(title_g, large, font_system);
                entries.push((title_g.into(), (screen_w - title_g_w) * 0.5, box_y + 8.0 * s, large, col_title));

                let start_y = box_y + large + 18.0 * s;
                let row_h   = medium + 16.0 * s;
                let val_x   = content_x + content_w * 0.50;

                let g_row = self.graphics_row;
                let tip_base_x = content_x - arrow_gap;
                let arrow_y_for = |ry: f32| -> (f32, f32, f32) { (tip_base_x, ry + medium * 0.35, arrow_size) };
                self.arrow_screen_pos = None;

                {
                    let ry = start_y;
                    let lc = if g_row == 0 { col_selected } else { col_normal };
                    if g_row == 0 { self.arrow_screen_pos = Some(arrow_y_for(ry)); }
                    entries.push(("Resolution".into(), content_x, ry, medium, lc));
                    let (label, _, _) = RES_PRESETS[self.res_preset_idx];
                    let vc = if g_row == 0 { col_selected } else { col_normal };
                    let val_str = if g_row == 0 { format!("< {} >", label) } else { label.into() };
                    entries.push((val_str, val_x, ry, medium, vc));
                    if g_row == 0 {
                        entries.push(("(Entree pour appliquer)".into(), val_x, ry + medium + 2.0, small * 0.70, col_dim));
                    }
                }
                {
                    let ry = start_y + row_h;
                    let lc = if g_row == 1 { col_selected } else { col_normal };
                    if g_row == 1 { self.arrow_screen_pos = Some(arrow_y_for(ry)); }
                    entries.push(("Mode affichage".into(), content_x, ry, medium, lc));
                    let vc = if g_row == 1 { col_selected } else { col_normal };
                    let val_str = if g_row == 1 { format!("< {} >", self.window_mode.label()) } else { self.window_mode.label().into() };
                    entries.push((val_str, val_x, ry, medium, vc));
                }
                {
                    let ry = start_y + row_h * 2.0;
                    let lc = if g_row == 2 { col_selected } else { col_normal };
                    if g_row == 2 { self.arrow_screen_pos = Some(arrow_y_for(ry)); }
                    entries.push(("Taille interface".into(), content_x, ry, medium, lc));
                    let vc = if g_row == 2 { col_selected } else { col_normal };
                    let val_str = if g_row == 2 { format!("< {:.2}x >", self.ui_scale) } else { format!("{:.2}x", self.ui_scale) };
                    entries.push((val_str, val_x, ry, medium, vc));
                }
                {
                    let ry = start_y + row_h * 3.0;
                    let lc = if g_row == 3 { col_selected } else { col_normal };
                    if g_row == 3 { self.arrow_screen_pos = Some(arrow_y_for(ry)); }
                    let mark = if self.fps_enabled { "[x]" } else { "[ ]" };
                    entries.push((format!("Limite FPS  {}", mark), content_x, ry, medium, lc));
                }
                {
                    let ry = start_y + row_h * 4.0;
                    let dim = !self.fps_enabled;
                    let lc = if dim { col_dim } else if g_row == 4 { col_selected } else { col_normal };
                    if !dim && g_row == 4 { self.arrow_screen_pos = Some(arrow_y_for(ry)); }
                    entries.push(("Valeur FPS".into(), content_x, ry, medium, lc));
                    let (val_str, vc) = if dim {
                        (fps_label(self.fps_millis), col_dim)
                    } else {
                        match self.fps_edit_mode {
                            FpsEditMode::Typing => {
                                let buf = if self.fps_typing_buf.is_empty() { "_".into() } else { format!("{}|", self.fps_typing_buf) };
                                (format!("{} fps", buf), col_typing)
                            }
                            FpsEditMode::Preset => {
                                let vc = if g_row == 4 { col_selected } else { col_normal };
                                (format!("< {} >", fps_label(self.fps_millis)), vc)
                            }
                        }
                    };
                    entries.push((val_str, val_x, ry, medium, vc));
                }
                {
                    let ry = start_y + row_h * 5.0;
                    let lc = if g_row == 5 { col_selected } else { col_normal };
                    if g_row == 5 { self.arrow_screen_pos = Some(arrow_y_for(ry)); }
                    let mark = if self.show_fps { "[x]" } else { "[ ]" };
                    entries.push((format!("Afficher FPS  {}", mark), content_x, ry, medium, lc));
                }
            }

            // ══════════════════════════════════════════
            // Seed
            // ══════════════════════════════════════════
            ActiveScreen::Seed => {
                self.arrow_screen_pos = None;

                let title_s = "SEED";
                let title_s_w = measure_text(title_s, large, font_system);
                entries.push((title_s.into(), (screen_w - title_s_w) * 0.5, box_y + 10.0 * s, large, col_title));

                let field_pad_x = 10.0 * s;
                let field_pad_y = 7.0 * s;
                let field_h     = medium + field_pad_y * 2.0;
                let field_x     = box_x + margin;
                let field_w     = box_w - margin * 2.0;
                let seed_y      = box_y + large + 26.0 * s;

                self.seed_field_rect = Some((field_x, seed_y, field_w, field_h));

                let inner_w = field_w - field_pad_x * 2.0;
                let char_w  = measure_text("A", medium, font_system).max(1.0);
                let max_visible_chars = ((inner_w / char_w).floor() as usize).max(1);

                if self.seed_cursor < self.seed_scroll {
                    self.seed_scroll = self.seed_cursor;
                }
                if self.seed_cursor > self.seed_scroll + max_visible_chars.saturating_sub(1) {
                    self.seed_scroll = self.seed_cursor.saturating_sub(max_visible_chars.saturating_sub(1));
                }

                let visible_start = self.seed_scroll.min(self.seed_input.len());
                let visible_end   = (self.seed_scroll + max_visible_chars).min(self.seed_input.len());
                let visible_text: String = self.seed_input[visible_start..visible_end].to_string();
                let cursor_in_view = self.seed_cursor.saturating_sub(self.seed_scroll);

                let (display_seed, seed_text_col) = if self.seed_selected {
                    (format!("[{}]", visible_text), col_selected)
                } else if self.seed_input.is_empty() && !self.seed_focused {
                    ("Entree pour editer...".into(), col_dim)
                } else if self.seed_focused && self.cursor_blink_visible {
                    let mut v = visible_text.clone();
                    let insert_pos = cursor_in_view.min(v.len());
                    v.insert(insert_pos, '|');
                    (v, col_normal)
                } else {
                    (visible_text.clone(), col_normal)
                };

                {
                    let rect = (field_x, seed_y, field_w, field_h, 10u32, 0usize);
                    let entry = self.clickable_rects.iter().position(|r| r.4 == 10);
                    if let Some(i) = entry { self.clickable_rects[i] = rect; }
                    else { self.clickable_rects.push(rect); }
                }

                let text_y = seed_y + field_pad_y;
                entries.push((display_seed, field_x + field_pad_x, text_y, medium, seed_text_col));

                let hint_y = seed_y + field_h + 5.0 * s;
                let hint = if self.seed_focused {
                    "[Entree] confirmer  [Echap] annuler"
                } else if self.selected_row == 0 {
                    "[Entree] editer  [Bas] Appliquer  [Haut] Aleatoire"
                } else {
                    "[Haut/Bas] naviguer  [Entree] valider"
                };
                let hint_w = measure_text(hint, small * 0.78, font_system);
                entries.push((hint.into(), box_x + (box_w - hint_w) * 0.5, hint_y, small * 0.78, col_dim));

                let arrow_size = medium * 0.75;
                let arrow_gap  = medium * 0.35;
                let btn_y_start = hint_y + small * 0.78 + 18.0 * s;

                if !self.seed_focused && self.selected_row == 0 {
                    self.arrow_screen_pos = Some((field_x - arrow_gap, seed_y + field_h * 0.5, arrow_size));
                }

                let apply_col = if !self.seed_focused && self.selected_row == 1 { col_selected } else { col_normal };
                if !self.seed_focused && self.selected_row == 1 {
                    self.arrow_screen_pos = Some((content_x - arrow_gap, btn_y_start + medium * 0.35, arrow_size));
                }
                let apply_w = measure_text("[ Appliquer ]", medium, font_system);
                {
                    let rect = (content_x - arrow_gap - 4.0 * s, btn_y_start - 2.0 * s,
                                apply_w + arrow_gap + 8.0 * s, medium + 4.0 * s, 1u32, 1usize);
                    let entry = self.clickable_rects.iter().position(|r| r.4 == 1);
                    if let Some(i) = entry { self.clickable_rects[i] = rect; }
                    else { self.clickable_rects.push(rect); }
                }
                entries.push(("[ Appliquer ]".into(), content_x, btn_y_start, medium, apply_col));

                let rand_y = btn_y_start + medium + 14.0 * s;
                let rand_col = if !self.seed_focused && self.selected_row == 2 { col_selected } else { col_normal };
                if !self.seed_focused && self.selected_row == 2 {
                    self.arrow_screen_pos = Some((content_x - arrow_gap, rand_y + medium * 0.35, arrow_size));
                }
                let rand_w = measure_text("[ Seed aleatoire ]", medium, font_system);
                {
                    let rect = (content_x - arrow_gap - 4.0 * s, rand_y - 2.0 * s,
                                rand_w + arrow_gap + 8.0 * s, medium + 4.0 * s, 2u32, 2usize);
                    let entry = self.clickable_rects.iter().position(|r| r.4 == 2);
                    if let Some(i) = entry { self.clickable_rects[i] = rect; }
                    else { self.clickable_rects.push(rect); }
                }
                entries.push(("[ Seed aleatoire ]".into(), content_x, rand_y, medium, rand_col));
            }
        }

        let _ = (col_xbox_a, col_xbox_b, col_xbox_x, col_xbox_y);

        entries
    }
}
