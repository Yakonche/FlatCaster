// src/app/mod.rs
pub mod gpu_state;
pub mod game_state;
pub mod render;
pub mod pause_menu;
pub mod settings;

use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::{WindowEvent, MouseScrollDelta},
    event_loop::ActiveEventLoop,
    window::{Window, WindowId, Icon},
    dpi::PhysicalSize,
};
use crate::config::*;

pub use gpu_state::GpuState;
pub use game_state::GameState;

pub struct App {
    pub window: Option<Arc<Window>>,
    pub gpu: Option<GpuState>,
    pub game: Option<GameState>,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        Self { window: None, gpu: None, game: None, should_quit: false }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let icon = load_icon();
            let attrs = Window::default_attributes()
                .with_title("2D First-Person Viewer")
                .with_inner_size(PhysicalSize::new(RES_WIDTH, RES_HEIGHT))
                .with_window_icon(icon);
            let window = Arc::new(event_loop.create_window(attrs).expect("Failed to create window"));

            let size = window.inner_size();
            let width = size.width.max(1);
            let height = size.height.max(1);

            self.gpu = Some(GpuState::new(window.clone()));
            self.game = Some(GameState::new(width, height));
            self.window = Some(window);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let game = match self.game.as_mut() {
            Some(g) => g,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(gpu) = self.gpu.as_mut() {
                    let w = size.width.max(1);
                    let h = size.height.max(1);
                    gpu.config.width = w;
                    gpu.config.height = h;
                    gpu.surface.configure(&gpu.device, &gpu.config);
                    gpu.raycaster.resize(&gpu.device, w);
                    gpu.renderer.hud_text.update_resolution(&gpu.queue, w, h);
                    game.width = w;
                    game.height = h;
                    game.strip_height = game.strip_height.clamp(50.0, h as f32 - 50.0);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                game.input_manager.on_key_event(&event);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                game.input_manager.on_mouse_button(button, state);
            }
            WindowEvent::CursorMoved { position, .. } => {
                game.input_manager.on_mouse_move(position.x, position.y);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let y = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(p) => p.y as f32 / 120.0,
                };
                game.input_manager.on_scroll(y);
            }
            WindowEvent::RedrawRequested => {
                render::render(self);
                if self.should_quit {
                    event_loop.exit();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

fn load_icon() -> Option<Icon> {
    let bytes = include_bytes!("../../assets/icon.ico");
    // Parse ICO: skip header (6 bytes) + first entry (16 bytes) to get image offset/size,
    // then decode the embedded PNG or BMP. Use a simple RGBA extraction via raw ICO parsing.
    parse_ico_to_icon(bytes)
}

fn parse_ico_to_icon(data: &[u8]) -> Option<Icon> {
    if data.len() < 6 { return None; }
    // ICO header: reserved(2) + type(2) + count(2)
    let count = u16::from_le_bytes([data[4], data[5]]) as usize;
    if count == 0 || data.len() < 6 + count * 16 { return None; }

    // Pick the largest image entry
    let mut best_size = 0u32;
    let mut best_offset = 0usize;
    let mut best_data_size = 0usize;

    for i in 0..count {
        let entry = &data[6 + i * 16..6 + (i + 1) * 16];
        let w = if entry[0] == 0 { 256u32 } else { entry[0] as u32 };
        let h = if entry[1] == 0 { 256u32 } else { entry[1] as u32 };
        let sz = w * h;
        let data_size = u32::from_le_bytes([entry[8], entry[9], entry[10], entry[11]]) as usize;
        let offset = u32::from_le_bytes([entry[12], entry[13], entry[14], entry[15]]) as usize;
        if sz > best_size {
            best_size = sz;
            best_offset = offset;
            best_data_size = data_size;
        }
    }

    if best_offset + best_data_size > data.len() { return None; }
    let img_data = &data[best_offset..best_offset + best_data_size];

    // Check if it's a PNG (starts with PNG magic)
    if img_data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
        decode_png_to_icon(img_data)
    } else {
        // BMP DIB header embedded in ICO
        decode_bmp_dib_to_icon(img_data)
    }
}

fn decode_png_to_icon(png: &[u8]) -> Option<Icon> {
    // Minimal PNG decoder for RGBA/RGB images
    // We'll use a simple approach: find IHDR for dimensions, then IDAT for pixels
    // For simplicity, delegate to a known-simple path using raw chunk parsing
    // Since we don't have the `image` crate, we'll try to use BMP fallback instead.
    // If the ICO contains a PNG, we need at least basic support.
    // As a fallback, return None and let the OS use the embedded exe icon.
    let _ = png;
    None
}

fn decode_bmp_dib_to_icon(dib: &[u8]) -> Option<Icon> {
    if dib.len() < 40 { return None; }
    // BITMAPINFOHEADER
    let width = i32::from_le_bytes([dib[4], dib[5], dib[6], dib[7]]).unsigned_abs();
    let height_raw = i32::from_le_bytes([dib[8], dib[9], dib[10], dib[11]]);
    // In ICO BMP the height is doubled (image + mask), actual height = abs(height)/2
    let height = (height_raw.unsigned_abs() / 2) as usize;
    let width = width as usize;
    let bit_count = u16::from_le_bytes([dib[14], dib[15]]) as usize;

    if bit_count != 32 && bit_count != 24 { return None; }

    let header_size = u32::from_le_bytes([dib[0], dib[1], dib[2], dib[3]]) as usize;
    let pixel_offset = header_size; // No color table for 24/32-bit
    let row_size = ((bit_count * width + 31) / 32) * 4;

    if pixel_offset + row_size * height > dib.len() { return None; }

    let mut rgba = vec![0u8; width * height * 4];
    for y in 0..height {
        // BMP rows are bottom-up
        let src_row = height - 1 - y;
        let src = &dib[pixel_offset + src_row * row_size..];
        for x in 0..width {
            let dst = (y * width + x) * 4;
            if bit_count == 32 {
                let s = x * 4;
                rgba[dst]     = src[s + 2]; // R
                rgba[dst + 1] = src[s + 1]; // G
                rgba[dst + 2] = src[s];     // B
                rgba[dst + 3] = src[s + 3]; // A
            } else {
                let s = x * 3;
                rgba[dst]     = src[s + 2]; // R
                rgba[dst + 1] = src[s + 1]; // G
                rgba[dst + 2] = src[s];     // B
                rgba[dst + 3] = 255;         // A
            }
        }
    }

    Icon::from_rgba(rgba, width as u32, height as u32).ok()
}
