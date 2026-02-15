// src/app/pause_menu/types.rs
//
// Tous les types de données, constantes et fonctions utilitaires du menu pause.

use winit::keyboard::KeyCode;
use gilrs::Button as GButton;
use crate::app::settings::{WindowMode, AppSettings};

// ──────────────────────────────────────────────
// Structures de données — bindings
// ──────────────────────────────────────────────

/// Une action logique du joueur (ce qu'on peut remapper)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameAction {
    Forward,
    Backward,
    Left,
    Right,
    RotLeft,
    RotRight,
    Sprint,
    Slow,
    Shoot,
    Shockwave,
    ZoomIn,
    ZoomOut,
}

impl GameAction {
    pub const ALL: &'static [GameAction] = &[
        GameAction::Forward,
        GameAction::Backward,
        GameAction::Left,
        GameAction::Right,
        GameAction::RotLeft,
        GameAction::RotRight,
        GameAction::Sprint,
        GameAction::Slow,
        GameAction::Shoot,
        GameAction::Shockwave,
        GameAction::ZoomIn,
        GameAction::ZoomOut,
    ];

    pub fn label(self) -> &'static str {
        match self {
            GameAction::Forward    => "Avancer",
            GameAction::Backward   => "Reculer",
            GameAction::Left       => "Aller à gauche",
            GameAction::Right      => "Aller à droite",
            GameAction::RotLeft    => "Tourner gauche",
            GameAction::RotRight   => "Tourner droite",
            GameAction::Sprint     => "Sprint",
            GameAction::Slow       => "Ralentir",
            GameAction::Shoot      => "Tirer",
            GameAction::Shockwave  => "Onde de choc",
            GameAction::ZoomIn     => "Zoom +",
            GameAction::ZoomOut    => "Zoom -",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyBind {
    Key(KeyCode),
    MouseButton(u32), // 1=gauche, 3=droit, 2=milieu
}

impl KeyBind {
    pub fn label(self) -> String {
        match self {
            KeyBind::Key(k) => format!("{:?}", k),
            KeyBind::MouseButton(1) => "Souris G".into(),
            KeyBind::MouseButton(3) => "Souris D".into(),
            KeyBind::MouseButton(2) => "Souris M".into(),
            KeyBind::MouseButton(n) => format!("Souris {}", n),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamepadBind {
    Button(GButton),
    /// Joystick gauche — direction spécifique
    StickUp,
    StickDown,
    StickLeft,
    StickRight,
    None,
}

impl GamepadBind {
    pub fn label(self) -> &'static str {
        match self {
            GamepadBind::Button(GButton::South)          => "A / Croix",
            GamepadBind::Button(GButton::East)           => "B / Rond",
            GamepadBind::Button(GButton::North)          => "Y / Triangle",
            GamepadBind::Button(GButton::West)           => "X / Carré",
            GamepadBind::Button(GButton::RightTrigger)   => "R1 / RB",
            GamepadBind::Button(GButton::LeftTrigger)    => "L1 / LB",
            GamepadBind::Button(GButton::RightTrigger2)  => "R2 / RT",
            GamepadBind::Button(GButton::LeftTrigger2)   => "L2 / LT",
            GamepadBind::Button(GButton::DPadUp)         => "↑ D-Pad",
            GamepadBind::Button(GButton::DPadDown)       => "↓ D-Pad",
            GamepadBind::Button(GButton::DPadLeft)       => "← D-Pad",
            GamepadBind::Button(GButton::DPadRight)      => "→ D-Pad",
            GamepadBind::Button(GButton::Start)          => "Start",
            GamepadBind::Button(GButton::Select)         => "Select",
            GamepadBind::StickUp                         => "🕹 ↑",
            GamepadBind::StickDown                       => "🕹 ↓",
            GamepadBind::StickLeft                       => "🕹 ←",
            GamepadBind::StickRight                      => "🕹 →",
            GamepadBind::None                            => "—",
            _                                            => "Bouton",
        }
    }
}

/// Entrée complète pour une action
#[derive(Debug, Clone)]
pub struct Binding {
    pub key:     Option<KeyBind>,
    pub gamepad: GamepadBind,
}

impl Binding {
    pub fn key(k: KeyBind) -> Self { Self { key: Some(k), gamepad: GamepadBind::None } }
    pub fn mouse(n: u32)   -> Self { Self { key: Some(KeyBind::MouseButton(n)), gamepad: GamepadBind::None } }
    pub fn with_pad(mut self, b: GButton) -> Self { self.gamepad = GamepadBind::Button(b); self }
}

// ──────────────────────────────────────────────
// Valeurs FPS prédéfinies
// ──────────────────────────────────────────────

pub const FPS_PRESETS: &[(&str, u32)] = &[
    ("Illimitée",  0),
    ("23.976 fps", 23976),
    ("24 fps",     24000),
    ("25 fps",     25000),
    ("29.97 fps",  29970),
    ("30 fps",     30000),
    ("50 fps",     50000),
    ("59.94 fps",  59940),
    ("60 fps",     60000),
    ("75 fps",     75000),
    ("100 fps",   100000),
    ("119.88 fps",119880),
    ("120 fps",   120000),
    ("144 fps",   144000),
    ("165 fps",   165000),
    ("240 fps",   240000),
    ("360 fps",   360000),
    ("480 fps",   480000),
    ("1000 fps", 1000000),
    ("Illimitée",       0),
];

pub fn fps_label(fps_millis: u32) -> String {
    if fps_millis == 0 { return "Illimitée".into(); }
    for &(label, val) in FPS_PRESETS {
        if val == fps_millis { return label.into(); }
    }
    let fps = fps_millis as f64 / 1000.0;
    if fps.fract() < 0.001 { format!("{} fps", fps as u32) }
    else                   { format!("{:.3} fps", fps) }
}

pub fn fps_millis_to_frame_duration(fps_millis: u32) -> Option<std::time::Duration> {
    if fps_millis == 0 { None }
    else {
        let secs = 1000.0 / fps_millis as f64;
        Some(std::time::Duration::from_secs_f64(secs))
    }
}

// ──────────────────────────────────────────────
// Résolutions prédéfinies
// ──────────────────────────────────────────────

pub const RES_PRESETS: &[(&str, u32, u32)] = &[
    ("1280 × 720",  1280,  720),
    ("1366 × 768",  1366,  768),
    ("1600 × 900",  1600,  900),
    ("1920 × 1080", 1920, 1080),
    ("2560 × 1440", 2560, 1440),
    ("3840 × 2160", 3840, 2160),
];

// ──────────────────────────────────────────────
// Navigation du menu principal
// ──────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainMenuItem {
    Resume,
    Controls,
    Graphics,
    Seed,
    Quit,
}

impl MainMenuItem {
    pub const ALL: &'static [MainMenuItem] = &[
        MainMenuItem::Resume,
        MainMenuItem::Controls,
        MainMenuItem::Graphics,
        MainMenuItem::Seed,
        MainMenuItem::Quit,
    ];
    pub fn label(self) -> &'static str {
        match self {
            MainMenuItem::Resume   => "Reprendre",
            MainMenuItem::Controls => "Contrôles",
            MainMenuItem::Graphics => "Graphismes",
            MainMenuItem::Seed     => "Seed",
            MainMenuItem::Quit     => "Quitter",
        }
    }
}

// ──────────────────────────────────────────────
// Écran actif
// ──────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveScreen {
    Main,
    Controls,
    Graphics,
    Seed,
}

// ──────────────────────────────────────────────
// Sous-états
// ──────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlCol { Key, Pad }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FpsEditMode { Preset, Typing }

// ──────────────────────────────────────────────
// État complet du menu pause
// ──────────────────────────────────────────────

pub struct PauseMenu {
    pub active_screen: ActiveScreen,

    // ─── Menu principal ───
    pub main_row: usize,

    // ─── Contrôles ───
    pub selected_row:        usize,
    pub control_col:         ControlCol,
    pub awaiting_rebind_key: Option<(usize, ControlCol)>,
    pub bindings:            Vec<(GameAction, Binding)>,

    // ─── Graphismes ───
    /// 0 = résolution, 1 = mode fenêtre, 2 = taille UI, 3 = FPS activé, 4 = valeur FPS
    pub graphics_row:    usize,
    pub res_preset_idx:  usize,
    /// Résolution saisie/sélectionnée (appliquée au clic sur Appliquer)
    pub pending_width:   u32,
    pub pending_height:  u32,
    pub window_mode:     WindowMode,
    /// Échelle de l'interface (0.5 – 3.0)
    pub ui_scale:        f32,
    pub fps_enabled:     bool,
    pub fps_millis:      u32,
    pub(super) fps_edit_mode:  FpsEditMode,
    pub(super) fps_typing_buf: String,
    pub(super) fps_preset_idx: usize,

    // ─── Seed ───
    pub seed_input:    String,
    pub seed_cursor:   usize,
    pub seed_selected: bool,
    /// Décalage de scroll horizontal dans le champ seed (en caractères depuis le début)
    pub seed_scroll:   usize,
    /// Indique si le champ seed est "actif" (focus clavier + curseur visible)
    pub seed_focused:  bool,

    // ─── Affichage FPS ───
    pub show_fps: bool,

    // ─── Défilement du sous-menu Contrôles ───
    pub controls_scroll: usize,

    // ─── Navigation ───
    pub(super) nav_cooldown: u32,

    // ─── Paramètre résolution appliqué : flag pour render.rs ───
    pub pending_resize: Option<(u32, u32)>,
    pub pending_window_mode: Option<WindowMode>,

    // ─── Position de la flèche (screen px) : mis à jour par build_hud ───
    // (x_centre_fleche, y_centre_ligne, taille_police_medium)
    pub arrow_screen_pos: Option<(f32, f32, f32)>,

    // ─── Rectangles des éléments cliquables (mis à jour par build_hud) ───
    // Chaque entrée : (x, y, w, h, action_id)
    // action_id : 0=items menu principal (row=index), 1=apply seed, 2=random seed,
    //             10=seed field click, 20=controls rows
    pub clickable_rects: Vec<(f32, f32, f32, f32, u32, usize)>,

    // ─── Rectangle du champ seed (pour le rendu du fond+bordure) ───
    pub seed_field_rect: Option<(f32, f32, f32, f32)>,

    // ─── Curseur clignotant : phase temporelle (mis à jour par build_hud) ───
    pub cursor_blink_visible: bool,
}

/// Retourné par `handle_input`
pub enum MenuEvent {
    None,
    ApplySeed(String),         // Applique la seed et ferme le menu
    ApplySeedInMenu(String),   // Applique la seed sans fermer le menu
    RandomSeed,
    Close,
    Quit,                      // Ferme le jeu complètement
}

impl PauseMenu {
    pub fn new(settings: &AppSettings, current_seed: &str) -> Self {
        let fps_preset_idx  = Self::find_nearest_fps_preset(settings.fps_millis);
        let res_preset_idx  = Self::find_nearest_res_preset(settings.res_width, settings.res_height);

        Self {
            active_screen: ActiveScreen::Main,
            main_row:      0,

            selected_row:        0,
            control_col:         ControlCol::Key,
            awaiting_rebind_key: None,
            bindings:            settings.bindings.clone(),

            graphics_row:    0,
            res_preset_idx,
            pending_width:   settings.res_width,
            pending_height:  settings.res_height,
            window_mode:     settings.window_mode,
            ui_scale:        settings.ui_scale,
            fps_enabled:     settings.fps_enabled,
            fps_millis:      settings.fps_millis,
            fps_edit_mode:   FpsEditMode::Preset,
            fps_typing_buf:  String::new(),
            fps_preset_idx,

            seed_input:    current_seed.to_string(),
            seed_cursor:   current_seed.len(),
            seed_selected: false,
            seed_scroll:   0,
            seed_focused:  false,

            show_fps: settings.show_fps,

            controls_scroll: 0,

            nav_cooldown: 0,

            pending_resize:      None,
            pending_window_mode: None,

            arrow_screen_pos:   None,
            clickable_rects:    Vec::new(),
            seed_field_rect:    None,
            cursor_blink_visible: true,
        }
    }

    pub fn find_nearest_fps_preset(fps_millis: u32) -> usize {
        FPS_PRESETS.iter().enumerate()
            .min_by_key(|(_, &(_, v))| (v as i64 - fps_millis as i64).abs())
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    pub(super) fn find_nearest_res_preset(w: u32, h: u32) -> usize {
        RES_PRESETS.iter().enumerate()
            .min_by_key(|(_, &(_, pw, ph))| {
                let dw = pw as i64 - w as i64;
                let dh = ph as i64 - h as i64;
                dw * dw + dh * dh
            })
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    pub fn box_rect(&self, screen_w: f32, screen_h: f32) -> (f32, f32, f32, f32) {
        let s = self.ui_scale;
        let (box_w, box_h) = match self.active_screen {
            ActiveScreen::Main => {
                (260.0 * s, 350.0 * s)
            }
            ActiveScreen::Controls => {
                (620.0 * s, 440.0 * s)
            }
            ActiveScreen::Graphics => {
                (520.0 * s, 380.0 * s)
            }
            ActiveScreen::Seed => {
                (520.0 * s, 270.0 * s)
            }
        };
        let box_x = (screen_w - box_w) * 0.5;
        let box_y = ((screen_h - box_h) * 0.5).max(20.0);
        (box_x, box_y, box_w, box_h)
    }
}
