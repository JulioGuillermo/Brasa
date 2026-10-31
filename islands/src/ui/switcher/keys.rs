//! What a key does in the window switcher.

use gtk4::gdk::Key;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    Next,
    Previous,
    Choose,
    Cancel,
}

pub fn action_for(key: Key) -> Option<Action> {
    let action = match key {
        Key::Tab | Key::Down | Key::Right | Key::j | Key::l => Action::Next,
        Key::ISO_Left_Tab | Key::Up | Key::Left | Key::k | Key::h => Action::Previous,
        Key::Return | Key::KP_Enter | Key::space => Action::Choose,
        Key::Escape => Action::Cancel,
        _ => return None,
    };
    Some(action)
}

pub fn is_super(key: Key) -> bool {
    matches!(key, Key::Super_L | Key::Super_R)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tab_goes_on_and_shift_tab_goes_back() {
        assert_eq!(action_for(Key::Tab), Some(Action::Next));
        assert_eq!(action_for(Key::ISO_Left_Tab), Some(Action::Previous));
        assert_eq!(action_for(Key::Return), Some(Action::Choose));
        assert_eq!(action_for(Key::Escape), Some(Action::Cancel));
        assert_eq!(action_for(Key::a), None);
    }
}
