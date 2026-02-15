// src/app/pause_menu/input.rs
//
// Gestion de tous les inputs du menu pause :
// clavier, souris, manette, navigation, remappage, seed.

use winit::keyboard::KeyCode;
use super::types::*;

impl PauseMenu {
    pub fn handle_input(
        &mut self,
        up: bool, down: bool,
        left: bool, right: bool,
        confirm: bool,
        escape: bool,
        back: bool,          // B/East manette ou Backspace — retour au menu principal
        chars: &[char],
        backspace: bool,
        ctrl_c: bool,
        ctrl_v: bool,
        clipboard_text: Option<&str>,
        new_key_pressed: Option<KeyCode>,
        new_mouse_pressed: Option<u32>,
        scroll_delta: f32,
        mouse_x: f32,
        mouse_y: f32,
        mouse_clicked: bool,  // clic gauche souris (edge)
        // Flèches pures (sans Z/S/Q/D) — pour déplacement curseur texte
        arrow_left: bool, arrow_right: bool,
        arrow_up: bool,   arrow_down: bool,
    ) -> MenuEvent {
        if self.nav_cooldown > 0 { self.nav_cooldown -= 1; }

        // ─── Navigation souris (hover + clic) ────────────────────────────────
        let mouse_event = self.handle_mouse(mouse_x, mouse_y, mouse_clicked);
        if !matches!(mouse_event, MenuEvent::None) {
            return mouse_event;
        }

        // B/East = retour arrière dans tous les sous-menus (sauf rebind en attente)
        let escape_or_back = escape || back;

        match self.active_screen {
            ActiveScreen::Main => self.handle_main(up, down, confirm, escape_or_back),
            ActiveScreen::Controls => self.handle_controls(
                up, down, left, right, confirm, escape_or_back,
                new_key_pressed, new_mouse_pressed, scroll_delta,
            ),
            ActiveScreen::Graphics => self.handle_graphics(
                up, down, left, right, confirm, escape_or_back,
                chars, backspace, scroll_delta,
            ),
            ActiveScreen::Seed => self.handle_seed(
                up, down, left, right, confirm, escape_or_back, chars, backspace, ctrl_c, ctrl_v, clipboard_text,
                arrow_left, arrow_right, arrow_up, arrow_down,
            ),
        }
    }

    /// Gère le survol et le clic souris sur les éléments cliquables.
    /// Retourne MenuEvent::None si rien d'actionnable.
    fn handle_mouse(&mut self, mx: f32, my: f32, clicked: bool) -> MenuEvent {
        let screen = self.active_screen;
        let valid_rect = |action: u32| -> bool {
            match screen {
                ActiveScreen::Main     => action == 0,
                ActiveScreen::Seed     => action == 1 || action == 2 || action == 10,
                ActiveScreen::Controls => action == 20,
                ActiveScreen::Graphics => false,
            }
        };

        let mut hovered: Option<(u32, usize)> = None;
        for &(rx, ry, rw, rh, action, idx) in &self.clickable_rects {
            if !valid_rect(action) { continue; }
            if mx >= rx && mx <= rx + rw && my >= ry && my <= ry + rh {
                hovered = Some((action, idx));
                break;
            }
        }

        if let Some((action, idx)) = hovered {
            match action {
                0       => { self.main_row = idx; }
                20      => { self.selected_row = idx; }
                10      => { self.selected_row = 0; }
                1       => { self.selected_row = 1; }
                2       => { self.selected_row = 2; }
                _       => {}
            }
        }

        if !clicked { return MenuEvent::None; }

        if let Some((10, _)) = hovered {
            self.seed_focused = true;
            return MenuEvent::None;
        }

        self.seed_focused = false;

        if let Some((action, idx)) = hovered {
            match action {
                0 => {
                    self.nav_cooldown = 10;
                    match MainMenuItem::ALL[idx] {
                        MainMenuItem::Resume   => return MenuEvent::Close,
                        MainMenuItem::Controls => {
                            self.active_screen   = ActiveScreen::Controls;
                            self.selected_row    = 0;
                            self.control_col     = ControlCol::Key;
                            self.controls_scroll = 0;
                            self.clickable_rects.retain(|r| r.4 == 0);
                            self.seed_field_rect = None;
                        }
                        MainMenuItem::Graphics => {
                            self.active_screen = ActiveScreen::Graphics;
                            self.graphics_row  = 0;
                            self.clickable_rects.retain(|r| r.4 == 0);
                            self.seed_field_rect = None;
                        }
                        MainMenuItem::Seed => {
                            self.active_screen = ActiveScreen::Seed;
                            self.selected_row  = 0;
                            self.seed_focused  = false;
                            self.clickable_rects.retain(|r| r.4 == 0);
                        }
                        MainMenuItem::Quit => return MenuEvent::Quit,
                    }
                }
                1 => {
                    if !self.seed_input.is_empty() {
                        return MenuEvent::ApplySeed(self.seed_input.clone());
                    }
                }
                2 => {
                    return MenuEvent::RandomSeed;
                }
                _ => {}
            }
        }

        MenuEvent::None
    }

    // ── Menu principal ──────────────────────────────────────

    fn handle_main(&mut self, up: bool, down: bool, confirm: bool, escape: bool) -> MenuEvent {
        let n = MainMenuItem::ALL.len();
        if down && self.nav_cooldown == 0 {
            self.main_row = (self.main_row + 1) % n;
            self.nav_cooldown = 8;
        }
        if up && self.nav_cooldown == 0 {
            self.main_row = (self.main_row + n - 1) % n;
            self.nav_cooldown = 8;
        }

        if escape {
            return MenuEvent::Close;
        }

        if confirm && self.nav_cooldown == 0 {
            self.nav_cooldown = 10;
            match MainMenuItem::ALL[self.main_row] {
                MainMenuItem::Resume   => return MenuEvent::Close,
                MainMenuItem::Controls => {
                    self.active_screen    = ActiveScreen::Controls;
                    self.selected_row     = 0;
                    self.control_col      = ControlCol::Key;
                    self.controls_scroll  = 0;
                }
                MainMenuItem::Graphics => {
                    self.active_screen = ActiveScreen::Graphics;
                    self.graphics_row  = 0;
                }
                MainMenuItem::Seed => {
                    self.active_screen = ActiveScreen::Seed;
                    self.selected_row  = 0;
                    self.seed_focused  = false;
                    self.clickable_rects.retain(|r| r.4 == 0);
                }
                MainMenuItem::Quit => return MenuEvent::Quit,
            }
        }

        MenuEvent::None
    }

    // ── Contrôles ──────────────────────────────────────────

    fn handle_controls(
        &mut self,
        up: bool, down: bool, left: bool, right: bool,
        confirm: bool, escape: bool,
        new_key_pressed: Option<KeyCode>,
        new_mouse_pressed: Option<u32>,
        scroll_delta: f32,
    ) -> MenuEvent {
        if let Some((row, col)) = self.awaiting_rebind_key {
            if escape {
                self.awaiting_rebind_key = None;
                return MenuEvent::None;
            }
            if let Some(key) = new_key_pressed {
                if col == ControlCol::Key {
                    self.bindings[row].1.key = Some(KeyBind::Key(key));
                }
                self.awaiting_rebind_key = None;
                return MenuEvent::None;
            }
            if let Some(btn) = new_mouse_pressed {
                if col == ControlCol::Key {
                    self.bindings[row].1.key = Some(KeyBind::MouseButton(btn));
                    self.awaiting_rebind_key = None;
                }
                return MenuEvent::None;
            }
            return MenuEvent::None;
        }

        if escape {
            self.active_screen = ActiveScreen::Main;
            self.controls_scroll = 0;
            return MenuEvent::None;
        }

        let n = self.bindings.len();

        if scroll_delta > 0.1 && self.nav_cooldown == 0 {
            if self.controls_scroll + 1 < n { self.controls_scroll += 1; }
            self.nav_cooldown = 4;
        }
        if scroll_delta < -0.1 && self.nav_cooldown == 0 {
            self.controls_scroll = self.controls_scroll.saturating_sub(1);
            self.nav_cooldown = 4;
        }

        if down && self.nav_cooldown == 0 {
            self.selected_row = (self.selected_row + 1) % n;
            if self.selected_row >= self.controls_scroll + self.controls_max_visible_rows() {
                self.controls_scroll = self.selected_row.saturating_sub(self.controls_max_visible_rows() - 1);
            }
            if self.selected_row == 0 {
                self.controls_scroll = 0;
            }
            self.nav_cooldown = 8;
        }
        if up && self.nav_cooldown == 0 {
            self.selected_row = (self.selected_row + n - 1) % n;
            if self.selected_row < self.controls_scroll {
                self.controls_scroll = self.selected_row;
            }
            if self.selected_row == n - 1 {
                let vis = self.controls_max_visible_rows();
                self.controls_scroll = n.saturating_sub(vis);
            }
            self.nav_cooldown = 8;
        }
        if left && self.nav_cooldown == 0 {
            self.control_col  = ControlCol::Key;
            self.nav_cooldown = 10;
        }
        if right && self.nav_cooldown == 0 {
            self.control_col  = ControlCol::Pad;
            self.nav_cooldown = 10;
        }
        if confirm {
            self.awaiting_rebind_key = Some((self.selected_row, self.control_col));
        }

        MenuEvent::None
    }

    /// Estimation conservative du nombre de lignes visibles dans le panneau Contrôles.
    fn controls_max_visible_rows(&self) -> usize {
        let s = self.ui_scale;
        let box_h_est = 500.0_f32 * s;
        let header_h  = (36.0 + 14.0 + (22.0 + 10.0) * 0.9) * s;
        let row_h2    = (22.0 + 8.0) * s;
        ((box_h_est - header_h) / row_h2).floor().max(1.0) as usize
    }

    // ── Graphismes ─────────────────────────────────────────

    fn handle_graphics(
        &mut self,
        up: bool, down: bool, left: bool, right: bool,
        confirm: bool, escape: bool,
        chars: &[char],
        backspace: bool,
        scroll_delta: f32,
    ) -> MenuEvent {
        const ROWS: usize = 6;

        if escape {
            if self.fps_edit_mode == FpsEditMode::Typing {
                self.fps_edit_mode = FpsEditMode::Preset;
                self.fps_typing_buf.clear();
            } else {
                self.active_screen = ActiveScreen::Main;
            }
            return MenuEvent::None;
        }

        if self.fps_edit_mode != FpsEditMode::Typing {
            if down && self.nav_cooldown == 0 {
                self.graphics_row = (self.graphics_row + 1) % ROWS;
                self.nav_cooldown = 8;
            }
            if up && self.nav_cooldown == 0 {
                self.graphics_row = (self.graphics_row + ROWS - 1) % ROWS;
                self.nav_cooldown = 8;
            }
        }

        use crate::app::settings::WindowMode;
        use super::types::{RES_PRESETS, FPS_PRESETS};

        match self.graphics_row {
            0 => {
                let n = RES_PRESETS.len();
                if right && self.nav_cooldown == 0 {
                    self.res_preset_idx = (self.res_preset_idx + 1) % n;
                    self.nav_cooldown = 8;
                    let (_, w, h) = RES_PRESETS[self.res_preset_idx];
                    self.pending_width  = w;
                    self.pending_height = h;
                }
                if left && self.nav_cooldown == 0 {
                    self.res_preset_idx = (self.res_preset_idx + n - 1) % n;
                    self.nav_cooldown = 8;
                    let (_, w, h) = RES_PRESETS[self.res_preset_idx];
                    self.pending_width  = w;
                    self.pending_height = h;
                }
                if scroll_delta > 0.1 && self.nav_cooldown == 0 {
                    self.res_preset_idx = (self.res_preset_idx + 1) % n;
                    self.nav_cooldown = 4;
                    let (_, w, h) = RES_PRESETS[self.res_preset_idx];
                    self.pending_width  = w;
                    self.pending_height = h;
                }
                if scroll_delta < -0.1 && self.nav_cooldown == 0 {
                    self.res_preset_idx = (self.res_preset_idx + n - 1) % n;
                    self.nav_cooldown = 4;
                    let (_, w, h) = RES_PRESETS[self.res_preset_idx];
                    self.pending_width  = w;
                    self.pending_height = h;
                }
                if confirm && self.nav_cooldown == 0 {
                    self.nav_cooldown = 15;
                    self.pending_resize = Some((self.pending_width, self.pending_height));
                }
            }
            1 => {
                let modes = WindowMode::all();
                let n = modes.len();
                let cur = modes.iter().position(|&m| m == self.window_mode).unwrap_or(0);
                if (right || confirm) && self.nav_cooldown == 0 {
                    self.window_mode = modes[(cur + 1) % n];
                    self.nav_cooldown = 10;
                    self.pending_window_mode = Some(self.window_mode);
                }
                if left && self.nav_cooldown == 0 {
                    self.window_mode = modes[(cur + n - 1) % n];
                    self.nav_cooldown = 10;
                    self.pending_window_mode = Some(self.window_mode);
                }
            }
            2 => {
                let step = 0.25_f32;
                if right && self.nav_cooldown == 0 {
                    self.ui_scale = (self.ui_scale + step).min(3.0);
                    self.nav_cooldown = 8;
                }
                if left && self.nav_cooldown == 0 {
                    self.ui_scale = (self.ui_scale - step).max(0.5);
                    self.nav_cooldown = 8;
                }
                if scroll_delta > 0.1 && self.nav_cooldown == 0 {
                    self.ui_scale = (self.ui_scale + step).min(3.0);
                    self.nav_cooldown = 4;
                }
                if scroll_delta < -0.1 && self.nav_cooldown == 0 {
                    self.ui_scale = (self.ui_scale - step).max(0.5);
                    self.nav_cooldown = 4;
                }
            }
            3 => {
                if (confirm || left || right) && self.nav_cooldown == 0 {
                    self.fps_enabled = !self.fps_enabled;
                    if !self.fps_enabled {
                        self.fps_edit_mode = FpsEditMode::Preset;
                        self.fps_typing_buf.clear();
                    }
                    self.nav_cooldown = 10;
                }
            }
            4 => {
                if !self.fps_enabled { return MenuEvent::None; }
                match self.fps_edit_mode {
                    FpsEditMode::Preset => {
                        let n = FPS_PRESETS.len();
                        if right && self.nav_cooldown == 0 {
                            self.fps_preset_idx = (self.fps_preset_idx + 1) % n;
                            self.fps_millis = FPS_PRESETS[self.fps_preset_idx].1;
                            self.nav_cooldown = 8;
                        }
                        if left && self.nav_cooldown == 0 {
                            self.fps_preset_idx = (self.fps_preset_idx + n - 1) % n;
                            self.fps_millis = FPS_PRESETS[self.fps_preset_idx].1;
                            self.nav_cooldown = 8;
                        }
                        if scroll_delta > 0.1 && self.nav_cooldown == 0 {
                            self.fps_preset_idx = (self.fps_preset_idx + 1) % n;
                            self.fps_millis = FPS_PRESETS[self.fps_preset_idx].1;
                            self.nav_cooldown = 4;
                        }
                        if scroll_delta < -0.1 && self.nav_cooldown == 0 {
                            self.fps_preset_idx = (self.fps_preset_idx + n - 1) % n;
                            self.fps_millis = FPS_PRESETS[self.fps_preset_idx].1;
                            self.nav_cooldown = 4;
                        }
                        for &c in chars {
                            if c.is_ascii_digit() {
                                self.fps_edit_mode = FpsEditMode::Typing;
                                self.fps_typing_buf.clear();
                                self.fps_typing_buf.push(c);
                                break;
                            }
                        }
                    }
                    FpsEditMode::Typing => {
                        for &c in chars {
                            if c.is_ascii_digit() {
                                if self.fps_typing_buf.len() < 6 { self.fps_typing_buf.push(c); }
                            } else if (c == '.' || c == ',') && !self.fps_typing_buf.contains('.') {
                                self.fps_typing_buf.push('.');
                            }
                        }
                        if backspace { self.fps_typing_buf.pop(); }
                        if confirm   { self.commit_fps_typing(); }
                        if left || right {
                            self.fps_edit_mode = FpsEditMode::Preset;
                            self.fps_typing_buf.clear();
                        }
                    }
                }
            }
            5 => {
                if (confirm || left || right) && self.nav_cooldown == 0 {
                    self.show_fps = !self.show_fps;
                    self.nav_cooldown = 10;
                }
            }
            _ => {}
        }

        MenuEvent::None
    }

    // ── Seed ───────────────────────────────────────────────

    fn handle_seed(
        &mut self,
        up: bool, down: bool,
        _left: bool, _right: bool,
        confirm: bool, escape: bool,
        chars: &[char],
        backspace: bool,
        ctrl_c: bool,
        ctrl_v: bool,
        clipboard_text: Option<&str>,
        arrow_left: bool, arrow_right: bool,
        arrow_up: bool,   arrow_down: bool,
    ) -> MenuEvent {
        const SEED_ROWS: usize = 3;
        const SEED_MAX_LEN: usize = 256;

        if escape {
            if self.seed_focused {
                self.seed_focused = false;
                return MenuEvent::None;
            }
            self.active_screen = ActiveScreen::Main;
            self.clickable_rects.retain(|r| r.4 == 0);
            self.seed_field_rect = None;
            return MenuEvent::None;
        }

        if self.seed_focused {
            if confirm {
                self.seed_focused = false;
                if !self.seed_input.is_empty() {
                    return MenuEvent::ApplySeedInMenu(self.seed_input.clone());
                }
                return MenuEvent::None;
            }

            if arrow_left && self.nav_cooldown == 0 {
                if self.seed_cursor > 0 { self.seed_cursor -= 1; }
                self.nav_cooldown = 3;
            }
            if arrow_right && self.nav_cooldown == 0 {
                if self.seed_cursor < self.seed_input.len() { self.seed_cursor += 1; }
                self.nav_cooldown = 3;
            }
            if arrow_up && self.nav_cooldown == 0 {
                self.seed_cursor = 0;
                self.nav_cooldown = 6;
            }
            if arrow_down && self.nav_cooldown == 0 {
                self.seed_cursor = self.seed_input.len();
                self.nav_cooldown = 6;
            }

            for &c in chars {
                if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                    if self.seed_selected {
                        self.seed_input.clear();
                        self.seed_cursor   = 0;
                        self.seed_scroll   = 0;
                        self.seed_selected = false;
                    }
                    if self.seed_input.len() < SEED_MAX_LEN {
                        self.seed_input.insert(self.seed_cursor, c);
                        self.seed_cursor += 1;
                    }
                }
            }

            if backspace {
                if self.seed_selected {
                    self.seed_input.clear();
                    self.seed_cursor   = 0;
                    self.seed_scroll   = 0;
                    self.seed_selected = false;
                } else if self.seed_cursor > 0 {
                    self.seed_cursor -= 1;
                    self.seed_input.remove(self.seed_cursor);
                }
            }

            if ctrl_c { self.seed_selected = true; }

            if ctrl_v {
                if let Some(text) = clipboard_text {
                    let filtered: String = text.chars()
                        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
                        .take(SEED_MAX_LEN)
                        .collect();
                    if self.seed_selected {
                        self.seed_cursor   = filtered.len();
                        self.seed_scroll   = 0;
                        self.seed_input    = filtered;
                        self.seed_selected = false;
                    } else {
                        for c in filtered.chars() {
                            if self.seed_input.len() >= SEED_MAX_LEN { break; }
                            self.seed_input.insert(self.seed_cursor, c);
                            self.seed_cursor += 1;
                        }
                    }
                }
            }

            return MenuEvent::None;
        }

        if down && self.nav_cooldown == 0 {
            self.selected_row = (self.selected_row + 1) % SEED_ROWS;
            self.nav_cooldown = 8;
        }
        if up && self.nav_cooldown == 0 {
            self.selected_row = (self.selected_row + SEED_ROWS - 1) % SEED_ROWS;
            self.nav_cooldown = 8;
        }

        if confirm && self.selected_row == 0 && self.nav_cooldown == 0 {
            self.seed_focused = true;
            self.nav_cooldown = 8;
            return MenuEvent::None;
        }
        if confirm && self.selected_row == 1 && self.nav_cooldown == 0 {
            if !self.seed_input.is_empty() {
                return MenuEvent::ApplySeed(self.seed_input.clone());
            }
        }
        if confirm && self.selected_row == 2 && self.nav_cooldown == 0 {
            return MenuEvent::RandomSeed;
        }

        for &c in chars {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                self.seed_focused  = true;
                self.selected_row  = 0;
                if self.seed_selected {
                    self.seed_input.clear();
                    self.seed_cursor   = 0;
                    self.seed_scroll   = 0;
                    self.seed_selected = false;
                }
                if self.seed_input.len() < SEED_MAX_LEN {
                    self.seed_input.insert(self.seed_cursor, c);
                    self.seed_cursor += 1;
                }
            }
        }
        if backspace && !self.seed_input.is_empty() {
            self.seed_focused = true;
            self.selected_row = 0;
            if self.seed_selected {
                self.seed_input.clear();
                self.seed_cursor   = 0;
                self.seed_scroll   = 0;
                self.seed_selected = false;
            } else if self.seed_cursor > 0 {
                self.seed_cursor -= 1;
                self.seed_input.remove(self.seed_cursor);
            }
        }

        MenuEvent::None
    }

    pub(super) fn commit_fps_typing(&mut self) {
        let buf = self.fps_typing_buf.trim().to_string();
        if let Ok(val) = buf.parse::<f64>() {
            let val = val.max(1.0).min(9999.0);
            self.fps_millis = (val * 1000.0).round() as u32;
            self.fps_preset_idx = Self::find_nearest_fps_preset(self.fps_millis);
        }
        self.fps_edit_mode = FpsEditMode::Preset;
        self.fps_typing_buf.clear();
    }
}
