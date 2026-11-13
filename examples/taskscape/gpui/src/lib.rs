//! Taskscape in GPUI: the main window and the capture bar, native from the first frame.
//!
//! The data and its rules are `taskscape_common`'s, linked directly (`Db`); the look is
//! DESIGN.md's tokens (`theme`), drawn with GPUI's own elements, text fields and animations.

pub mod capture;
pub mod db;
pub mod keys;
pub mod model;
pub mod motion;
pub mod parts;
pub mod settings;
pub mod text_field;
pub mod theme;
pub mod ui;
pub mod workspace;

mod filter;
mod inspector;
mod pane;

mod sheet;
mod titlebar;
mod trash;

use std::borrow::Cow;
use std::rc::Rc;

use gpui::{App, AssetSource, Global, SharedString};

pub use db::Db;
pub use workspace::{Surface, Workspace};

/// What the app reports as the user changes things: the contracts' domain events, with the
/// store's ids (`taskCompleted {id, done}`). Nothing listens in the app itself; a test harness
/// may (`set_reporter`).
pub type Reporter = Rc<dyn Fn(&str, serde_json::Value)>;

#[derive(Default)]
struct ReportGlobal(Option<Reporter>);
impl Global for ReportGlobal {}

pub fn set_reporter(cx: &mut App, r: Reporter) {
    cx.set_global(ReportGlobal(Some(r)));
}

pub fn report(cx: &App, name: &str, fields: serde_json::Value) {
    if let Some(r) = cx.try_global::<ReportGlobal>().and_then(|g| g.0.clone()) {
        r(name, fields);
    }
}

/// The icons (Material Symbols, as SVG), embedded.
pub struct Assets;

macro_rules! icons {
    ($($name:literal),* $(,)?) => {
        const ICONS: &[(&str, &[u8])] = &[$(($name, include_bytes!(concat!("../assets/icons/", $name, ".svg")))),*];
    };
}
icons!(
    "close", "delete", "folder_open", "add", "open_in_new", "unfold_more", "tune", "search", "screenshot_monitor",
    "chevron_right", "chevron_left", "check", "undo", "redo", "task_alt", "restore", "restart_alt",
    "radio_button_unchecked", "last_page", "edit", "delete_forever", "arrow_forward", "update",
    "subdirectory_arrow_right", "schedule", "remove", "open_in_full", "note_add", "drive_file_rename",
    "drag_indicator", "bolt", "attach_file", "add_link", "vertical_split", "swap_horiz", "settings", "notes",
    "more_horiz", "left_click", "delete_sweep", "content_copy", "casino", "account_tree", "expand_more", "link",
    "image", "description", "info", "dark_mode", "light_mode", "contrast", "minimize", "crop_square",
    "filter_list", "sort", "keyboard_return", "right_panel_open", "right_panel_close", "keyboard", "upload_file",
    "folder", "check_box", "check_box_outline_blank", "format_bold", "format_italic", "code",
    "format_list_bulleted", "keyboard_command_key", "arrow_upward", "arrow_downward",
);

impl AssetSource for Assets {
    fn load(&self, path: &str) -> anyhow::Result<Option<Cow<'static, [u8]>>> {
        let name = path.trim_start_matches("icons/").trim_end_matches(".svg");
        Ok(ICONS.iter().find(|(n, _)| *n == name).map(|(_, b)| Cow::Borrowed(*b)))
    }
    fn list(&self, _: &str) -> anyhow::Result<Vec<SharedString>> {
        Ok(ICONS.iter().map(|(n, _)| SharedString::from(format!("icons/{n}.svg"))).collect())
    }
}

/// The system UI font (DESIGN.md, "Type"): macOS's own; elsewhere the family fontconfig gives
/// for `sans-serif`, which is what a browser's `system-ui` resolves to there.
pub fn system_font() -> SharedString {
    if cfg!(target_os = "macos") {
        return ".SystemUIFont".into();
    }
    static FAMILY: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    FAMILY
        .get_or_init(|| {
            std::process::Command::new("fc-match")
                .args(["-f", "%{family[0]}", "sans-serif"])
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| ".SystemUIFont".into())
        })
        .clone()
        .into()
}

/// Key bindings and actions shared by every window.
pub fn init(cx: &mut App) {
    cx.bind_keys(text_field::bindings());
    cx.bind_keys(keys::fixed_bindings());
}
