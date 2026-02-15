// src/app/settings.rs
//
// Sauvegarde/chargement des paramètres dans config.ini (format clé=valeur simple).
// Aucune dépendance externe : lecture/écriture manuelle.

use std::collections::HashMap;
use std::fs;

use crate::app::pause_menu::{GameAction, KeyBind, GamepadBind, Binding};
use winit::keyboard::KeyCode;
use gilrs::Button as GButton;

pub const CONFIG_PATH: &str = "config.ini";

// ──────────────────────────────────────────────
// Struct regroupant tous les paramètres
// ──────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowMode {
    Windowed,
    Fullscreen,
    BorderlessFullscreen,
}

impl WindowMode {
    pub fn label(self) -> &'static str {
        match self {
            WindowMode::Windowed             => "Fenêtré",
            WindowMode::Fullscreen           => "Plein écran",
            WindowMode::BorderlessFullscreen => "Plein écran fenêtré",
        }
    }
    pub fn all() -> &'static [WindowMode] {
        &[WindowMode::Windowed, WindowMode::Fullscreen, WindowMode::BorderlessFullscreen]
    }
    fn to_str(self) -> &'static str {
        match self {
            WindowMode::Windowed             => "windowed",
            WindowMode::Fullscreen           => "fullscreen",
            WindowMode::BorderlessFullscreen => "borderless",
        }
    }
    fn from_str(s: &str) -> Self {
        match s {
            "fullscreen" => WindowMode::Fullscreen,
            "borderless" => WindowMode::BorderlessFullscreen,
            _            => WindowMode::Windowed,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppSettings {
    // ─── Graphismes ───
    pub window_mode:   WindowMode,
    pub res_width:     u32,
    pub res_height:    u32,
    /// Échelle de l'interface (1.0 = normal). Multiplie toutes les tailles de police du menu.
    pub ui_scale:      f32,

    // ─── FPS ───
    pub fps_enabled:   bool,
    pub fps_millis:    u32,
    pub show_fps:      bool,

    // ─── Contrôles (dans le même ordre que GameAction::ALL) ───
    /// Chaque entrée : (key_bind_opt, gamepad_bind)
    pub bindings: Vec<(GameAction, Binding)>,
}

impl Default for AppSettings {
    fn default() -> Self {
        use KeyCode::*;
        use GameAction::*;
        let bindings = vec![
            (Forward,   Binding { key: Some(KeyBind::Key(KeyZ)),        gamepad: GamepadBind::StickUp    }),
            (Backward,  Binding { key: Some(KeyBind::Key(KeyS)),        gamepad: GamepadBind::StickDown  }),
            (Left,      Binding { key: Some(KeyBind::Key(KeyQ)),        gamepad: GamepadBind::StickLeft  }),
            (Right,     Binding { key: Some(KeyBind::Key(KeyD)),        gamepad: GamepadBind::StickRight }),
            (RotLeft,   Binding::key(KeyBind::Key(ArrowLeft)).with_pad(GButton::DPadLeft)),
            (RotRight,  Binding::key(KeyBind::Key(ArrowRight)).with_pad(GButton::DPadRight)),
            (Sprint,    Binding::key(KeyBind::Key(ControlLeft)).with_pad(GButton::South)),
            (Slow,      Binding::key(KeyBind::Key(AltLeft)).with_pad(GButton::East)),
            (Shoot,     Binding::mouse(1).with_pad(GButton::RightTrigger)),
            (Shockwave, Binding::mouse(3).with_pad(GButton::LeftTrigger)),
            (ZoomIn,    Binding::key(KeyBind::Key(Equal)).with_pad(GButton::RightTrigger2)),
            (ZoomOut,   Binding::key(KeyBind::Key(Minus)).with_pad(GButton::LeftTrigger2)),
        ];
        Self {
            window_mode: WindowMode::Windowed,
            res_width:   1900,
            res_height:  1000,
            ui_scale:    2.0,
            fps_enabled: true,
            fps_millis:  120_000,
            show_fps:    true,
            bindings,
        }
    }
}

// ──────────────────────────────────────────────
// Sauvegarde
// ──────────────────────────────────────────────

pub fn save(settings: &AppSettings) {
    let mut lines = Vec::<String>::new();

    lines.push("[graphics]".into());
    lines.push(format!("window_mode={}", settings.window_mode.to_str()));
    lines.push(format!("res_width={}",   settings.res_width));
    lines.push(format!("res_height={}",  settings.res_height));
    lines.push(format!("ui_scale={}",    settings.ui_scale));

    lines.push(String::new());
    lines.push("[fps]".into());
    lines.push(format!("fps_enabled={}", if settings.fps_enabled { "1" } else { "0" }));
    lines.push(format!("fps_millis={}",  settings.fps_millis));
    lines.push(format!("show_fps={}",    if settings.show_fps { "1" } else { "0" }));

    lines.push(String::new());
    lines.push("[controls]".into());
    for (action, binding) in &settings.bindings {
        let key_str = match &binding.key {
            Some(KeyBind::Key(k))         => format!("key:{:?}", k),
            Some(KeyBind::MouseButton(n)) => format!("mouse:{}", n),
            None                          => "none".into(),
        };
        let pad_str = match binding.gamepad {
            GamepadBind::Button(b)     => format!("pad:{:?}", b),
            GamepadBind::StickUp       => "pad:StickUp".into(),
            GamepadBind::StickDown     => "pad:StickDown".into(),
            GamepadBind::StickLeft     => "pad:StickLeft".into(),
            GamepadBind::StickRight    => "pad:StickRight".into(),
            GamepadBind::None          => "pad:none".into(),
        };
        lines.push(format!("{:?}={},{}", action, key_str, pad_str));
    }

    let content = lines.join("\n") + "\n";
    if let Err(e) = fs::write(CONFIG_PATH, &content) {
        log::warn!("Impossible d'écrire config.ini : {}", e);
    }
}

// ──────────────────────────────────────────────
// Chargement
// ──────────────────────────────────────────────

pub fn load() -> AppSettings {
    let mut s = AppSettings::default();

    let text = match fs::read_to_string(CONFIG_PATH) {
        Ok(t)  => t,
        Err(_) => return s, // premier lancement : valeurs par défaut
    };

    let mut map: HashMap<String, String> = HashMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('[') || line.starts_with('#') { continue; }
        if let Some((k, v)) = line.split_once('=') {
            map.insert(k.trim().to_string(), v.trim().to_string());
        }
    }

    // ─── Graphismes ───
    if let Some(v) = map.get("window_mode") {
        s.window_mode = WindowMode::from_str(v);
    }
    if let Some(v) = map.get("res_width") {
        if let Ok(n) = v.parse::<u32>() { s.res_width = n.max(640); }
    }
    if let Some(v) = map.get("res_height") {
        if let Ok(n) = v.parse::<u32>() { s.res_height = n.max(480); }
    }
    if let Some(v) = map.get("ui_scale") {
        if let Ok(f) = v.parse::<f32>() { s.ui_scale = f.clamp(0.5, 3.0); }
    }

    // ─── FPS ───
    if let Some(v) = map.get("fps_enabled") {
        s.fps_enabled = v == "1" || v == "true";
    }
    if let Some(v) = map.get("fps_millis") {
        if let Ok(n) = v.parse::<u32>() { s.fps_millis = n; }
    }
    if let Some(v) = map.get("show_fps") {
        s.show_fps = v == "1" || v == "true";
    }

    // ─── Contrôles ───
    for (action, binding) in &mut s.bindings {
        let key_name = format!("{:?}", action);
        if let Some(val) = map.get(&key_name) {
            let parts: Vec<&str> = val.splitn(2, ',').collect();
            if parts.len() >= 1 {
                binding.key = parse_key_bind(parts[0]);
            }
            if parts.len() >= 2 {
                binding.gamepad = parse_gamepad_bind(parts[1]);
            }
        }
    }

    s
}

// ──────────────────────────────────────────────
// Parseurs internes
// ──────────────────────────────────────────────

fn parse_key_bind(s: &str) -> Option<KeyBind> {
    if s == "none" { return None; }
    if let Some(rest) = s.strip_prefix("mouse:") {
        if let Ok(n) = rest.parse::<u32>() { return Some(KeyBind::MouseButton(n)); }
    }
    if let Some(rest) = s.strip_prefix("key:") {
        if let Some(k) = keycode_from_str(rest) { return Some(KeyBind::Key(k)); }
    }
    None
}

fn parse_gamepad_bind(s: &str) -> GamepadBind {
    if s == "pad:none"        { return GamepadBind::None; }
    if s == "pad:StickUp"     { return GamepadBind::StickUp; }
    if s == "pad:StickDown"   { return GamepadBind::StickDown; }
    if s == "pad:StickLeft"   { return GamepadBind::StickLeft; }
    if s == "pad:StickRight"  { return GamepadBind::StickRight; }
    if let Some(rest) = s.strip_prefix("pad:") {
        if let Some(b) = gbutton_from_str(rest) { return GamepadBind::Button(b); }
    }
    GamepadBind::None
}

/// Convertit une chaîne Debug-style de KeyCode en KeyCode.
/// On ne couvre que les codes utilisés par défaut + ceux couramment remappés.
fn keycode_from_str(s: &str) -> Option<KeyCode> {
    use KeyCode::*;
    Some(match s {
        "KeyA" => KeyA, "KeyB" => KeyB, "KeyC" => KeyC, "KeyD" => KeyD,
        "KeyE" => KeyE, "KeyF" => KeyF, "KeyG" => KeyG, "KeyH" => KeyH,
        "KeyI" => KeyI, "KeyJ" => KeyJ, "KeyK" => KeyK, "KeyL" => KeyL,
        "KeyM" => KeyM, "KeyN" => KeyN, "KeyO" => KeyO, "KeyP" => KeyP,
        "KeyQ" => KeyQ, "KeyR" => KeyR, "KeyS" => KeyS, "KeyT" => KeyT,
        "KeyU" => KeyU, "KeyV" => KeyV, "KeyW" => KeyW, "KeyX" => KeyX,
        "KeyY" => KeyY, "KeyZ" => KeyZ,
        "Digit0" => Digit0, "Digit1" => Digit1, "Digit2" => Digit2,
        "Digit3" => Digit3, "Digit4" => Digit4, "Digit5" => Digit5,
        "Digit6" => Digit6, "Digit7" => Digit7, "Digit8" => Digit8,
        "Digit9" => Digit9,
        "F1" => F1, "F2" => F2, "F3" => F3, "F4" => F4,
        "F5" => F5, "F6" => F6, "F7" => F7, "F8" => F8,
        "F9" => F9, "F10" => F10, "F11" => F11, "F12" => F12,
        "ArrowUp" => ArrowUp, "ArrowDown" => ArrowDown,
        "ArrowLeft" => ArrowLeft, "ArrowRight" => ArrowRight,
        "Space" => Space, "Enter" => Enter, "Escape" => Escape,
        "Backspace" => Backspace, "Tab" => Tab,
        "ShiftLeft" => ShiftLeft, "ShiftRight" => ShiftRight,
        "ControlLeft" => ControlLeft, "ControlRight" => ControlRight,
        "AltLeft" => AltLeft, "AltRight" => AltRight,
        "Equal" => Equal, "Minus" => Minus,
        "BracketLeft" => BracketLeft, "BracketRight" => BracketRight,
        "Semicolon" => Semicolon, "Quote" => Quote,
        "Comma" => Comma, "Period" => Period, "Slash" => Slash,
        "Backslash" => Backslash, "Backquote" => Backquote,
        "Numpad0" => Numpad0, "Numpad1" => Numpad1, "Numpad2" => Numpad2,
        "Numpad3" => Numpad3, "Numpad4" => Numpad4, "Numpad5" => Numpad5,
        "Numpad6" => Numpad6, "Numpad7" => Numpad7, "Numpad8" => Numpad8,
        "Numpad9" => Numpad9,
        "NumpadAdd" => NumpadAdd, "NumpadSubtract" => NumpadSubtract,
        "NumpadMultiply" => NumpadMultiply, "NumpadDivide" => NumpadDivide,
        "NumpadEnter" => NumpadEnter,
        "Home" => Home, "End" => End, "PageUp" => PageUp, "PageDown" => PageDown,
        "Insert" => Insert, "Delete" => Delete,
        _ => return None,
    })
}

fn gbutton_from_str(s: &str) -> Option<GButton> {
    use GButton::*;
    Some(match s {
        "South"          => South,
        "East"           => East,
        "North"          => North,
        "West"           => West,
        "RightTrigger"   => RightTrigger,
        "LeftTrigger"    => LeftTrigger,
        "RightTrigger2"  => RightTrigger2,
        "LeftTrigger2"   => LeftTrigger2,
        "DPadUp"         => DPadUp,
        "DPadDown"       => DPadDown,
        "DPadLeft"       => DPadLeft,
        "DPadRight"      => DPadRight,
        "Start"          => Start,
        "Select"         => Select,
        "LeftThumb"      => LeftThumb,
        "RightThumb"     => RightThumb,
        _                => return None,
    })
}
