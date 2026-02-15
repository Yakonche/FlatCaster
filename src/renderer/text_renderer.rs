// src/renderer/text_renderer.rs
//
// Wrapper autour de glyphon 0.6 pour rendre du texte TTF (Jersey10-Regular.ttf)
// directement dans le render pass wgpu existant.

use glyphon::{
    Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics,
    Resolution, Shaping, SwashCache, TextArea, TextAtlas, TextBounds,
    TextRenderer as GlyphonRenderer, Viewport,
};

pub struct HudTextRenderer {
    pub font_system: FontSystem,
    pub swash_cache: SwashCache,
    pub atlas: TextAtlas,
    pub viewport: Viewport,
    pub renderer: GlyphonRenderer,
    cache: Cache,
}

impl HudTextRenderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
        font_bytes: &[u8],
    ) -> Self {
        let mut font_system = FontSystem::new();
        // Charger la police custom (Jersey10-Regular.ttf)
        font_system.db_mut().load_font_data(font_bytes.to_vec());

        let swash_cache = SwashCache::new();
        let cache = Cache::new(device);
        let viewport = Viewport::new(device, &cache);
        let mut atlas = TextAtlas::new(device, queue, &cache, surface_format);
        let renderer = GlyphonRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);

        Self { font_system, swash_cache, atlas, viewport, renderer, cache }
    }

    /// Retourne la largeur réelle en pixels d'un texte à une taille donnée.
    pub fn measure_text(&mut self, text: &str, size_px: f32) -> f32 {
        let mut buf = Buffer::new(&mut self.font_system, Metrics::new(size_px, size_px * 1.2));
        buf.set_size(&mut self.font_system, None, None);
        buf.set_text(
            &mut self.font_system,
            text,
            Attrs::new().family(Family::Name("Jersey 10")),
            Shaping::Basic,
        );
        buf.shape_until_scroll(&mut self.font_system, false);
        buf.layout_runs()
            .map(|run| run.line_w)
            .fold(0.0_f32, f32::max)
    }

    /// Mettre à jour la résolution (à appeler sur resize).
    pub fn update_resolution(&mut self, queue: &wgpu::Queue, width: u32, height: u32) {
        self.viewport.update(queue, Resolution { width, height });
    }

    /// Prépare et dessine une liste de textes (screen-space, pixels).
    /// `texts` : Vec<(texte, x_px, y_px, taille_px, couleur_rgba)>
    pub fn draw<'pass>(
        &'pass mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        pass: &mut wgpu::RenderPass<'pass>,
        texts: &[TextEntry],
        screen_width: u32,
        screen_height: u32,
    ) {
        // S'assurer que la résolution est à jour
        self.viewport.update(queue, Resolution { width: screen_width, height: screen_height });

        // Construire un Buffer glyphon par texte
        let buffers: Vec<Buffer> = texts.iter().map(|t| {
            let mut buf = Buffer::new(&mut self.font_system, Metrics::new(t.size_px, t.size_px * 1.2));
            buf.set_size(&mut self.font_system, Some(screen_width as f32), Some(screen_height as f32));
            buf.set_text(
                &mut self.font_system,
                &t.text,
                Attrs::new().family(Family::Name("Jersey 10")),
                Shaping::Basic,
            );
            buf.shape_until_scroll(&mut self.font_system, false);
            buf
        }).collect();

        let areas: Vec<TextArea> = texts.iter().zip(buffers.iter()).map(|(t, buf)| {
            TextArea {
                buffer: buf,
                left: t.x_px,
                top: t.y_px,
                scale: 1.0,
                bounds: TextBounds {
                    left: 0,
                    top: 0,
                    right: screen_width as i32,
                    bottom: screen_height as i32,
                },
                default_color: Color::rgba(t.color[0], t.color[1], t.color[2], t.color[3]),
                custom_glyphs: &[],
            }
        }).collect();

        self.atlas.trim();

        if let Err(e) = self.renderer.prepare(
            device,
            queue,
            &mut self.font_system,
            &mut self.atlas,
            &self.viewport,
            areas,
            &mut self.swash_cache,
        ) {
            log::warn!("glyphon prepare error: {:?}", e);
            return;
        }

        if let Err(e) = self.renderer.render(&self.atlas, &self.viewport, pass) {
            log::warn!("glyphon render error: {:?}", e);
        }
    }
}

/// Une entrée de texte à afficher en screen-space (pixels).
pub struct TextEntry {
    pub text: String,
    /// Position X en pixels depuis le haut-gauche de l'écran
    pub x_px: f32,
    /// Position Y en pixels depuis le haut-gauche de l'écran
    pub y_px: f32,
    /// Taille de police en pixels
    pub size_px: f32,
    /// Couleur RGBA (0-255)
    pub color: [u8; 4],
}

impl TextEntry {
    pub fn white(text: impl Into<String>, x_px: f32, y_px: f32, size_px: f32) -> Self {
        Self { text: text.into(), x_px, y_px, size_px, color: [255, 255, 255, 255] }
    }
}
