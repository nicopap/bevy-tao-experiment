use bevy::ecs::entity::Entity;
use bevy::input::{
    keyboard::{KeyCode, KeyboardInput},
    mouse::MouseButton,
    touch::{ForceTouch, TouchInput, TouchPhase},
    ButtonState,
};
use bevy::math::Vec2;
use bevy::window::{CursorIcon, WindowLevel, WindowTheme};

use tao::window::Window;
use tao::{event::KeyEvent, keyboard::Key};

pub fn convert_keyboard_input(keyboard_input: &KeyEvent, window: Entity) -> KeyboardInput {
    KeyboardInput {
        scan_code: keyboard_input.physical_key.to_scancode().unwrap(),
        state: convert_element_state(keyboard_input.state),
        key_code: convert_virtual_key_code(keyboard_input.logical_key.clone()),
        window,
    }
}

pub fn convert_element_state(element_state: tao::event::ElementState) -> ButtonState {
    match element_state {
        tao::event::ElementState::Pressed => ButtonState::Pressed,
        tao::event::ElementState::Released => ButtonState::Released,
        _ => unimplemented!("A new version of tao added variants to ElementState"),
    }
}

pub fn convert_mouse_button(mouse_button: tao::event::MouseButton) -> MouseButton {
    match mouse_button {
        tao::event::MouseButton::Left => MouseButton::Left,
        tao::event::MouseButton::Right => MouseButton::Right,
        tao::event::MouseButton::Middle => MouseButton::Middle,
        tao::event::MouseButton::Other(val) => MouseButton::Other(val),
        _ => unimplemented!("A new version of tao added variants to MouseButton"),
    }
}

pub fn convert_touch_input(
    touch_input: tao::event::Touch,
    location: tao::dpi::LogicalPosition<f64>,
) -> TouchInput {
    TouchInput {
        phase: match touch_input.phase {
            tao::event::TouchPhase::Started => TouchPhase::Started,
            tao::event::TouchPhase::Moved => TouchPhase::Moved,
            tao::event::TouchPhase::Ended => TouchPhase::Ended,
            tao::event::TouchPhase::Cancelled => TouchPhase::Canceled,
            _ => unimplemented!("A new version of tao added variants to TouchPhase"),
        },
        position: Vec2::new(location.x as f32, location.y as f32),
        force: touch_input.force.map(|f| match f {
            tao::event::Force::Calibrated {
                force,
                max_possible_force,
                altitude_angle,
                ..
            } => ForceTouch::Calibrated {
                force,
                max_possible_force,
                altitude_angle,
            },
            tao::event::Force::Normalized(x) => ForceTouch::Normalized(x),
            _ => unimplemented!("A new version of tao added variants to ForceTouch"),
        }),
        id: touch_input.id,
    }
}

pub fn convert_virtual_key_code(virtual_key_code: Key) -> Option<KeyCode> {
    let key = match virtual_key_code {
        Key::Character("1") => KeyCode::Key1,
        Key::Character("2") => KeyCode::Key2,
        Key::Character("3") => KeyCode::Key3,
        Key::Character("4") => KeyCode::Key4,
        Key::Character("5") => KeyCode::Key5,
        Key::Character("6") => KeyCode::Key6,
        Key::Character("7") => KeyCode::Key7,
        Key::Character("8") => KeyCode::Key8,
        Key::Character("9") => KeyCode::Key9,
        Key::Character("0") => KeyCode::Key0,
        Key::Character("A") => KeyCode::A,
        Key::Character("B") => KeyCode::B,
        Key::Character("C") => KeyCode::C,
        Key::Character("D") => KeyCode::D,
        Key::Character("E") => KeyCode::E,
        Key::Character("F") => KeyCode::F,
        Key::Character("G") => KeyCode::G,
        Key::Character("H") => KeyCode::H,
        Key::Character("I") => KeyCode::I,
        Key::Character("J") => KeyCode::J,
        Key::Character("K") => KeyCode::K,
        Key::Character("L") => KeyCode::L,
        Key::Character("M") => KeyCode::M,
        Key::Character("N") => KeyCode::N,
        Key::Character("O") => KeyCode::O,
        Key::Character("P") => KeyCode::P,
        Key::Character("Q") => KeyCode::Q,
        Key::Character("R") => KeyCode::R,
        Key::Character("S") => KeyCode::S,
        Key::Character("T") => KeyCode::T,
        Key::Character("U") => KeyCode::U,
        Key::Character("V") => KeyCode::V,
        Key::Character("W") => KeyCode::W,
        Key::Character("X") => KeyCode::X,
        Key::Character("Y") => KeyCode::Y,
        Key::Character("Z") => KeyCode::Z,
        Key::Character("+") => KeyCode::Plus,
        Key::Character("*") => KeyCode::Asterisk,
        Key::Character("^") => KeyCode::Caret,
        Key::Character("[") => KeyCode::BracketLeft,
        Key::Character("'") => KeyCode::Apostrophe,
        Key::Character("\\") => KeyCode::Backslash,
        Key::Character(":") => KeyCode::Colon,
        Key::Character(",") => KeyCode::Comma,
        Key::Character("-") => KeyCode::Minus,
        Key::Character("=") => KeyCode::Equals,
        Key::Character("`") => KeyCode::Grave,
        Key::Character(".") => KeyCode::Period,
        Key::Character("]") => KeyCode::BracketRight,
        Key::Character(";") => KeyCode::Semicolon,
        Key::Character("/") => KeyCode::Slash,
        Key::Escape => KeyCode::Escape,
        Key::F1 => KeyCode::F1,
        Key::F2 => KeyCode::F2,
        Key::F3 => KeyCode::F3,
        Key::F4 => KeyCode::F4,
        Key::F5 => KeyCode::F5,
        Key::F6 => KeyCode::F6,
        Key::F7 => KeyCode::F7,
        Key::F8 => KeyCode::F8,
        Key::F9 => KeyCode::F9,
        Key::F10 => KeyCode::F10,
        Key::F11 => KeyCode::F11,
        Key::F12 => KeyCode::F12,
        Key::F13 => KeyCode::F13,
        Key::F14 => KeyCode::F14,
        Key::F15 => KeyCode::F15,
        Key::F16 => KeyCode::F16,
        Key::F17 => KeyCode::F17,
        Key::F18 => KeyCode::F18,
        Key::F19 => KeyCode::F19,
        Key::F20 => KeyCode::F20,
        Key::F21 => KeyCode::F21,
        Key::F22 => KeyCode::F22,
        Key::F23 => KeyCode::F23,
        Key::F24 => KeyCode::F24,
        Key::PrintScreen => KeyCode::Snapshot,
        Key::ScrollLock => KeyCode::Scroll,
        Key::Pause => KeyCode::Pause,
        Key::Insert => KeyCode::Insert,
        Key::Home => KeyCode::Home,
        Key::Delete => KeyCode::Delete,
        Key::End => KeyCode::End,
        Key::PageDown => KeyCode::PageDown,
        Key::PageUp => KeyCode::PageUp,
        Key::ArrowLeft => KeyCode::Left,
        Key::ArrowUp => KeyCode::Up,
        Key::ArrowRight => KeyCode::Right,
        Key::ArrowDown => KeyCode::Down,
        Key::Backspace => KeyCode::Back,
        Key::Enter => KeyCode::Return,
        Key::Space => KeyCode::Space,
        Key::Compose => KeyCode::Compose,
        Key::NumLock => KeyCode::Numlock,
        Key::Convert => KeyCode::Convert,
        Key::KanaMode => KeyCode::Kana,
        Key::KanjiMode => KeyCode::Kanji,
        Key::Alt => KeyCode::AltLeft,
        Key::Control => KeyCode::ControlLeft,
        Key::Shift => KeyCode::ShiftLeft,
        Key::Super => KeyCode::SuperLeft,
        Key::LaunchMail => KeyCode::Mail,
        Key::MediaPlay => KeyCode::MediaSelect,
        Key::MediaStop => KeyCode::MediaStop,
        Key::AudioVolumeMute => KeyCode::Mute,
        Key::GoHome => KeyCode::MyComputer,
        Key::BrowserForward => KeyCode::NavigateForward,
        Key::BrowserBack => KeyCode::NavigateBackward,
        Key::MediaTrackNext => KeyCode::NextTrack,
        Key::MediaPlayPause => KeyCode::PlayPause,
        Key::Power => KeyCode::Power,
        Key::MediaTrackPrevious => KeyCode::PrevTrack,
        Key::Tab => KeyCode::Tab,
        Key::BrowserStop => KeyCode::Stop,
        Key::AudioVolumeDown => KeyCode::VolumeDown,
        Key::AudioVolumeUp => KeyCode::VolumeUp,
        Key::WakeUp => KeyCode::Wake,
        Key::Copy => KeyCode::Copy,
        Key::Paste => KeyCode::Paste,
        Key::Cut => KeyCode::Cut,
        _ => return None,
    };
    Some(key)
}

pub fn convert_cursor_icon(cursor_icon: CursorIcon) -> tao::window::CursorIcon {
    match cursor_icon {
        CursorIcon::Default => tao::window::CursorIcon::Default,
        CursorIcon::Crosshair => tao::window::CursorIcon::Crosshair,
        CursorIcon::Hand => tao::window::CursorIcon::Hand,
        CursorIcon::Arrow => tao::window::CursorIcon::Arrow,
        CursorIcon::Move => tao::window::CursorIcon::Move,
        CursorIcon::Text => tao::window::CursorIcon::Text,
        CursorIcon::Wait => tao::window::CursorIcon::Wait,
        CursorIcon::Help => tao::window::CursorIcon::Help,
        CursorIcon::Progress => tao::window::CursorIcon::Progress,
        CursorIcon::NotAllowed => tao::window::CursorIcon::NotAllowed,
        CursorIcon::ContextMenu => tao::window::CursorIcon::ContextMenu,
        CursorIcon::Cell => tao::window::CursorIcon::Cell,
        CursorIcon::VerticalText => tao::window::CursorIcon::VerticalText,
        CursorIcon::Alias => tao::window::CursorIcon::Alias,
        CursorIcon::Copy => tao::window::CursorIcon::Copy,
        CursorIcon::NoDrop => tao::window::CursorIcon::NoDrop,
        CursorIcon::Grab => tao::window::CursorIcon::Grab,
        CursorIcon::Grabbing => tao::window::CursorIcon::Grabbing,
        CursorIcon::AllScroll => tao::window::CursorIcon::AllScroll,
        CursorIcon::ZoomIn => tao::window::CursorIcon::ZoomIn,
        CursorIcon::ZoomOut => tao::window::CursorIcon::ZoomOut,
        CursorIcon::EResize => tao::window::CursorIcon::EResize,
        CursorIcon::NResize => tao::window::CursorIcon::NResize,
        CursorIcon::NeResize => tao::window::CursorIcon::NeResize,
        CursorIcon::NwResize => tao::window::CursorIcon::NwResize,
        CursorIcon::SResize => tao::window::CursorIcon::SResize,
        CursorIcon::SeResize => tao::window::CursorIcon::SeResize,
        CursorIcon::SwResize => tao::window::CursorIcon::SwResize,
        CursorIcon::WResize => tao::window::CursorIcon::WResize,
        CursorIcon::EwResize => tao::window::CursorIcon::EwResize,
        CursorIcon::NsResize => tao::window::CursorIcon::NsResize,
        CursorIcon::NeswResize => tao::window::CursorIcon::NeswResize,
        CursorIcon::NwseResize => tao::window::CursorIcon::NwseResize,
        CursorIcon::ColResize => tao::window::CursorIcon::ColResize,
        CursorIcon::RowResize => tao::window::CursorIcon::RowResize,
    }
}

pub fn set_window_level(window_level: WindowLevel, tao_window: &Window) {
    let (on_top, on_bottom) = match window_level {
        WindowLevel::AlwaysOnBottom => (false, true),
        WindowLevel::Normal => (false, false),
        WindowLevel::AlwaysOnTop => (true, false),
    };
    tao_window.set_always_on_top(on_top);
    tao_window.set_always_on_bottom(on_bottom);
}

pub fn convert_tao_theme(theme: tao::window::Theme) -> WindowTheme {
    match theme {
        tao::window::Theme::Light => WindowTheme::Light,
        tao::window::Theme::Dark => WindowTheme::Dark,
        _ => unimplemented!("A new version of tao added variants to Theme"),
    }
}

pub fn convert_window_theme(theme: WindowTheme) -> tao::window::Theme {
    match theme {
        WindowTheme::Light => tao::window::Theme::Light,
        WindowTheme::Dark => tao::window::Theme::Dark,
    }
}
