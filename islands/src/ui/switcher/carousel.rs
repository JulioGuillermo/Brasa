//! The cards on a drum turning about a level axis: the chosen one faces the viewer, full size, and the
//! others lie on the drum above and below it, turned away and fading. `position` says which card is in
//! front; between two whole numbers the drum is turning. A short list is an arc, not a whole ring, so
//! the cards never meet behind the drum.

use std::cell::{Cell, RefCell};

use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::{glib, graphene, gsk};

pub const CARD_WIDTH: i32 = 260;
pub const CARD_HEIGHT: i32 = 150;
const DRUM_RADIUS: f32 = 270.0;
const DEGREES_PER_CARD: f64 = 34.0;
const PERSPECTIVE_DEPTH: f32 = 900.0;
const FULLY_SHOWN_CARDS: f64 = 1.2;
const FADE_CARDS: f64 = 1.4;
const FADE_AT_THE_BACK: f64 = 0.6;
pub const FEWEST_CARDS_FOR_A_RING: usize = 6;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct Carousel {
        pub cards: RefCell<Vec<gtk4::Widget>>,
        pub position: Cell<f64>,
        pub placed: RefCell<Vec<graphene::Rect>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Carousel {
        const NAME: &'static str = "BrasaCarousel";
        type Type = super::Carousel;
        type ParentType = gtk4::Widget;
    }

    impl ObjectImpl for Carousel {
        fn dispose(&self) {
            for card in self.cards.take() {
                card.unparent();
            }
        }
    }

    impl WidgetImpl for Carousel {
        fn measure(&self, _orientation: gtk4::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
            (0, 0, -1, -1)
        }

        fn size_allocate(&self, width: i32, height: i32, _baseline: i32) {
            let cards = self.cards.borrow();
            let centre = graphene::Point::new(width as f32 / 2.0, height as f32 / 2.0);
            let mut placed = Vec::with_capacity(cards.len());
            for (index, card) in cards.iter().enumerate() {
                let offset = offset_from_front(index, cards.len(), self.position.get());
                let transform = on_the_drum(centre, offset);
                let card_area = graphene::Rect::new(0.0, 0.0, CARD_WIDTH as f32, CARD_HEIGHT as f32);
                placed.push(transform.transform_bounds(&card_area));
                card.allocate(CARD_WIDTH, CARD_HEIGHT, -1, Some(transform));
            }
            *self.placed.borrow_mut() = placed;
        }

        fn snapshot(&self, snapshot: &gtk4::Snapshot) {
            let cards = self.cards.borrow();
            let position = self.position.get();
            for index in back_to_front(cards.len(), position) {
                let opacity = opacity_at(offset_from_front(index, cards.len(), position));
                if opacity <= 0.0 {
                    continue;
                }
                snapshot.push_opacity(opacity);
                self.obj().snapshot_child(&cards[index], snapshot);
                snapshot.pop();
            }
        }
    }
}

glib::wrapper! {
    pub struct Carousel(ObjectSubclass<imp::Carousel>)
        @extends gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for Carousel {
    fn default() -> Self {
        glib::Object::builder().property("hexpand", true).property("vexpand", true).build()
    }
}

impl Carousel {
    pub fn set_cards(&self, cards: Vec<gtk4::Widget>) {
        for old in self.imp().cards.take() {
            old.unparent();
        }
        for card in &cards {
            card.set_parent(self);
            card.set_can_target(false);
        }
        self.imp().cards.replace(cards);
        self.queue_allocate();
    }

    pub fn set_position(&self, position: f64) {
        self.imp().position.set(position);
        self.queue_allocate();
        self.queue_draw();
    }

    /// The card drawn at that point, the front one winning where two overlap.
    pub fn card_at(&self, x: f64, y: f64) -> Option<usize> {
        let placed = self.imp().placed.borrow();
        let position = self.imp().position.get();
        let point = graphene::Point::new(x as f32, y as f32);
        back_to_front(placed.len(), position)
            .into_iter()
            .rev()
            .filter(|index| opacity_at(offset_from_front(*index, placed.len(), position)) > 0.0)
            .find(|index| placed[*index].contains_point(&point))
    }
}

/// How many cards away from the front one; on a ring, going round the shorter way.
fn offset_from_front(index: usize, count: usize, position: f64) -> f64 {
    if count < FEWEST_CARDS_FOR_A_RING {
        return index as f64 - position;
    }
    let count = count as f64;
    let offset = (index as f64 - position).rem_euclid(count);
    if offset > count / 2.0 { offset - count } else { offset }
}

/// Faded with the distance from the front, gone before a ring's cards could meet behind the drum.
fn opacity_at(offset: f64) -> f64 {
    let distance = offset.abs();
    let opacity = if distance <= FULLY_SHOWN_CARDS {
        1.0 - (1.0 - FADE_AT_THE_BACK) * distance / FULLY_SHOWN_CARDS
    } else {
        FADE_AT_THE_BACK * (1.0 - (distance - FULLY_SHOWN_CARDS) / FADE_CARDS)
    };
    opacity.clamp(0.0, 1.0)
}

fn back_to_front(count: usize, position: f64) -> Vec<usize> {
    let mut order: Vec<usize> = (0..count).collect();
    order.sort_by(|first, second| {
        let first = offset_from_front(*first, count, position).abs();
        let second = offset_from_front(*second, count, position).abs();
        second.total_cmp(&first)
    });
    order
}

/// The card's place on the drum, `offset` cards round from the front, seen in perspective from
/// in front of the centre.
fn on_the_drum(centre: graphene::Point, offset: f64) -> gsk::Transform {
    let angle = (offset * DEGREES_PER_CARD) as f32;
    let radians = angle.to_radians();
    let height = DRUM_RADIUS * radians.sin();
    let depth = DRUM_RADIUS * (radians.cos() - 1.0);
    gsk::Transform::new()
        .translate(&centre)
        .perspective(PERSPECTIVE_DEPTH)
        .translate_3d(&graphene::Point3D::new(0.0, height, depth))
        .rotate_3d(-angle, &graphene::Vec3::x_axis())
        .translate(&graphene::Point::new(-CARD_WIDTH as f32 / 2.0, -CARD_HEIGHT as f32 / 2.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn on_a_ring_offsets_go_round_the_shorter_way() {
        assert_eq!(offset_from_front(0, 7, 0.0), 0.0);
        assert_eq!(offset_from_front(1, 7, 0.0), 1.0);
        assert_eq!(offset_from_front(6, 7, 0.0), -1.0);
        assert_eq!(offset_from_front(0, 7, 6.0), 1.0);
        assert_eq!(offset_from_front(2, 7, 9.0), 0.0);
    }

    #[test]
    fn a_short_list_is_an_arc() {
        assert_eq!(offset_from_front(2, 3, 0.0), 2.0);
        assert_eq!(offset_from_front(0, 3, 2.0), -2.0);
    }

    #[test]
    fn the_front_card_is_solid_and_the_far_ones_are_gone() {
        assert_eq!(opacity_at(0.0), 1.0);
        assert!(opacity_at(1.0) < 1.0);
        assert!(opacity_at(-1.0) > 0.0);
        assert_eq!(opacity_at(3.0), 0.0);
    }

    #[test]
    fn the_front_card_is_drawn_last() {
        assert_eq!(back_to_front(3, 1.0).last(), Some(&1));
    }
}
