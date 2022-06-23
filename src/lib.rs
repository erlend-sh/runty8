#![doc = include_str!("../README.md")]
#![allow(clippy::new_without_default)]
// #![deny(missing_docs)]
pub mod app;
pub mod runtime;
pub mod ui;
pub use screen::Resources;

mod draw;
pub mod editor;
mod font;
mod graphics;
pub mod screen;
use crate::editor::serialize::Serialize;
use app::AppCompat;
use glium::glutin::event::{ElementState, VirtualKeyCode};
use runtime::{
    draw_context::DrawData,
    flags::Flags,
    map::Map,
    sprite_sheet::{Color, Sprite, SpriteSheet},
    state::State,
};
use std::fmt::Debug;

/// Mouse buttons
#[derive(Clone, Copy, Debug)]
pub enum MouseButton {
    // TODO: Handle other mouse buttons? idk
    Left,
    Right,
    Middle,
}

/// Mouse-related events
#[derive(Clone, Copy, Debug)]
pub enum MouseEvent {
    /// Mouse move event.
    // Contains the current position of the mouse.
    Move {
        ///
        x: i32,
        ///
        y: i32,
    },
    /// Mouse button pressed
    Down(MouseButton),
    /// Mouse button released
    Up(MouseButton),
}

/// Keyboard keys
#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq)]
pub enum Key {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Control,
}

impl Key {
    pub(crate) fn from_virtual_keycode(key: VirtualKeyCode) -> Option<Self> {
        match key {
            VirtualKeyCode::A => Some(Self::A),
            VirtualKeyCode::B => Some(Self::B),
            VirtualKeyCode::C => Some(Self::C),
            VirtualKeyCode::D => Some(Self::D),
            VirtualKeyCode::E => Some(Self::E),
            VirtualKeyCode::F => Some(Self::F),
            VirtualKeyCode::G => Some(Self::G),
            VirtualKeyCode::H => Some(Self::H),
            VirtualKeyCode::I => Some(Self::I),
            VirtualKeyCode::J => Some(Self::J),
            VirtualKeyCode::K => Some(Self::K),
            VirtualKeyCode::L => Some(Self::L),
            VirtualKeyCode::M => Some(Self::M),
            VirtualKeyCode::N => Some(Self::N),
            VirtualKeyCode::O => Some(Self::O),
            VirtualKeyCode::P => Some(Self::P),
            VirtualKeyCode::Q => Some(Self::Q),
            VirtualKeyCode::R => Some(Self::R),
            VirtualKeyCode::S => Some(Self::S),
            VirtualKeyCode::T => Some(Self::T),
            VirtualKeyCode::U => Some(Self::U),
            VirtualKeyCode::V => Some(Self::V),
            VirtualKeyCode::W => Some(Self::W),
            VirtualKeyCode::X => Some(Self::X),
            VirtualKeyCode::Y => Some(Self::Y),
            VirtualKeyCode::Z => Some(Self::Z),
            VirtualKeyCode::LControl => Some(Self::Control),
            // VirtualKeyCode::Escape => todo!(),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct KeyboardEvent {
    pub key: Key,
    pub state: KeyState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyState {
    Up,
    Down,
}

impl KeyState {
    fn from_state(state: ElementState) -> Self {
        match state {
            ElementState::Pressed => Self::Down,
            ElementState::Released => Self::Up,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Event {
    Mouse(MouseEvent),
    Keyboard(KeyboardEvent),
    Tick { delta_millis: f64 },
}

fn create_directory(path: &str) {
    if let Err(e) = std::fs::create_dir(path) {
        println!("Couldn't create directory {}, error: {:?}", path, e);
    };
}

/// Run a Pico8 application
// TODO: add example
// pub fn run_app<T: AppCompat + 'static>(assets_path: String) {
//     create_directory(&assets_path);

//     let draw_data = DrawData::new();
//     let (map, sprite_flags, sprite_sheet) = (
//         create_map(&assets_path),
//         create_sprite_flags(&assets_path),
//         create_sprite_sheet(&assets_path),
//     );

//     crate::screen::run_app::<T>(assets_path, map, sprite_flags, sprite_sheet, draw_data);
// }

#[macro_export]
macro_rules! run_app2 {
    ($path:literal, $t:ty) => {
        use $crate::editor::serialize::Serialize;
        use $crate::runtime::draw_context::DrawData;
        use $crate::runtime::flags::Flags;
        use $crate::runtime::map::Map;
        use $crate::runtime::sprite_sheet::SpriteSheet;

        fn create_sprite_flags_static() -> Flags {
            let content = include_str!(concat!($path, "/", "sprite_flags.txt"));

            Flags::deserialize(content).unwrap()
        }
        fn create_map_static() -> Map {
            let content = include_str!(concat!($path, "/", "map.txt"));

            Map::deserialize(content).unwrap()
        }

        fn create_sprite_sheet_static() -> SpriteSheet {
            let content = include_str!(concat!($path, "/", "sprite_sheet.txt"));

            SpriteSheet::deserialize(content).unwrap()
        }

        fn create_sprite_flags(assets_path: &str) -> Flags {
            if let Ok(content) = std::fs::read_to_string(&format!(
                "{}{}{}",
                assets_path,
                std::path::MAIN_SEPARATOR,
                Flags::file_name()
            )) {
                Flags::deserialize(&content).unwrap()
            } else {
                Flags::new()
            }
        }

        fn create_map(assets_path: &str) -> Map {
            let path = format!(
                "{}{}{}",
                assets_path,
                std::path::MAIN_SEPARATOR,
                Map::file_name()
            );

            if let Ok(content) = std::fs::read_to_string(&path) {
                Map::deserialize(&content).unwrap()
            } else {
                println!("Couldn't read map from {}", path);
                Map::new()
            }
        }

        fn create_sprite_sheet(assets_path: &str) -> SpriteSheet {
            let path = format!(
                "{}{}{}",
                assets_path,
                std::path::MAIN_SEPARATOR,
                SpriteSheet::file_name()
            );

            if let Ok(content) = std::fs::read_to_string(&path) {
                SpriteSheet::deserialize(&content).unwrap()
            } else {
                println!("Couldn't read sprite sheet from {}", path);
                SpriteSheet::new()
            }
        }

        let draw_data = DrawData::new();
        let (map, sprite_flags, sprite_sheet) = if cfg!(bundle_config) {
            (
                create_map_static(),
                create_sprite_flags_static(),
                create_sprite_sheet_static(),
            )
        } else {
            (
                create_map($path),
                create_sprite_flags($path),
                create_sprite_sheet($path),
            )
        };

        $crate::screen::run_app::<$t>($path.to_owned(), map, sprite_flags, sprite_sheet, draw_data);
    };
}

/* UTILS */
pub(crate) fn write_and_log(file_name: &str, contents: &str) {
    print!("Writing {file_name}... ");
    std::fs::write(&file_name, contents).unwrap();
    println!("success.")
}
