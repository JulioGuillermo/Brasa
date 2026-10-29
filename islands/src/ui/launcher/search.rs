//! The search field.

pub fn search_entry() -> gtk4::SearchEntry {
    gtk4::SearchEntry::builder().placeholder_text("Search apps").hexpand(true).css_classes(["island-search"]).build()
}
