use std::collections::HashSet;

use winit::{
    event::{ElementState, KeyEvent, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

/// Manages the current input state of the window.
pub struct InputState {
    /// The keys currently being held down.
    keys_held: HashSet<KeyCode>,

    /// The keys currently pressed down this frame.
    keys_pressed: HashSet<KeyCode>,
    /// The keys currently released this frame.
    keys_released: HashSet<KeyCode>,

    /// The last known position of the mouse.
    last_mouse: Option<(f32, f32)>,
}

impl InputState {
    /// Creates a new [`InputState`] manager.
    pub(crate) fn new() -> Self {
        Self {
            keys_held: HashSet::new(),
            keys_pressed: HashSet::new(),
            keys_released: HashSet::new(),
            last_mouse: None,
        }
    }

    /// Flushes all keys and mouse states to begin a new frame.
    pub(crate) fn flush(&mut self) {
        self.keys_pressed.drain();
        self.keys_released.drain();
    }

    /// Handles an incoming [`WindowEvent`].
    pub(crate) fn window_event(&mut self, event: &WindowEvent) {
        use WindowEvent as WE;

        match event {
            WE::KeyboardInput { event, .. } => self.keyboard_event(event),

            _ => {}
        }
    }

    /// Handles an incoming [`KeyEvent`].
    fn keyboard_event(&mut self, event: &KeyEvent) {
        let PhysicalKey::Code(code) = event.physical_key else {
            return;
        };

        match event.state {
            ElementState::Pressed if !event.repeat => {
                self.keys_held.insert(code);
                self.keys_pressed.insert(code);
            }
            ElementState::Released => {
                self.keys_released.insert(code);
                self.keys_held.remove(&code);
            }
            _ => {}
        }
    }

    /// Returns if the given key code was pressed this frame.
    pub fn key_pressed(&self, code: KeyCode) -> bool {
        self.keys_pressed.contains(&code)
    }

    /// Returns if the given key code was released this frame.
    pub fn key_released(&self, code: KeyCode) -> bool {
        self.keys_released.contains(&code)
    }

    /// Returns if the given key code is currently being held down.
    pub fn key_held(&self, code: KeyCode) -> bool {
        self.keys_held.contains(&code)
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}
