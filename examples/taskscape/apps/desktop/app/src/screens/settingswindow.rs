//! contract: omnicui/contracts/settingswindow.contract.yaml (id: app.settingswindow). The desktop Settings window's content (decision 19): the title, the section sidebar with the search, and the chosen section's page, General, Capture, Shortcuts or About.
//!
//! The General page draws the window: its title, sidebar, search and discard controls, in its window
//! mode (four sections). The other sections are their own pages, drawn into the pane with their own
//! title and sidebar outside it (their pages carry the same chrome when a test shows them alone).
use crate::screens::settingsabout::SettingsAbout;
use crate::screens::settingscapture::SettingsCapture;
use crate::screens::settingsgeneral::{self as general, SettingsGeneral};
use crate::screens::settingsshortcuts::SettingsShortcuts;
use crate::App;
use egui::{vec2, Event, Key, Rect, UiBuilder};
use taskscape_ui::omnicui::settingsgeneral::parts as gp;
use taskscape_ui::omnicui::settingswindow::{parts, SettingswindowEvent, SettingswindowProps};
use taskscape_ui::{as_part, Look, Mounted};

/// The General page's parts, under the window's names.
const RENAMES: &[(&str, &str)] = &[
    (gp::HEADING, parts::TITLE),
    (gp::DISCARDALL, parts::DISCARDALL),
    (general::SIDEBAR, parts::SIDEBAR),
    (gp::SECTIONNAV, parts::TABLIST),
    (gp::SEARCH, parts::SEARCH),
    (gp::CLEARSEARCH, parts::SEARCHCLEAR),
    (gp::GENERALTAB, parts::TABGENERAL),
    (general::CAPTURE_TAB, parts::TABCAPTURE),
    (gp::SHORTCUTSTAB, parts::TABSHORTCUTS),
    (gp::ABOUTTAB, parts::TABABOUT),
    (gp::GENERALMATCHES, parts::GENERALMATCHES),
    (gp::CONTENTPANEL, parts::CONTENT),
    (gp::EMPTYTITLE, parts::EMPTYSTATE),
    (gp::EMPTYSUBTITLE, parts::EMPTYHINT),
    (gp::APPEARANCEHEADER, parts::APPEARANCEHEADING),
    (gp::APPEARANCEDISCARD, parts::APPEARANCEDISCARD),
    (gp::APPEARANCEGROUP, parts::APPEARANCEGROUP),
    (general::THEME_ROW, parts::THEMEROW),
    (gp::THEMETITLE, parts::THEMETITLE),
    (gp::THEMEDESCRIPTION, parts::THEMEHINT),
    (gp::THEMESYSTEM, parts::THEMESYSTEM),
    (gp::THEMELIGHT, parts::THEMELIGHT),
    (gp::THEMEDARK, parts::THEMEDARK),
    (general::REDUCED_ROW, parts::REDUCEDROW),
    (gp::REDUCEDMOTIONTITLE, parts::REDUCEDTITLE),
    (gp::REDUCEDMOTIONDESCRIPTION, parts::REDUCEDHINT),
    (gp::REDUCEDMOTIONSYSTEM, parts::REDUCEDSYSTEM),
    (gp::REDUCEDMOTIONON, parts::REDUCEDON),
    (gp::REDUCEDMOTIONOFF, parts::REDUCEDOFF),
    (gp::TASKSHEADER, parts::TASKSHEADING),
    (gp::TASKSDISCARD, parts::TASKSDISCARD),
    (gp::TASKSGROUP, parts::TASKSGROUP),
    (general::SHOW_ROW, parts::SHOWCOMPLETEDROW),
    (gp::SHOWCOMPLETEDTITLE, parts::SHOWCOMPLETEDTITLE),
    (gp::SHOWCOMPLETEDDESCRIPTION, parts::SHOWCOMPLETEDHINT),
    (gp::SHOWCOMPLETEDSWITCH, parts::SHOWCOMPLETED),
    // The other sections' pages, as the window's tests read them.
    ("app.settingscapture.captureModeLabel", parts::CAPTUREMODELABEL),
    ("app.settingsshortcuts.rowNewTask", parts::SHORTCUTSNEWTASK),
    ("app.settingsabout.appName", parts::ABOUTAPPNAME),
];

/// The title bar's height and the sidebar's width of each section's own page: the page is drawn that
/// far up and to the left of the pane, so that only its pane shows.
const ABOUT_CHROME: (f32, f32) = (154.0, 46.0);
const CAPTURE_CHROME: (f32, f32) = (153.0, 46.0);
const SHORTCUTS_CHROME: (f32, f32) = (taskscape_ui::rem(10.929), taskscape_ui::rem(3.286));

pub struct SettingsWindow {
    general: SettingsGeneral,
    capture: SettingsCapture,
    shortcuts: SettingsShortcuts,
    about: SettingsAbout,
    /// The section whose page drew last frame: a page that takes the keyboard when it first draws
    /// gives it back to the window.
    last_pane: String,
}

impl Default for SettingsWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingsWindow {
    pub fn new() -> Self {
        let mut general = SettingsGeneral::new();
        general.window = true;
        let (mut capture, mut shortcuts, mut about) = (SettingsCapture::new(), SettingsShortcuts::new(), SettingsAbout::new());
        capture.embedded = true;
        shortcuts.embedded = true;
        about.embedded = true;
        about.set_data_folder(taskscape_storage::paths::root_dir().display().to_string());
        Self { general, capture, shortcuts, about, last_pane: "general".into() }
    }

    /// One frame of the window's content. A close (Escape, the close chord) sets `app.settings_window`
    /// to None and reports it.
    pub fn show(&mut self, app: &mut App, ui: &mut egui::Ui, look: Look) -> Vec<SettingswindowEvent> {
        let mut events = vec![];
        if app.settings_window.is_none() {
            app.settings_window = Some("general".into());
        }
        as_part(ui, RENAMES, |ui| {
            self.general.show(app, ui, look);
        });
        for (key, value) in self.general.writes.drain(..) {
            // The switch is stored as true or false; the event says on or off.
            let value = match value.as_str() {
                "true" => "on".to_owned(),
                "false" => "off".to_owned(),
                _ => value,
            };
            events.push(SettingswindowEvent::SettingChanged { key: key.to_owned(), value });
        }
        if let Some(reason) = self.general.closed.take() {
            app.settings_window = None;
            events.push(SettingswindowEvent::WindowClosed { reason: reason.to_owned() });
            return events;
        }

        let pane = app.settings_window.clone().unwrap_or_default();
        if let Some(content) = self.general.content {
            let focus_before = ui.memory(|m| m.focused());
            let first = self.last_pane != pane;
            // The window keeps its own keys: the page must not close the window or move to its own search.
            let events_before = ui.input_mut(|i| {
                let all = i.events.clone();
                i.events.retain(|e| !matches!(e, Event::Key { key: Key::Escape | Key::F | Key::W, .. }));
                all
            });
            let chrome = match pane.as_str() {
                "capture" => CAPTURE_CHROME,
                "shortcuts" => SHORTCUTS_CHROME,
                _ => ABOUT_CHROME,
            };
            let page = Rect::from_min_max(content.min - vec2(chrome.0, chrome.1), content.max);
            as_part(ui, RENAMES, |ui| ui.scope_builder(UiBuilder::new().max_rect(page), |ui| {
                ui.set_clip_rect(content);
                match pane.as_str() {
                    "capture" => {
                        self.capture.show(app, ui, look);
                    }
                    "shortcuts" => {
                        self.shortcuts.show(app, ui, look);
                    }
                    _ => {
                        self.about.show(app, ui, look);
                    }
                }
            }));
            ui.input_mut(|i| i.events = events_before);
            if first {
                if let Some(id) = focus_before {
                    ui.memory_mut(|m| m.request_focus(id));
                }
            }
            // A page may name another section (its own hidden sidebar); the window's sidebar is the one shown.
            if app.settings_window.is_none() {
                app.settings_window = Some(pane.clone());
            }
        }
        self.last_pane = pane;
        events
    }
}

/// The test's mount: the window on the props' data set, with a theme saved before it opened.
struct Mount {
    app: App,
    screen: SettingsWindow,
}

impl Mounted for Mount {
    fn frame(&mut self, ui: &mut egui::Ui, look: Look, emit: &mut dyn FnMut(&'static str, serde_json::Value)) {
        for e in self.screen.show(&mut self.app, ui, look) {
            let (n, f) = e.to_json();
            emit(n, f);
        }
    }
}

pub fn mount(mut app: App, props: serde_json::Value) -> Result<Box<dyn Mounted>, String> {
    let p: SettingswindowProps = serde_json::from_value(props).map_err(|e| e.to_string())?;
    if p.storedTheme != "none" {
        app.data.set_setting("theme", &p.storedTheme).map_err(|e| e.to_string())?;
    }
    app.settings_window = Some("general".into());
    Ok(Box::new(Mount { app, screen: SettingsWindow::new() }))
}
