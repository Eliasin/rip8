use sdl2::keyboard::{KeyboardState, Scancode};

use std::collections::HashMap;

pub type Key = u8;

pub trait Keyboard {
    fn is_key_pressed(&self, key: Key) -> bool;
}

impl<'a> Keyboard for &'a dyn Keyboard {
    fn is_key_pressed(&self, key: Key) -> bool {
        (*self).is_key_pressed(key)
    }
}

pub struct SDL2Keyboard<'a> {
    keyboard_state: KeyboardState<'a>,
    key_map: HashMap<Key, Scancode>,
}

impl Keyboard for SDL2Keyboard<'_> {
    fn is_key_pressed(&self, key: Key) -> bool {
        let scancode = self
            .key_map
            .get(&key)
            .expect(format!("Unmapped key {}", key).as_str());

        self.keyboard_state.is_scancode_pressed(*scancode)
    }
}

impl SDL2Keyboard<'_> {
    pub fn new<'a>(keyboard_state: KeyboardState<'a>) -> SDL2Keyboard<'a> {
        let key_map: HashMap<Key, Scancode> = [
            (0x1, Scancode::Num1),
            (0x2, Scancode::Num2),
            (0x3, Scancode::Num3),
            (0xC, Scancode::Num4),
            (0x4, Scancode::Q),
            (0x5, Scancode::W),
            (0x6, Scancode::E),
            (0xD, Scancode::R),
            (0x7, Scancode::A),
            (0x8, Scancode::S),
            (0x9, Scancode::D),
            (0xE, Scancode::F),
            (0xA, Scancode::Z),
            (0x0, Scancode::X),
            (0xB, Scancode::C),
            (0xF, Scancode::V),
        ]
        .iter()
        .cloned()
        .collect();
        SDL2Keyboard {
            keyboard_state: keyboard_state,
            key_map: key_map,
        }
    }
}
