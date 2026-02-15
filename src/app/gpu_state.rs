// src/app/gpu_state.rs
use std::sync::Arc;
use winit::window::Window;
use crate::renderer::Renderer;
use crate::raycasting::RayCaster;

pub struct GpuState {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub renderer: Renderer,
    pub raycaster: RayCaster,
}

impl GpuState {
    pub fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);

        // Sur Windows avec HDR activé, Vulkan peut exposer un espace couleur étendu (scRGB)
        // qui cause des couleurs délavées lors d'une capture d'écran par ShareX ou similaire.
        // DX12 via DXGI gère correctement la déclaration SDR au système HDR Windows.
        // On utilise DX12 en priorité sur Windows, avec Vulkan en fallback.
        #[cfg(target_os = "windows")]
        let preferred_backends = wgpu::Backends::DX12;
        #[cfg(not(target_os = "windows"))]
        let preferred_backends = wgpu::Backends::VULKAN | wgpu::Backends::METAL;

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: preferred_backends,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).expect("Failed to create surface");

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .expect("No suitable GPU adapter found");

        log::info!("Using adapter: {}", adapter.get_info().name);

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Main Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
            },
            None,
        ))
        .expect("Failed to create device");

        let surface_caps = surface.get_capabilities(&adapter);

        // Log des formats et alpha modes disponibles pour diagnostic HDR
        log::info!("Formats surface disponibles : {:?}", surface_caps.formats);
        log::info!("Alpha modes disponibles : {:?}", surface_caps.alpha_modes);

        // Forcer un format SDR 8-bit pour éviter les problèmes de couleurs avec HDR Windows.
        // On préfère Bgra8UnormSrgb ou Rgba8UnormSrgb, puis les variantes Unorm non-sRGB,
        // en dernier recours le premier format disponible.
        let surface_format = surface_caps.formats.iter()
            .find(|f| **f == wgpu::TextureFormat::Bgra8UnormSrgb || **f == wgpu::TextureFormat::Rgba8UnormSrgb)
            .or_else(|| surface_caps.formats.iter().find(|f| **f == wgpu::TextureFormat::Bgra8Unorm || **f == wgpu::TextureFormat::Rgba8Unorm))
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        log::info!("Format surface sélectionné : {:?}", surface_format);

        // Forcer CompositeAlphaMode::Opaque pour éviter l'espace couleur HDR étendu
        let alpha_mode = surface_caps.alpha_modes.iter()
            .find(|m| **m == wgpu::CompositeAlphaMode::Opaque)
            .copied()
            .unwrap_or(surface_caps.alpha_modes[0]);

        log::info!("Alpha mode sélectionné : {:?}", alpha_mode);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoNoVsync,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let renderer = Renderer::new(&device, &queue, surface_format);
        let raycaster = RayCaster::new(&device, width);

        Self { surface, device, queue, config, renderer, raycaster }
    }
}
