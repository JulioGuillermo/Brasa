//! The window switcher: an island hanging from the left edge of the focused monitor, halfway down,
//! with the open windows on a turning drum and the chosen one in front. Super+Tab opens it on this
//! workspace's windows and Super+Shift+Tab on all of them; each press turns to the next, and letting
//! Super go focuses the one in front. Opened by its own key it stays until Enter, a click or Escape.

mod card;
mod carousel;
mod keys;

use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};
use std::time::Duration;

use gtk4::prelude::*;
use gtk4::{gdk, glib, graphene};
use gtk4_layer_shell::{Edge as LayerEdge, KeyboardMode, Layer, LayerShell};

use crate::config;
use crate::runtime::workers;
use crate::services::compositor::{self, Scope, Window};
use crate::ui::shell::click_away::ClickAway;
use crate::ui::shell::frame::Frame;
use crate::ui::shell::island::parse_color;
use crate::ui::shell::monitors;
use crate::ui::shell::outline::Outline;
use crate::ui::shell::spring::Spring;
use carousel::{CARD_HEIGHT, CARD_WIDTH, Carousel, FEWEST_CARDS_FOR_A_RING};
use keys::Action;

const PADDING: f64 = 26.0;
const REACH: f64 = CARD_WIDTH as f64 + 2.0 * PADDING;
const LONGEST: f64 = 620.0;
const SHORTEST: f64 = CARD_HEIGHT as f64 + 2.0 * PADDING;
const SHOULDER: f64 = 110.0;
const RADIUS: f64 = 38.0;
const SCREEN_MARGIN: f64 = 40.0;
const SLACK: f64 = 30.0;
/// As in the islands: the window reaches past the screen edge, so a fractional scale cannot leave a
/// half drawn border on it.
const OVERHANG: f64 = 2.0;
const CONTENT_FADE_SECONDS: f64 = 0.12;
const MAX_FRAME_SECONDS: f64 = 1.0 / 30.0;
const SUPER_CHECK_EVERY: Duration = Duration::from_millis(40);
const KEYBOARD_RELEASE_WAIT: Duration = Duration::from_millis(120);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Opening {
    /// Super+Tab: the window in front gets the focus when Super is let go.
    WhileSuperHeld,
    /// Its own key: it stays until a window is chosen or it is closed.
    UntilChosen,
}

#[derive(Clone, Copy)]
struct Painted {
    outline: Outline,
    color: gdk::RGBA,
    middle: f64,
}

struct Springs {
    reach: Spring,
    length: Spring,
    shoulder: Spring,
    radius: Spring,
    position: Spring,
}

impl Springs {
    fn all(&mut self) -> [&mut Spring; 5] {
        [&mut self.reach, &mut self.length, &mut self.shoulder, &mut self.radius, &mut self.position]
    }
}

struct State {
    opening: Option<Opening>,
    windows: Vec<Window>,
    cards: Vec<gtk4::Widget>,
    chosen: usize,
    /// Where the drum turns to: it only grows or shrinks, so turning past the last card goes on round.
    turned_to: f64,
    longest: f64,
    springs: Springs,
    ticking: bool,
    last_frame: Option<f64>,
    super_watch: Option<glib::SourceId>,
}

pub struct Switcher {
    me: Weak<Switcher>,
    window: gtk4::Window,
    frame: Frame,
    carousel: Carousel,
    painted: Rc<Cell<Painted>>,
    state: RefCell<State>,
    click_away: ClickAway,
}

impl Switcher {
    pub fn new(app: &gtk4::Application) -> Rc<Switcher> {
        let painted = Rc::new(Cell::new(Painted {
            outline: Outline { center: 0.0, width: 0.0, height: 0.0, shoulder: 0.0, radius: 0.0 },
            color: gdk::RGBA::BLACK,
            middle: 0.0,
        }));
        let for_painter = Rc::clone(&painted);
        let painter = Box::new(move |snapshot: &gtk4::Snapshot| {
            let painted = for_painter.get();
            snapshot.save();
            snapshot.translate(&graphene::Point::new(OVERHANG as f32, painted.middle as f32));
            snapshot.rotate(-90.0);
            painted.outline.paint(snapshot, &painted.color);
            snapshot.restore();
        });
        let carousel = Carousel::default();
        carousel.set_opacity(0.0);
        let frame = Frame::new(&carousel, painter);
        let window = gtk4::Window::builder()
            .application(app)
            .css_classes(["brasa-island", "brasa-switcher"])
            .default_width((OVERHANG + REACH + SLACK) as i32)
            .child(&frame)
            .build();

        let switcher = Rc::new_cyclic(|me: &Weak<Switcher>| Switcher {
            me: me.clone(),
            window,
            frame,
            carousel,
            painted,
            state: RefCell::new(State {
                opening: None,
                windows: Vec::new(),
                cards: Vec::new(),
                chosen: 0,
                turned_to: 0.0,
                longest: LONGEST,
                springs: Springs {
                    reach: Spring::new(0.0),
                    length: Spring::new(0.0),
                    shoulder: Spring::new(0.0),
                    radius: Spring::new(0.0),
                    position: Spring::new(0.0),
                },
                ticking: false,
                last_frame: None,
                super_watch: None,
            }),
            click_away: ClickAway::new(app, close_of(me.clone())),
        });
        switcher.place_on_screen();
        switcher.listen();
        switcher
    }

    fn place_on_screen(&self) {
        self.window.init_layer_shell();
        self.window.set_namespace(Some("brasa-switcher"));
        self.window.set_layer(Layer::Overlay);
        self.window.set_exclusive_zone(-1);
        for edge in [LayerEdge::Left, LayerEdge::Top, LayerEdge::Bottom] {
            self.window.set_anchor(edge, true);
        }
        self.window.set_margin(LayerEdge::Left, -(OVERHANG as i32));
    }

    pub fn is_open(&self) -> bool {
        self.state.borrow().opening.is_some()
    }

    /// Super+Tab and Super+Shift+Tab: opens it, or turns it to the next window when it is open.
    pub fn switch(&self, scope: Scope) {
        if self.is_open() {
            self.turn(1);
        } else {
            self.open(scope, Opening::WhileSuperHeld);
        }
    }

    /// Its own key: every window, until one is chosen.
    pub fn toggle(&self) {
        if self.is_open() {
            self.close();
        } else {
            self.open(Scope::All, Opening::UntilChosen);
        }
    }

    fn open(&self, scope: Scope, opening: Opening) {
        let windows = compositor::windows(scope);
        if windows.is_empty() {
            return;
        }
        let Some(monitor) = monitors::focused_or_first() else { return };
        let skips_the_focused_one = opening == Opening::WhileSuperHeld && windows.len() > 1 && windows[0].focused;
        let chosen = usize::from(skips_the_focused_one);
        let cards: Vec<gtk4::Widget> = windows.iter().map(card::card).collect();
        self.carousel.set_cards(cards.clone());
        self.carousel.set_position(chosen as f64);
        {
            let mut state = self.state.borrow_mut();
            state.opening = Some(opening);
            state.windows = windows;
            state.cards = cards;
            state.chosen = chosen;
            state.turned_to = chosen as f64;
            state.springs.position = Spring::new(chosen as f64);
        }
        self.mark_chosen();
        self.place_on(&monitor);

        self.window.set_keyboard_mode(KeyboardMode::Exclusive);
        self.click_away.show();
        self.window.present();
        if opening == Opening::WhileSuperHeld {
            self.watch_super();
        }
        self.fit();
    }

    fn place_on(&self, monitor: &gdk::Monitor) {
        self.window.set_monitor(Some(monitor));
        let height = monitor.geometry().height() as f64;
        let longest = LONGEST.min(height - 2.0 * SHOULDER - SCREEN_MARGIN).max(SHORTEST);
        self.state.borrow_mut().longest = longest;
        let mut painted = self.painted.get();
        painted.middle = height / 2.0;
        painted.color = parse_color(&config::load().color);
        self.painted.set(painted);
    }

    pub fn close(&self) {
        self.stop_watching_super();
        self.state.borrow_mut().opening = None;
        self.window.set_keyboard_mode(KeyboardMode::None);
        self.click_away.hide();
        self.fit();
    }

    /// Focuses the window in front once the switcher has let the keyboard go: letting it go makes
    /// Hyprland give the focus back to the window that had it, which would undo an earlier focus.
    fn choose(&self) {
        let chosen = {
            let state = self.state.borrow();
            state.windows.get(state.chosen).cloned()
        };
        self.close();
        let Some(chosen) = chosen else { return };
        glib::timeout_add_local_once(KEYBOARD_RELEASE_WAIT, move || {
            workers::run(move || compositor::focus(&chosen));
        });
    }

    fn turn(&self, step: i64) {
        {
            let mut state = self.state.borrow_mut();
            let count = state.windows.len();
            if count == 0 {
                return;
            }
            state.chosen = (state.chosen as i64 + step).rem_euclid(count as i64) as usize;
            state.turned_to = if count >= FEWEST_CARDS_FOR_A_RING {
                state.turned_to + step as f64
            } else {
                state.chosen as f64
            };
        }
        self.mark_chosen();
        self.fit();
    }

    fn mark_chosen(&self) {
        let state = self.state.borrow();
        for (index, card) in state.cards.iter().enumerate() {
            if index == state.chosen {
                card.add_css_class("chosen");
            } else {
                card.remove_css_class("chosen");
            }
        }
    }

    // ---- Keys, clicks and the wheel

    fn listen(&self) {
        let typing = gtk4::EventControllerKey::new();
        typing.set_propagation_phase(gtk4::PropagationPhase::Capture);
        let me = self.me.clone();
        typing.connect_key_pressed(move |_, key, _, _| {
            let used = me.upgrade().is_some_and(|switcher| switcher.on_key(key));
            if used { glib::Propagation::Stop } else { glib::Propagation::Proceed }
        });
        let me = self.me.clone();
        typing.connect_key_released(move |_, key, _, _| {
            if let Some(switcher) = me.upgrade() {
                if keys::is_super(key) {
                    switcher.on_super_let_go();
                }
            }
        });
        self.window.add_controller(typing);

        let click = gtk4::GestureClick::new();
        click.set_button(0);
        let me = self.me.clone();
        click.connect_pressed(move |_, _, x, y| {
            if let Some(switcher) = me.upgrade() {
                switcher.on_press(x, y);
            }
        });
        self.window.add_controller(click);

        let wheel = gtk4::EventControllerScroll::new(
            gtk4::EventControllerScrollFlags::VERTICAL | gtk4::EventControllerScrollFlags::DISCRETE,
        );
        let me = self.me.clone();
        wheel.connect_scroll(move |_, _, dy| {
            if let Some(switcher) = me.upgrade() {
                switcher.turn(if dy > 0.0 { 1 } else { -1 });
            }
            glib::Propagation::Stop
        });
        self.window.add_controller(wheel);
    }

    fn on_key(&self, key: gdk::Key) -> bool {
        if !self.is_open() {
            return false;
        }
        match keys::action_for(key) {
            Some(Action::Next) => self.turn(1),
            Some(Action::Previous) => self.turn(-1),
            Some(Action::Choose) => self.choose(),
            Some(Action::Cancel) => self.close(),
            None => return false,
        }
        true
    }

    fn on_press(&self, x: f64, y: f64) {
        if !self.is_open() {
            return;
        }
        let body = self.body();
        if !body.contains_point(&graphene::Point::new(x as f32, y as f32)) {
            self.close();
            return;
        }
        let content = self.content_area();
        let card = self.carousel.card_at(x - content.x() as f64, y - content.y() as f64);
        if let Some(card) = card {
            self.state.borrow_mut().chosen = card;
            self.choose();
        }
    }

    // ---- Letting Super go

    fn on_super_let_go(&self) {
        if self.state.borrow().opening == Some(Opening::WhileSuperHeld) {
            self.choose();
        }
    }

    /// Super may be let go before the switcher has the keyboard, and then no key release reaches it:
    /// so, once it has the keyboard, it also looks at the modifiers held.
    fn watch_super(&self) {
        self.stop_watching_super();
        let me = self.me.clone();
        let watch = glib::timeout_add_local(SUPER_CHECK_EVERY, move || {
            let Some(switcher) = me.upgrade() else { return glib::ControlFlow::Break };
            if switcher.window.is_active() && !super_held() {
                switcher.state.borrow_mut().super_watch = None;
                switcher.on_super_let_go();
                return glib::ControlFlow::Break;
            }
            glib::ControlFlow::Continue
        });
        self.state.borrow_mut().super_watch = Some(watch);
    }

    fn stop_watching_super(&self) {
        let watch = self.state.borrow_mut().super_watch.take();
        if let Some(watch) = watch {
            watch.remove();
        }
    }

    // ---- Shape and animation

    fn retarget(&self) {
        let mut state = self.state.borrow_mut();
        let is_open = state.opening.is_some();
        let longest = state.longest;
        let turned_to = state.turned_to;
        let springs = &mut state.springs;
        springs.position.target = turned_to;
        if is_open {
            springs.reach.target = REACH;
            springs.length.target = longest;
            springs.shoulder.target = SHOULDER;
            springs.radius.target = RADIUS;
        } else {
            springs.reach.target = 0.0;
            springs.length.target = 0.0;
            springs.shoulder.target = 0.0;
            springs.radius.target = 0.0;
        }
    }

    fn fit(&self) {
        let mut state = self.state.borrow_mut();
        if state.ticking {
            return;
        }
        state.ticking = true;
        state.last_frame = None;
        let me = self.me.clone();
        self.frame.add_tick_callback(move |_, clock| match me.upgrade() {
            Some(switcher) => switcher.on_frame(clock),
            None => glib::ControlFlow::Break,
        });
    }

    fn on_frame(&self, clock: &gdk::FrameClock) -> glib::ControlFlow {
        let now = clock.frame_time() as f64 / 1e6;
        let seconds = {
            let mut state = self.state.borrow_mut();
            let seconds = state.last_frame.map_or(0.0, |last| (now - last).min(MAX_FRAME_SECONDS));
            state.last_frame = Some(now);
            seconds
        };
        self.retarget();
        let (springs_moving, position) = {
            let mut state = self.state.borrow_mut();
            let mut moving = false;
            for spring in state.springs.all() {
                spring.step(seconds);
                moving |= !spring.at_rest();
            }
            (moving, state.springs.position.value)
        };
        self.carousel.set_position(position);
        let fading = self.fade_cards(seconds);
        self.show_outline();
        if fading || springs_moving {
            return glib::ControlFlow::Continue;
        }
        self.state.borrow_mut().ticking = false;
        if !self.is_open() {
            self.window.set_visible(false);
            self.carousel.set_cards(Vec::new());
            self.state.borrow_mut().cards.clear();
        }
        glib::ControlFlow::Break
    }

    fn fade_cards(&self, seconds: f64) -> bool {
        let target = if self.is_open() { 1.0 } else { 0.0 };
        let step = seconds / CONTENT_FADE_SECONDS;
        let opacity = self.carousel.opacity();
        let opacity = if target > opacity { (opacity + step).min(target) } else { (opacity - step).max(target) };
        self.carousel.set_opacity(opacity);
        opacity != target
    }

    fn show_outline(&self) {
        let (reach, length, shoulder, radius) = {
            let springs = &self.state.borrow().springs;
            (springs.reach.value, springs.length.value, springs.shoulder.value, springs.radius.value)
        };
        let mut painted = self.painted.get();
        painted.outline = Outline {
            center: 0.0,
            width: length as f32,
            height: reach as f32,
            shoulder: shoulder as f32,
            radius: radius as f32,
        };
        self.painted.set(painted);
        self.frame.place_content(self.content_area(), self.body());
    }

    /// The cards' area at full size, in the window: the island's body once it has grown.
    fn content_area(&self) -> graphene::Rect {
        let longest = self.state.borrow().longest;
        let middle = self.painted.get().middle;
        graphene::Rect::new(OVERHANG as f32, (middle - longest / 2.0) as f32, REACH as f32, longest as f32)
    }

    /// The island's body as the springs have it now, shoulders left out.
    fn body(&self) -> graphene::Rect {
        let painted = self.painted.get();
        let outline = painted.outline;
        let top = painted.middle as f32 - outline.width / 2.0;
        graphene::Rect::new(OVERHANG as f32, top, outline.height, outline.width)
    }
}

fn super_held() -> bool {
    let keyboard = gdk::Display::default().and_then(|display| display.default_seat()).and_then(|seat| seat.keyboard());
    keyboard.is_some_and(|keyboard| keyboard.modifier_state().contains(gdk::ModifierType::SUPER_MASK))
}

fn close_of(switcher: Weak<Switcher>) -> impl Fn() {
    move || {
        if let Some(switcher) = switcher.upgrade() {
            switcher.close();
        }
    }
}
