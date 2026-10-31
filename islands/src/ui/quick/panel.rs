//! The quick settings window: a layer-shell surface covering the focused monitor, so a click anywhere
//! else closes it. The outline grows out of a bottom corner, driven by springs, and the card is shown
//! clipped to it. Built anew at every opening, so it shows this machine's tiles and panel.conf's.

use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};
use std::time::Duration;

use gtk4::prelude::*;
use gtk4::{gdk, glib, graphene};
use gtk4_layer_shell::{Edge as LayerEdge, KeyboardMode, Layer, LayerShell};

use super::card::Card;
use super::corner::{Corner, OVERHANG, Placement};
use super::keys;
use crate::config::{self, Side};
use crate::runtime::events::EventSender;
use crate::services::process;
use crate::ui::shell::click_away::ClickAway;
use crate::ui::shell::frame::Frame;
use crate::ui::shell::island::parse_color;
use crate::ui::shell::monitors;
use crate::ui::shell::spring::Spring;

const PADDING: f64 = 22.0;
const RADIUS: f64 = 34.0;
const SHOULDER: f64 = 90.0;
const CONTENT_FADE_SECONDS: f64 = 0.15;
const MAX_FRAME_SECONDS: f64 = 1.0 / 30.0;
const SLIDERS_EVERY: Duration = Duration::from_millis(400);
const SWITCHES_EVERY: Duration = Duration::from_millis(2000);
const LEAVE_DELAY: Duration = Duration::from_millis(400);

#[derive(Clone, Copy)]
struct Painted {
    corner: Corner,
    placement: Placement,
    color: gdk::RGBA,
}

struct Springs {
    width: Spring,
    height: Spring,
    shoulder: Spring,
    radius: Spring,
}

impl Springs {
    fn all(&mut self) -> [&mut Spring; 4] {
        [&mut self.width, &mut self.height, &mut self.shoulder, &mut self.radius]
    }
}

struct State {
    is_open: bool,
    /// Opened by resting the pointer in the corner: it closes when the pointer leaves it.
    opened_by_pointer: bool,
    leaving: Option<glib::SourceId>,
    on_right: bool,
    card: Option<Rc<Card>>,
    springs: Springs,
    ticking: bool,
    last_frame: Option<f64>,
    screen: (f64, f64),
    watches: Vec<glib::SourceId>,
}

pub struct Panel {
    me: Weak<Panel>,
    window: gtk4::Window,
    frame: Frame,
    holder: gtk4::Box,
    events: EventSender,
    painted: Rc<Cell<Painted>>,
    state: RefCell<State>,
    click_away: ClickAway,
}

impl Panel {
    pub fn new(app: &gtk4::Application, events: &EventSender) -> Rc<Panel> {
        let painted = Rc::new(Cell::new(Painted {
            corner: Corner::closed(),
            placement: Placement::in_window(0.0, 0.0, true),
            color: gdk::RGBA::BLACK,
        }));
        let for_painter = Rc::clone(&painted);
        let painter = Box::new(move |snapshot: &gtk4::Snapshot| {
            let painted = for_painter.get();
            snapshot.save();
            painted.placement.enter_corner_frame(snapshot);
            painted.corner.paint(snapshot, &painted.color);
            snapshot.restore();
        });
        let holder = gtk4::Box::builder().opacity(0.0).build();
        let frame = Frame::new(&holder, painter);
        let window = gtk4::Window::builder().application(app).css_classes(["brasa-quick"]).child(&frame).build();

        let panel = Rc::new_cyclic(|me| Panel {
            me: me.clone(),
            window,
            frame,
            holder,
            events: events.clone(),
            painted,
            state: RefCell::new(State {
                is_open: false,
                opened_by_pointer: false,
                leaving: None,
                on_right: true,
                card: None,
                springs: Springs {
                    width: Spring::new(0.0),
                    height: Spring::new(0.0),
                    shoulder: Spring::new(0.0),
                    radius: Spring::new(0.0),
                },
                ticking: false,
                last_frame: None,
                screen: (0.0, 0.0),
                watches: Vec::new(),
            }),
            click_away: ClickAway::new(app, close_of(me.clone())),
        });
        panel.place_on_screen();
        panel.listen();
        panel
    }

    fn place_on_screen(&self) {
        self.window.init_layer_shell();
        self.window.set_namespace(Some("brasa-quick"));
        self.window.set_layer(Layer::Overlay);
        self.window.set_exclusive_zone(-1);
    }

    fn listen(&self) {
        let outside = gtk4::GestureClick::new();
        let me = self.me.clone();
        outside.connect_pressed(move |_, _, x, y| {
            if let Some(panel) = me.upgrade() {
                panel.on_press(x, y);
            }
        });
        self.window.add_controller(outside);

        let typing = gtk4::EventControllerKey::new();
        typing.set_propagation_phase(gtk4::PropagationPhase::Capture);
        let me = self.me.clone();
        typing.connect_key_pressed(move |_, key, _, _| {
            let used = me.upgrade().is_some_and(|panel| panel.on_key(key));
            if used { glib::Propagation::Stop } else { glib::Propagation::Proceed }
        });
        self.window.add_controller(typing);

        let pointer = gtk4::EventControllerMotion::new();
        let me = self.me.clone();
        pointer.connect_motion(move |_, x, y| {
            if let Some(panel) = me.upgrade() {
                panel.on_pointer_at(x, y);
            }
        });
        let me = self.me.clone();
        pointer.connect_leave(move |_| {
            if let Some(panel) = me.upgrade() {
                panel.close_soon_if_opened_by_pointer();
            }
        });
        self.window.add_controller(pointer);
    }

    fn card(&self) -> Option<Rc<Card>> {
        self.state.borrow().card.clone()
    }

    pub fn is_open(&self) -> bool {
        self.state.borrow().is_open
    }

    // ---- Opening and closing

    /// Super+a and the bar's gear: on its own monitor, or the focused one.
    pub fn toggle(&self) {
        if self.is_open() {
            self.close();
            return;
        }
        let settings = config::load();
        let monitor = monitors::find(&settings.quick_monitor).or_else(monitors::focused_or_first);
        if let Some(monitor) = monitor {
            self.open(&monitor, settings, false);
        }
    }

    /// The pointer rested in the hot corner of that monitor.
    pub fn open_by_pointer(&self, monitor: &gdk::Monitor) {
        if !self.is_open() {
            self.open(monitor, config::load(), true);
        }
    }

    fn open(&self, monitor: &gdk::Monitor, settings: config::Config, by_pointer: bool) {
        let on_right = settings.quick_side == Side::Right;
        let card = self.build_card(settings.quick_size);
        self.show_card(&card);
        {
            let mut state = self.state.borrow_mut();
            state.on_right = on_right;
            state.is_open = true;
            state.opened_by_pointer = by_pointer;
            state.card = Some(Rc::clone(&card));
        }
        let mut painted = self.painted.get();
        painted.color = parse_color(&settings.color);
        self.painted.set(painted);

        self.cover(monitor, on_right);
        if by_pointer {
            self.window.set_keyboard_mode(KeyboardMode::OnDemand);
        } else {
            self.window.set_keyboard_mode(KeyboardMode::Exclusive);
            self.click_away.show();
        }
        self.window.present();
        card.pager.refresh();
        card.refresh_sliders();
        self.watch();
        card.pager.focus_tile(0);
        self.fit();
    }

    fn build_card(&self, size: config::Size) -> Rc<Card> {
        let for_command = self.me.clone();
        let for_resize = self.me.clone();
        Card::new(
            size,
            &self.events,
            Rc::new(move |command| {
                if let Some(panel) = for_command.upgrade() {
                    panel.close_and_run(command);
                }
            }),
            Rc::new(move || {
                if let Some(panel) = for_resize.upgrade() {
                    panel.fit();
                }
            }),
        )
    }

    fn show_card(&self, card: &Card) {
        while let Some(old) = self.holder.first_child() {
            self.holder.remove(&old);
        }
        self.holder.append(&card.root);
        self.holder.set_opacity(0.0);
    }

    /// The monitor's size, moved OVERHANG past its bottom and side edges.
    fn cover(&self, monitor: &gdk::Monitor, on_right: bool) {
        self.window.set_monitor(Some(monitor));
        let geometry = monitor.geometry();
        self.window.set_default_size(geometry.width(), geometry.height());
        self.state.borrow_mut().screen = (geometry.width() as f64, geometry.height() as f64);
        let anchored = [
            (LayerEdge::Right, on_right),
            (LayerEdge::Left, !on_right),
            (LayerEdge::Bottom, true),
            (LayerEdge::Top, false),
        ];
        for (edge, is_anchored) in anchored {
            self.window.set_anchor(edge, is_anchored);
            self.window.set_margin(edge, if is_anchored { -(OVERHANG as i32) } else { 0 });
        }
    }

    pub fn close(&self) {
        self.cancel_leaving();
        self.state.borrow_mut().is_open = false;
        self.stop_watching();
        self.window.set_keyboard_mode(KeyboardMode::None);
        self.click_away.hide();
        self.fit();
    }

    fn close_and_run(&self, command: &str) {
        self.close();
        process::detach(command);
    }

    fn back_or_close(&self) {
        match self.card() {
            Some(card) if card.showing_menu() => card.show_tiles(),
            _ => self.close(),
        }
    }

    // ---- Events

    pub fn on_tested(&self, test: &str, passed: bool) {
        if let Some(card) = self.card() {
            card.on_tested(test, passed);
        }
    }

    fn on_press(&self, x: f64, y: f64) {
        let painted = self.painted.get();
        let (corner_x, corner_y) = painted.placement.to_corner(x, y);
        if !painted.corner.holds(corner_x, corner_y) {
            self.close();
        }
    }

    fn on_pointer_at(&self, x: f64, y: f64) {
        let painted = self.painted.get();
        let (corner_x, corner_y) = painted.placement.to_corner(x, y);
        if painted.corner.holds(corner_x, corner_y) {
            self.cancel_leaving();
        } else {
            self.close_soon_if_opened_by_pointer();
        }
    }

    pub fn close_soon_if_opened_by_pointer(&self) {
        let state = self.state.borrow();
        if !state.is_open || !state.opened_by_pointer || state.leaving.is_some() {
            return;
        }
        drop(state);
        let me = self.me.clone();
        let source = glib::timeout_add_local_once(LEAVE_DELAY, move || {
            let Some(panel) = me.upgrade() else { return };
            panel.state.borrow_mut().leaving = None;
            panel.close();
        });
        self.state.borrow_mut().leaving = Some(source);
    }

    fn cancel_leaving(&self) {
        let leaving = self.state.borrow_mut().leaving.take();
        if let Some(source) = leaving {
            source.remove();
        }
    }

    fn on_key(&self, key: gdk::Key) -> bool {
        if key == gdk::Key::Escape {
            self.back_or_close();
            return true;
        }
        match self.card() {
            Some(card) => keys::press(&card, &self.window, key),
            None => false,
        }
    }

    // ---- Following changes made elsewhere while open

    fn watch(&self) {
        self.stop_watching();
        let me = self.me.clone();
        let sliders = glib::timeout_add_local(SLIDERS_EVERY, move || {
            if let Some(card) = me.upgrade().and_then(|panel| panel.card()) {
                card.refresh_sliders();
            }
            glib::ControlFlow::Continue
        });
        let me = self.me.clone();
        let switches = glib::timeout_add_local(SWITCHES_EVERY, move || {
            if let Some(card) = me.upgrade().and_then(|panel| panel.card()) {
                card.pager.refresh();
            }
            glib::ControlFlow::Continue
        });
        self.state.borrow_mut().watches = vec![sliders, switches];
    }

    fn stop_watching(&self) {
        let watches = std::mem::take(&mut self.state.borrow_mut().watches);
        for watch in watches {
            watch.remove();
        }
    }

    // ---- Shape and animation

    fn card_size(&self) -> (f64, f64) {
        let width = self.holder.measure(gtk4::Orientation::Horizontal, -1).1;
        let height = self.holder.measure(gtk4::Orientation::Vertical, width).1;
        (width as f64, height as f64)
    }

    fn retarget(&self) {
        let is_open = self.is_open();
        let (width, height) = self.card_size();
        let mut state = self.state.borrow_mut();
        let springs = &mut state.springs;
        if is_open {
            springs.width.target = width + 2.0 * PADDING;
            springs.height.target = height + 2.0 * PADDING;
            springs.shoulder.target = SHOULDER;
            springs.radius.target = RADIUS;
        } else {
            for spring in springs.all() {
                spring.target = 0.0;
            }
        }
    }

    pub fn fit(&self) {
        let mut state = self.state.borrow_mut();
        if state.ticking {
            return;
        }
        state.ticking = true;
        state.last_frame = None;
        let me = self.me.clone();
        self.frame.add_tick_callback(move |_, clock| match me.upgrade() {
            Some(panel) => panel.on_frame(clock),
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
        let springs_moving = {
            let mut state = self.state.borrow_mut();
            let mut moving = false;
            for spring in state.springs.all() {
                spring.step(seconds);
                moving |= !spring.at_rest();
            }
            moving
        };
        let fading = self.fade_card(seconds);
        self.show_outline();
        if fading || springs_moving {
            return glib::ControlFlow::Continue;
        }
        self.state.borrow_mut().ticking = false;
        if !self.is_open() {
            self.window.set_visible(false);
        }
        glib::ControlFlow::Break
    }

    fn fade_card(&self, seconds: f64) -> bool {
        let target = if self.is_open() { 1.0 } else { 0.0 };
        let step = seconds / CONTENT_FADE_SECONDS;
        let opacity = self.holder.opacity();
        let opacity = if target > opacity { (opacity + step).min(target) } else { (opacity - step).max(target) };
        self.holder.set_opacity(opacity);
        opacity != target
    }

    /// The outline as the springs have it now, and the card inside it.
    fn show_outline(&self) {
        let (on_right, (screen_width, screen_height), width, height, shoulder, radius) = {
            let state = self.state.borrow();
            let springs = &state.springs;
            (
                state.on_right,
                state.screen,
                springs.width.value,
                springs.height.value,
                springs.shoulder.value,
                springs.radius.value,
            )
        };
        let placement = Placement::in_window(screen_width, screen_height, on_right);
        let mut painted = self.painted.get();
        painted.placement = placement;
        painted.corner = Corner { width: width as f32, height: height as f32, shoulder: shoulder as f32, radius: radius as f32 };
        self.painted.set(painted);

        let (card_width, card_height) = self.card_size();
        let card_x = if on_right { placement.x - PADDING - card_width } else { placement.x + PADDING };
        let card_y = placement.y - PADDING - card_height;
        let body_x = if on_right { placement.x - width } else { placement.x };
        let body_y = placement.y - height;
        let content_area = graphene::Rect::new(card_x.round() as f32, card_y.round() as f32, card_width as f32, card_height as f32);
        let visible_area = graphene::Rect::new(body_x as f32, body_y as f32, width as f32, height as f32);
        self.frame.place_content(content_area, visible_area);
    }
}

fn close_of(panel: Weak<Panel>) -> impl Fn() {
    move || {
        if let Some(panel) = panel.upgrade() {
            panel.close();
        }
    }
}
