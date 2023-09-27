use bevy::prelude::KeyCode;

pub(super) const fn keycode_to_digit(keycode: &KeyCode) -> Option<usize> {
    match keycode {
        KeyCode::Key1 => Some(0),
        KeyCode::Key2 => Some(1),
        KeyCode::Key3 => Some(2),
        KeyCode::Key4 => Some(3),
        KeyCode::Key5 => Some(4),
        KeyCode::Key6 => Some(5),
        KeyCode::Key7 => Some(6),
        KeyCode::Key8 => Some(7),
        KeyCode::Key9 => Some(8),
        KeyCode::Key0 => Some(9),
        _ => None
    }
}