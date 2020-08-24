use crate::renderer::RenderBackend;
use std::collections::HashSet;
use winit::event::VirtualKeyCode;

pub struct KeyboardContext {
    pressed_keys: HashSet<VirtualKeyCode>,
    last_pressed: Option<VirtualKeyCode>,
    current_pressed: Option<VirtualKeyCode>,
}

impl KeyboardContext {
    fn new() -> Self {
        Self {
            pressed_keys: HashSet::with_capacity(4),
            last_pressed: None,
            current_pressed: None,
        }
    }

    pub fn set_key_state(&mut self, keycode: VirtualKeyCode, pressed: bool) {
        if pressed {
            self.pressed_keys.insert(keycode);
            self.last_pressed = self.current_pressed;
            self.current_pressed = Some(key);
        } else {
            self.pressed_keys.remove(&keycode);
            self.current_pressed = None;
        }
    }

    pub fn is_pressed(&self, keycode: VirtualKeyCode) -> bool {
        self.pressed_keys.contains(&keycode)
    }

    pub fn is_key_repeated(&self) -> bool {
        if self.last_pressed.is_some() {
            self.last_pressed == self.current_pressed
        } else {
            false
        }
    }
}

pub struct Context {
    backend: Box<dyn RenderBackend>,
    keyboard: KeyboardContext,
}

impl Context {
    pub fn new() -> Context {
        Self {
            backend: RenderBackend::new(),
            keyboard: KeyboardContext::new(),
        }
    }

    pub fn keyboard(&self) -> &KeyboardContext {
        &self.keyboard
    }

    pub fn keyboard_mut(&mut self) -> &mut KeyboardContext {
        &mut self.keyboard
    }
}
