//! Keeps the notification centre across restarts of the island: the notices and Do Not Disturb in
//! ~/.local/state/brasa/notifications.json, their pictures as PNG files beside it. The file has the format
//! of the Python island's, so either version reads the other's.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use gtk4::glib;
use serde::{Deserialize, Serialize};

use super::notice::Notice;
use crate::runtime::workers;

const STATE_FILE: &str = "notifications.json";
const PICTURES_FOLDER: &str = "notification-pictures";

static LATEST: Mutex<Option<Saved>> = Mutex::new(None);
static WRITING: Mutex<()> = Mutex::new(());

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Saved {
    pub notices: Vec<Notice>,
    pub closed_by_app: Vec<u32>,
    pub dnd: bool,
}

#[derive(Serialize, Deserialize)]
struct StateFile {
    dnd: bool,
    closed_by_app: Vec<u32>,
    notices: Vec<Record>,
}

#[derive(Serialize, Deserialize)]
struct Record {
    id: u32,
    app: String,
    summary: String,
    body: String,
    app_icon: String,
    image_path: String,
    desktop_entry: String,
    urgency: u8,
}

pub fn default_folder() -> PathBuf {
    let state_home = match std::env::var_os("XDG_STATE_HOME") {
        Some(folder) if !folder.is_empty() => PathBuf::from(folder),
        _ => glib::home_dir().join(".local/state"),
    };
    state_home.join("brasa")
}

/// Returns at once; the file is written on a worker, and the last state asked for is the one that stays.
pub fn save_soon(saved: Saved) {
    *LATEST.lock().unwrap() = Some(saved);
    workers::run(write_latest);
}

fn write_latest() {
    let _writing = WRITING.lock().unwrap();
    let Some(saved) = LATEST.lock().unwrap().take() else { return };
    let _ = save(&default_folder(), &saved);
}

pub fn save(folder: &Path, saved: &Saved) -> std::io::Result<()> {
    let records: Vec<Record> = saved.notices.iter().map(|notice| record_of(folder, notice)).collect();
    let pictures_in_use: HashSet<String> = records.iter().map(|record| record.image_path.clone()).collect();
    let mut closed_by_app = saved.closed_by_app.clone();
    closed_by_app.sort_unstable();
    let state = StateFile { dnd: saved.dnd, closed_by_app, notices: records };

    fs::create_dir_all(folder)?;
    let path = folder.join(STATE_FILE);
    let unfinished = folder.join(format!("{STATE_FILE}.new"));
    fs::write(&unfinished, serde_json::to_vec(&state)?)?;
    fs::rename(unfinished, path)?;
    forget_unused_pictures(folder, &pictures_in_use);
    Ok(())
}

pub fn load(folder: &Path) -> Saved {
    read_state(folder).map(saved_from).unwrap_or_default()
}

fn read_state(folder: &Path) -> Option<StateFile> {
    let contents = fs::read(folder.join(STATE_FILE)).ok()?;
    serde_json::from_slice(&contents).ok()
}

fn saved_from(state: StateFile) -> Saved {
    let notices = state.notices.into_iter().map(notice_of).collect();
    Saved { notices, closed_by_app: state.closed_by_app, dnd: state.dnd }
}

fn notice_of(record: Record) -> Notice {
    Notice {
        id: record.id,
        app: record.app,
        summary: record.summary,
        body: record.body,
        app_icon: record.app_icon,
        image_path: record.image_path,
        desktop_entry: record.desktop_entry,
        urgency: record.urgency,
        ..Notice::default()
    }
}

fn record_of(folder: &Path, notice: &Notice) -> Record {
    let image_path = picture_file(folder, notice).unwrap_or_else(|| notice.image_path.clone());
    Record {
        id: notice.id,
        app: notice.app.clone(),
        summary: notice.summary.clone(),
        body: notice.body.clone(),
        app_icon: notice.app_icon.clone(),
        image_path,
        desktop_entry: notice.desktop_entry.clone(),
        urgency: notice.urgency,
    }
}

/// Writes the notice's raw pixels as a PNG, once; None when it has none or the write fails.
fn picture_file(folder: &Path, notice: &Notice) -> Option<String> {
    let picture = notice.image_data.as_ref()?;
    let pictures = folder.join(PICTURES_FOLDER);
    let path = pictures.join(format!("{}.png", picture.content_name()));
    if !path.exists() {
        fs::create_dir_all(&pictures).ok()?;
        picture.save_png(&path).ok()?;
    }
    Some(path.to_string_lossy().into_owned())
}

fn forget_unused_pictures(folder: &Path, in_use: &HashSet<String>) {
    let Ok(entries) = fs::read_dir(folder.join(PICTURES_FOLDER)) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if !in_use.contains(path.to_string_lossy().as_ref()) {
            let _ = fs::remove_file(path);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use gtk4::prelude::*;

    use super::*;
    use crate::services::notifications::picture::Picture;

    fn scratch_folder(name: &str) -> PathBuf {
        let folder = std::env::temp_dir().join(format!("brasa-store-test-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&folder);
        folder
    }

    fn notice(id: u32, summary: &str) -> Notice {
        Notice { id, app: "Files".into(), summary: summary.into(), urgency: 2, ..Notice::default() }
    }

    fn red_picture() -> Arc<Picture> {
        let hint = (1i32, 1i32, 4i32, true, 8i32, 4i32, vec![255u8, 0, 0, 255]).to_variant();
        Arc::new(Picture::from_hint(&hint).unwrap())
    }

    #[test]
    fn the_centre_comes_back_as_it_was_saved() {
        let folder = scratch_folder("round-trip");
        let saved = Saved { notices: vec![notice(1, "One"), notice(4, "Four")], closed_by_app: vec![4], dnd: true };
        save(&folder, &saved).unwrap();
        assert_eq!(load(&folder), saved);
        let _ = fs::remove_dir_all(folder);
    }

    #[test]
    fn nothing_saved_is_an_empty_centre() {
        assert_eq!(load(&scratch_folder("nothing")), Saved::default());
    }

    #[test]
    fn the_file_has_the_python_islands_format() {
        let folder = scratch_folder("format");
        save(&folder, &Saved { notices: vec![notice(1, "One")], closed_by_app: vec![], dnd: false }).unwrap();
        let written = fs::read_to_string(folder.join(STATE_FILE)).unwrap();
        let expected = r#"{"dnd":false,"closed_by_app":[],"notices":[{"id":1,"app":"Files","summary":"One","body":"","app_icon":"","image_path":"","desktop_entry":"","urgency":2}]}"#;
        assert_eq!(written, expected);
        let _ = fs::remove_dir_all(folder);
    }

    #[test]
    fn a_picture_is_kept_as_a_png_and_unused_ones_are_removed() {
        let folder = scratch_folder("pictures");
        let with_picture = Notice { image_data: Some(red_picture()), ..notice(1, "Pic") };
        save(&folder, &Saved { notices: vec![with_picture], ..Saved::default() }).unwrap();
        let kept = load(&folder).notices[0].image_path.clone();
        assert!(kept.ends_with(".png") && Path::new(&kept).exists());

        save(&folder, &Saved { notices: vec![notice(1, "Pic")], ..Saved::default() }).unwrap();
        assert!(!Path::new(&kept).exists());
        let _ = fs::remove_dir_all(folder);
    }
}
