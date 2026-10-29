//! The widget an island or the quick panel is made of: it paints the outline (through the GPU, with
//! GSK paths and gradients) and shows its content clipped to it, so the content is revealed as the
//! outline grows instead of spilling over it. Its owner tells it where the content and the clip go.

use std::cell::{Cell, RefCell};

use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::{glib, graphene, gsk};

pub type Painter = Box<dyn Fn(&gtk4::Snapshot)>;

mod imp {
    use super::*;

    pub struct Frame {
        pub content: RefCell<Option<gtk4::Widget>>,
        pub painter: RefCell<Option<Painter>>,
        pub content_area: Cell<graphene::Rect>,
        pub visible_area: Cell<graphene::Rect>,
    }

    impl Default for Frame {
        fn default() -> Self {
            Frame {
                content: RefCell::new(None),
                painter: RefCell::new(None),
                content_area: Cell::new(graphene::Rect::zero()),
                visible_area: Cell::new(graphene::Rect::zero()),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Frame {
        const NAME: &'static str = "BrasaFrame";
        type Type = super::Frame;
        type ParentType = gtk4::Widget;
    }

    impl ObjectImpl for Frame {
        fn dispose(&self) {
            if let Some(content) = self.content.take() {
                content.unparent();
            }
        }
    }

    impl WidgetImpl for Frame {
        fn measure(&self, _orientation: gtk4::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
            (0, 0, -1, -1)
        }

        fn size_allocate(&self, _width: i32, _height: i32, _baseline: i32) {
            let Some(content) = self.content.borrow().clone() else { return };
            let area = self.content_area.get();
            let at = gsk::Transform::new().translate(&graphene::Point::new(area.x(), area.y()));
            content.allocate(area.width() as i32, area.height() as i32, -1, Some(at));
        }

        fn snapshot(&self, snapshot: &gtk4::Snapshot) {
            if let Some(paint) = self.painter.borrow().as_ref() {
                paint(snapshot);
            }
            let Some(content) = self.content.borrow().clone() else { return };
            snapshot.push_clip(&self.visible_area.get());
            self.obj().snapshot_child(&content, snapshot);
            snapshot.pop();
        }
    }
}

glib::wrapper! {
    pub struct Frame(ObjectSubclass<imp::Frame>)
        @extends gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Frame {
    pub fn new(content: &impl IsA<gtk4::Widget>, painter: Painter) -> Self {
        let frame: Frame = glib::Object::builder().property("hexpand", true).property("vexpand", true).build();
        content.set_parent(&frame);
        frame.imp().content.replace(Some(content.clone().upcast()));
        frame.imp().painter.replace(Some(painter));
        frame
    }

    /// Where the content is laid out, at its full size, and the part of it that is drawn.
    pub fn place_content(&self, content_area: graphene::Rect, visible_area: graphene::Rect) {
        self.imp().content_area.set(content_area);
        self.imp().visible_area.set(visible_area);
        self.queue_allocate();
        self.queue_draw();
    }
}
