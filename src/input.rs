// src/input.rs
use winit::event::{ElementState, KeyEvent, MouseButton};
use winit::keyboard::{KeyCode, PhysicalKey};
use gilrs::{Gilrs, Button, Axis, Event as GilrsEvent};

/// Holds the current frame's input state
#[derive(Default)]
pub struct InputState {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub rot_left: bool,
    pub rot_right: bool,

    // Actions
    pub shoot: bool,
    pub shockwave: bool,

    // Modifiers / Zoom
    pub sprint: bool,
    pub slow: bool,
    pub zoom_in: bool,
    pub zoom_out: bool,

    // Gamepad analog
    pub move_x: f32,
    pub move_y: f32,
    pub look_x: f32,
    pub look_y: f32,

    // Mouse
    pub scroll_delta: f32,
    pub mouse_x: f64,
    pub mouse_y: f64,
    pub mouse_left_down: bool,
    pub mouse_left_clicked: bool,  // edge : vrai uniquement le frame du clic
    /// Angle (en radians) vers la position souris depuis le centre de la vue 2D, calculé par game_update
    pub aim_angle: Option<f32>,

    // Menu — tous en edge (un seul déclenchement par appui)
    pub escape_pressed: bool,
    pub menu_up: bool,
    pub menu_down: bool,
    pub menu_left: bool,
    pub menu_right: bool,
    pub menu_confirm: bool,       // edge : Enter ou Space
    pub menu_back: bool,          // B / East — retour arrière dans le menu
    /// Flèche gauche pure (sans Q/A) — déplacement curseur texte
    pub arrow_left: bool,
    /// Flèche droite pure (sans D) — déplacement curseur texte
    pub arrow_right: bool,
    /// Flèche haut pure (sans Z/W) — déplacement curseur texte
    pub arrow_up: bool,
    /// Flèche bas pure (sans S) — déplacement curseur texte
    pub arrow_down: bool,

    // Saisie texte (pour l'éditeur de seed dans le menu pause)
    pub chars_typed: Vec<char>,
    pub backspace_pressed: bool,
    pub ctrl_c: bool,
    pub ctrl_v: bool,
    /// Dernière touche physique pressée ce frame (pour le rebind)
    pub new_key_pressed: Option<winit::keyboard::KeyCode>,
    /// Dernier bouton souris pressé ce frame (pour le rebind)
    pub new_mouse_pressed: Option<u32>,
}

pub struct InputManager {
    pub gilrs: Option<Gilrs>,
    keys: std::collections::HashSet<KeyCode>,
    mouse_buttons: std::collections::HashSet<u32>,
    pub scroll_delta: f32,
    escape_edge: bool,

    // ── Edges clavier pour le menu (un seul déclenchement par appui) ──
    key_up_edge:    bool,
    key_down_edge:  bool,
    key_left_edge:  bool,
    key_right_edge: bool,
    // ── Edges flèches pures (sans les touches lettres) ──
    arrow_left_edge:  bool,
    arrow_right_edge: bool,
    arrow_up_edge:    bool,
    arrow_down_edge:  bool,
    // ── Edge confirm (Enter/Space) ──
    confirm_edge: bool,

    // ── Edges manette pour le menu (détectés via événements, pas is_pressed) ──
    gamepad_start_edge:  bool,  // Start  → escape_pressed
    gamepad_east_edge:   bool,  // B/East → menu_back
    gamepad_south_edge:  bool,  // A/South → menu_confirm
    gamepad_up_edge:     bool,
    gamepad_down_edge:   bool,
    gamepad_left_edge:   bool,
    gamepad_right_edge:  bool,

    // Joystick analogique pour le menu (cooldown interne)
    stick_nav_cooldown: u32,

    mouse_x: f64,
    mouse_y: f64,
    mouse_left_edge: bool,
    // Saisie texte
    chars_typed: Vec<char>,
    backspace_edge: bool,
    ctrl_c_edge: bool,
    ctrl_v_edge: bool,
    new_key_edge: Option<KeyCode>,
    new_mouse_edge: Option<u32>,
}

impl InputManager {
    pub fn new() -> Self {
        let gilrs = Gilrs::new().ok();
        if let Some(ref g) = gilrs {
            for (_id, gamepad) in g.gamepads() {
                log::info!("Gamepad detected: {} ({})", gamepad.name(), gamepad.id());
            }
        }
        Self {
            gilrs,
            keys: std::collections::HashSet::new(),
            mouse_buttons: std::collections::HashSet::new(),
            scroll_delta: 0.0,
            escape_edge: false,
            key_up_edge:    false,
            key_down_edge:  false,
            key_left_edge:  false,
            key_right_edge: false,
            gamepad_start_edge:  false,
            gamepad_east_edge:   false,
            gamepad_south_edge:  false,
            gamepad_up_edge:     false,
            gamepad_down_edge:   false,
            gamepad_left_edge:   false,
            gamepad_right_edge:  false,
            stick_nav_cooldown:  0,
            arrow_left_edge:  false,
            arrow_right_edge: false,
            arrow_up_edge:    false,
            arrow_down_edge:  false,
            confirm_edge:     false,
            mouse_x: 0.0,
            mouse_y: 0.0,
            mouse_left_edge: false,
            chars_typed: Vec::new(),
            backspace_edge: false,
            ctrl_c_edge: false,
            ctrl_v_edge: false,
            new_key_edge: None,
            new_mouse_edge: None,
        }
    }

    pub fn on_key_event(&mut self, event: &KeyEvent) {
        if let PhysicalKey::Code(code) = event.physical_key {
            match event.state {
                ElementState::Pressed => {
                    if code == KeyCode::Escape {
                        self.escape_edge = true;
                    }
                    if code == KeyCode::Backspace {
                        self.backspace_edge = true;
                    }
                    // Flèches et Z/S/Q/D : edge pour la navigation menu
                    if code == KeyCode::ArrowUp   || code == KeyCode::KeyZ || code == KeyCode::KeyW {
                        self.key_up_edge = true;
                    }
                    if code == KeyCode::ArrowDown || code == KeyCode::KeyS {
                        self.key_down_edge = true;
                    }
                    if code == KeyCode::ArrowLeft  || code == KeyCode::KeyQ || code == KeyCode::KeyA { self.key_left_edge  = true; }
                    if code == KeyCode::ArrowRight || code == KeyCode::KeyD { self.key_right_edge = true; }
                    // Flèches pures (sans lettres) : pour déplacement curseur texte
                    if code == KeyCode::ArrowLeft  { self.arrow_left_edge  = true; }
                    if code == KeyCode::ArrowRight { self.arrow_right_edge = true; }
                    if code == KeyCode::ArrowUp    { self.arrow_up_edge    = true; }
                    if code == KeyCode::ArrowDown  { self.arrow_down_edge  = true; }
                    // Confirm edge : Enter ou Space
                    if code == KeyCode::Enter || code == KeyCode::NumpadEnter || code == KeyCode::Space {
                        self.confirm_edge = true;
                    }
                    // Ctrl+C / Ctrl+V
                    let ctrl = self.keys.contains(&KeyCode::ControlLeft)
                        || self.keys.contains(&KeyCode::ControlRight);
                    if ctrl && code == KeyCode::KeyC { self.ctrl_c_edge = true; }
                    if ctrl && code == KeyCode::KeyV { self.ctrl_v_edge = true; }

                    // Saisie texte (pour l'éditeur de seed)
                    if let winit::keyboard::Key::Character(ref s) = event.logical_key {
                        for c in s.chars() {
                            self.chars_typed.push(c);
                        }
                    }

                    self.new_key_edge = Some(code);
                    self.keys.insert(code);
                }
                ElementState::Released => {
                    self.keys.remove(&code);
                }
            }
        }
    }

    pub fn on_mouse_button(&mut self, button: MouseButton, state: ElementState) {
        let idx = match button {
            MouseButton::Left => 1,
            MouseButton::Right => 3,
            MouseButton::Middle => 2,
            MouseButton::Back => 4,
            MouseButton::Forward => 5,
            _ => 0,
        };
        match state {
            ElementState::Pressed => {
                self.mouse_buttons.insert(idx);
                self.new_mouse_edge = Some(idx);
                if idx == 1 { self.mouse_left_edge = true; }
            }
            ElementState::Released => { self.mouse_buttons.remove(&idx); }
        }
    }

    pub fn on_mouse_move(&mut self, x: f64, y: f64) {
        self.mouse_x = x;
        self.mouse_y = y;
    }

    pub fn on_scroll(&mut self, delta: f32) {
        self.scroll_delta += delta;
    }

    pub fn poll_gamepad(&mut self) {
        if let Some(ref mut gilrs) = self.gilrs {
            while let Some(GilrsEvent { event, .. }) = gilrs.next_event() {
                use gilrs::EventType;
                match event {
                    EventType::ButtonPressed(btn, _) => {
                        match btn {
                            Button::Start      => self.gamepad_start_edge = true,
                            Button::East       => self.gamepad_east_edge  = true,
                            Button::South      => self.gamepad_south_edge = true,
                            Button::DPadUp     => self.gamepad_up_edge    = true,
                            Button::DPadDown   => self.gamepad_down_edge  = true,
                            Button::DPadLeft   => self.gamepad_left_edge  = true,
                            Button::DPadRight  => self.gamepad_right_edge = true,
                            _                  => {}
                        }
                    }
                    EventType::Connected    => log::info!("Manette connectée"),
                    EventType::Disconnected => log::info!("Manette déconnectée"),
                    _ => {}
                }
            }
        } else {
            if let Ok(g) = Gilrs::new() {
                self.gilrs = Some(g);
            }
        }
    }

    pub fn get_state(&mut self) -> InputState {
        let mut state = InputState::default();

        // Keyboard bindings (ZQSD + arrows)
        state.forward = self.keys.contains(&KeyCode::KeyZ) || self.keys.contains(&KeyCode::KeyW);
        state.backward = self.keys.contains(&KeyCode::KeyS);
        state.left = self.keys.contains(&KeyCode::KeyQ) || self.keys.contains(&KeyCode::KeyA);
        state.right = self.keys.contains(&KeyCode::KeyD);
        state.rot_left = self.keys.contains(&KeyCode::ArrowLeft);
        state.rot_right = self.keys.contains(&KeyCode::ArrowRight);
        state.sprint = self.keys.contains(&KeyCode::ControlLeft);
        state.slow = self.keys.contains(&KeyCode::AltLeft);

        // Mouse
        // Le click gauche est désormais géré par game_update pour orienter/avancer le joueur
        state.shockwave = self.mouse_buttons.contains(&3);
        state.mouse_x = self.mouse_x;
        state.mouse_y = self.mouse_y;
        state.mouse_left_down = self.mouse_buttons.contains(&1);
        state.mouse_left_clicked = self.mouse_left_edge;
        self.mouse_left_edge = false;

        // Scroll
        state.scroll_delta = self.scroll_delta;
        self.scroll_delta = 0.0;

        // Escape edge detection
        state.escape_pressed = self.escape_edge;
        self.escape_edge = false;

        // Saisie texte
        state.chars_typed = std::mem::take(&mut self.chars_typed);
        state.backspace_pressed = self.backspace_edge;
        self.backspace_edge = false;
        state.ctrl_c = self.ctrl_c_edge;
        self.ctrl_c_edge = false;
        state.ctrl_v = self.ctrl_v_edge;
        self.ctrl_v_edge = false;
        state.new_key_pressed = self.new_key_edge.take();
        state.new_mouse_pressed = self.new_mouse_edge.take();

        // Menu clavier (edge-triggered pour flèches, Z, S/Q/D — comme la manette D-pad)
        if self.key_up_edge    { state.menu_up    = true; self.key_up_edge    = false; }
        if self.key_down_edge  { state.menu_down  = true; self.key_down_edge  = false; }
        if self.key_left_edge  { state.menu_left  = true; self.key_left_edge  = false; }
        if self.key_right_edge { state.menu_right = true; self.key_right_edge = false; }
        // Flèches pures (pour déplacement curseur dans le champ seed)
        if self.arrow_left_edge  { state.arrow_left  = true; self.arrow_left_edge  = false; }
        if self.arrow_right_edge { state.arrow_right = true; self.arrow_right_edge = false; }
        if self.arrow_up_edge    { state.arrow_up    = true; self.arrow_up_edge    = false; }
        if self.arrow_down_edge  { state.arrow_down  = true; self.arrow_down_edge  = false; }
        // Confirm : edge (un seul déclenchement par appui)
        if self.confirm_edge { state.menu_confirm = true; self.confirm_edge = false; }

        // Gamepad Start → escape_pressed (edge)
        if self.gamepad_start_edge {
            state.escape_pressed = true;
            self.gamepad_start_edge = false;
        }

        // Gamepad B/East → menu_back (edge)
        if self.gamepad_east_edge {
            state.menu_back = true;
            self.gamepad_east_edge = false;
        }

        // Gamepad A/South → menu_confirm (edge, séparé de sprint en jeu)
        if self.gamepad_south_edge {
            state.menu_confirm = true;
            self.gamepad_south_edge = false;
        }

        // Gamepad D-Pad → menu navigation (edges)
        if self.gamepad_up_edge    { state.menu_up    = true; self.gamepad_up_edge    = false; }
        if self.gamepad_down_edge  { state.menu_down  = true; self.gamepad_down_edge  = false; }
        if self.gamepad_left_edge  { state.menu_left  = true; self.gamepad_left_edge  = false; }
        if self.gamepad_right_edge { state.menu_right = true; self.gamepad_right_edge = false; }

        // Joystick gauche → menu navigation avec cooldown (zone morte 0.4)
        if self.stick_nav_cooldown > 0 {
            self.stick_nav_cooldown -= 1;
        }
        if let Some(ref gilrs) = self.gilrs {
            if let Some((_id, gamepad)) = gilrs.gamepads().next() {
                let sx = gamepad.value(Axis::LeftStickX);
                let sy = -gamepad.value(Axis::LeftStickY); // Y inversé : haut = négatif sur la plupart des manettes
                const DEAD: f32 = 0.55;
                if self.stick_nav_cooldown == 0 {
                    if sy < -DEAD { state.menu_up    = true; self.stick_nav_cooldown = 30; }
                    if sy >  DEAD { state.menu_down  = true; self.stick_nav_cooldown = 30; }
                    if sx < -DEAD { state.menu_left  = true; self.stick_nav_cooldown = 30; }
                    if sx >  DEAD { state.menu_right = true; self.stick_nav_cooldown = 30; }
                }
            }
        }

        // Gamepad (état continu — uniquement pour le jeu, pas le menu)
        if let Some(ref gilrs) = self.gilrs {
            if let Some((_id, gamepad)) = gilrs.gamepads().next() {
                state.move_x = gamepad.value(Axis::LeftStickX);
                state.move_y = -gamepad.value(Axis::LeftStickY);
                state.look_x = gamepad.value(Axis::RightStickX);
                state.look_y = -gamepad.value(Axis::RightStickY);

                if gamepad.is_pressed(Button::RightTrigger2) { state.zoom_in  = true; }
                if gamepad.is_pressed(Button::LeftTrigger2)  { state.zoom_out = true; }
                if gamepad.is_pressed(Button::RightTrigger)  { state.shoot    = true; }
                if gamepad.is_pressed(Button::LeftTrigger)   { state.shockwave = true; }
                // Sprint/Slow : continu, uniquement hors menu (jeu)
                if gamepad.is_pressed(Button::South) { state.sprint = true; }
                if gamepad.is_pressed(Button::East)  { state.slow   = true; }
            }
        }

        state
    }
}
