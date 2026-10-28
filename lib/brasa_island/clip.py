"""A container that lays its child out at full size but only draws the part inside `visible_area`, so
the island's content is revealed as the outline grows instead of spilling over it."""
import gi

gi.require_version("Graphene", "1.0")
from gi.repository import Graphene, Gtk


class Clip(Gtk.Widget):
    def __init__(self, child: Gtk.Widget):
        super().__init__()
        self.child = child
        self.visible_area = Graphene.Rect().init(0, 0, 0, 0)
        child.set_parent(self)

    def show_only(self, x: float, y: float, width: float, height: float):
        self.visible_area = Graphene.Rect().init(x, y, max(width, 0), max(height, 0))
        self.queue_draw()

    def do_measure(self, orientation, for_size):
        return self.child.measure(orientation, for_size)

    def do_size_allocate(self, width, height, baseline):
        self.child.allocate(width, height, baseline, None)

    def do_snapshot(self, snapshot):
        snapshot.push_clip(self.visible_area)
        self.snapshot_child(self.child, snapshot)
        snapshot.pop()
