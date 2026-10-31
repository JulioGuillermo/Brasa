#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Scope {
    /// The workspace showing on the focused monitor.
    Workspace,
    All,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Window {
    /// Hyprland's address, or sway's con_id.
    pub id: String,
    /// The class on Hyprland, the app_id on sway: what finds its .desktop file.
    pub app_id: String,
    pub title: String,
    pub workspace: String,
    pub focused: bool,
    pub pid: Option<u32>,
    /// The name the Wayland window list gives it, to capture its picture; empty when unknown.
    pub capture_id: String,
}
