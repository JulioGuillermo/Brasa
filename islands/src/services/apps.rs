//! The installed apps (their .desktop files): listing, ranking a search, launching.

use std::process::{Command, Stdio};

use gtk4::gdk::prelude::*;
use gtk4::gio;
use gtk4::glib;
use gio_unix::DesktopAppInfo;

use crate::runtime::workers;

const TERMINAL: &str = "foot";
const ISLAND_ONLY_VARIABLE: &str = "LD_PRELOAD";

pub struct InstalledApp {
    pub info: DesktopAppInfo,
    pub search_text: SearchText,
}

/// The texts of an app a search looks in, already lowercased.
#[derive(Clone)]
pub struct SearchText {
    name: String,
    generic_name: String,
    description: String,
    executable: String,
    keywords: Vec<String>,
}

impl SearchText {
    pub fn new(name: &str, generic_name: &str, description: &str, executable: &str, keywords: &[&str]) -> Self {
        SearchText {
            name: name.to_lowercase(),
            generic_name: generic_name.to_lowercase(),
            description: description.to_lowercase(),
            executable: executable.to_lowercase(),
            keywords: keywords.iter().map(|keyword| keyword.to_lowercase()).collect(),
        }
    }

    fn other_texts(&self) -> impl Iterator<Item = &String> {
        [&self.generic_name, &self.description, &self.executable].into_iter().chain(&self.keywords)
    }
}

/// How well an app matches a search; the earlier the variant, the better.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Rank {
    NameStarts,
    WordStarts,
    NameContains,
    Elsewhere,
}

/// Every app that wants to be shown, by display name.
pub fn installed() -> Vec<InstalledApp> {
    let mut apps: Vec<InstalledApp> = gio::AppInfo::all()
        .into_iter()
        .filter_map(|info| info.downcast::<DesktopAppInfo>().ok())
        .filter(|info| info.should_show())
        .map(installed_app)
        .collect();
    apps.sort_by(|first, second| first.search_text.name.cmp(&second.search_text.name));
    apps
}

fn installed_app(info: DesktopAppInfo) -> InstalledApp {
    let keywords = info.keywords();
    let keywords: Vec<&str> = keywords.iter().map(|keyword| keyword.as_str()).collect();
    let search_text = SearchText::new(
        &info.display_name(),
        info.generic_name().as_deref().unwrap_or(""),
        info.description().as_deref().unwrap_or(""),
        &info.executable().to_string_lossy(),
        &keywords,
    );
    InstalledApp { info, search_text }
}

/// `search` must already be trimmed and lowercased.
pub fn rank(app: &SearchText, search: &str) -> Option<Rank> {
    if app.name.starts_with(search) {
        return Some(Rank::NameStarts);
    }
    if app.name.split_whitespace().any(|word| word.starts_with(search)) {
        return Some(Rank::WordStarts);
    }
    if app.name.contains(search) {
        return Some(Rank::NameContains);
    }
    if app.other_texts().any(|text| text.contains(search)) {
        return Some(Rank::Elsewhere);
    }
    None
}

/// Positions in `apps` of those that match, best first; apps of equal rank keep their order.
pub fn matching(apps: &[SearchText], search: &str) -> Vec<usize> {
    let search = search.trim().to_lowercase();
    if search.is_empty() {
        return (0..apps.len()).collect();
    }
    let mut ranked: Vec<(Rank, usize)> = apps
        .iter()
        .enumerate()
        .filter_map(|(position, app)| rank(app, &search).map(|rank| (rank, position)))
        .collect();
    ranked.sort();
    ranked.into_iter().map(|(_rank, position)| position).collect()
}

pub fn launch(app: &DesktopAppInfo) {
    if app.boolean("Terminal") {
        launch_in_terminal(app);
        return;
    }
    let Some(display) = gtk4::gdk::Display::default() else { return };
    let context = display.app_launch_context();
    context.unsetenv(ISLAND_ONLY_VARIABLE);
    let _ = app.launch(&[], Some(&context));
}

fn launch_in_terminal(app: &DesktopAppInfo) {
    let commandline = app.commandline().unwrap_or_else(|| app.executable());
    let without_codes = without_field_codes(&commandline.to_string_lossy());
    let Ok(words) = glib::shell_parse_argv(&without_codes) else { return };
    let mut command = Command::new("setsid");
    command.args(["-f", TERMINAL, "-e"]).args(words);
    command.stdout(Stdio::null()).stderr(Stdio::null());
    command.env_remove(ISLAND_ONLY_VARIABLE);
    workers::run(move || {
        let _ = command.status();
    });
}

fn without_field_codes(commandline: &str) -> String {
    let mut kept = String::with_capacity(commandline.len());
    let mut characters = commandline.chars().peekable();
    while let Some(character) = characters.next() {
        let starts_field_code = character == '%' && characters.peek().is_some_and(char::is_ascii_alphabetic);
        if starts_field_code {
            characters.next();
        } else {
            kept.push(character);
        }
    }
    kept
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(name: &str, generic_name: &str, description: &str, executable: &str, keywords: &[&str]) -> SearchText {
        SearchText::new(name, generic_name, description, executable, keywords)
    }

    fn named(name: &str) -> SearchText {
        text(name, "", "", "", &[])
    }

    #[test]
    fn name_start_beats_word_start_beats_contains_beats_elsewhere() {
        let apps = [
            text("Zed", "Editor", "", "", &[]),
            named("Dark Fire"),
            named("Fireplace"),
            named("Backfire"),
        ];
        assert_eq!(matching(&apps, "fire"), vec![2, 1, 3]);
        assert_eq!(matching(&apps, "edit"), vec![0]);
    }

    #[test]
    fn equal_ranks_keep_their_order() {
        let apps = [named("Alpha one"), named("Alpha two"), named("Alpha three")];
        assert_eq!(matching(&apps, "alpha"), vec![0, 1, 2]);
    }

    #[test]
    fn search_is_trimmed_and_case_insensitive() {
        let apps = [named("Firefox"), named("Files")];
        assert_eq!(matching(&apps, "  FIRE "), vec![0]);
    }

    #[test]
    fn empty_search_lists_everything() {
        let apps = [named("A"), named("B")];
        assert_eq!(matching(&apps, "  "), vec![0, 1]);
    }

    #[test]
    fn looks_elsewhere_in_every_other_text() {
        let app = text("Thing", "Browser", "Surf the web", "thing-bin", &["Internet"]);
        for search in ["brow", "surf", "thing-b", "internet"] {
            assert_eq!(rank(&app, search), Some(Rank::Elsewhere), "{search}");
        }
        assert_eq!(rank(&app, "missing"), None);
    }

    #[test]
    fn field_codes_are_removed() {
        assert_eq!(without_field_codes("htop %U --flag %f"), "htop  --flag ");
        assert_eq!(without_field_codes("run 100%"), "run 100%");
    }
}
