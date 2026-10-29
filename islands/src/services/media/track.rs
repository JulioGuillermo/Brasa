//! What a player says about its track (MPRIS metadata) and its state.

use gtk4::glib;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Status {
    Playing,
    Paused,
    Stopped,
}

impl Status {
    pub fn from_mpris(text: &str) -> Status {
        match text {
            "Playing" => Status::Playing,
            "Paused" => Status::Paused,
            _ => Status::Stopped,
        }
    }
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct Track {
    pub id: String,
    pub title: String,
    pub artists: Vec<String>,
    pub album: String,
    /// 0 when the player does not say.
    pub length_seconds: f64,
    pub art_url: String,
    /// The page it plays from, for a browser.
    pub page_url: String,
}

impl Track {
    /// metadata: the a{sv} of the player's Metadata property.
    pub fn from_metadata(metadata: &glib::Variant) -> Track {
        let metadata = glib::VariantDict::new(Some(metadata));
        let text = |key: &str| {
            metadata
                .lookup_value(key, None)
                .and_then(|value| value.str().map(str::to_string))
                .unwrap_or_default()
        };
        Track {
            id: text("mpris:trackid"),
            title: text("xesam:title"),
            artists: metadata
                .lookup_value("xesam:artist", None)
                .and_then(|value| value.get::<Vec<String>>())
                .unwrap_or_default(),
            album: text("xesam:album"),
            length_seconds: length_of(&metadata),
            art_url: text("mpris:artUrl"),
            page_url: text("xesam:url"),
        }
    }

    /// Title and artists: what tells one track from another.
    pub fn same_song(&self, other: &Track) -> bool {
        self.title == other.title && self.artists == other.artists
    }
}

/// mpris:length is microseconds; players send it as x, t or even i.
fn length_of(metadata: &glib::VariantDict) -> f64 {
    let Some(value) = metadata.lookup_value("mpris:length", None) else { return 0.0 };
    let microseconds = value
        .get::<i64>()
        .or_else(|| value.get::<u64>().map(|length| length as i64))
        .or_else(|| value.get::<i32>().map(i64::from))
        .unwrap_or(0);
    microseconds.max(0) as f64 / 1e6
}

#[cfg(test)]
mod tests {
    use super::*;
    use gtk4::glib::prelude::*;

    #[test]
    fn reads_mpris_metadata() {
        let dict = glib::VariantDict::new(None);
        dict.insert_value("xesam:title", &"Song".to_variant());
        dict.insert_value("xesam:artist", &vec!["A".to_string(), "B".to_string()].to_variant());
        dict.insert_value("mpris:length", &259_000_000u64.to_variant());
        let track = Track::from_metadata(&dict.end());
        assert_eq!(track.title, "Song");
        assert_eq!(track.artists, vec!["A", "B"]);
        assert_eq!(track.length_seconds, 259.0);
        assert_eq!(track.album, "");
    }
}
