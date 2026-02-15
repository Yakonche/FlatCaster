// src/app/game_state.rs
use std::time::Instant;
use crate::config::*;
use crate::player::Player;
use crate::map::MapHandler;
use crate::entities::EntityManager;
use crate::input::InputManager;
use super::pause_menu::PauseMenu;
use super::settings::{self, AppSettings};

pub struct GameState {
    pub player: Player,
    pub map: MapHandler,
    pub entity_manager: EntityManager,
    pub input_manager: InputManager,
    pub seed: String,
    pub width: u32,
    pub height: u32,
    pub strip_height: f32,
    pub zoom_level: f32,
    pub freeze_entities: bool,
    pub paused: bool,
    pub pause_menu: PauseMenu,
    pub gc_timer: u32,
    pub frame_count: u32,
    pub last_fps_time: Instant,
    pub last_frame_time: Instant,
    pub fps: f32,
    pub time_ms: u32,
    pub start_time: Instant,
    pub is_dragging_separator: bool,
    pub hover_separator: bool,
    /// FPS en millièmes (120000 = 120 fps). Synchronisé depuis pause_menu.fps_millis.
    pub fps_millis: u32,
    /// La limite est-elle active ? Synchronisé depuis pause_menu.fps_enabled.
    pub fps_enabled: bool,
    /// Instant du dernier frame rendu (pour le throttle FPS)
    pub last_throttle_time: Instant,
    /// Paramètres persistants (chargés depuis config.ini)
    pub settings: AppSettings,
}

impl GameState {
    pub fn new(width: u32, height: u32) -> Self {
        let seed = generate_random_seed();
        println!("--- SEED DU MONDE : {} ---", seed);

        // Charge les paramètres depuis config.ini (valeurs par défaut si absent)
        let loaded = settings::load();
        let fps_millis  = loaded.fps_millis;
        let fps_enabled = loaded.fps_enabled;
        let pause_menu  = PauseMenu::new(&loaded, &seed);

        Self {
            player: Player::new(),
            map: MapHandler::new(&seed),
            entity_manager: EntityManager::new(&seed),
            input_manager: InputManager::new(),
            pause_menu,
            seed,
            width,
            height,
            strip_height: DEFAULT_STRIP_HEIGHT,
            zoom_level: 1.0,
            freeze_entities: false,
            paused: false,
            gc_timer: 0,
            frame_count: 0,
            last_fps_time: Instant::now(),
            last_frame_time: Instant::now(),
            fps: 0.0,
            time_ms: 0,
            start_time: Instant::now(),
            is_dragging_separator: false,
            hover_separator: false,
            fps_millis,
            fps_enabled,
            last_throttle_time: Instant::now(),
            settings: loaded,
        }
    }

    /// Recrée la carte et les entités avec une nouvelle seed, replace le joueur.
    pub fn apply_seed(&mut self, new_seed: String) {
        println!("--- NOUVELLE SEED : {} ---", new_seed);
        self.seed = new_seed.clone();
        self.map = MapHandler::new(&new_seed);
        self.entity_manager = EntityManager::new(&new_seed);
        self.player = Player::new();
        self.time_ms = 0;
        self.start_time = Instant::now();
        self.pause_menu.seed_input = new_seed;
    }

    /// Sauvegarde les paramètres actuels dans config.ini
    pub fn save_settings(&mut self) {
        self.settings.fps_enabled  = self.pause_menu.fps_enabled;
        self.settings.fps_millis   = self.pause_menu.fps_millis;
        self.settings.show_fps     = self.pause_menu.show_fps;
        self.settings.window_mode  = self.pause_menu.window_mode;
        self.settings.res_width    = self.pause_menu.pending_width;
        self.settings.res_height   = self.pause_menu.pending_height;
        self.settings.ui_scale     = self.pause_menu.ui_scale;
        self.settings.bindings     = self.pause_menu.bindings.clone();
        settings::save(&self.settings);
    }
}
