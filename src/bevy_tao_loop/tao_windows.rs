#![warn(missing_docs)]

use bevy::ecs::entity::Entity;

use bevy::utils::{tracing::warn, HashMap};
use bevy::window::{CursorGrabMode, Window, WindowMode, WindowPosition, WindowResolution};

use tao::{
    dpi::{LogicalSize, PhysicalPosition},
    monitor::{MonitorHandle, VideoMode},
};

use super::converters::convert_window_theme;

/// A resource which maps window entities to [`winit`] library windows.
#[derive(Debug, Default)]
pub struct TaoWindows {
    /// Stores [`winit`] windows by window identifier.
    pub windows: HashMap<tao::window::WindowId, tao::window::Window>,
    /// Maps entities to `winit` window identifiers.
    pub entity_to_tao: HashMap<Entity, tao::window::WindowId>,
    /// Maps `winit` window identifiers to entities.
    pub tao_to_entity: HashMap<tao::window::WindowId, Entity>,

    // Some winit functions, such as `set_window_icon` can only be used from the main thread. If
    // they are used in another thread, the app will hang. This marker ensures `TaoWindows` is
    // only ever accessed with bevy's non-send functions and in NonSend systems.
    _not_send_sync: core::marker::PhantomData<*const ()>,
}

impl TaoWindows {
    /// Creates a `winit` window and associates it with our entity.
    pub fn create_window(
        &mut self,
        event_loop: &tao::event_loop::EventLoopWindowTarget<()>,
        entity: Entity,
        window: &Window,
    ) -> &tao::window::Window {
        let mut tao_window_builder = tao::window::WindowBuilder::new();

        tao_window_builder = match window.mode {
            WindowMode::BorderlessFullscreen => tao_window_builder.with_fullscreen(Some(
                tao::window::Fullscreen::Borderless(event_loop.primary_monitor()),
            )),
            WindowMode::Fullscreen => {
                tao_window_builder.with_fullscreen(Some(tao::window::Fullscreen::Exclusive(
                    get_best_videomode(&event_loop.primary_monitor().unwrap()),
                )))
            }
            WindowMode::SizedFullscreen => tao_window_builder.with_fullscreen(Some(
                tao::window::Fullscreen::Exclusive(get_fitting_videomode(
                    &event_loop.primary_monitor().unwrap(),
                    window.width() as u32,
                    window.height() as u32,
                )),
            )),
            WindowMode::Windowed => {
                if let Some(position) = tao_window_position(
                    &window.position,
                    &window.resolution,
                    event_loop.available_monitors(),
                    event_loop.primary_monitor(),
                    None,
                ) {
                    tao_window_builder = tao_window_builder.with_position(position);
                }

                let logical_size = LogicalSize::new(window.width(), window.height());
                if let Some(sf) = window.resolution.scale_factor_override() {
                    tao_window_builder.with_inner_size(logical_size.to_physical::<f64>(sf))
                } else {
                    tao_window_builder.with_inner_size(logical_size)
                }
            }
        };

        tao_window_builder = tao_window_builder
            .with_theme(window.window_theme.map(convert_window_theme))
            .with_resizable(window.resizable)
            .with_decorations(window.decorations);

        let constraints = window.resize_constraints.check_constraints();
        let min_inner_size = LogicalSize {
            width: constraints.min_width,
            height: constraints.min_height,
        };
        let max_inner_size = LogicalSize {
            width: constraints.max_width,
            height: constraints.max_height,
        };

        let tao_window_builder =
            if constraints.max_width.is_finite() && constraints.max_height.is_finite() {
                tao_window_builder
                    .with_min_inner_size(min_inner_size)
                    .with_max_inner_size(max_inner_size)
            } else {
                tao_window_builder.with_min_inner_size(min_inner_size)
            };

        let tao_window_builder = tao_window_builder.with_title(window.title.as_str());
        let tao_window = dbg!(tao_window_builder).build(event_loop).unwrap();

        // Do not set the grab mode on window creation if it's none, this can fail on mobile
        if window.cursor.grab_mode != CursorGrabMode::None {
            attempt_grab(&tao_window, window.cursor.grab_mode);
        }

        tao_window.set_cursor_visible(window.cursor.visible);

        // Do not set the cursor hittest on window creation if it's false, as it will always fail on some
        // platforms and log an unfixable warning.
        if !window.cursor.hit_test {
            if let Err(err) = tao_window.set_ignore_cursor_events(window.cursor.hit_test) {
                warn!(
                    "Could not set cursor hit test for window {:?}: {:?}",
                    window.title, err
                );
            }
        }

        self.entity_to_tao.insert(entity, tao_window.id());
        self.tao_to_entity.insert(tao_window.id(), entity);

        self.windows
            .entry(tao_window.id())
            .insert(tao_window)
            .into_mut()
    }

    /// Get the winit window that is associated with our entity.
    pub fn get_window(&self, entity: Entity) -> Option<&tao::window::Window> {
        self.entity_to_tao
            .get(&entity)
            .and_then(|tao_id| self.windows.get(tao_id))
    }

    /// Get the entity associated with the winit window id.
    ///
    /// This is mostly just an intermediary step between us and winit.
    pub fn get_window_entity(&self, tao_id: tao::window::WindowId) -> Option<Entity> {
        self.tao_to_entity.get(&tao_id).cloned()
    }

    /// Remove a window from winit.
    ///
    /// This should mostly just be called when the window is closing.
    pub fn remove_window(&mut self, entity: Entity) -> Option<tao::window::Window> {
        let tao_id = self.entity_to_tao.remove(&entity)?;
        // Don't remove from tao_to_window_id, to track that we used to know about this winit window
        self.windows.remove(&tao_id)
    }
}

/// Gets the "best" video mode which fits the given dimensions.
///
/// The heuristic for "best" prioritizes width, height, and refresh rate in that order.
pub fn get_fitting_videomode(monitor: &MonitorHandle, width: u32, height: u32) -> VideoMode {
    let mut modes = monitor.video_modes().collect::<Vec<_>>();

    fn abs_diff(a: u32, b: u32) -> u32 {
        if a > b {
            return a - b;
        }
        b - a
    }

    modes.sort_by(|a, b| {
        use std::cmp::Ordering::*;
        match abs_diff(a.size().width, width).cmp(&abs_diff(b.size().width, width)) {
            Equal => {
                match abs_diff(a.size().height, height).cmp(&abs_diff(b.size().height, height)) {
                    Equal => b.refresh_rate().cmp(&a.refresh_rate()),
                    default => default,
                }
            }
            default => default,
        }
    });

    modes.first().unwrap().clone()
}

/// Gets the "best" videomode from a monitor.
///
/// The heuristic for "best" prioritizes width, height, and refresh rate in that order.
pub fn get_best_videomode(monitor: &MonitorHandle) -> VideoMode {
    let mut modes = monitor.video_modes().collect::<Vec<_>>();
    modes.sort_by(|a, b| {
        use std::cmp::Ordering::*;
        match b.size().width.cmp(&a.size().width) {
            Equal => match b.size().height.cmp(&a.size().height) {
                Equal => b.refresh_rate().cmp(&a.refresh_rate()),
                default => default,
            },
            default => default,
        }
    });

    modes.first().unwrap().clone()
}

pub(crate) fn attempt_grab(tao_window: &tao::window::Window, grab_mode: CursorGrabMode) {
    let grab_result = match grab_mode {
        bevy::window::CursorGrabMode::None => tao_window.set_cursor_grab(false),
        bevy::window::CursorGrabMode::Confined => tao_window.set_cursor_grab(true),
        bevy::window::CursorGrabMode::Locked => tao_window.set_cursor_grab(true),
    };

    if let Err(err) = grab_result {
        let err_desc = match grab_mode {
            bevy::window::CursorGrabMode::Confined | bevy::window::CursorGrabMode::Locked => "grab",
            bevy::window::CursorGrabMode::None => "ungrab",
        };

        bevy::utils::tracing::error!("Unable to {} cursor: {}", err_desc, err);
    }
}

/// Compute the physical window position for a given [`WindowPosition`].
// Ideally we could generify this across window backends, but we only really have winit atm
// so whatever.
pub fn tao_window_position(
    position: &WindowPosition,
    resolution: &WindowResolution,
    mut available_monitors: impl Iterator<Item = MonitorHandle>,
    primary_monitor: Option<MonitorHandle>,
    current_monitor: Option<MonitorHandle>,
) -> Option<PhysicalPosition<i32>> {
    match position {
        WindowPosition::Automatic => {
            /* Window manager will handle position */
            None
        }
        WindowPosition::Centered(monitor_selection) => {
            use bevy::window::MonitorSelection::*;
            let maybe_monitor = match monitor_selection {
                Current => {
                    if current_monitor.is_none() {
                        warn!("Can't select current monitor on window creation or cannot find current monitor!");
                    }
                    current_monitor
                }
                Primary => primary_monitor,
                Index(n) => available_monitors.nth(*n),
            };

            if let Some(monitor) = maybe_monitor {
                let screen_size = monitor.size();

                // We use the monitors scale factor here since WindowResolution.scale_factor
                // is not yet populated when windows are created at plugin setup
                let scale_factor = monitor.scale_factor();

                // Logical to physical window size
                let (width, height): (u32, u32) =
                    LogicalSize::new(resolution.width(), resolution.height())
                        .to_physical::<u32>(scale_factor)
                        .into();

                let position = PhysicalPosition {
                    x: screen_size.width.saturating_sub(width) as f64 / 2.
                        + monitor.position().x as f64,
                    y: screen_size.height.saturating_sub(height) as f64 / 2.
                        + monitor.position().y as f64,
                };

                Some(position.cast::<i32>())
            } else {
                warn!("Couldn't get monitor selected with: {monitor_selection:?}");
                None
            }
        }
        WindowPosition::At(position) => {
            Some(PhysicalPosition::new(position[0] as f64, position[1] as f64).cast::<i32>())
        }
    }
}
