"""A menu tile opened: its choices in a list, the current one lit, and a way back to the tiles."""
from gi.repository import Gtk, Pango

from . import shell
from .items import Menu


def light_if(button: Gtk.Button, lit: bool):
    if lit:
        button.add_css_class("lit")


class MenuView(Gtk.Box):
    """on_back() for the back arrow; on_chosen(choice) after a choice ran."""

    def __init__(self, on_back, on_chosen):
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=8, css_classes=["quick-menu"])
        self.on_chosen = on_chosen
        back = Gtk.Button(icon_name="go-previous-symbolic", css_classes=["quick-round"], tooltip_text="Back")
        back.connect("clicked", lambda _button: on_back())
        self.title = Gtk.Label(xalign=0, hexpand=True, css_classes=["quick-title"])
        header = Gtk.Box(spacing=8)
        header.append(back)
        header.append(self.title)
        self.list = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=4)
        scroller = Gtk.ScrolledWindow(child=self.list, hscrollbar_policy=Gtk.PolicyType.NEVER,
                                      propagate_natural_height=True, max_content_height=260)
        self.append(header)
        self.append(scroller)

    def show_menu(self, menu: Menu):
        self.title.set_text(menu.name)
        while child := self.list.get_first_child():
            self.list.remove(child)
        for choice in menu.choices:
            label = Gtk.Label(label=choice.label, xalign=0, ellipsize=Pango.EllipsizeMode.END)
            button = Gtk.Button(child=label, css_classes=["quick-choice"])
            button.connect("clicked", lambda _button, choice=choice: self.choose(choice))
            shell.test(choice.current, lambda lit, button=button: light_if(button, lit))
            self.list.append(button)

    def choose(self, choice):
        shell.run(choice.command)
        self.on_chosen(choice)
