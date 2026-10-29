//! The one process: it builds the sections once, places them in islands as island.conf says, starts the
//! reader and worker threads, and hands each event they send to the part it is for.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use gtk4::prelude::*;
use gtk4::{gdk, gio, glib};

use crate::commands::Command;
use crate::config::{self, Config};
use crate::runtime::events::{self, Event, EventReceiver};
use crate::runtime::{reader, workers};
use crate::ui::launcher::Launcher;
use crate::ui::music::Music;
use crate::ui::notifications::Notices;
use crate::ui::quick::Quick;
use crate::ui::shell::island::{Island, Sections};
use crate::ui::shell::monitors;

const ISLAND_STYLE: &str = include_str!("../style/island.css");
const QUICK_STYLE: &str = include_str!("../style/quick.css");
/// Super+d, then releasing Super, asks twice in a row: the second one is ignored.
const DOUBLE_TOGGLE: Duration = Duration::from_millis(600);

pub struct App {
    gtk: gtk4::Application,
    music: Rc<Music>,
    notices: Rc<Notices>,
    launcher: RefCell<Option<Rc<Launcher>>>,
    quick: Rc<Quick>,
    main_island: RefCell<Option<Rc<Island>>>,
    launcher_island: RefCell<Option<Rc<Island>>>,
    last_launcher_toggle: RefCell<Option<Instant>>,
}

impl App {
    pub fn start(gtk: &gtk4::Application) -> Rc<App> {
        std::mem::forget(gtk.hold());
        load_style();
        workers::start();
        let (sender, receiver) = events::channel();
        reader::start(&sender);

        let app = Rc::new_cyclic(|me: &std::rc::Weak<App>| {
            let on_peek = me.clone();
            let on_resize = me.clone();
            let on_change = me.clone();
            let on_close = me.clone();
            App {
                gtk: gtk.clone(),
                music: Music::new(
                    &sender,
                    move || with(&on_peek, |app| app.tell_main(Island::peek)),
                    move || with(&on_resize, |app| app.tell_main(Island::fit)),
                ),
                notices: Notices::new(
                    move || with(&on_change, |app| app.tell_main(Island::relayout)),
                    move || with(&on_close, |app| app.tell_main(Island::close_view)),
                ),
                launcher: RefCell::new(None),
                quick: Quick::new(gtk, &sender),
                main_island: RefCell::new(None),
                launcher_island: RefCell::new(None),
                last_launcher_toggle: RefCell::new(None),
            }
        });
        app.build(&config::load());
        app.receive(receiver);
        app
    }

    pub fn run(self: &Rc<Self>, command: Command) {
        match command {
            Command::Start => {}
            Command::Toggle => self.tell_main(Island::toggle),
            Command::Centre => self.tell_main(Island::toggle_centre),
            Command::Dnd => self.notices.toggle_dnd(),
            Command::Launcher => self.toggle_launcher(),
            Command::Peek => self.tell_main(Island::peek),
            Command::Hide => self.islands().iter().for_each(|island| island.close_all()),
            Command::Reload => self.reload(),
            Command::Quick => self.quick.toggle(),
        }
    }

    fn build(self: &Rc<Self>, settings: &Config) {
        let me = Rc::downgrade(self);
        let launcher = Launcher::new(settings.launcher_view, settings.launcher_size, move || {
            with(&me, |app| app.close_launcher());
        });
        let launcher_place = settings.launcher.then(|| settings.launcher_place.clone());
        let music_sections = Sections {
            music: Some(Rc::clone(&self.music)),
            notices: Some(Rc::clone(&self.notices)),
            launcher: None,
        };
        let launcher_sections = Sections { launcher: Some(Rc::clone(&launcher)), ..Sections::default() };

        if launcher_place.as_ref() == Some(&settings.place) {
            let shared = Sections { launcher: Some(Rc::clone(&launcher)), ..music_sections };
            let island = Island::new(&self.gtk, settings.place.clone(), settings, shared);
            *self.launcher_island.borrow_mut() = Some(Rc::clone(&island));
            *self.main_island.borrow_mut() = Some(island);
        } else {
            *self.main_island.borrow_mut() = Some(Island::new(&self.gtk, settings.place.clone(), settings, music_sections));
            *self.launcher_island.borrow_mut() =
                launcher_place.map(|place| Island::new(&self.gtk, place, settings, launcher_sections));
        }
        *self.launcher.borrow_mut() = Some(launcher);
    }

    fn islands(&self) -> Vec<Rc<Island>> {
        let mut islands: Vec<Rc<Island>> = self.main_island.borrow().iter().cloned().collect();
        if let Some(launcher_island) = self.launcher_island.borrow().as_ref() {
            if !islands.iter().any(|island| Rc::ptr_eq(island, launcher_island)) {
                islands.push(Rc::clone(launcher_island));
            }
        }
        islands
    }

    /// island.conf again: the islands are built anew where it now says, and show where they are.
    fn reload(self: &Rc<Self>) {
        for island in self.islands() {
            island.destroy();
        }
        self.main_island.borrow_mut().take();
        self.launcher_island.borrow_mut().take();
        self.build(&config::load());
        for island in self.islands() {
            island.show_off_soon();
        }
        self.quick.reload();
    }

    fn tell_main(&self, action: impl FnOnce(&Island)) {
        let island = self.main_island.borrow().clone();
        if let Some(island) = island {
            action(&island);
        }
    }

    fn toggle_launcher(&self) {
        let now = Instant::now();
        let mut last = self.last_launcher_toggle.borrow_mut();
        if last.is_some_and(|last| now - last < DOUBLE_TOGGLE) {
            return;
        }
        *last = Some(now);
        drop(last);
        let island = self.launcher_island.borrow().clone();
        if let Some(island) = island {
            island.toggle_launcher(true);
        }
    }

    fn close_launcher(&self) {
        let island = self.launcher_island.borrow().clone();
        if let Some(island) = island {
            island.close_view();
        }
    }

    // ---- Events from the other threads

    fn receive(self: &Rc<Self>, receiver: EventReceiver) {
        let me = Rc::downgrade(self);
        glib::spawn_future_local(async move {
            while let Ok(event) = receiver.recv().await {
                let Some(app) = me.upgrade() else { return };
                app.handle(event);
            }
        });
    }

    fn handle(&self, event: Event) {
        match event {
            Event::FocusedMonitor(connector) => monitors::set_focused(connector),
            Event::Volume(level) => self.music.show_volume(level),
            Event::Cover { url, picture } => self.music.show_cover(&url, picture.as_ref()),
            Event::Tested { test, passed } => self.quick.on_tested(&test, passed),
        }
    }
}

fn with(app: &std::rc::Weak<App>, action: impl FnOnce(&Rc<App>)) {
    if let Some(app) = app.upgrade() {
        action(&app);
    }
}

fn load_style() {
    let Some(display) = gdk::Display::default() else { return };
    for style in [ISLAND_STYLE, QUICK_STYLE] {
        let provider = gtk4::CssProvider::new();
        provider.load_from_string(style);
        gtk4::style_context_add_provider_for_display(&display, &provider, gtk4::STYLE_PROVIDER_PRIORITY_USER);
    }
}

pub fn application_id() -> String {
    let session = std::env::var("WAYLAND_DISPLAY").unwrap_or_else(|_| "wayland-0".into());
    format!("dev.brasa.islands.{}", session.replace('-', "_"))
}

pub fn flags() -> gio::ApplicationFlags {
    gio::ApplicationFlags::HANDLES_COMMAND_LINE
}
