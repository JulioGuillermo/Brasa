//! Finds a track's cover (mpris:artUrl, else the thumbnail of a YouTube video) and loads it on a worker
//! as a centred square texture, so it never changes the island's layout.

use std::process::Command;

use gtk4::gdk_pixbuf::{InterpType, Pixbuf, PixbufLoader};
use gtk4::prelude::*;
use gtk4::{gdk, glib};

use super::track::Track;
use crate::runtime::events::{self, Event, EventSender};
use crate::runtime::workers;

const DOWNLOAD_TIMEOUT_SECONDS: &str = "5";
const SIDE_PIXELS: i32 = 256;
const YOUTUBE_THUMBNAILS: &str = "https://i.ytimg.com/vi/";
const YOUTUBE_HOSTS: [&str; 4] = ["youtube.com", "www.youtube.com", "music.youtube.com", "m.youtube.com"];

/// Empty when the track has no cover to show.
pub fn cover_url(track: &Track) -> String {
    if track.art_url.is_empty() {
        return youtube_thumbnail(&track.page_url);
    }
    track.art_url.clone()
}

/// Loads the cover on a worker and sends Event::Cover; picture is None when it cannot be read.
pub fn load(events: &EventSender, url: &str) {
    let events = events.clone();
    let url = url.to_string();
    workers::run(move || {
        let picture = fetch(&url).and_then(|data| square_texture(&data, &url));
        events::send(&events, Event::Cover { url, picture });
    });
}

fn youtube_thumbnail(page_url: &str) -> String {
    let Some((host, path, query)) = split_url(page_url) else { return String::new() };
    let video = if host == "youtu.be" {
        path.trim_matches('/').to_string()
    } else if YOUTUBE_HOSTS.contains(&host.as_str()) {
        query_value(query, "v").unwrap_or_default().to_string()
    } else {
        return String::new();
    };
    if video.is_empty() {
        return String::new();
    }
    format!("{YOUTUBE_THUMBNAILS}{video}/hqdefault.jpg")
}

/// "https://host:1/path?query" -> ("host", "/path", "query")
fn split_url(url: &str) -> Option<(String, &str, &str)> {
    let (_scheme, rest) = url.split_once("://")?;
    let authority_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let (authority, after) = rest.split_at(authority_end);
    let after = after.split('#').next().unwrap_or("");
    let (path, query) = after.split_once('?').unwrap_or((after, ""));
    let host_and_port = authority.rsplit('@').next().unwrap_or(authority);
    let host = host_and_port.split(':').next().unwrap_or(host_and_port);
    Some((host.to_lowercase(), path, query))
}

fn query_value<'a>(query: &'a str, key: &str) -> Option<&'a str> {
    query.split('&').filter_map(|pair| pair.split_once('=')).find(|(name, _)| *name == key).map(|(_, value)| value)
}

fn fetch(url: &str) -> Option<Vec<u8>> {
    if url.starts_with("file://") {
        let path = glib::filename_from_uri(url).ok()?.0;
        return std::fs::read(path).ok();
    }
    let output = Command::new("curl")
        .args(["-sfL", "--max-time", DOWNLOAD_TIMEOUT_SECONDS, url])
        .output()
        .ok()?;
    output.status.success().then_some(output.stdout)
}

fn square_texture(data: &[u8], url: &str) -> Option<gdk::Texture> {
    let loader = PixbufLoader::new();
    loader.write(data).ok()?;
    loader.close().ok()?;
    let mut image = loader.pixbuf()?;
    if url.starts_with(YOUTUBE_THUMBNAILS) {
        image = without_letterbox(&image)?;
    }
    let side = image.width().min(image.height());
    let left = (image.width() - side) / 2;
    let top = (image.height() - side) / 2;
    let centre = image.new_subpixbuf(left, top, side, side);
    let scaled = centre.scale_simple(SIDE_PIXELS, SIDE_PIXELS, InterpType::Bilinear)?;
    Some(gdk::Texture::for_pixbuf(&scaled))
}

fn without_letterbox(image: &Pixbuf) -> Option<Pixbuf> {
    let height = (image.width() * 9 / 16).min(image.height());
    let top = (image.height() - height) / 2;
    Some(image.new_subpixbuf(0, top, image.width(), height))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn track(art_url: &str, page_url: &str) -> Track {
        Track { art_url: art_url.into(), page_url: page_url.into(), ..Track::default() }
    }

    #[test]
    fn art_url_wins() {
        let track = track("file:///tmp/a.png", "https://youtu.be/abc");
        assert_eq!(cover_url(&track), "file:///tmp/a.png");
    }

    #[test]
    fn youtube_pages_give_a_thumbnail() {
        let expected = "https://i.ytimg.com/vi/abc/hqdefault.jpg";
        assert_eq!(cover_url(&track("", "https://youtu.be/abc")), expected);
        assert_eq!(cover_url(&track("", "https://www.youtube.com/watch?x=1&v=abc")), expected);
        assert_eq!(cover_url(&track("", "https://music.youtube.com/watch?v=abc#t=3")), expected);
    }

    #[test]
    fn other_pages_give_nothing() {
        assert_eq!(cover_url(&track("", "https://example.com/watch?v=abc")), "");
        assert_eq!(cover_url(&track("", "https://www.youtube.com/watch")), "");
        assert_eq!(cover_url(&track("", "")), "");
    }
}
