//! brasa-islands [start|toggle|centre|dnd|launcher|peek|hide|reload|quick]
//! The first call starts the one process; later calls hand it their command and leave.

mod app;
mod commands;
mod config;
mod runtime;
mod services;
mod ui;

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::glib;
use gtk4::prelude::*;

use app::App;
use commands::Command;

const USAGE: &str = "usage: brasa-islands [start|toggle|centre|dnd|launcher|peek|hide|reload|quick]";

fn main() -> glib::ExitCode {
    let word = std::env::args().nth(1);
    if Command::parse(word.as_deref()).is_none() {
        eprintln!("{USAGE}");
        return glib::ExitCode::FAILURE;
    }
    let gtk = gtk4::Application::builder().application_id(app::application_id()).flags(app::flags()).build();
    let running: Rc<RefCell<Option<Rc<App>>>> = Rc::new(RefCell::new(None));

    gtk.connect_command_line(move |gtk, command_line| {
        let arguments = command_line.arguments();
        let word = arguments.get(1).and_then(|argument| argument.to_str());
        let Some(command) = Command::parse(word) else { return glib::ExitCode::FAILURE };
        let app = running.borrow_mut().get_or_insert_with(|| App::start(gtk)).clone();
        app.run(command);
        glib::ExitCode::SUCCESS
    });
    gtk.run()
}
