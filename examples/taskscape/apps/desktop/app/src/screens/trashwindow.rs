//! contract: omnicui/contracts/trashwindow.contract.yaml (id: app.trashwindow). The Trash window: the Trash list as the content of its own OS window, opened from the title bar or the palette (decision 8).
use crate::screens::trash::Trash;
use crate::App;
use egui::{vec2, Key, Rect, Role, Sense, ViewportCommand, WidgetInfo};
use taskscape_ui::omnicui::trash::{parts as trash_parts, TrashEvent};
use taskscape_ui::omnicui::trashwindow::{parts, TrashwindowEvent, TrashwindowProps};
use taskscape_ui::{as_part, part, rem, Look, Mounted};

/// The Trash list's header strip (its glyph and "Trash" title), which the Trash list draws at its top.
const HEADER_H: f32 = rem(3.286);

/// The Trash list's part names, renamed to the window's: the same nodes, under the names the contract gives them.
const LIST_NAMES: [(&str, &str); 9] = [
    (trash_parts::TITLE, parts::TITLE),
    (trash_parts::ROW, parts::ROW),
    (trash_parts::RESTOREITEM, parts::RESTOREROW),
    (trash_parts::PURGEITEM, parts::PURGEROW),
    (trash_parts::LIST, parts::LISTBODY),
    (trash_parts::ACTIONSTRIP, parts::ACTIONSTRIP),
    (trash_parts::RESTOREALL, parts::RESTOREALL),
    (trash_parts::EMPTYTRASH, parts::EMPTYTRASH),
    (trash_parts::EMPTYSTATE, parts::EMPTYSTATE),
];

/// The second row's parts. The Trash list names its rows by title, so the second row takes the names of
/// whichever demo row is second now: after a restore or a purge the list moves up, and the other row
/// carries no second-row name.
fn second_row_names(app: &App) -> Vec<(&'static str, &'static str)> {
    match app.data.trash.get(1).map(|t| t.title.as_str()) {
        Some("Call the printer about samples") => vec![
            (trash_parts::ROWPRINTER, parts::SECONDROW),
            (trash_parts::RESTOREPRINTER, parts::SECONDRESTOREROW),
            (trash_parts::PURGEPRINTER, parts::SECONDPURGEROW),
        ],
        Some("Print the old flyer") => vec![
            (trash_parts::ROWFLYER, parts::SECONDROW),
            (trash_parts::RESTOREFLYER, parts::SECONDRESTOREROW),
            (trash_parts::PURGEFLYER, parts::SECONDPURGEROW),
        ],
        _ => vec![],
    }
}

/// The Escape key or the platform's Close Window chord (Ctrl W, Cmd W on macOS), as the close reason.
/// The chord reads the platform's own modifier: Control elsewhere, Command on macOS.
fn close_request(ui: &egui::Ui) -> Option<&'static str> {
    ui.input(|i| {
        let platform = if cfg!(target_os = "macos") { i.modifiers.mac_cmd || i.modifiers.command } else { i.modifiers.ctrl || i.modifiers.command };
        if i.key_pressed(Key::Escape) {
            Some("escape")
        } else if platform && i.key_pressed(Key::W) {
            Some("shortcut")
        } else {
            None
        }
    })
}

/// The Trash window's state: the Trash list it holds, and the events waiting to be reported.
pub struct TrashWindow {
    /// Room at the header's left for the OS's window buttons (a real window on macOS).
    pub lead: f32,
    trash: Trash,
    /// Events raised outside a frame (an open request, a close request); reported on the next frame.
    queued: Vec<TrashwindowEvent>,
    /// A second open request asked for the window to come to the front.
    focus: bool,
}

impl TrashWindow {
    pub fn new() -> Self {
        Self { trash: Trash::new(), queued: vec![], focus: false, lead: 0.0 }
    }

    /// An open request (from the title bar or the palette): opens the window, or focuses it when it is
    /// already open. Reports `opened` with whether a window was made.
    pub fn open(&mut self, app: &mut App) {
        let created = !app.trash_window;
        app.trash_window = true;
        if !created {
            self.focus = true;
        }
        self.queued.push(TrashwindowEvent::Opened { created });
    }

    /// Closes the window (Escape, Ctrl W, or the OS close request) and reports why.
    pub fn close(&mut self, app: &mut App, reason: &str) {
        app.trash_window = false;
        self.queued.push(TrashwindowEvent::Closed { reason: reason.to_owned() });
    }

    /// The events raised outside a frame, taken.
    pub fn take_events(&mut self) -> Vec<TrashwindowEvent> {
        std::mem::take(&mut self.queued)
    }

    /// One frame: draws the window's content while the window is open, and returns the events it raised.
    pub fn show(&mut self, app: &mut App, ui: &mut egui::Ui, look: Look) -> Vec<TrashwindowEvent> {
        let mut events = self.take_events();
        if !app.trash_window {
            return events;
        }
        if std::mem::take(&mut self.focus) {
            ui.ctx().send_viewport_cmd(ViewportCommand::Focus);
        }
        if let Some(reason) = close_request(ui) {
            self.close(app, reason);
            events.append(&mut self.take_events());
            return events;
        }

        // The header strip is the window's title bar (decision 2: the OS draws the window controls, not this).
        let full = ui.max_rect();
        let header = Rect::from_min_size(full.min, vec2(full.width(), HEADER_H));
        let bar = ui.interact(header, ui.scope_id().with("trashwindow-titlebar"), Sense::hover());
        bar.widget_info(|| WidgetInfo::new(Role::GenericContainer));
        part(ui, &bar, parts::TITLEBAR);

        // What is trashed before this frame: the list's removals are read back from the data after it.
        let before: Vec<(String, String)> = app.data.trash.iter().map(|t| (t.id.clone(), t.title.clone())).collect();
        let mut names: Vec<(&str, &str)> = LIST_NAMES.to_vec();
        names.extend(second_row_names(app));
        self.trash.lead = self.lead;
        let trash_events = as_part(ui, &names, |ui| self.trash.show(app, ui, look));
        if let Some(count) = self.trash.empty_asked.take() {
            events.push(TrashwindowEvent::EmptyAsked { count: count as i64 });
        }

        // Empty Trash asks first and its accept removes every root in one go; that is not a per-row purge.
        let all = before.len() as i64;
        let emptied = before.len() > 1 && trash_events.iter().any(|e| matches!(e, TrashEvent::Purged { count } if *count == all));
        for (id, title) in &before {
            if app.data.trash.iter().any(|t| &t.id == id) {
                continue;
            }
            if app.data.task(id).is_some() {
                events.push(TrashwindowEvent::Restored { title: title.clone() });
            } else if !emptied {
                events.push(TrashwindowEvent::Purged { title: title.clone() });
            }
        }
        events
    }
}

/// The test's mount: the app on its data set, with the Trash window opened as the props say.
struct Mount {
    app: App,
    screen: TrashWindow,
}

impl Mounted for Mount {
    fn frame(&mut self, ui: &mut egui::Ui, look: Look, emit: &mut dyn FnMut(&'static str, serde_json::Value)) {
        for e in self.screen.show(&mut self.app, ui, look) {
            let (n, f) = e.to_json();
            emit(n, f);
        }
    }

    /// The OS close request: closes the window; the closed event goes out on the next frame.
    fn close(&mut self) {
        self.screen.close(&mut self.app, "os");
    }
}

pub fn mount(mut app: App, props: serde_json::Value) -> Result<Box<dyn Mounted>, String> {
    let p: TrashwindowProps = serde_json::from_value(props).map_err(|e| e.to_string())?;
    let mut screen = TrashWindow::new();
    for _ in 0..p.openRequests {
        screen.open(&mut app);
    }
    Ok(Box::new(Mount { app, screen }))
}
