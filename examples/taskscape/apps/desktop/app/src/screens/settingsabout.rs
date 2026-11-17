//! contract: omnicui/contracts/settingsabout.contract.yaml (id: app.settingsabout). The About pane of the Settings window: the section list with its search, the Application group with the data folder row, and the Reset group.
use crate::data::Settings;
use crate::App;
use egui::{pos2, vec2, Align2, Color32, CornerRadius, FontFamily, FontId, Galley, Id, Key, Modifiers, Painter, Pos2, Rect, Response, Role, Sense, Shape, Stroke, StrokeKind, TextEdit, Ui, WidgetInfo};
use std::sync::Arc;
use taskscape_ui::omnicui::settingsabout::{parts, SettingsaboutEvent, SettingsaboutProps};
use taskscape_ui::{icon, part, weight, Look, Mounted, Tokens};

const APP_DESCRIPTION: &str = "A task manager for lists, projects and subtasks.";
const RESTORE_DESCRIPTION: &str = "Puts appearance, capture and task preferences — and every keyboard shortcut — back the way they shipped. Your tasks are untouched.";
const APP_KEYWORDS: &str = "about version application name";
const RESET_KEYWORDS: &str = "reset clear defaults factory revert erase preferences";
const NAV_W: f32 = 154.0;

/// The section list, in the desktop's order. Capture is desktop only; General, Capture and Shortcuts
/// draw their own panes on their own screens, so here they show nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Pane {
    General,
    Capture,
    Shortcuts,
    About,
}

impl Pane {
    const ALL: [Pane; 4] = [Pane::General, Pane::Capture, Pane::Shortcuts, Pane::About];

    fn label(self) -> &'static str {
        match self {
            Pane::General => "General",
            Pane::Capture => "Capture",
            Pane::Shortcuts => "Shortcuts",
            Pane::About => "About",
        }
    }
    fn icon(self) -> &'static str {
        match self {
            Pane::General => "tune",
            Pane::Capture => "image",
            Pane::Shortcuts => "keyboard",
            Pane::About => "info",
        }
    }
    fn part(self) -> &'static str {
        match self {
            Pane::General => parts::NAVGENERAL,
            Pane::Capture => parts::NAVCAPTURE,
            Pane::Shortcuts => parts::NAVSHORTCUTS,
            Pane::About => parts::NAVABOUT,
        }
    }
}

/// The About pane of the Settings window. Its state is what the screen keeps between frames.
pub struct SettingsAbout {
    pane: Pane,
    /// Drawn inside the Settings window, which has its own sidebar: the page stays on its own section and
    /// ignores its sidebar, which sits out of view.
    pub embedded: bool,
    query: String,
    /// A click on a section with matches narrows the search to it; the heading then drops its prefix.
    scoped: bool,
    confirming: bool,
    /// Focus moves to Cancel when the confirmation opens, and back to Reset when it closes.
    focus_cancel: bool,
    focus_reset: bool,
    /// The settings as the screen opened: the discard control puts these back.
    open: Option<Settings>,
    /// The widget that had keyboard focus at the end of the last frame. egui drops focus on Escape before
    /// the frame runs, so the Escape rules read it from here.
    focus_seen: Option<Id>,
    /// The tab index that takes keyboard focus in the next frame.
    tab_focus: Option<usize>,
    version: String,
    data_folder: String,
}

impl SettingsAbout {
    pub fn new() -> Self {
        Self {
            pane: Pane::About,
            embedded: false,
            query: String::new(),
            scoped: false,
            confirming: false,
            focus_cancel: false,
            focus_reset: false,
            open: None,
            focus_seen: None,
            tab_focus: None,
            version: "0.1.0".into(),
            data_folder: "/Users/you/.taskscape".into(),
        }
    }

    /// One frame: the Settings window's About page on the app. Returns the events raised this frame.
    pub fn show(&mut self, app: &mut App, ui: &mut Ui, look: Look) -> Vec<SettingsaboutEvent> {
        if self.embedded {
            self.pane = Pane::About;
        }
        let mut events = vec![];
        let t = look.t;
        let prev_focus = self.focus_seen;
        // (Tab, arrows and Escape are read here before egui's own handling of them.)
        let search_id = Id::new(parts::SEARCH);
        let search_was_focused = prev_focus == Some(search_id);
        // Escape in the search field clears the query first; with no query it closes the window (desktop).
        if search_was_focused && ui.input(|i| i.key_pressed(Key::Escape)) {
            ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape));
            if self.query.is_empty() {
                app.settings_window = None;
            } else {
                self.query.clear();
                self.scoped = false;
            }
        }
        if app.settings_window.is_none() {
            self.focus_seen = None;
            return events;
        }
        let baseline = self.open.get_or_insert_with(|| app.data.settings.clone()).clone();
        let changed = app.data.settings != baseline;

        let full = ui.available_rect_before_wrap();
        ui.allocate_rect(full, Sense::hover());
        let p = ui.painter().clone();
        let header = Rect::from_min_size(full.min, vec2(full.width(), 46.0));
        let body = Rect::from_min_max(pos2(full.left(), header.bottom()), full.max);
        let nav = Rect::from_min_size(body.min, vec2(NAV_W, body.height()));
        let panel = Rect::from_min_max(pos2(nav.right(), body.top()), body.max);

        tag(ui, full, Sense::hover(), Role::GenericContainer, "Settings", None, None, parts::WINDOW);
        p.rect_filled(full, 0.0, t.surface_2);
        p.rect_filled(header, 0.0, t.surface_1);
        p.hline(header.x_range(), header.bottom(), Stroke::new(1.0, t.edge_1));
        p.rect_filled(nav, 0.0, t.surface_1);
        p.vline(nav.right(), nav.y_range(), Stroke::new(1.0, t.edge_1));

        // Header: the title, and the discard control while something differs from the open-time values.
        let title = paint_text(&p, pos2(header.left() + 18.0, header.center().y), Align2::LEFT_CENTER, "Settings", 16.0, 600, t.content_1);
        tag(ui, title, Sense::hover(), Role::Heading, "Settings", None, None, parts::TITLE);
        if changed {
            let r = Rect::from_center_size(pos2(header.right() - 32.0, header.center().y), vec2(28.0, 28.0));
            let resp = tag(ui, r, Sense::click(), Role::Button, "Discard all changes", None, None, parts::DISCARDALL);
            if resp.hovered() {
                p.rect_filled(r, CornerRadius::same(7), t.wash_1);
            }
            paint_icon(&p, r.center(), "undo", 19.0, t.content_1);
            if resp.clicked() {
                let base = baseline.clone();
                write_prefs(app, prefs_of(&base));
            }
        }

        // Search: the well with its query field, and the clear control while the query is not empty.
        let well = Rect::from_min_size(nav.min + vec2(8.0, 8.0), vec2(NAV_W - 16.0, 24.5));
        p.rect_filled(well, CornerRadius::same(4), t.wash_2);
        // Command-F (Ctrl-F elsewhere) focuses the search field.
        if ui.input(|i| i.key_pressed(Key::F) && (i.modifiers.command || i.modifiers.ctrl || i.modifiers.mac_cmd)) {
            ui.memory_mut(|m| m.request_focus(search_id));
        }
        let searching_now = !self.query.trim().is_empty();
        paint_icon(&p, pos2(well.left() + 16.0, well.center().y), "search", 14.0, t.content_3);
        let edit_w = well.right() - if searching_now { 26.0 } else { 6.0 } - (well.left() + 28.0);
        let edit_rect = Rect::from_center_size(pos2(well.left() + 28.0 + edit_w / 2.0, well.center().y), vec2(edit_w, 16.0));
        let edit = TextEdit::singleline(&mut self.query)
            .id(search_id)
            .frame(egui::Frame::NONE)
            .font(FontId::new(12.0, weight(400)))
            .text_color(t.content_1)
            .hint_text(egui::RichText::new("Search").color(t.content_3))
            .desired_width(edit_rect.width());
        let search_resp: Response = ui.put(edit_rect, edit);
        if search_resp.changed() {
            self.scoped = false;
        }
        if search_resp.has_focus() {
            p.rect_stroke(well, CornerRadius::same(4), Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
        }
        let q_now = self.query.clone();
        ui.ctx().accesskit_node_builder(search_id, |n| n.set_value(q_now));
        search_resp.widget_info(|| WidgetInfo::labeled(Role::TextInput, true, "Search settings"));
        part(ui, &search_resp, parts::SEARCH);

        if !self.query.is_empty() {
            let r = Rect::from_center_size(pos2(well.right() - 12.0, well.center().y), vec2(20.0, 20.0));
            let resp = tag(ui, r, Sense::click(), Role::Button, "Clear search", None, None, parts::SEARCHCLEAR);
            paint_icon(&p, r.center(), "close", 14.0, t.content_3);
            if resp.clicked() {
                self.query.clear();
                self.scoped = false;
                ui.memory_mut(|m| m.request_focus(search_id));
            }
        }

        // The query's state: which sections match, and what the groups show.
        let tokens: Vec<String> = self.query.trim().to_lowercase().split_whitespace().map(str::to_owned).collect();
        let searching = !tokens.is_empty();
        let hit = |hay: &str| searching && tokens.iter().all(|k| hay.contains(k.as_str()));
        let app_hit = hit(&format!("taskscape {APP_DESCRIPTION} {APP_KEYWORDS}").to_lowercase());
        let reset_hit = hit(&format!("restore defaults {RESTORE_DESCRIPTION} {RESET_KEYWORDS}").to_lowercase());
        let about_count = app_hit as usize + reset_hit as usize;
        let count_of = |pane: Pane| if pane == Pane::About { about_count } else { 0 };
        let pane_empty = searching && count_of(self.pane) == 0;
        let app_shown = !searching || app_hit;
        let reset_shown = !searching || reset_hit;
        let scoped = self.scoped;

        // Section list: arrows move between the tabs, and only the selected tab takes the Tab key.
        let tab_ids: Vec<Id> = Pane::ALL.iter().map(|pane| Id::new(pane.part())).collect();
        let tab_at = |f: Option<Id>| f.and_then(|f| tab_ids.iter().position(|id| *id == f));
        let focused = ui.ctx().memory(|m| m.focused());
        let sel_idx = Pane::ALL.iter().position(|pane| *pane == self.pane).unwrap_or(0);
        // The tab that takes keyboard focus. A focus change lands in the frame after the keys moved it, as the
        // confirmation's Cancel focus does.
        let pending_focus = self.tab_focus.take();
        let mut want_focus: Option<usize> = None;
        // Tab from the search field lands on the selected tab only: the other tabs take no Tab stop.
        // Tab from the search field is taken here: it lands on the selected tab, and the other tabs take no Tab stop.
        let tabbed_out = search_was_focused && ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Tab));
        if search_was_focused && (tabbed_out || tab_at(focused).is_some_and(|i| i != sel_idx)) {
            want_focus = Some(sel_idx);
        }
        if let Some(i) = tab_at(focused) {
            let n = Pane::ALL.len();
            let up = ui.input_mut(|inp| inp.consume_key(Modifiers::NONE, Key::ArrowUp));
            let down = ui.input_mut(|inp| inp.consume_key(Modifiers::NONE, Key::ArrowDown));
            let next = if up {
                Some((i + n - 1) % n)
            } else if down {
                Some((i + 1) % n)
            } else {
                None
            };
            if let Some(j) = next {
                let to = Pane::ALL[j];
                self.choose(to, count_of(to), searching);
                want_focus = Some(j);
            }
        }
        for (i, pane) in Pane::ALL.iter().copied().enumerate() {
            let r = Rect::from_min_size(pos2(nav.left() + 8.0, well.bottom() + 8.0 + i as f32 * 32.0), vec2(NAV_W - 16.0, 30.0));
            let selected = self.pane == pane;
            let count = count_of(pane);
            let dimmed = searching && count == 0;
            let alpha = if dimmed { 0.4 } else { 1.0 };
            let sense = Sense::click();
            let resp = tag(ui, r, sense, Role::Button, pane.label(), None, Some(selected), pane.part());
            if pending_focus == Some(i) || want_focus == Some(i) {
                resp.request_focus();
            }
            let accent_on = !(searching && !scoped);
            if selected && accent_on {
                p.rect_filled(r, CornerRadius::same(7), t.accent_500.gamma_multiply(0.1));
            } else if resp.hovered() && !selected {
                p.rect_filled(r, CornerRadius::same(7), t.wash_1);
            }
            let ink = |c: Color32| c.gamma_multiply(alpha);
            paint_icon(&p, pos2(r.left() + 18.0, r.center().y), pane.icon(), 16.0, ink(t.content_3));
            paint_text(&p, pos2(r.left() + 36.0, r.center().y), Align2::LEFT_CENTER, pane.label(), 14.0, 400, ink(t.content_1));
            if pane == Pane::About && searching && about_count > 0 {
                let s = about_count.to_string();
                let g = p.layout_no_wrap(s.clone(), FontId::new(11.0, weight(600)), t.content_2);
                let bw = g.size().x + 8.0;
                let br = Rect::from_center_size(pos2(r.right() - 6.0 - bw / 2.0, r.center().y), vec2(bw, 16.0));
                p.rect_filled(br, CornerRadius::same(8), t.wash_2);
                p.text(br.center(), Align2::CENTER_CENTER, s.clone(), FontId::new(11.0, weight(600)), t.content_2);
                tag(ui, br, Sense::hover(), Role::GenericContainer, &s, None, None, parts::ABOUTMATCHES);
            }
            if resp.clicked() {
                self.choose(pane, count, searching);
            }
        }

        // The panel: the About content, or the empty result of a search.
        let panel_inner = panel.shrink2(vec2(18.0, 12.0));
        tag(ui, panel, Sense::hover(), Role::GenericContainer, "About", None, None, parts::PANEL);
        if pane_empty {
            let y0 = panel_inner.top() + 63.0;
            let caption = format!("Nothing here for “{}”.", self.query.trim());
            let box_r = Rect::from_min_size(pos2(panel.left(), y0), vec2(panel.width(), 90.0));
            tag(ui, box_r, Sense::hover(), Role::GenericContainer, "No settings match", None, None, parts::EMPTYSTATE);
            paint_icon(&p, pos2(panel.center().x, y0 + 15.0), "search", 30.0, t.content_3);
            paint_text(&p, pos2(panel.center().x, y0 + 45.0), Align2::CENTER_CENTER, "No settings match", 15.0, 500, t.content_2);
            paint_text(&p, pos2(panel.center().x, y0 + 66.0), Align2::CENTER_CENTER, &caption, 12.0, 400, t.content_3);
        } else if self.pane == Pane::About {
            let x = panel_inner.left();
            let w = panel_inner.width();
            let mut y = panel_inner.top();
            let heading = |name: &str| if searching && !scoped { format!("About · {name}") } else { name.to_owned() };

            if app_shown {
                y = group_label(ui, &p, &t, x, y, w, &heading("Application"), parts::APPLICATIONHEADER);
                let card = Rect::from_min_size(pos2(x, y), vec2(w, 111.0));
                let slot = p.add(Shape::Noop);

                // The application row: name, description and version.
                let name_r = paint_text(&p, pos2(x + 8.0, y + 8.0), Align2::LEFT_TOP, "Taskscape", 14.0, 400, t.content_1);
                tag(ui, name_r, Sense::hover(), Role::GenericContainer, "Taskscape", None, None, parts::APPNAME);
                let desc = wrapped(&p, APP_DESCRIPTION, 11.5, 400, t.content_2, w - 76.0);
                let desc_r = Rect::from_min_size(pos2(x + 8.0, y + 29.5), desc.size());
                p.galley(desc_r.min, desc, t.content_2);
                tag(ui, desc_r, Sense::hover(), Role::GenericContainer, APP_DESCRIPTION, None, None, parts::APPDESCRIPTION);
                let ver = if self.version.is_empty() { "—".to_owned() } else { self.version.clone() };
                let ver_r = paint_text(&p, pos2(x + w - 8.0, y + 27.5), Align2::RIGHT_CENTER, &ver, 11.5, 400, t.content_2);
                tag(ui, ver_r, Sense::hover(), Role::GenericContainer, &ver, None, None, parts::VERSION);
                p.hline(x..=x + w, y + 55.0, Stroke::new(1.0, t.edge_1));

                // The data folder row (desktop): the path, and a control that opens it in the file manager.
                let fy = y + 56.0;
                paint_text(&p, pos2(x + 8.0, fy + 8.0), Align2::LEFT_TOP, "Data folder", 14.0, 400, t.content_1);
                let path = if self.data_folder.is_empty() { "—".to_owned() } else { self.data_folder.clone() };
                let path_r = paint_text(&p, pos2(x + 8.0, fy + 27.5), Align2::LEFT_TOP, &path, 11.5, 400, t.content_2);
                tag(ui, path_r, Sense::hover(), Role::GenericContainer, &path, None, None, parts::DATAFOLDERPATH);
                if !self.data_folder.is_empty() {
                    let br = Rect::from_center_size(pos2(x + w - 22.0, fy + 27.5), vec2(28.0, 28.0));
                    let open_name = format!("Open {} in the file manager", self.data_folder);
                    // Opening the folder happens in the OS file manager, outside the app; only the pane stays as it was.
                    let resp = tag(ui, br, Sense::click(), Role::Button, &open_name, Some(self.data_folder.as_str()), None, parts::PATHBUTTON);
                    if resp.hovered() {
                        p.rect_filled(br, CornerRadius::same(7), t.wash_1);
                    }
                    paint_icon(&p, br.center(), "folder_open", 16.0, t.content_2);
                }

                let cr = CornerRadius::same(10);
                p.set(slot, Shape::Vec(vec![Shape::rect_filled(card, cr, t.surface_2), Shape::rect_stroke(card, cr, Stroke::new(1.0, t.edge_2), StrokeKind::Inside)]));
                y += 111.0 + 18.0;
            }

            if reset_shown {
                y = group_label(ui, &p, &t, x, y, w, &heading("Reset"), parts::RESETHEADER);
                let desc = wrapped(&p, RESTORE_DESCRIPTION, 11.5, 400, t.content_2, w - 16.0);
                let dh = desc.size().y;
                let card_h = 8.0 + 19.5 + 2.0 + dh + 7.0 + 30.0 + 8.0;
                let card = Rect::from_min_size(pos2(x, y), vec2(w, card_h));
                let slot = p.add(Shape::Noop);

                let title_r = paint_text(&p, pos2(x + 8.0, y + 8.0), Align2::LEFT_TOP, "Restore defaults", 14.0, 400, t.content_1);
                tag(ui, title_r, Sense::hover(), Role::GenericContainer, "Restore defaults", None, None, parts::RESTOREDEFAULTS);
                let desc_r = Rect::from_min_size(pos2(x + 8.0, y + 29.5), desc.size());
                p.galley(desc_r.min, desc, t.content_2);
                tag(ui, desc_r, Sense::hover(), Role::GenericContainer, RESTORE_DESCRIPTION, None, None, parts::RESTOREDESCRIPTION);

                let by = y + 8.0 + 19.5 + 2.0 + dh + 7.0;
                let row_r = Rect::from_min_size(pos2(x + 8.0, by), vec2(w - 16.0, 30.0));
                if !self.confirming {
                    if self.focus_reset {
                        ui.memory_mut(|m| m.request_focus(Id::new(parts::RESETBUTTON)));
                        self.focus_reset = false;
                    }
                    let resp = tag(ui, row_r, Sense::click(), Role::Button, "Reset all settings…", Some("Reset all settings…"), None, parts::RESETBUTTON);
                    let border = if resp.hovered() { t.edge_3 } else { t.edge_2 };
                    button_paint(&p, row_r, &t, t.surface_3, Some(border), resp.has_focus(), "Reset all settings…", t.content_1);
                    if resp.clicked() {
                        self.confirming = true;
                        self.focus_cancel = true;
                    }
                } else {
                    if self.focus_cancel {
                        ui.memory_mut(|m| m.request_focus(Id::new(parts::CANCELRESET)));
                        self.focus_cancel = false;
                    }
                    let half = (w - 16.0 - 8.0) / 2.0;
                    let cancel_r = Rect::from_min_size(pos2(x + 8.0, by), vec2(half, 30.0));
                    let confirm_r = Rect::from_min_size(pos2(x + 16.0 + half, by), vec2(half, 30.0));
                    let esc = ui.input(|i| i.key_pressed(Key::Escape));
                    let enter = ui.input(|i| i.key_pressed(Key::Enter));
                    let cancel = tag(ui, cancel_r, Sense::click(), Role::Button, "Cancel", Some("Cancel"), None, parts::CANCELRESET);
                    let confirm = tag(ui, confirm_r, Sense::click(), Role::Button, "Reset all settings", Some("Reset all settings"), None, parts::CONFIRMRESET);
                    let cb = if cancel.hovered() { t.edge_3 } else { t.edge_2 };
                    button_paint(&p, cancel_r, &t, t.surface_3, Some(cb), cancel.has_focus(), "Cancel", t.content_1);
                    let fill = if confirm.hovered() { t.danger_600 } else { t.danger_500 };
                    button_paint(&p, confirm_r, &t, fill, None, confirm.has_focus(), "Reset all settings", t.on_accent);
                    let cancel_was_focused = prev_focus == Some(Id::new(parts::CANCELRESET));
                    let confirm_was_focused = prev_focus == Some(Id::new(parts::CONFIRMRESET));
                    if cancel.clicked() || ((cancel_was_focused || cancel.has_focus()) && esc) || ((confirm_was_focused || confirm.has_focus()) && esc) {
                        self.confirming = false;
                        self.focus_reset = true;
                    } else if confirm.clicked() && !enter {
                        // Enter arms but never confirms (decision 10); Space or a click confirms.
                        self.reset_all(app, &mut events);
                    }
                }

                let cr = CornerRadius::same(10);
                p.set(slot, Shape::Vec(vec![Shape::rect_filled(card, cr, t.surface_2), Shape::rect_stroke(card, cr, Stroke::new(1.0, t.edge_2), StrokeKind::Inside)]));
            }
        }

        self.tab_focus = want_focus;
        self.focus_seen = ui.ctx().memory(|m| m.focused());
        events
    }

    /// Selects a section. While searching, a section with matches narrows the search to it; one without clears it.
    fn choose(&mut self, next: Pane, count: usize, searching: bool) {
        self.pane = next;
        if !searching {
            return;
        }
        if count > 0 {
            self.scoped = true;
        } else {
            self.query.clear();
            self.scoped = false;
        }
    }

    /// Confirmed Reset all settings: the preferences go back to their shipped values; tasks are untouched.
    fn reset_all(&mut self, app: &mut App, events: &mut Vec<SettingsaboutEvent>) {
        write_prefs(app, vec![("theme", "system".into()), ("reduced_motion", "system".into()), ("show_completed", "true".into())]);
        self.confirming = false;
        self.focus_reset = true;
        events.push(SettingsaboutEvent::ResetAll { scope: "all".into() });
    }
}

/// The three preferences as stored, under their keys.
fn prefs_of(s: &Settings) -> Vec<(&'static str, String)> {
    vec![("theme", s.theme.clone()), ("reduced_motion", s.reduced_motion.clone()), ("show_completed", s.show_completed.to_string())]
}

/// Writes preferences through the app's data; a refused save goes to the toast and stops the writes.
fn write_prefs(app: &mut App, prefs: Vec<(&'static str, String)>) {
    for (key, value) in prefs {
        if let Err(e) = app.data.set_setting(key, &value) {
            app.refused(e);
            return;
        }
    }
}

fn paint_text(p: &Painter, pos: Pos2, align: Align2, s: &str, px: f32, w: u16, color: Color32) -> Rect {
    p.text(pos, align, s, FontId::new(px, weight(w)), color)
}

fn paint_icon(p: &Painter, center: Pos2, name: &str, px: f32, color: Color32) {
    let g = icon(name, px);
    p.text(center, Align2::CENTER_CENTER, g.text(), FontId::new(px, FontFamily::Name("Icons".into())), color);
}

/// Text laid out to wrap within a width; the caller paints it and reads its size.
fn wrapped(p: &Painter, s: &str, px: f32, w: u16, color: Color32, width: f32) -> Arc<Galley> {
    p.layout(s.to_owned(), FontId::new(px, weight(w)), color, width)
}

/// Tags a rect as a contract part. The node's label is the name; `value` is the text when it differs.
fn tag(ui: &Ui, rect: Rect, sense: Sense, role: Role, label: &str, value: Option<&str>, selected: Option<bool>, part_id: &str) -> Response {
    let response = ui.interact(rect, Id::new(part_id), sense);
    if let Some(on) = selected {
        // A button's selected state is set on the node itself (WidgetInfo carries it for checkboxes only).
        ui.ctx().accesskit_node_builder(response.id, |n| n.set_selected(on));
    }
    let label = label.to_owned();
    let value = value.map(str::to_owned);
    response.widget_info(|| {
        let mut info = match selected {
            Some(s) => WidgetInfo::selected(role, true, s, label.clone()),
            None => WidgetInfo::labeled(role, true, label.clone()),
        };
        info.current_text_value = value.clone();
        info
    });
    part(ui, &response, part_id);
    response
}

/// An eyebrow label over a card, with a rule to its right. Returns the y where the card starts.
fn group_label(ui: &Ui, p: &Painter, t: &Tokens, x: f32, y: f32, w: f32, label: &str, part_id: &str) -> f32 {
    let r = paint_text(p, pos2(x, y + 8.0), Align2::LEFT_CENTER, &label.to_uppercase(), 10.5, 600, t.content_3);
    tag(ui, r, Sense::hover(), Role::Heading, label, None, None, part_id);
    p.hline(r.right() + 8.0..=x + w, y + 8.0, Stroke::new(1.0, t.edge_2));
    y + 24.0
}

/// A button's box: a fill, an optional border, the focus ring and its centred label.
fn button_paint(p: &Painter, r: Rect, t: &Tokens, fill: Color32, border: Option<Color32>, focus: bool, label: &str, color: Color32) {
    let cr = CornerRadius::same(7);
    p.rect_filled(r, cr, fill);
    if let Some(b) = border {
        p.rect_stroke(r, cr, Stroke::new(1.0, b), StrokeKind::Inside);
    }
    if focus {
        p.rect_stroke(r, cr, Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
    }
    paint_text(p, r.center(), Align2::CENTER_CENTER, label, 12.5, 600, color);
}

impl Default for SettingsAbout {
    fn default() -> Self {
        Self::new()
    }
}

/// The screen as the test mounts it: the app on the demo data set, with the props' version and data folder.
struct Mount {
    app: App,
    screen: SettingsAbout,
}

impl Mounted for Mount {
    fn frame(&mut self, ui: &mut Ui, look: Look, emit: &mut dyn FnMut(&'static str, serde_json::Value)) {
        for e in self.screen.show(&mut self.app, ui, look) {
            let (n, f) = e.to_json();
            emit(n, f);
        }
    }
}

pub fn mount(mut app: App, props: serde_json::Value) -> Result<Box<dyn Mounted>, String> {
    let p: SettingsaboutProps = serde_json::from_value(props).map_err(|e| e.to_string())?;
    // The start state the contract needs (as the web harness sets it): appearance at System, Show completed
    // off, so that a reset changes something for the discard control to revert.
    let _ = app.data.set_setting("theme", "system");
    let _ = app.data.set_setting("reduced_motion", "system");
    let _ = app.data.set_setting("show_completed", "false");
    // The Settings window is open on its About page (app.settings_window names the page shown).
    app.settings_window = Some("about".to_owned());
    let mut screen = SettingsAbout::new();
    screen.version = p.version;
    screen.data_folder = p.dataFolder;
    Ok(Box::new(Mount { app, screen }))
}
