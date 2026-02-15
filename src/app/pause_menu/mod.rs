// src/app/pause_menu/mod.rs

mod types;
mod input;
mod hud_builder;

// Re-exports publics (même interface qu'avant pour le reste du crate)
pub use types::{
    GameAction, KeyBind, GamepadBind, Binding,
    fps_millis_to_frame_duration,
    ActiveScreen,
    PauseMenu, MenuEvent,
};

// Re-exports supplémentaires utilisés par d'autres modules du crate
pub use types::{FPS_PRESETS, RES_PRESETS, fps_label, MainMenuItem, ControlCol, FpsEditMode};
