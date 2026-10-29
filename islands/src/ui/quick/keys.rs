//! The panel without a mouse. Arrows move between tiles, across pages, and down to the sliders and the
//! buttons at the bottom; Page Up / Page Down turn a page; Enter or Space press. In a menu, up and down
//! walk the choices and left goes back. On a slider, left and right change it.

use gtk4::gdk::Key;
use gtk4::prelude::*;

use super::card::Card;
use super::grid::Arrow;

/// Whether the key was used. Escape is the panel's own.
pub fn press(card: &Card, window: &gtk4::Window, key: Key) -> bool {
    if card.showing_menu() {
        return in_menu(card, window, key);
    }
    if key == Key::Page_Down || key == Key::Page_Up {
        let page = card.pager.current_page();
        let target_page = if key == Key::Page_Down { page + 1 } else { page.saturating_sub(1) };
        card.pager.focus_tile(target_page * card.pager.per_page());
        return true;
    }
    let Some(arrow) = arrow_of(key) else { return false };
    window.set_focus_visible(true);
    move_by_arrow(card, focused(window).as_ref(), arrow)
}

fn arrow_of(key: Key) -> Option<Arrow> {
    match key {
        Key::Left => Some(Arrow::Left),
        Key::Right => Some(Arrow::Right),
        Key::Up => Some(Arrow::Up),
        Key::Down => Some(Arrow::Down),
        _ => None,
    }
}

fn in_menu(card: &Card, window: &gtk4::Window, key: Key) -> bool {
    if key == Key::Left || key == Key::BackSpace {
        card.show_tiles();
        return true;
    }
    if key == Key::Up || key == Key::Down {
        window.set_focus_visible(true);
        card.menu.move_focus(focused(window).as_ref(), if key == Key::Down { 1 } else { -1 });
        return true;
    }
    false
}

fn move_by_arrow(card: &Card, focus: Option<&gtk4::Widget>, arrow: Arrow) -> bool {
    if let Some(tile) = card.pager.focused_index(focus) {
        from_tile(card, tile, arrow);
        return true;
    }
    if let Some(number) = position_of(focus, card.sliders.iter().map(|slider| slider.scale.upcast_ref())) {
        return from_slider(card, number, arrow);
    }
    if let Some(number) = position_of(focus, card.footer.buttons.iter().map(|button| button.upcast_ref())) {
        from_footer(card, number, arrow);
        return true;
    }
    card.pager.focus_tile(0);
    true
}

fn position_of<'a>(focus: Option<&gtk4::Widget>, mut widgets: impl Iterator<Item = &'a gtk4::Widget>) -> Option<usize> {
    let focus = focus?;
    widgets.position(|widget| widget == focus)
}

fn from_tile(card: &Card, index: usize, arrow: Arrow) {
    match card.pager.neighbour(index, arrow) {
        Some(target) => card.pager.focus_tile(target),
        None => below_tiles(card, 0),
    }
}

fn from_slider(card: &Card, number: usize, arrow: Arrow) -> bool {
    match arrow {
        Arrow::Left | Arrow::Right => return false,
        Arrow::Up => above(card, number),
        Arrow::Down => below_tiles(card, number + 1),
    }
    true
}

fn from_footer(card: &Card, number: usize, arrow: Arrow) {
    let buttons = &card.footer.buttons;
    match arrow {
        Arrow::Up => above(card, card.sliders.len()),
        Arrow::Left => grab(&buttons[number.saturating_sub(1)]),
        Arrow::Right => grab(&buttons[(number + 1).min(buttons.len() - 1)]),
        Arrow::Down => {}
    }
}

/// The slider of that number, or the bottom buttons after the last one.
fn below_tiles(card: &Card, slider_number: usize) {
    match card.sliders.get(slider_number) {
        Some(slider) => grab(&slider.scale),
        None => grab(&card.footer.buttons[0]),
    }
}

/// What sits above the slider of that number: the slider before it, or the tiles.
fn above(card: &Card, slider_number: usize) {
    if slider_number > 0 {
        grab(&card.sliders[slider_number - 1].scale);
        return;
    }
    let pager = &card.pager;
    let last_of_page = ((pager.current_page() + 1) * pager.per_page()).min(pager.tiles.len()) - 1;
    pager.focus_tile(last_of_page);
}

fn focused(window: &gtk4::Window) -> Option<gtk4::Widget> {
    GtkWindowExt::focus(window)
}

fn grab(widget: &impl IsA<gtk4::Widget>) {
    widget.grab_focus();
}
