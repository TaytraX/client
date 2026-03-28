use winit::keyboard::KeyCode;

pub struct Player(bool, bool, bool, bool);

impl Player {
    pub fn new() -> Self {
        Player(false, false, false, false)
    }

    pub fn handle_key(&mut self, code: KeyCode, key_state: bool) {
        let Player(front, back, left, right) = self;
        match code {
            KeyCode::KeyW => *front = key_state,
            KeyCode::KeyA => *left = key_state,
            KeyCode::KeyS => *back = key_state,
            KeyCode::KeyD => *right = key_state,
            KeyCode::KeyF => *back = key_state,
            _ => {},
        }
    }

    pub fn get(&self) -> (bool, bool, bool, bool) {
        (self.0, self.1, self.2, self.3)
    }
}