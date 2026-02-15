// src/app/render/mod.rs

mod arrow;
mod frame_limiter;
mod game_update;
mod map_data;

use std::time::Instant;
use winit::window::CursorIcon;
use bytemuck;
use crate::config::*;
use crate::renderer::{WallInstance, MapLineUniforms, FovUniforms, TextEntry};
use crate::raycasting::RayCaster;
use super::App;
use super::pause_menu::{MenuEvent, ActiveScreen};

pub fn render(app: &mut App) {
    let gpu = app.gpu.as_mut().unwrap();
    let game = app.game.as_mut().unwrap();

    // ─── FPS throttle ───────────────────────────────────────────────────────
    frame_limiter::throttle_fps(game);

    // Delta Time
    let now = Instant::now();
    let dt = now.duration_since(game.last_frame_time).as_secs_f32();
    game.last_frame_time = now;

    game.time_ms = game.start_time.elapsed().as_millis() as u32;

    // FPS counter
    game.frame_count += 1;
    let elapsed = game.last_fps_time.elapsed().as_secs_f32();
    if elapsed >= 1.0 {
        game.fps = game.frame_count as f32 / elapsed;
        game.frame_count = 0;
        game.last_fps_time = Instant::now();
    }

    // Animate map (rotating circular mazes)
    game.map.update_animations(dt);

    game.input_manager.poll_gamepad();
    let mut input = game.input_manager.get_state();

    // ─── Pause toggle ────────────────────────────────────────────────────────
    let just_opened_pause = input.escape_pressed && !game.paused;
    if just_opened_pause {
        game.paused = true;
        game.pause_menu.active_screen = ActiveScreen::Main;
        game.pause_menu.main_row      = 0;
        game.pause_menu.seed_input    = game.seed.clone();
        game.pause_menu.seed_cursor   = game.seed.len();
    }

    // ─── Logique du menu pause ───────────────────────────────────────────────
    if game.paused {
        game.pause_menu.cursor_blink_visible = (game.time_ms / 500) % 2 == 0;

        let event = game.pause_menu.handle_input(
            input.menu_up,
            input.menu_down,
            input.menu_left,
            input.menu_right,
            input.menu_confirm,
            input.escape_pressed && !just_opened_pause,
            input.menu_back,
            &input.chars_typed,
            input.backspace_pressed,
            input.ctrl_c,
            input.ctrl_v,
            None, // clipboard
            input.new_key_pressed,
            input.new_mouse_pressed,
            input.scroll_delta,
            input.mouse_x as f32,
            input.mouse_y as f32,
            input.mouse_left_clicked,
            input.arrow_left,
            input.arrow_right,
            input.arrow_up,
            input.arrow_down,
        );

        game.fps_millis  = game.pause_menu.fps_millis;
        game.fps_enabled = game.pause_menu.fps_enabled;

        if let Some((w, h)) = game.pause_menu.pending_resize.take() {
            if let Some(window) = &app.window {
                let _ = window.request_inner_size(winit::dpi::PhysicalSize::new(w, h));
            }
        }

        if let Some(mode) = game.pause_menu.pending_window_mode.take() {
            if let Some(window) = &app.window {
                use super::settings::WindowMode;
                match mode {
                    WindowMode::Fullscreen => {
                        window.set_fullscreen(Some(winit::window::Fullscreen::Exclusive(
                            winit::monitor::VideoModeHandle::clone(
                                &window.current_monitor()
                                    .and_then(|m| m.video_modes().next())
                                    .expect("no video mode")
                            )
                        )));
                    }
                    WindowMode::BorderlessFullscreen => {
                        window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
                    }
                    WindowMode::Windowed => {
                        window.set_fullscreen(None);
                    }
                }
            }
        }

        match event {
            MenuEvent::Close => {
                game.save_settings();
                game.paused = false;
            }
            MenuEvent::ApplySeed(seed) => {
                game.apply_seed(seed);
                gpu.raycaster.invalidate_cache();
                game.save_settings();
                game.paused = false;
            }
            MenuEvent::ApplySeedInMenu(seed) => {
                game.apply_seed(seed);
                gpu.raycaster.invalidate_cache();
                game.save_settings();
            }
            MenuEvent::RandomSeed => {
                let new_seed = crate::config::generate_random_seed();
                game.pause_menu.seed_input  = new_seed.clone();
                game.pause_menu.seed_cursor = new_seed.len();
                game.apply_seed(new_seed);
                gpu.raycaster.invalidate_cache();
                game.save_settings();
                game.paused = false;
            }
            MenuEvent::Quit => {
                game.save_settings();
                app.should_quit = true;
                return;
            }
            MenuEvent::None => {}
        }
    }

    // ─── Separator dragging ──────────────────────────────────────────────────
    let separator_y = game.height as f32 - game.strip_height;
    let mouse_y = input.mouse_y as f32;

    if (mouse_y - separator_y).abs() < 10.0 {
        game.hover_separator = true;
        if let Some(w) = &app.window { w.set_cursor(CursorIcon::NsResize); }
    } else {
        game.hover_separator = false;
        if !game.is_dragging_separator {
            if let Some(w) = &app.window { w.set_cursor(CursorIcon::Default); }
        }
    }

    if input.mouse_left_down && game.hover_separator {
        game.is_dragging_separator = true;
    }
    if !input.mouse_left_down {
        game.is_dragging_separator = false;
    }
    if game.is_dragging_separator {
        let new_h = game.height as f32 - mouse_y;
        game.strip_height = new_h.clamp(50.0, game.height as f32 - 50.0);
    }

    // ─── Zoom ────────────────────────────────────────────────────────────────
    if !game.paused {
        if input.scroll_delta != 0.0 {
            if input.scroll_delta > 0.0 { game.zoom_level *= 1.1; }
            else { game.zoom_level /= 1.1; }
        }
        if input.zoom_in  { game.zoom_level *= 1.05; }
        if input.zoom_out { game.zoom_level /= 1.05; }
        game.zoom_level = game.zoom_level.clamp(0.1, 50.0);
    }

    // ─── Game logic update ───────────────────────────────────────────────────
    if !game.paused {
        game_update::update_game(game, &mut input, dt);
    }

    let (px, py) = game.player.pos();
    let entity_segments = game.entity_manager.get_ray_segments((px, py));
    gpu.raycaster.update_map(
        &gpu.device, &gpu.queue, &mut game.map,
        &entity_segments,
        px, py,
        game.zoom_level,
        game.width, game.height,
    );

    // ─── Données 2D dynamiques ───────────────────────────────────────────────
    let mut dynamic_data = map_data::build_dynamic_data(game);
    map_data::push_separator_line(&mut dynamic_data, game);

    // ─── HUD textes ──────────────────────────────────────────────────────────
    let mut hud_texts: Vec<TextEntry> = Vec::new();

    if game.pause_menu.show_fps {
        let font_size_px = 52.0;
        let fps_str = format!("{:.0}", game.fps);
        let fps_w_est = fps_str.len() as f32 * font_size_px * 0.55;
        let fps_x_px = game.width as f32 - fps_w_est - 4.0;
        hud_texts.push(TextEntry::white(fps_str, fps_x_px, 2.0, font_size_px));
    }

    if game.paused {
        let sw = game.width as f32;
        let sh = game.height as f32;

        let menu_entries = game.pause_menu.build_hud(sw, sh, game.fps, game.time_ms, &mut gpu.renderer.hud_text.font_system);

        // ─── Overlay : fond + champ seed + flèche animée ─────────────────────
        {
            let (bx, by, bw, bh) = game.pause_menu.box_rect(sw, sh);
            let tex_w = sw as u32;
            let tex_h = sh as u32;
            let x0 = bx.max(0.0) as u32;
            let y0 = by.max(0.0) as u32;
            let x1 = (bx + bw).min(sw) as u32;
            let y1 = (by + bh).min(sh) as u32;
            let mut rgba = vec![0u8; (tex_w * tex_h * 4) as usize];

            // Fond sombre semi-transparent
            for row in y0..y1 {
                for col in x0..x1 {
                    let idx = ((row * tex_w + col) * 4) as usize;
                    rgba[idx]     = 8;
                    rgba[idx + 1] = 8;
                    rgba[idx + 2] = 18;
                    rgba[idx + 3] = 215;
                }
            }

            // Champ seed : fond noir + bordure colorée
            if let Some((fx, fy, fw, fh)) = game.pause_menu.seed_field_rect {
                let fx0 = fx.max(0.0) as u32;
                let fy0 = fy.max(0.0) as u32;
                let fx1 = (fx + fw).min(sw) as u32;
                let fy1 = (fy + fh).min(sh) as u32;
                for row in fy0..fy1 {
                    for col in fx0..fx1 {
                        let idx = ((row * tex_w + col) * 4) as usize;
                        if idx + 3 < rgba.len() {
                            rgba[idx]     = 0;
                            rgba[idx + 1] = 0;
                            rgba[idx + 2] = 0;
                            rgba[idx + 3] = 255;
                        }
                    }
                }
                let border_col: [u8; 4] = if game.pause_menu.seed_focused {
                    [255, 220, 60, 255]
                } else {
                    [160, 160, 170, 255]
                };
                let thickness: u32 = 2;
                for t in 0..thickness {
                    for col in fx0..fx1 {
                        let row = fy0 + t;
                        if row < fy1 {
                            let idx = ((row * tex_w + col) * 4) as usize;
                            if idx + 3 < rgba.len() { rgba[idx]=border_col[0]; rgba[idx+1]=border_col[1]; rgba[idx+2]=border_col[2]; rgba[idx+3]=border_col[3]; }
                        }
                        let row = fy1.saturating_sub(1 + t);
                        if row >= fy0 {
                            let idx = ((row * tex_w + col) * 4) as usize;
                            if idx + 3 < rgba.len() { rgba[idx]=border_col[0]; rgba[idx+1]=border_col[1]; rgba[idx+2]=border_col[2]; rgba[idx+3]=border_col[3]; }
                        }
                    }
                    for row in fy0..fy1 {
                        let col = fx0 + t;
                        if col < fx1 {
                            let idx = ((row * tex_w + col) * 4) as usize;
                            if idx + 3 < rgba.len() { rgba[idx]=border_col[0]; rgba[idx+1]=border_col[1]; rgba[idx+2]=border_col[2]; rgba[idx+3]=border_col[3]; }
                        }
                        let col = fx1.saturating_sub(1 + t);
                        if col >= fx0 {
                            let idx = ((row * tex_w + col) * 4) as usize;
                            if idx + 3 < rgba.len() { rgba[idx]=border_col[0]; rgba[idx+1]=border_col[1]; rgba[idx+2]=border_col[2]; rgba[idx+3]=border_col[3]; }
                        }
                    }
                }
            }

            // Flèche animée
            if let Some((ax, ay, font_size)) = game.pause_menu.arrow_screen_pos {
                let phase = {
                    let t = (game.time_ms % 1200) as f32 / 1200.0;
                    (t * std::f32::consts::PI * 2.0).cos() * 0.5 + 0.5
                };
                arrow::draw_arrow(&mut rgba, tex_w, tex_h, ax, ay, font_size, phase);
            }
            gpu.renderer.update_overlay_texture(&gpu.device, &gpu.queue, tex_w, tex_h, &rgba);
        }

        for (text, x, y, size, color) in menu_entries {
            hud_texts.push(TextEntry { text, x_px: x, y_px: y, size_px: size, color });
        }
    }

    gpu.renderer.update_dynamic_lines(&gpu.device, &gpu.queue, &dynamic_data);

    // ─── Surface ─────────────────────────────────────────────────────────────
    let surface_tex = match gpu.surface.get_current_texture() {
        Ok(t) => t,
        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
            gpu.surface.configure(&gpu.device, &gpu.config);
            return;
        }
        Err(e) => {
            log::error!("Surface error: {}", e);
            return;
        }
    };
    let view = surface_tex.texture.create_view(&Default::default());

    let mut encoder = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Frame"),
    });

    gpu.raycaster.cast(
        &mut encoder, &gpu.queue,
        px, py, game.player.angle,
        game.width,
    );

    let z_buffer = gpu.raycaster.read_z_buffer(&gpu.device);
    let entity_data = game.entity_manager.get_render_data(
        &z_buffer,
        game.player.pos(),
        game.player.angle,
        gpu.raycaster.num_rays,
        game.width,
        gpu.raycaster.screen_dist,
    );

    let wall_instances = build_wall_instances_from_raycaster(&gpu.raycaster, &gpu.device);
    let num_wall_instances = wall_instances.len();

    if num_wall_instances > 0 {
        gpu.renderer.ensure_wall_buffer(&gpu.device, num_wall_instances);
        if let Some(buf) = &gpu.renderer.wall_instance_buffer {
            gpu.queue.write_buffer(buf, 0, bytemuck::cast_slice(&wall_instances));
        }
    }

    let entity_instances: Vec<WallInstance> = entity_data.iter().map(|d| WallInstance {
        x_offset: d[0],
        width: d[1],
        height: d[2],
        distance: d[3],
        color: [d[4], d[5], d[6]],
    }).collect();
    let num_entity_instances = entity_instances.len();

    if num_entity_instances > 0 {
        gpu.renderer.ensure_entity_buffer(&gpu.device, num_entity_instances);
        if let Some(buf) = &gpu.renderer.entity_instance_buffer {
            gpu.queue.write_buffer(buf, 0, bytemuck::cast_slice(&entity_instances));
        }
    }

    // Viewports
    let strip_px = game.strip_height as u32;
    let map_h = (game.height - strip_px).max(1);

    let center_x = game.width as f32 / 2.0;
    let center_y = map_h as f32 / 2.0;
    let world_offset_x = center_x - px * game.zoom_level;
    let world_offset_y = center_y - py * game.zoom_level;

    gpu.queue.write_buffer(
        &gpu.renderer.map_lines_uniform_buffer, 0,
        bytemuck::bytes_of(&MapLineUniforms {
            offset: [world_offset_x, world_offset_y],
            tile_size: TILE_SIZE * game.zoom_level,
            _pad1: 0.0,
            resolution: [game.width as f32, map_h as f32],
            _pad2: [0.0; 2],
        }),
    );

    let step = (gpu.raycaster.num_rays / 300).max(1);
    let num_fan_points = gpu.raycaster.num_rays / step;
    let num_fov_verts = num_fan_points.saturating_sub(1) * 3;
    gpu.queue.write_buffer(
        &gpu.renderer.fov_uniform_buffer, 0,
        bytemuck::bytes_of(&FovUniforms {
            player_pos: [px, py],
            offset: [world_offset_x, world_offset_y],
            zoom: game.zoom_level,
            _pad1: 0.0,
            resolution: [game.width as f32, map_h as f32],
            step: step as i32,
            max_ray: gpu.raycaster.num_rays as i32,
        }),
    );

    let map_lines_bg = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("MapLines BG"),
        layout: &gpu.renderer.map_lines_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: gpu.raycaster.segment_buffer.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: gpu.renderer.map_lines_uniform_buffer.as_entire_binding() },
        ],
    });

    let fov_bg = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("FOV BG"),
        layout: &gpu.renderer.fov_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: gpu.raycaster.hit_buffer.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: gpu.renderer.fov_uniform_buffer.as_entire_binding() },
        ],
    });

    // ─── Render pass ─────────────────────────────────────────────────────────
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Main"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            ..Default::default()
        });

        // 1. Bottom: vue pseudo-3D
        if strip_px > 0 {
            pass.set_viewport(0.0, (game.height - strip_px) as f32, game.width as f32, strip_px as f32, 0.0, 1.0);
            pass.set_pipeline(&gpu.renderer.render3d_pipeline);
            pass.set_bind_group(0, &gpu.renderer.render3d_bind_group, &[]);
            pass.set_vertex_buffer(0, gpu.renderer.quad_vbo.slice(..));

            if num_wall_instances > 0 {
                if let Some(buf) = &gpu.renderer.wall_instance_buffer {
                    pass.set_vertex_buffer(1, buf.slice(..));
                    pass.draw(0..4, 0..num_wall_instances as u32);
                }
            }
            if num_entity_instances > 0 {
                if let Some(buf) = &gpu.renderer.entity_instance_buffer {
                    pass.set_vertex_buffer(1, buf.slice(..));
                    pass.draw(0..4, 0..num_entity_instances as u32);
                }
            }
        }

        // 2. Top: vue 2D du dessus
        if map_h > 0 {
            pass.set_viewport(0.0, 0.0, game.width as f32, map_h as f32, 0.0, 1.0);

            if gpu.raycaster.num_rays > 0 {
                pass.set_pipeline(&gpu.renderer.fov_pipeline);
                pass.set_bind_group(0, &fov_bg, &[]);
                pass.draw(0..num_fov_verts as u32, 0..1);
            }

            if gpu.raycaster.num_segments > 0 {
                pass.set_pipeline(&gpu.renderer.map_lines_pipeline);
                pass.set_bind_group(0, &map_lines_bg, &[]);
                pass.draw(0..(gpu.raycaster.num_segments as u32 * 2), 0..1);
            }

            if !dynamic_data.is_empty() {
                if let Some(ref bg) = gpu.renderer.dynamic_bind_group {
                    pass.set_pipeline(&gpu.renderer.map_lines_pipeline);
                    pass.set_bind_group(0, bg, &[]);
                    let vert_count = (dynamic_data.len() / 5) * 2;
                    pass.draw(0..vert_count as u32, 0..1);
                }
            }
        }

        // 3. Fond du menu pause (overlay)
        if game.paused {
            if let Some(ref bg) = gpu.renderer.overlay_bind_group {
                pass.set_viewport(0.0, 0.0, game.width as f32, game.height as f32, 0.0, 1.0);
                pass.set_pipeline(&gpu.renderer.overlay_pipeline);
                pass.set_bind_group(0, bg, &[]);
                pass.draw(0..6, 0..1);
            }
        }

        // 4. HUD texte TTF
        if !hud_texts.is_empty() {
            pass.set_viewport(0.0, 0.0, game.width as f32, game.height as f32, 0.0, 1.0);
            gpu.renderer.hud_text.draw(
                &gpu.device,
                &gpu.queue,
                &mut pass,
                &hud_texts,
                game.width,
                game.height,
            );
        }
    }

    gpu.queue.submit(std::iter::once(encoder.finish()));
    surface_tex.present();
}

pub fn build_wall_instances_from_raycaster(raycaster: &RayCaster, device: &wgpu::Device) -> Vec<WallInstance> {
    let slice = raycaster.readback_buffer.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        tx.send(result).unwrap();
    });
    device.poll(wgpu::Maintain::Wait);

    if rx.recv().unwrap().is_err() {
        return Vec::new();
    }

    let data = slice.get_mapped_range();
    let floats: &[f32] = bytemuck::cast_slice(&data);
    let num_rays = raycaster.num_rays;

    let mut instances = Vec::with_capacity(num_rays);
    for i in 0..num_rays {
        let base = i * 7;
        if base + 6 >= floats.len() { break; }
        let x_offset = floats[base];
        if x_offset <= -5.0 { continue; }

        instances.push(WallInstance {
            x_offset,
            width: floats[base + 1] + 0.5,
            height: floats[base + 2],
            distance: floats[base + 3],
            color: [floats[base + 4], floats[base + 5], floats[base + 6]],
        });
    }

    drop(data);
    raycaster.readback_buffer.unmap();
    instances
}
