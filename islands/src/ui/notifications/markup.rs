//! The text of a notice body made safe for a label: no inline images, and plain text when the markup is broken.

use gtk4::{glib, pango};

pub fn safe_markup(text: &str) -> String {
    let without_images = strip_image_tags(text);
    if pango::parse_markup(&without_images, '\0').is_ok() {
        without_images
    } else {
        glib::markup_escape_text(&without_images).to_string()
    }
}

/// Removes every <img ...> tag, whatever the case of its name.
fn strip_image_tags(text: &str) -> String {
    let lowercase = text.to_ascii_lowercase();
    let mut kept = String::with_capacity(text.len());
    let mut from = 0;
    while let Some(found) = image_tag_start(&lowercase, from) {
        kept.push_str(&text[from..found]);
        match lowercase[found..].find('>') {
            Some(length) => from = found + length + 1,
            None => return kept,
        }
    }
    kept.push_str(&text[from..]);
    kept
}

fn image_tag_start(lowercase: &str, from: usize) -> Option<usize> {
    let mut search_from = from;
    while let Some(offset) = lowercase[search_from..].find("<img") {
        let start = search_from + offset;
        let next = lowercase[start + 4..].chars().next();
        if !next.is_some_and(|letter| letter.is_alphanumeric() || letter == '_') {
            return Some(start);
        }
        search_from = start + 4;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_image_tags_in_any_case() {
        assert_eq!(strip_image_tags(r#"a <img src="x.png"/> b <IMG alt=y> c"#), "a  b  c");
    }

    #[test]
    fn keeps_tags_that_only_start_like_an_image() {
        assert_eq!(strip_image_tags("<imgx>text</imgx>"), "<imgx>text</imgx>");
    }

    #[test]
    fn keeps_valid_markup() {
        assert_eq!(safe_markup("<b>bold</b>"), "<b>bold</b>");
    }

    #[test]
    fn escapes_broken_markup() {
        assert_eq!(safe_markup("a < b & c"), "a &lt; b &amp; c");
    }
}
