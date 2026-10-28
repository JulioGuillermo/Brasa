"""The panel without a mouse. Arrows move between tiles, across pages, and down to the sliders and the
buttons at the bottom; Page Up / Page Down turn a page; Enter or Space press; Escape goes back from a
menu, or closes. On a slider, left and right change it."""
from gi.repository import Gdk

ARROWS = {Gdk.KEY_Left: "left", Gdk.KEY_Right: "right", Gdk.KEY_Up: "up", Gdk.KEY_Down: "down"}


class Keys:
    """panel: the Panel, read for its parts (pager, menu, sliders, footer buttons) as they are now."""

    def __init__(self, panel):
        self.panel = panel

    def press(self, keyval) -> bool:
        panel = self.panel
        if keyval == Gdk.KEY_Escape:
            panel.back_or_close()
            return True
        if panel.showing_menu:
            return self.in_menu(keyval)
        if keyval in (Gdk.KEY_Page_Down, Gdk.KEY_Page_Up):
            step = 1 if keyval == Gdk.KEY_Page_Down else -1
            panel.pager.focus_tile((panel.pager.current + step) * panel.pager.per_page)
            return True
        if keyval not in ARROWS:
            return False
        panel.set_focus_visible(True)
        return self.arrow(ARROWS[keyval])

    def in_menu(self, keyval) -> bool:
        menu = self.panel.menu
        focus = self.panel.get_focus()
        if keyval in (Gdk.KEY_Left, Gdk.KEY_BackSpace):
            self.panel.show_tiles()
            return True
        if keyval in (Gdk.KEY_Up, Gdk.KEY_Down):
            self.panel.set_focus_visible(True)
            return menu.move_focus(focus, 1 if keyval == Gdk.KEY_Down else -1)
        return False

    def arrow(self, key) -> bool:
        panel = self.panel
        focus = panel.get_focus()
        tile = panel.pager.focused_index(focus)
        if tile is not None:
            return self.from_tile(tile, key)
        scales = [slider.scale for slider in panel.sliders]
        if focus in scales:
            return self.from_slider(scales.index(focus), key)
        if focus in panel.footer_buttons:
            return self.from_footer(panel.footer_buttons.index(focus), key)
        panel.pager.focus_tile(0)
        return True

    def from_tile(self, index, key) -> bool:
        target = self.panel.pager.neighbour(index, key)
        if target is None:
            self.below_tiles(0)
        else:
            self.panel.pager.focus_tile(target)
        return True

    def from_slider(self, number, key) -> bool:
        if key in ("left", "right"):
            return False
        if key == "up":
            self.above(number)
        else:
            self.below_tiles(number + 1)
        return True

    def from_footer(self, number, key) -> bool:
        buttons = self.panel.footer_buttons
        if key == "up":
            self.above(len(self.panel.sliders))
        elif key == "left":
            buttons[max(number - 1, 0)].grab_focus()
        elif key == "right":
            buttons[min(number + 1, len(buttons) - 1)].grab_focus()
        return True

    def below_tiles(self, slider_number):
        """The slider of that number, or the bottom buttons after the last one."""
        sliders = self.panel.sliders
        if slider_number < len(sliders):
            sliders[slider_number].scale.grab_focus()
        else:
            self.panel.footer_buttons[0].grab_focus()

    def above(self, slider_number):
        """What sits above the slider of that number: the slider before it, or the tiles."""
        if slider_number > 0:
            self.panel.sliders[slider_number - 1].scale.grab_focus()
        else:
            pager = self.panel.pager
            pager.focus_tile(min((pager.current + 1) * pager.per_page, len(pager.tiles)) - 1)
