//! One island window: a layer-shell surface on a screen edge whose outline grows out of the edge to fit
//! what it shows. It hosts the music and notifications, the app launcher, or both when they share a
//! place. Springs drive every size, so the outline flows between states.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::{Rc, Weak};
use std::time::Duration;

use gtk4::prelude::*;
use gtk4::{cairo, gdk, glib, graphene};
use gtk4_layer_shell::{Edge as LayerEdge, KeyboardMode, Layer, LayerShell};

use super::frame::Frame;
use super::monitors;
use super::outline::Outline;
use super::shapes::Shape;
use super::spring::Spring;
use crate::config::{Align, Config, Edge, Place};
use crate::ui::launcher::Launcher;
use crate::ui::music::{Music, View};
use crate::ui::notifications::Notices;

const SECTION_SPACING: i32 = 12;
const SLACK: f64 = 40.0;
/// The window reaches this far past the screen edge, so its own border, which a fractional scale can
/// leave half drawn, stays off screen.
const OVERHANG: f64 = 2.0;
const HOT_ZONE_HEIGHT: f64 = 3.0;
const HOVER_DWELL: Duration = Duration::from_millis(120);
const LEAVE_DELAY: Duration = Duration::from_millis(400);
const SHORTCUT_IDLE: Duration = Duration::from_millis(5000);
const SHOW_OFF_DELAY: Duration = Duration::from_millis(150);
const CONTENT_FADE_SECONDS: f64 = 0.15;
const MAX_FRAME_SECONDS: f64 = 1.0 / 30.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    /// Super+m: the music, and the notification centre when it is on.
    Main,
    /// Super+n: the music and the notification centre, even when it is off.
    Centre,
    Launcher,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Timer {
    Dwell,
    Leave,
    Idle,
    Peek,
    ShowOff,
}

#[derive(Default, Clone)]
pub struct Sections {
    pub music: Option<Rc<Music>>,
    pub notices: Option<Rc<Notices>>,
    pub launcher: Option<Rc<Launcher>>,
}

impl Sections {
    fn widgets(&self) -> Vec<gtk4::Widget> {
        let mut widgets = Vec::new();
        if let Some(music) = &self.music {
            widgets.push(music.widget());
        }
        if let Some(notices) = &self.notices {
            widgets.push(notices.widget());
        }
        if let Some(launcher) = &self.launcher {
            widgets.push(launcher.widget());
        }
        widgets
    }
}

struct Springs {
    width: Spring,
    height: Spring,
    shoulder: Spring,
    radius: Spring,
    inset: Spring,
}

impl Springs {
    fn all(&mut self) -> [&mut Spring; 5] {
        [&mut self.width, &mut self.height, &mut self.shoulder, &mut self.radius, &mut self.inset]
    }
}

struct State {
    shape: Shape,
    content_shape: Shape,
    mode: Option<Mode>,
    peeking: bool,
    pointer_inside: bool,
    springs: Springs,
    ticking: bool,
    last_frame: Option<f64>,
    monitor: String,
}

pub struct Island {
    me: Weak<Island>,
    window: gtk4::Window,
    frame: Frame,
    content: gtk4::Box,
    place: Place,
    config: Config,
    sections: Sections,
    window_width: f64,
    window_height: f64,
    drawn: Rc<Cell<Outline>>,
    state: RefCell<State>,
    timers: RefCell<HashMap<Timer, glib::SourceId>>,
}

impl Island {
    pub fn new(app: &gtk4::Application, place: Place, config: &Config, sections: Sections) -> Rc<Self> {
        let shapes = shapes_for(&sections, config);
        let window_width = shapes.iter().map(|shape| shape.full_width()).fold(0.0, f64::max) + 2.0 * SLACK;
        let window_height =
            shapes.iter().map(|shape| shape.dimensions().max_height).fold(0.0, f64::max) + SLACK + OVERHANG;
        let at_top = place.edge == Edge::Top;
        let edge_y = if at_top { OVERHANG } else { window_height - OVERHANG };
        let hidden = Shape::Hidden.dimensions();

        let drawn = Rc::new(Cell::new(Outline {
            center: (window_width / 2.0) as f32,
            width: hidden.body_width as f32,
            height: 0.0,
            shoulder: 0.0,
            radius: 0.0,
        }));
        let color = parse_color(&config.color);
        let painted = Rc::clone(&drawn);
        let painter = Box::new(move |snapshot: &gtk4::Snapshot| {
            snapshot.save();
            snapshot.translate(&graphene::Point::new(0.0, edge_y as f32));
            if !at_top {
                snapshot.scale(1.0, -1.0);
            }
            painted.get().paint(snapshot, &color);
            snapshot.restore();
        });

        let content = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(SECTION_SPACING)
            .opacity(0.0)
            .build();
        let frame = Frame::new(&content, painter);
        let window = gtk4::Window::builder()
            .application(app)
            .css_classes(["brasa-island"])
            .default_width(window_width as i32)
            .default_height(window_height as i32)
            .child(&frame)
            .build();

        let first_shape = shapes.get(1).copied().unwrap_or(Shape::Hidden);
        let island = Rc::new_cyclic(|me| Island {
            me: me.clone(),
            window,
            frame,
            content,
            place,
            config: config.clone(),
            sections,
            window_width,
            window_height,
            drawn,
            state: RefCell::new(State {
                shape: Shape::Hidden,
                content_shape: first_shape,
                mode: None,
                peeking: false,
                pointer_inside: false,
                springs: Springs {
                    width: Spring::new(hidden.body_width),
                    height: Spring::new(0.0),
                    shoulder: Spring::new(0.0),
                    radius: Spring::new(0.0),
                    inset: Spring::new(hidden.padding.0),
                },
                ticking: false,
                last_frame: None,
                monitor: String::new(),
            }),
            timers: RefCell::new(HashMap::new()),
        });
        island.place_on_screen();
        island.take_sections();
        island.listen_to_pointer();
        let me = island.me.clone();
        island.window.connect_realize(move |_| {
            if let Some(island) = me.upgrade() {
                island.update_input_region();
            }
        });
        island.relayout();
        island.window.present();
        island
    }

    pub fn hosts_music(&self) -> bool {
        self.sections.music.is_some()
    }

    pub fn hosts_launcher(&self) -> bool {
        self.sections.launcher.is_some()
    }

    fn at_top(&self) -> bool {
        self.place.edge == Edge::Top
    }

    fn edge_y(&self) -> f64 {
        if self.at_top() { OVERHANG } else { self.window_height - OVERHANG }
    }

    fn is_open(&self) -> bool {
        self.state.borrow().mode.is_some()
    }

    // ---- Window

    fn place_on_screen(&self) {
        let window = &self.window;
        window.init_layer_shell();
        window.set_namespace(Some("brasa-island"));
        window.set_layer(Layer::Overlay);
        window.set_exclusive_zone(-1);
        window.set_keyboard_mode(KeyboardMode::None);
        window.set_anchor(LayerEdge::Top, self.at_top());
        window.set_anchor(LayerEdge::Bottom, !self.at_top());
        window.set_anchor(LayerEdge::Left, self.place.align == Align::Left);
        window.set_anchor(LayerEdge::Right, self.place.align == Align::Right);
        let screen_edge = if self.at_top() { LayerEdge::Top } else { LayerEdge::Bottom };
        window.set_margin(screen_edge, -(OVERHANG as i32));
        if !self.place.monitor.is_empty() {
            window.set_monitor(monitors::find(&self.place.monitor).as_ref());
            self.state.borrow_mut().monitor = self.place.monitor.clone();
        }
    }

    /// Without a monitor of its own, the island shows up on the focused one.
    fn follow_focus(&self) {
        if !self.place.monitor.is_empty() {
            return;
        }
        let connector = monitors::focused_connector();
        let Some(monitor) = monitors::find(&connector) else { return };
        if connector == self.state.borrow().monitor {
            return;
        }
        self.state.borrow_mut().monitor = connector;
        self.window.set_monitor(Some(&monitor));
    }

    /// Nearest the edge the music, then the notifications; the launcher stands alone.
    fn take_sections(&self) {
        let mut widgets = self.sections.widgets();
        if !self.at_top() {
            widgets.reverse();
        }
        for widget in &widgets {
            self.content.append(widget);
        }
        if let Some(notices) = &self.sections.notices {
            notices.set_edge(self.at_top());
        }
    }

    /// Gives the sections back before the window goes, for the next island to take.
    pub fn destroy(&self) {
        self.close_all();
        for widget in self.sections.widgets() {
            self.content.remove(&widget);
        }
        if let Some(music) = &self.sections.music {
            music.set_shown(false);
        }
        self.window.destroy();
    }

    // ---- What it shows

    pub fn relayout(&self) {
        let (mode, peeking) = {
            let state = self.state.borrow();
            (state.mode, state.peeking)
        };
        if mode == Some(Mode::Launcher) {
            self.show_launcher();
            return;
        }
        if let Some(launcher) = &self.sections.launcher {
            launcher.widget().set_visible(false);
        }
        let (Some(music), Some(notices)) = (&self.sections.music, &self.sections.notices) else {
            self.show_shape(Shape::Hidden);
            return;
        };
        let open_main = matches!(mode, Some(Mode::Main | Mode::Centre));
        let notices_on = self.showing_centre() || notices.has_toasts();
        let music_on = self.config.media && (open_main || peeking || (notices_on && music.is_playing()));
        music.set_shown(music_on);
        music.widget().set_visible(music_on);
        music.set_view(if open_main { View::Full } else { View::Line });
        notices.widget().set_visible(notices_on);
        if !music_on && !notices_on {
            self.show_shape(Shape::Hidden);
        } else if music_on && !open_main && !notices_on {
            self.show_shape(Shape::Line);
        } else {
            self.show_shape(Shape::Card);
        }
    }

    fn show_launcher(&self) {
        if let (Some(music), Some(notices)) = (&self.sections.music, &self.sections.notices) {
            music.set_shown(false);
            music.widget().set_visible(false);
            notices.widget().set_visible(false);
        }
        if let Some(launcher) = &self.sections.launcher {
            launcher.widget().set_visible(true);
        }
        self.show_shape(Shape::Launcher(self.config.launcher_size));
    }

    fn show_shape(&self, shape: Shape) {
        if self.state.borrow().shape == Shape::Hidden && shape != Shape::Hidden {
            self.follow_focus();
        }
        {
            let mut state = self.state.borrow_mut();
            state.shape = shape;
            if shape != Shape::Hidden {
                state.content_shape = shape;
            }
        }
        self.fit();
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
            Some(island) => island.on_frame(clock),
            None => glib::ControlFlow::Break,
        });
    }

    fn content_height(&self, content_shape: Shape) -> f64 {
        let width = content_shape.content_width() as i32;
        self.content.measure(gtk4::Orientation::Vertical, width).1 as f64
    }

    fn retarget(&self) {
        let (shape, content_shape) = {
            let state = self.state.borrow();
            (state.shape, state.content_shape)
        };
        let dimensions = shape.dimensions();
        let from_edge = content_shape.dimensions().padding.0;
        let height = if shape == Shape::Hidden {
            0.0
        } else {
            (self.content_height(content_shape) + 2.0 * from_edge).min(dimensions.max_height)
        };
        let mut state = self.state.borrow_mut();
        let springs = &mut state.springs;
        springs.width.target = dimensions.body_width;
        springs.height.target = height;
        springs.shoulder.target = dimensions.shoulder;
        springs.radius.target = dimensions.radius;
        springs.inset.target = from_edge;
    }

    // ---- Opening and closing

    /// Super+m: music and notifications.
    pub fn toggle(&self) {
        self.toggle_mode(Mode::Main);
    }

    /// Super+n and the bar's bell: music and the notification centre, even when it is off.
    pub fn toggle_centre(&self) {
        self.toggle_mode(Mode::Centre);
    }

    fn toggle_mode(&self, mode: Mode) {
        if !self.hosts_music() {
            return;
        }
        if self.state.borrow().mode == Some(mode) {
            self.close_view();
            return;
        }
        self.open_view(mode, false);
        self.close_when_idle();
    }

    fn showing_centre(&self) -> bool {
        match self.state.borrow().mode {
            Some(Mode::Centre) => true,
            Some(Mode::Main) => self.config.notification_centre,
            _ => false,
        }
    }

    pub fn toggle_launcher(&self, grab_keyboard: bool) {
        if !self.hosts_launcher() {
            return;
        }
        if self.state.borrow().mode == Some(Mode::Launcher) {
            self.close_view();
            return;
        }
        self.open_view(Mode::Launcher, grab_keyboard);
        if !grab_keyboard {
            self.close_when_idle();
        }
    }

    fn close_when_idle(&self) {
        if !self.state.borrow().pointer_inside {
            self.later(Timer::Idle, SHORTCUT_IDLE, |island| island.close_view());
        }
    }

    fn open_view(&self, mode: Mode, grab_keyboard: bool) {
        self.cancel(Timer::Idle);
        self.state.borrow_mut().mode = Some(mode);
        if let Some(notices) = &self.sections.notices {
            notices.set_centre(self.showing_centre());
        }
        if mode == Mode::Launcher {
            if let Some(launcher) = &self.sections.launcher {
                launcher.opened();
            }
        }
        let keyboard = if mode == Mode::Launcher && grab_keyboard { KeyboardMode::Exclusive } else { KeyboardMode::None };
        self.window.set_keyboard_mode(keyboard);
        self.relayout();
    }

    pub fn close_view(&self) {
        self.cancel(Timer::Idle);
        self.state.borrow_mut().mode = None;
        self.window.set_keyboard_mode(KeyboardMode::None);
        if let Some(notices) = &self.sections.notices {
            notices.set_centre(false);
        }
        self.relayout();
    }

    /// Shows the current track for a moment.
    pub fn peek(&self) {
        if !self.hosts_music() || !self.config.media || self.config.peek_seconds <= 0.0 {
            return;
        }
        self.state.borrow_mut().peeking = true;
        self.relayout();
        let shown_for = Duration::from_secs_f64(self.config.peek_seconds);
        self.later(Timer::Peek, shown_for, |island| {
            island.state.borrow_mut().peeking = false;
            island.relayout();
        });
    }

    pub fn close_all(&self) {
        for (_, source) in self.timers.borrow_mut().drain() {
            source.remove();
        }
        self.state.borrow_mut().peeking = false;
        self.close_view();
    }

    /// Opens for a moment, to show where it now is.
    pub fn show_off_soon(&self) {
        self.later(Timer::ShowOff, SHOW_OFF_DELAY, |island| {
            if island.hosts_music() {
                island.toggle();
            } else {
                island.toggle_launcher(false);
            }
        });
    }

    // ---- Pointer

    fn listen_to_pointer(&self) {
        let pointer = gtk4::EventControllerMotion::new();
        let me = self.me.clone();
        pointer.connect_enter(move |_, _, _| {
            if let Some(island) = me.upgrade() {
                island.on_enter();
            }
        });
        let me = self.me.clone();
        pointer.connect_leave(move |_| {
            if let Some(island) = me.upgrade() {
                island.on_leave();
            }
        });
        self.window.add_controller(pointer);
    }

    fn hover_opens(&self) -> Option<Mode> {
        if self.hosts_music() && self.config.hover {
            return Some(Mode::Main);
        }
        if self.hosts_launcher() && self.config.launcher_hover {
            return Some(Mode::Launcher);
        }
        None
    }

    fn on_enter(&self) {
        self.state.borrow_mut().pointer_inside = true;
        if let Some(notices) = &self.sections.notices {
            notices.set_hovered(true);
        }
        self.cancel(Timer::Leave);
        self.cancel(Timer::Idle);
        let showing_notices = self.state.borrow().shape == Shape::Card && !self.is_open();
        let Some(mode) = self.hover_opens() else { return };
        if !self.is_open() && !showing_notices {
            self.later(Timer::Dwell, HOVER_DWELL, move |island| island.open_view(mode, false));
        }
    }

    fn on_leave(&self) {
        self.state.borrow_mut().pointer_inside = false;
        if let Some(notices) = &self.sections.notices {
            notices.set_hovered(false);
        }
        self.cancel(Timer::Dwell);
        if self.is_open() {
            self.later(Timer::Leave, LEAVE_DELAY, |island| island.close_view());
        }
    }

    fn later(&self, timer: Timer, delay: Duration, action: impl FnOnce(&Island) + 'static) {
        self.cancel(timer);
        let me = self.me.clone();
        let source = glib::timeout_add_local_once(delay, move || {
            let Some(island) = me.upgrade() else { return };
            island.timers.borrow_mut().remove(&timer);
            action(&island);
        });
        self.timers.borrow_mut().insert(timer, source);
    }

    fn cancel(&self, timer: Timer) {
        if let Some(source) = self.timers.borrow_mut().remove(&timer) {
            source.remove();
        }
    }

    // ---- Animation

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
        let fading = self.fade_content(seconds);
        self.show_outline();
        self.update_input_region();
        let resizing = self.sections.music.as_ref().is_some_and(|music| music.is_resizing());
        if fading || resizing || springs_moving {
            return glib::ControlFlow::Continue;
        }
        self.state.borrow_mut().ticking = false;
        glib::ControlFlow::Break
    }

    fn fade_content(&self, seconds: f64) -> bool {
        let target = if self.state.borrow().shape == Shape::Hidden { 0.0 } else { 1.0 };
        let step = seconds / CONTENT_FADE_SECONDS;
        let opacity = self.content.opacity();
        let opacity = if target > opacity { (opacity + step).min(target) } else { (opacity - step).max(target) };
        self.content.set_opacity(opacity);
        self.content.set_can_target(opacity > 0.0);
        opacity != target
    }

    fn outline(&self) -> Outline {
        let state = self.state.borrow();
        let springs = &state.springs;
        Outline {
            center: (self.window_width / 2.0) as f32,
            width: springs.width.value as f32,
            height: springs.height.value as f32,
            shoulder: springs.shoulder.value as f32,
            radius: springs.radius.value as f32,
        }
    }

    /// The outline as the springs have it now, and the content inside it.
    fn show_outline(&self) {
        let outline = self.outline();
        self.drawn.set(outline);
        let (content_shape, inset) = {
            let state = self.state.borrow();
            (state.content_shape, state.springs.inset.value)
        };
        let width = content_shape.content_width();
        let height = self.content_height(content_shape);
        let x = ((self.window_width - width) / 2.0).round();
        let y = if self.at_top() { self.edge_y() + inset } else { self.edge_y() - inset - height };
        let body = outline.body();
        let body_top = if self.at_top() { self.edge_y() } else { self.edge_y() - body.height() as f64 };
        let content_area = graphene::Rect::new(x as f32, y.round() as f32, width as f32, height as f32);
        let visible_area = graphene::Rect::new(body.x(), body_top as f32, body.width(), body.height());
        self.frame.place_content(content_area, visible_area);
    }

    /// Only the outline takes the pointer; with nothing shown, a thin line along the edge where the
    /// pointer opens it.
    fn update_input_region(&self) {
        let Some(surface) = self.window.surface() else { return };
        let outline = self.outline();
        let (x, width, mut height) = if outline.height >= 1.0 {
            (outline.left() as f64, outline.full_width() as f64, outline.height as f64)
        } else if self.hover_opens().is_some() {
            let line_width = Shape::Line.dimensions().body_width;
            (self.window_width / 2.0 - line_width / 2.0, line_width, HOT_ZONE_HEIGHT)
        } else {
            (0.0, 0.0, 0.0)
        };
        height += if width > 0.0 { OVERHANG } else { 0.0 };
        let y = if self.at_top() { 0.0 } else { self.window_height - height };
        let area = cairo::RectangleInt::new(x.round() as i32, y.round() as i32, width.round() as i32, height.round() as i32);
        surface.set_input_region(Some(&cairo::Region::create_rectangle(&area)));
    }
}

fn shapes_for(sections: &Sections, config: &Config) -> Vec<Shape> {
    let mut shapes = vec![Shape::Hidden];
    if sections.music.is_some() {
        shapes.extend([Shape::Line, Shape::Card]);
    }
    if sections.launcher.is_some() {
        shapes.push(Shape::Launcher(config.launcher_size));
    }
    shapes
}

pub fn parse_color(color: &str) -> gdk::RGBA {
    gdk::RGBA::parse(color).unwrap_or(gdk::RGBA::BLACK)
}
