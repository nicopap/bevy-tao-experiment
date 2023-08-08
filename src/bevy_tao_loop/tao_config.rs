use bevy::ecs::system::Resource;

/// A resource for configuring usage of the [`winit`] library.
#[derive(Debug, Resource)]
pub struct TaoSettings {
    /// Configures `winit` to return control to the caller after exiting the
    /// event loop, enabling [`App::run()`](bevy_app::App::run()) to return.
    ///
    /// By default, [`return_from_run`](Self::return_from_run) is `false` and *Bevy*
    /// will use `winit`'s
    /// [`EventLoop::run()`](https://docs.rs/winit/latest/winit/event_loop/struct.EventLoop.html#method.run)
    /// to initiate the event loop.
    /// [`EventLoop::run()`](https://docs.rs/winit/latest/winit/event_loop/struct.EventLoop.html#method.run)
    /// will never return but will terminate the process after the event loop exits.
    ///
    /// Setting [`return_from_run`](Self::return_from_run) to `true` will cause *Bevy*
    /// to use `winit`'s
    /// [`EventLoopExtRunReturn::run_return()`](https://docs.rs/winit/latest/winit/platform/run_return/trait.EventLoopExtRunReturn.html#tymethod.run_return)
    /// instead which is strongly discouraged by the `winit` authors.
    ///
    /// # Supported platforms
    ///
    /// This feature is only available on the following desktop `target_os` configurations:
    /// `windows`, `macos`, `linux`, `dragonfly`, `freebsd`, `netbsd`, and `openbsd`.
    ///
    /// Setting [`return_from_run`](Self::return_from_run) to `true` on
    /// unsupported platforms will cause [`App::run()`](bevy_app::App::run()) to panic!
    pub return_from_run: bool,
    /// Configures how the winit event loop updates while the window is focused.
    pub focused_mode: UpdateMode,
    /// Configures how the winit event loop updates while the window is *not* focused.
    pub unfocused_mode: UpdateMode,
}
impl Default for TaoSettings {
    fn default() -> Self {
        TaoSettings {
            return_from_run: false,
            focused_mode: UpdateMode::Continuous,
            unfocused_mode: UpdateMode::Continuous,
        }
    }
}

/// Configure how the winit event loop should update.
#[derive(Debug)]
pub enum UpdateMode {
    /// The event loop will update continuously, running as fast as possible.
    Continuous,
}
