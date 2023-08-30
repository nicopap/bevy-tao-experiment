//! Render the webview

use bevy::ecs::prelude::*;
use bevy::render::render_resource::{PipelineCache, SpecializedRenderPipelines, TextureFormat};
use bevy::render::renderer::RenderAdapter;
use bevy::render::Extract;
use bevy::window::{CompositeAlphaMode, PresentMode, PrimaryWindow, Window};
use bevy::{
    render::renderer::{RenderDevice, RenderInstance},
    window::RawHandleWrapper,
};
use wgpu::Surface;

use crate::bevy_tao_loop::WebviewRawHandles;

#[derive(Resource)]
struct ExtractedWebviewHandles {
    handle: RawHandleWrapper,
    window_data: WindowData,
}
#[derive(Clone, Copy)]
struct WindowData {
    physical_width: u32,
    physical_height: u32,
    alpha_mode: CompositeAlphaMode,
    present_mode: PresentMode,
}

fn extract(
    mut cmds: Commands,
    handles: Extract<Option<Res<WebviewRawHandles>>>,
    primary_window: Extract<Query<&Window, With<PrimaryWindow>>>,
) {
    if let (Some(handles), Ok(window)) = (*handles, primary_window.get_single()) {
        cmds.insert_resource(ExtractedWebviewHandles {
            handle: handles.0.clone(),
            window_data: WindowData {
                physical_width: window.physical_width(),
                physical_height: window.physical_height(),
                alpha_mode: window.composite_alpha_mode,
                present_mode: window.present_mode,
            },
        });
    }
}

struct WebviewSurface {
    surface: wgpu::Surface,
    texture: wgpu::SurfaceTexture,
    format: TextureFormat,
}

fn prepare_webview(
    render_device: Res<RenderDevice>,
    render_instance: Res<RenderInstance>,
    render_adapter: Res<RenderAdapter>,
    pipeline_cache: Res<PipelineCache>,
    webview: Option<Res<ExtractedWebviewHandles>>,
    mut webview_surface: Local<Option<WebviewSurface>>,
) {
    let Some(webview) = webview else {
        return;
    };
    let webview_surface = if let Some(wvs) = &mut *webview_surface {
        reconfigure_surface(
            &wvs.surface,
            webview.window_data,
            &render_device,
            &render_adapter,
            &render_instance,
        );
        wvs
    } else {
        // NOTE: On some OSes this MUST be called from the main thread.
        // As of wgpu 0.15, only failable if the given window is a HTML canvas and obtaining a WebGPU or WebGL2 context fails.
        let surface = unsafe { render_instance.create_surface(&webview.handle.get_handle()) }
            .expect("Failed to create wgpu surface");
        let surface = configure_surface(
            surface,
            webview.window_data,
            &render_device,
            &render_adapter,
            &render_instance,
        );
        *webview_surface = Some(surface);
        webview_surface.as_mut().unwrap()
    };
}
fn configure_surface(
    surface: Surface,
    window: WindowData,
    render_device: &RenderDevice,
    render_adapter: &RenderAdapter,
    render_instance: &RenderInstance,
) -> WebviewSurface {
    let caps = surface.get_capabilities(&render_adapter);
    let formats = caps.formats;
    // For future HDR output support, we'll need to request a format that supports HDR,
    // but as of wgpu 0.15 that is not yet supported.
    // Prefer sRGB formats for surfaces, but fall back to first available format if no sRGB formats are available.
    let mut format = *formats.get(0).expect("No supported formats for surface");
    for available_format in formats {
        // Rgba8UnormSrgb and Bgra8UnormSrgb and the only sRGB formats wgpu exposes that we can use for surfaces.
        if available_format == TextureFormat::Rgba8UnormSrgb
            || available_format == TextureFormat::Bgra8UnormSrgb
        {
            format = available_format;
            break;
        }
    }
    let surface_configuration = wgpu::SurfaceConfiguration {
        format,
        width: window.physical_width,
        height: window.physical_height,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        present_mode: match window.present_mode {
            PresentMode::Fifo => wgpu::PresentMode::Fifo,
            PresentMode::Mailbox => wgpu::PresentMode::Mailbox,
            PresentMode::Immediate => wgpu::PresentMode::Immediate,
            PresentMode::AutoVsync => wgpu::PresentMode::AutoVsync,
            PresentMode::AutoNoVsync => wgpu::PresentMode::AutoNoVsync,
        },
        alpha_mode: match window.alpha_mode {
            CompositeAlphaMode::Auto => wgpu::CompositeAlphaMode::Auto,
            CompositeAlphaMode::Opaque => wgpu::CompositeAlphaMode::Opaque,
            CompositeAlphaMode::PreMultiplied => wgpu::CompositeAlphaMode::PreMultiplied,
            CompositeAlphaMode::PostMultiplied => wgpu::CompositeAlphaMode::PostMultiplied,
            CompositeAlphaMode::Inherit => wgpu::CompositeAlphaMode::Inherit,
        },
        view_formats: if !format.is_srgb() {
            vec![format.add_srgb_suffix()]
        } else {
            vec![]
        },
    };

    // This is an ugly hack to work around drivers that don't support MSAA.
    // This should be removed once https://github.com/bevyengine/bevy/issues/7194 lands and we're doing proper
    // feature detection for MSAA.
    // When removed, we can also remove the `.after(prepare_windows)` of `prepare_core_3d_depth_textures` and `prepare_prepass_textures`
    // let sample_flags = render_adapter
    //     .get_texture_format_features(surface_configuration.format)
    //     .flags;

    // if !sample_flags.sample_count_supported(msaa.samples()) {
    //     let fallback = if sample_flags.sample_count_supported(Msaa::default().samples()) {
    //         Msaa::default()
    //     } else {
    //         Msaa::Off
    //     };

    //     let fallback_str = if fallback == Msaa::Off {
    //         "disabling MSAA".to_owned()
    //     } else {
    //         format!("MSAA {}x", fallback.samples())
    //     };

    //     bevy_log::warn!(
    //         "MSAA {}x is not supported on this device. Falling back to {}.",
    //         msaa.samples(),
    //         fallback_str,
    //     );
    //     *msaa = fallback;
    // }

    render_device.configure_surface(&surface, &surface_configuration);
    let frame = surface
        .get_current_texture()
        .expect("Error configuring surface");
    surface.set_swapchain_texture(frame);
    surface.swap_chain_texture_format = Some(format);
}
fn reconfigure_surface(
    surface: &Surface,
    window: WindowData,
    render_device: &RenderDevice,
    render_adapter: &RenderAdapter,
    render_instance: &RenderInstance,
) {
    // A recurring issue is hitting `wgpu::SurfaceError::Timeout` on certain Linux
    // mesa driver implementations. This seems to be a quirk of some drivers.
    // We'd rather keep panicking when not on Linux mesa, because in those case,
    // the `Timeout` is still probably the symptom of a degraded unrecoverable
    // application state.
    // see https://github.com/bevyengine/bevy/pull/5957
    // and https://github.com/gfx-rs/wgpu/issues/1218
    #[cfg(target_os = "linux")]
    let may_erroneously_timeout = || {
        render_instance
            .enumerate_adapters(wgpu::Backends::VULKAN)
            .any(|adapter| {
                let name = adapter.get_info().name;
                name.starts_with("AMD") || name.starts_with("Intel")
            })
    };

    // TODO
    // if not_already_configured || window.size_changed || window.present_mode_changed {
    match surface.get_current_texture() {
        Ok(frame) => {
            window.set_swapchain_texture(frame);
        }
        Err(wgpu::SurfaceError::Outdated) => {
            render_device.configure_surface(surface, &surface_configuration);
            let frame = surface
                .get_current_texture()
                .expect("Error reconfiguring surface");
            window.set_swapchain_texture(frame);
        }
        #[cfg(target_os = "linux")]
        Err(wgpu::SurfaceError::Timeout) if may_erroneously_timeout() => {
            bevy::utils::tracing::trace!(
                "Couldn't get swap chain texture. This is probably a quirk \
                        of your Linux GPU driver, so it can be safely ignored."
            );
        }
        Err(err) => {
            panic!("Couldn't get swap chain texture, operation unrecoverable: {err}");
        }
    }
}
