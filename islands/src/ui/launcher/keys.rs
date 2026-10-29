//! What the keys do in the launcher: move the choice, or close.

use gtk4::gdk::Key;

use crate::config::LauncherView;

const PAGE_ROWS: i32 = 3;

pub enum KeyAction {
    MoveChoice(i32),
    Close,
}

pub fn action_for(key: Key, view: LauncherView, columns: u32) -> Option<KeyAction> {
    let columns = columns as i32;
    let step = match key {
        Key::Down => columns,
        Key::Up => -columns,
        Key::Page_Down => columns * PAGE_ROWS,
        Key::Page_Up => -columns * PAGE_ROWS,
        Key::Escape => return Some(KeyAction::Close),
        Key::Right | Key::Tab if view == LauncherView::Grid => 1,
        Key::Left | Key::ISO_Left_Tab if view == LauncherView::Grid => -1,
        _ => return None,
    };
    Some(KeyAction::MoveChoice(step))
}
