"""A menu tile opened: its choices in a list, the current one lit, and a way back to the tiles."""
from gi.repository import Gtk, Pango

from . import glyphs, shell
from .items import Menu

MAX_LIST_HEIGHT = 300


class MenuView(Gtk.Box):
    """on_back() for the back arrow; on_chosen(choice) after a choice ran."""

    def __init__(self, on_back, on_chosen):
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=8, css_classes=["quick-menu"])
        self.on_chosen = on_chosen
        self.choice_buttons = []
        self.back = Gtk.Button(label=glyphs.BACK, css_classes=["quick-round"], tooltip_text="Back")
        self.back.connect("clicked", lambda _button: on_back())
        self.title = Gtk.Label(xalign=0, hexpand=True, css_classes=["quick-title"])
        header = Gtk.Box(spacing=8)
        header.append(self.back)
        header.append(self.title)
        self.list = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=4)
        scroller = Gtk.ScrolledWindow(child=self.list, hscrollbar_policy=Gtk.PolicyType.NEVER,
                                      propagate_natural_height=True, max_content_height=MAX_LIST_HEIGHT)
        self.append(header)
        self.append(scroller)

    def show_menu(self, menu: Menu):
        self.title.set_text(menu.name)
        while child := self.list.get_first_child():
            self.list.remove(child)
        self.choice_buttons = []
        for choice in menu.choices:
            label = Gtk.Label(label=choice.label, xalign=0, ellipsize=Pango.EllipsizeMode.END)
            button = Gtk.Button(child=label, css_classes=["quick-choice"])
            button.connect("clicked", lambda _button, choice=choice: self.choose(choice))
            shell.test(choice.current, lambda lit, button=button: self.light(button, lit))
            self.list.append(button)
            self.choice_buttons.append(button)

    def light(self, button: Gtk.Button, lit: bool):
        if not lit:
            return
        button.add_css_class("lit")
        if button.get_root() is not None and button.get_root().get_focus() in self.choice_buttons:
            button.grab_focus()

    def choose(self, choice):
        shell.run(choice.command)
        self.on_chosen(choice)

    # ---- Keyboard

    def focus_first(self):
        if self.choice_buttons:
            self.choice_buttons[0].grab_focus()

    def move_focus(self, focus, step) -> bool:
        """Up and down through the back arrow and the choices."""
        order = [self.back, *self.choice_buttons]
        if focus not in order:
            self.focus_first()
            return True
        index = max(0, min(order.index(focus) + step, len(order) - 1))
        order[index].grab_focus()
        return True
