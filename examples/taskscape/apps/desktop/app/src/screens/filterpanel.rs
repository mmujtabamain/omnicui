//! contract: omnicui/contracts/filterpanel.contract.yaml (id: app.filterpanel). Filter & Sort in the main window's aside, beside the task list pane it edits: the title bar with the list tabs, the pane, and the panel or its empty state.
#![allow(deprecated)]
use crate::state::{Aside, View};
use crate::App;
use std::collections::HashSet;
use taskscape_ui::omnicui::filterpanel::{parts, FilterpanelEvent, FilterpanelProps};
use taskscape_ui::{icon, part, text, weight, Look};
use egui::{pos2, vec2, Align2, Color32, CornerRadius, FontFamily, FontId, Key, Layout, Pos2, Rect, Response, Role, Sense, Stroke, StrokeKind, WidgetInfo};

const ASIDE_W: f32 = 320.0;

const SORTS: [(&str, &str, &str); 5] = [
    (parts::SORTMANUAL, "Manual", "manual"),
    (parts::SORTCREATED, "Date created", "created"),
    (parts::SORTUPDATED, "Date updated", "updated"),
    (parts::SORTALPHABETICAL, "Alphabetical", "title"),
    (parts::SORTDONELAST, "Done last", "done"),
];

const RANGES: [(&str, &str, &str); 5] = [
    (parts::RANGEANYTIME, "Any time", "any"),
    (parts::RANGELAST24HOURS, "Last 24 hours", "today"),
    (parts::RANGELAST7DAYS, "Last 7 days", "week"),
    (parts::RANGELAST30DAYS, "Last 30 days", "month"),
    (parts::RANGECUSTOM, "Custom…", "custom"),
];

const ROW_PARTS: [(&str, &str); 7] = [
    ("Pick up prescription", parts::ROWPICKUP),
    ("Book bike service", parts::ROWBIKE),
    ("Buy groceries for the week", parts::ROWGROCERIES),
    ("Sourdough loaf", parts::ROWSOURDOUGH),
    ("Oat milk", parts::ROWOATMILK),
    ("Post the parcel to Mum", parts::ROWPOST),
    ("Return library books", parts::ROWRETURNLIBRARY),
];

#[derive(Clone, Copy, PartialEq)]
pub enum Scope {
    List,
    Project,
    All,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Fields {
    Both,
    Title,
    Notes,
}

#[derive(Clone, Copy, PartialEq)]
enum Style {
    Primary,
    Neutral,
    Ghost,
}

/// The screen's own state: the search scope and fields, the open menus, the day count, and which rows are collapsed.
pub struct FilterPanel {
    scope: Scope,
    fields: Fields,
    queries: Vec<String>,
    collapsed: HashSet<String>,
    sort_open: bool,
    range_open: bool,
    options_open: bool,
    days: u32,
    days_text: String,
    updated: bool,
    /// Custom range was just picked: the day field takes the keyboard.
    focus_days: bool,
    /// The search field had the keyboard at the end of the last frame (Escape empties it).
    search_had_focus: bool,
}

/// One task as the pane reads it: owned, in the list's stored order.
struct Item {
    id: String,
    title: String,
    notes: String,
    done: bool,
    parent: Option<String>,
    has_att: bool,
}

enum Pred {
    Query(String, Fields),
    Show(String, bool),
    Refine(String, bool),
}

struct Row {
    id: String,
    title: String,
    note: String,
    done: bool,
    depth: usize,
    has_kids: bool,
    expanded: bool,
    kids_visible: usize,
    kids_done: usize,
}

#[derive(Default)]
struct Section {
    heading: Option<String>,
    rows: Vec<Row>,
}

#[derive(Default)]
struct Model {
    sections: Vec<Section>,
    visible: usize,
    total: usize,
    done: usize,
    matches: usize,
}

#[derive(Clone, Copy)]
struct RowFlags {
    primary: bool,
    first: bool,
    parent: bool,
    done: bool,
    searching: bool,
}

impl FilterPanel {
    pub fn new() -> Self {
        Self {
            scope: Scope::List,
            fields: Fields::Both,
            queries: vec![String::new(), String::new()],
            collapsed: HashSet::new(),
            sort_open: false,
            range_open: false,
            options_open: false,
            days: 7,
            days_text: "7".into(),
            updated: false,
            focus_days: false,
            search_had_focus: false,
        }
    }

    /// One frame: the title bar, the task list pane(s), and the Filter & Sort panel or its empty state in the aside.
    pub fn show(&mut self, app: &mut App, ui: &mut egui::Ui, look: Look) -> Vec<FilterpanelEvent> {
        if ui.input(|i| i.key_pressed(Key::Escape)) {
            self.sort_open = false;
            self.range_open = false;
            self.options_open = false;
        }
        ui.vertical(|ui| {
            self.title_bar(app, ui, look);
            let body_h = ui.available_height();
            ui.horizontal(|ui| {
                let aside = app.ui.aside != Aside::None;
                let pane_w = (ui.available_width() - if aside { ASIDE_W } else { 0.0 }).max(0.0);
                ui.allocate_ui_with_layout(vec2(pane_w, body_h), Layout::top_down(egui::Align::Min), |ui| self.panes(app, ui, look));
                if aside {
                    ui.allocate_ui_with_layout(vec2(ASIDE_W, body_h), Layout::top_down(egui::Align::Min), |ui| self.aside(app, ui, look));
                }
            });
        });
        Vec::new()
    }

    fn title_bar(&mut self, app: &mut App, ui: &mut egui::Ui, look: Look) {
        let project = app.ui.project.clone();
        let pname = project.as_deref().and_then(|p| app.data.project(p)).map(|p| p.name.clone()).unwrap_or_else(|| "Project".into());
        let tabs: Vec<(String, String)> = match project.as_deref() {
            Some(p) => app.data.lists_of(p).iter().map(|l| (l.id.clone(), l.name.clone())).collect(),
            None => vec![],
        };
        let active = app.active_list();
        let aside = app.ui.aside;
        let split_on = app.ui.split_on();
        ui.horizontal(|ui| {
            ui.set_height(40.0);
            ui.add_space(8.0);
            let w = text_w(ui, &pname, 13.0, 500) + 36.0;
            pill(ui, look, parts::PROJECTSWITCH, &pname, "", "unfold_more", w, Style::Ghost, true);
            for (id, name) in &tabs {
                let tw = text_w(ui, name, 13.0, 500) + 24.0;
                let on = active.as_deref() == Some(id.as_str());
                let key = match name.as_str() {
                    "Errands" => parts::LISTTABERRANDS,
                    "Reading" => parts::LISTTABREADING,
                    "Garden" => parts::LISTTABGARDEN,
                    _ => "",
                };
                if tab(ui, look, key, name, tw, on).clicked() {
                    app.ui.open_list(Some(id.clone()), None);
                }
            }
            if icon_button(ui, look, parts::NEWLIST, "New list", "add", None, true).clicked() {
                app.sheet = Some(crate::app::Sheet::NewList);
            }
            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                pill(ui, look, parts::ACCOUNTBUTTON, "?", "", "", 28.0, Style::Ghost, true);
                if icon_button(ui, look, parts::SETTINGS, "Settings", "settings", None, true).clicked() {
                    app.settings_window = Some("general".into());
                }
                let shown = aside != Aside::None;
                let hide_name = if shown { "Hide preview panel" } else { "Show preview panel" };
                if icon_button(ui, look, parts::HIDEPREVIEW, hide_name, "right_panel_close", Some(shown), true).clicked() {
                    app.ui.aside = if shown { Aside::None } else { Aside::Preview };
                }
                if icon_button(ui, look, parts::SPLITVIEW, "Split view", "vertical_split", Some(split_on), true).clicked() {
                    toggle_split(app);
                }
                if icon_button(ui, look, parts::PALETTEBUTTON, "Command palette", "keyboard_command_key", None, true).clicked() {
                    app.ui.palette_open = true;
                }
                if icon_button(ui, look, parts::TRASHBUTTON, "Trash", "delete", None, true).clicked() {
                    app.trash_window = true;
                }
            });
        });
    }

    /// One pane per list on screen (two in split view), side by side.
    fn panes(&mut self, app: &mut App, ui: &mut egui::Ui, look: Look) {
        let n = app.ui.panes.len().max(1);
        let models: Vec<Model> = (0..n).map(|i| self.model_for(app, i)).collect();
        ui.columns(n, |cols| {
            for (i, col) in cols.iter_mut().enumerate() {
                self.pane(app, col, look, i, &models[i]);
            }
        });
    }

    fn pane(&mut self, app: &mut App, ui: &mut egui::Ui, look: Look, i: usize, m: &Model) {
        let primary = i == 0;
        let name = app.ui.panes.get(i).cloned().flatten().and_then(|id| app.data.list(&id).map(|l| l.name.clone())).unwrap_or_default();
        let pane_active = app.ui.panes.get(i).cloned().flatten().map(|id| view_active(&app.ui.view(&id))).unwrap_or(false);
        self.search(app, ui, look, i, &name, m.matches);
        let searching = self.queries.get(i).is_some_and(|q| !q.trim().is_empty());
        let flat = || m.sections.iter().flat_map(|s| s.rows.iter());
        let first_row = flat().next().map(|r| r.id.clone());
        let first_parent = flat().find(|r| r.has_kids).map(|r| r.id.clone());
        let first_done = flat().find(|r| r.done).map(|r| r.id.clone());
        let row_count: usize = m.sections.iter().map(|s| s.rows.len()).sum();
        let rows_h = (ui.available_height() - 34.0).max(0.0);
        egui::ScrollArea::vertical().id_salt(("rows", i)).auto_shrink([false, false]).max_height(rows_h).show(ui, |ui| {
            for s in &m.sections {
                if let Some(h) = &s.heading {
                    let hr = ui.label(text(h.clone(), 12.0, 600));
                    let gp = if primary { group_part(h) } else { "" };
                    node(ui, &hr, Role::Heading, true, h, None, gp);
                }
                for r in &s.rows {
                    let f = RowFlags {
                        primary,
                        first: first_row.as_deref() == Some(r.id.as_str()),
                        parent: first_parent.as_deref() == Some(r.id.as_str()),
                        done: first_done.as_deref() == Some(r.id.as_str()),
                        searching,
                    };
                    self.row(app, ui, look, r, f);
                }
            }
            if row_count == 0 {
                ui.add_space(24.0);
                if searching {
                    let title = format!("No matches in {name}");
                    let r = center_text(ui, &title, 14.0, 600, look.t.content_1);
                    node(ui, &r, Role::Status, true, &title, None, if primary { parts::EMPTYSTATE } else { "" });
                    let hint = center_text(ui, "Try a different search or scope", 12.0, 500, look.t.content_2);
                    node(ui, &hint, Role::GenericContainer, true, "Try a different search or scope", None, if primary { parts::NOMATCHHINT } else { "" });
                } else {
                    let r = center_text(ui, "Nothing here yet", 14.0, 600, look.t.content_1);
                    node(ui, &r, Role::Status, true, "Nothing here yet", None, if primary { parts::EMPTYSTATE } else { "" });
                }
            }
        });
        footer(app, ui, look, primary, m, pane_active);
    }

    fn search(&mut self, app: &mut App, ui: &mut egui::Ui, look: Look, i: usize, name: &str, matches: usize) {
        let t = look.t;
        let primary = i == 0;
        while self.queries.len() <= i {
            self.queries.push(String::new());
        }
        let searching = !self.queries[i].trim().is_empty();
        let escape = ui.input(|k| k.key_pressed(Key::Escape));
        let find = ui.input(|k| (k.modifiers.command || k.modifiers.ctrl || k.modifiers.mac_cmd) && k.key_pressed(Key::F));
        ui.horizontal(|ui| {
            ui.add_space(12.0);
            let width = ui.available_width() - 12.0;
            egui::Frame::new().fill(t.wash_2).corner_radius(CornerRadius::same(18)).inner_margin(egui::Margin::symmetric(10, 5)).show(ui, |ui| {
                ui.set_width(width - 20.0);
                ui.horizontal(|ui| {
                    let (gr, _) = ui.allocate_exact_size(vec2(18.0, 18.0), Sense::hover());
                    glyph(ui, gr.center(), "search", 16.0, t.content_2);
                    let trailing = 26.0 + 2.0 + 26.0 + 26.0 + if searching { 56.0 } else { 0.0 } + 16.0;
                    let fw = (ui.available_width() - trailing).max(60.0);
                    let sid = ui.scope_id().with(("search", i));
                    let was_focused = ui.memory(|m| m.has_focus(sid));
                    let resp = ui.add(egui::TextEdit::singleline(&mut self.queries[i]).id(sid).frame(egui::Frame::NONE).desired_width(fw).hint_text(format!("Search {name}")));
                    if primary {
                        part(ui, &resp, parts::SEARCHFIELD);
                        if find && !resp.has_focus() {
                            resp.request_focus();
                        }
                    }
                    if escape && (was_focused || self.search_had_focus) {
                        self.queries[i].clear();
                    }
                    if primary {
                        self.search_had_focus = resp.has_focus();
                    }
                    if searching {
                        let count = matches.to_string();
                        let (cr, _) = ui.allocate_exact_size(vec2(30.0, 18.0), Sense::hover());
                        let cn = ui.interact(cr, ui.scope_id().with(("count", i)), Sense::hover());
                        ui.painter().text(cr.right_center(), Align2::RIGHT_CENTER, &count, FontId::new(12.0, weight(600)), t.content_2);
                        node(ui, &cn, Role::GenericContainer, true, &count, None, if primary { parts::SEARCHCOUNT } else { "" });
                        if icon_button(ui, look, if primary { parts::SEARCHCLEAR } else { "" }, "Clear search", "close", None, true).clicked() {
                            self.queries[i].clear();
                        }
                    }
                    let opt = icon_button(ui, look, if primary { parts::SEARCHOPTIONS } else { "" }, "Search options", "manage_search", None, true);
                    if opt.clicked() {
                        self.options_open = !self.options_open;
                        self.sort_open = false;
                        self.range_open = false;
                    }
                    if self.options_open && primary {
                        let at = opt.rect.left_bottom() + vec2(-150.0, 4.0);
                        popup(ui, "app.filterpanel.options", at, |ui| {
                            if menu_item(ui, look, parts::SCOPEWHOLEPROJECT, "Whole project") {
                                self.scope = Scope::Project;
                                self.options_open = false;
                            }
                            if menu_item(ui, look, "", "This list") {
                                self.scope = Scope::List;
                                self.options_open = false;
                            }
                            if menu_item(ui, look, "", "All projects") {
                                self.scope = Scope::All;
                                self.options_open = false;
                            }
                            if menu_item(ui, look, "", "Title & notes") {
                                self.fields = Fields::Both;
                                self.options_open = false;
                            }
                            if menu_item(ui, look, "", "Title only") {
                                self.fields = Fields::Title;
                                self.options_open = false;
                            }
                            if menu_item(ui, look, parts::FIELDSNOTESONLY, "Notes only") {
                                self.fields = Fields::Notes;
                                self.options_open = false;
                            }
                        });
                    }
                    let (sr, _) = ui.allocate_exact_size(vec2(1.0, 18.0), Sense::hover());
                    ui.painter().rect_filled(sr, CornerRadius::ZERO, t.edge_2);
                    if icon_button(ui, look, if primary { parts::NEWTASK } else { "" }, "New task", "add", None, true).clicked() {
                        app.ui.draft_list = app.active_list();
                    }
                });
            });
        });
    }

    fn row(&mut self, app: &mut App, ui: &mut egui::Ui, look: Look, r: &Row, f: RowFlags) {
        let t = look.t;
        let h = if r.note.is_empty() { 34.0 } else { 46.0 };
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), h), Sense::hover());
        let indent = r.depth as f32 * 24.0;
        let cy = rect.top() + 17.0;

        let row_r = ui.interact(rect, ui.scope_id().with(("row", &r.id)), Sense::hover());
        node(ui, &row_r, Role::GenericContainer, true, &r.title, None, if f.primary && f.first { parts::ROW } else { "" });
        if f.primary {
            let named = row_part(&r.title);
            if !named.is_empty() {
                let nr = ui.interact(rect, ui.scope_id().with(("named", &r.id)), Sense::hover());
                node(ui, &nr, Role::GenericContainer, true, &r.title, None, named);
            }
        }

        // The round check: done fills with the done lamp and draws a check.
        let circle = Rect::from_center_size(pos2(rect.left() + 22.0 + indent, cy), vec2(16.0, 16.0));
        let cb = ui.interact(circle, ui.scope_id().with(("done", &r.id)), Sense::click());
        let fill = if r.done { t.done_lamp_1 } else { t.surface_3 };
        let edge = if r.done { t.done_lamp_1 } else { t.content_3 };
        ui.painter().circle(circle.center(), 8.0, fill, Stroke::new(1.0, edge));
        if r.done {
            glyph(ui, circle.center(), "check", 12.0, t.on_accent);
        }
        if cb.has_focus() {
            ui.painter().circle_stroke(circle.center(), 10.0, Stroke::new(2.0, t.focus_1));
        }
        node(ui, &cb, Role::CheckBox, true, "Select", Some(r.done), if f.primary && f.done { parts::DONECHECK } else { "" });
        if cb.clicked() {
            if let Err(e) = app.data.update_task(&r.id, None, None, Some(!r.done)) {
                app.refused(e);
            }
        }

        // The title (a click previews the task) and its note.
        let tx = circle.right() + 8.0;
        let right_w = if r.has_kids && !f.searching { 120.0 } else { 36.0 };
        let title_rect = Rect::from_min_max(pos2(tx, rect.top() + 6.0), pos2(rect.right() - right_w, rect.top() + 26.0));
        let tc = if r.done { t.content_3 } else { t.content_1 };
        let galley = ui.painter().layout_no_wrap(r.title.clone(), FontId::new(14.0, weight(500)), tc);
        let tw = galley.size().x.min(title_rect.width());
        ui.painter().with_clip_rect(title_rect).galley(title_rect.min, galley, tc);
        if r.done {
            let y = title_rect.top() + 10.0;
            ui.painter().line_segment([pos2(tx, y), pos2(tx + tw, y)], Stroke::new(1.0, t.content_3));
        }
        let tr = ui.interact(title_rect, ui.scope_id().with(("title", &r.id)), Sense::click());
        if tr.clicked() {
            app.ui.preview = Some(r.id.clone());
        }
        node(ui, &tr, Role::Button, true, &r.title, None, "");
        if !r.note.is_empty() {
            let nr = Rect::from_min_max(pos2(tx, rect.top() + 26.0), pos2(title_rect.right(), rect.top() + 42.0));
            ui.painter().with_clip_rect(nr).text(nr.left_center(), Align2::LEFT_CENTER, &r.note, FontId::new(12.0, weight(400)), t.content_2);
        }

        // The subtask toggle and the done count of the visible children.
        if r.has_kids && !f.searching {
            let tg_rect = Rect::from_center_size(pos2(rect.right() - 46.0, cy), vec2(22.0, 22.0));
            let tg = ui.interact(tg_rect, ui.scope_id().with(("toggle", &r.id)), Sense::click());
            if tg.hovered() {
                ui.painter().rect_filled(tg_rect, CornerRadius::same(6), t.wash_1);
            }
            let c = tg_rect.center();
            let pts = if r.expanded {
                vec![c + vec2(-4.0, -2.0), c + vec2(4.0, -2.0), c + vec2(0.0, 3.0)]
            } else {
                vec![c + vec2(-2.0, -4.0), c + vec2(-2.0, 4.0), c + vec2(3.0, 0.0)]
            };
            ui.painter().add(egui::Shape::convex_polygon(pts, t.content_2, Stroke::NONE));
            if tg.clicked() {
                if r.expanded {
                    self.collapsed.insert(r.id.clone());
                } else {
                    self.collapsed.remove(&r.id);
                }
            }
            node(ui, &tg, Role::Button, true, if r.expanded { "Collapse" } else { "Expand" }, None, if f.primary && f.parent { parts::SUBTASKTOGGLE } else { "" });
            if r.kids_visible > 0 {
                let label = format!("{} / {}", r.kids_done, r.kids_visible);
                let bw = text_w(ui, &label, 12.0, 600) + 14.0;
                let br = Rect::from_min_size(pos2(tg_rect.left() - 8.0 - bw, cy - 9.0), vec2(bw, 18.0));
                ui.painter().rect_filled(br, CornerRadius::same(9), t.wash_2);
                ui.painter().text(br.center(), Align2::CENTER_CENTER, &label, FontId::new(12.0, weight(600)), t.content_2);
                let bn = ui.interact(br, ui.scope_id().with(("count", &r.id)), Sense::hover());
                node(ui, &bn, Role::GenericContainer, true, &label, None, if f.primary && f.parent { parts::SUBTASKCOUNT } else { "" });
            }
        }

        // The More button.
        let mr = Rect::from_center_size(pos2(rect.right() - 16.0, cy), vec2(24.0, 24.0));
        let mo = ui.interact(mr, ui.scope_id().with(("more", &r.id)), Sense::click());
        if mo.hovered() {
            ui.painter().rect_filled(mr, CornerRadius::same(7), t.wash_1);
        }
        glyph(ui, mr.center(), "more_horiz", 16.0, t.content_2);
        node(ui, &mo, Role::Button, true, "More", None, if f.primary && f.first { parts::ROWMORE } else { "" });
    }

    fn aside(&mut self, app: &mut App, ui: &mut egui::Ui, look: Look) {
        if app.ui.aside == Aside::Filter {
            self.panel(app, ui, look);
        } else {
            self.preview_empty(ui, look);
        }
    }

    /// The preview column's empty state, shown while the panel is closed.
    fn preview_empty(&self, ui: &mut egui::Ui, look: Look) {
        let t = look.t;
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), ui.available_height()), Sense::hover());
        ui.painter().rect_filled(rect, CornerRadius::ZERO, t.surface_2);
        let c = rect.center();
        glyph(ui, c - vec2(0.0, 26.0), "ads_click", 28.0, t.content_3);
        ui.painter().text(c + vec2(0.0, 2.0), Align2::CENTER_CENTER, "No task selected", FontId::new(14.0, weight(600)), t.content_1);
        ui.painter().text(c + vec2(0.0, 22.0), Align2::CENTER_CENTER, "Select a task to inspect it", FontId::new(12.0, weight(500)), t.content_2);
        let r = ui.interact(rect, ui.scope_id().with("previewEmpty"), Sense::hover());
        node(ui, &r, Role::Status, true, "No task selected", None, parts::PREVIEWEMPTY);
    }

    /// The Filter & Sort panel: sort, show, refine and date, edited on every list on screen as one.
    /// The Filter & Sort panel alone, as the main window's aside shows it.
    pub fn panel(&mut self, app: &mut App, ui: &mut egui::Ui, look: Look) {
        let t = look.t;
        let names: Vec<String> = app.ui.panes.iter().flatten().filter_map(|id| app.data.list(id).map(|l| l.name.clone())).collect();
        let subtitle = if names.is_empty() { "No list open".to_string() } else { names.join(", ") };
        let target = targets(app);
        let view: View = target.first().map(|id| app.ui.view(id)).unwrap_or_default();
        let view_on = target.iter().any(|id| view_active(&app.ui.view(id)));
        let summary = self.summary(app);
        let sort_on = view.sort != "manual" || view.descending;
        let manual = view.sort == "manual";

        let avail = ui.available_height();
        egui::Frame::new().fill(t.surface_2).show(ui, |ui| {
            // Header: the title, the target lists, and the close button.
            ui.horizontal(|ui| {
                ui.set_height(38.0);
                ui.add_space(12.0);
                let (ir, _) = ui.allocate_exact_size(vec2(18.0, 18.0), Sense::hover());
                glyph(ui, ir.center(), "tune", 18.0, t.content_2);
                ui.vertical(|ui| {
                    let h = ui.label(text("Filter & Sort", 14.0, 600));
                    node(ui, &h, Role::Heading, true, "Filter & Sort", None, parts::PANELTITLE);
                    let s = ui.label(text(subtitle.clone(), 12.0, 500));
                    node(ui, &s, Role::GenericContainer, true, &subtitle, None, parts::PANELSUBTITLE);
                });
                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(8.0);
                    if icon_button(ui, look, parts::CLOSEBUTTON, "Close", "close", None, true).clicked() {
                        app.ui.aside = Aside::Preview;
                    }
                });
            });

            egui::ScrollArea::vertical().id_salt("filter-body").auto_shrink([false, false]).max_height((avail - 124.0).max(0.0)).show(ui, |ui| {
            egui::Frame::new().inner_margin(egui::Margin::symmetric(12, 6)).show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 10.0;

                // Sort
                if section_title(ui, look, "Sort", parts::SECTIONSORT, sort_on, parts::CLEARSORT, "Clear sort") {
                    patch(app, |v: &mut View| {
                        v.sort = "manual".into();
                        v.descending = false;
                    });
                }
                ui.horizontal(|ui| {
                    let w = ui.available_width() - 60.0;
                    let lead = if manual { "drag_indicator" } else { "sort" };
                    let mr = pill(ui, look, parts::SORTMENU, sort_label(&view.sort), lead, "unfold_more", w, Style::Neutral, true);
                    if mr.clicked() {
                        self.sort_open = !self.sort_open;
                        self.range_open = false;
                    }
                    if self.sort_open {
                        popup(ui, "app.filterpanel.sortmenu", mr.rect.left_bottom() + vec2(0.0, 2.0), |ui| {
                            for (p, l, v) in SORTS {
                                if menu_item(ui, look, p, l) {
                                    let v = v.to_string();
                                    patch(app, move |x: &mut View| x.sort = v.clone());
                                    self.sort_open = false;
                                }
                            }
                        });
                    }
                    if icon_button(ui, look, parts::SORTASCENDING, "Ascending", "arrow_upward", Some(!manual && !view.descending), !manual).clicked() {
                        patch(app, |x: &mut View| x.descending = false);
                    }
                    if icon_button(ui, look, parts::SORTDESCENDING, "Descending", "arrow_downward", Some(!manual && view.descending), !manual).clicked() {
                        patch(app, |x: &mut View| x.descending = true);
                    }
                });

                // Show
                if section_title(ui, look, "Show", parts::SECTIONSHOW, view.show != "all", parts::CLEARSHOW, "Clear show") {
                    patch(app, |x: &mut View| x.show = "all".into());
                }
                let show_idx = match view.show.as_str() {
                    "active" => 1,
                    "done" => 2,
                    _ => 0,
                };
                if let Some(i) = segments(ui, look, &[(parts::SHOWALL, "All", show_idx == 0), (parts::SHOWACTIVE, "Active", show_idx == 1), (parts::SHOWDONE, "Done", show_idx == 2)]) {
                    let v = ["all", "active", "done"][i];
                    patch(app, move |x: &mut View| x.show = v.to_string());
                }

                // Refine: each row is Any, With or Without.
                if section_title(ui, look, "Refine", parts::SECTIONREFINE, !view.refine.is_empty(), parts::CLEARREFINE, "Clear refine") {
                    patch(app, |x: &mut View| x.refine.clear());
                }
                for (key, lead, label) in [("notes", "notes", "Notes"), ("attachments", "attach_file", "Attachments"), ("subtasks", "account_tree", "Subtasks")] {
                    let cur = tri_of(&view, key);
                    ui.horizontal(|ui| {
                        let (ir, _) = ui.allocate_exact_size(vec2(105.0, 24.0), Sense::hover());
                        glyph(ui, ir.left_center() + vec2(8.0, 0.0), lead, 14.0, t.content_2);
                        let lc = if cur == "any" { t.content_2 } else { t.content_1 };
                        ui.painter().text(ir.left_center() + vec2(22.0, 0.0), Align2::LEFT_CENTER, label, FontId::new(13.0, weight(500)), lc);
                        let ids = tri_parts(key);
                        let opts = [(ids[0], "Any", cur == "any"), (ids[1], "With", cur == "with"), (ids[2], "Without", cur == "without")];
                        if let Some(i) = segments(ui, look, &opts) {
                            let v = ["any", "with", "without"][i];
                            patch(app, move |x: &mut View| set_tri(x, key, v));
                        }
                    });
                }

                // Date: the field, the range, and the day count while the range is custom.
                if section_title(ui, look, "Date", parts::SECTIONDATE, view.date != "any", parts::CLEARDATE, "Clear date") {
                    patch(app, |x: &mut View| x.date = "any".into());
                    self.updated = false;
                    self.days = 7;
                    self.days_text = "7".into();
                }
                let upd = self.updated;
                if let Some(i) = segments(ui, look, &[(parts::DATECREATED, "Created", !upd), (parts::DATEUPDATED, "Updated", upd)]) {
                    self.updated = i == 1;
                }
                let rw = ui.available_width();
                let rr = pill(ui, look, parts::DATERANGEBUTTON, range_label(&view.date), "schedule", "unfold_more", rw, Style::Neutral, true);
                if rr.clicked() {
                    self.range_open = !self.range_open;
                    self.sort_open = false;
                }
                if self.range_open {
                    popup(ui, "app.filterpanel.rangemenu", rr.rect.left_bottom() + vec2(0.0, 2.0), |ui| {
                        for (p, l, v) in RANGES {
                            if menu_item(ui, look, p, l) {
                                let v = v.to_string();
                                if v == "custom" {
                                    self.focus_days = true;
                                }
                                patch(app, move |x: &mut View| x.date = v.clone());
                                self.range_open = false;
                            }
                        }
                    });
                }
                if view.date == "custom" {
                    ui.horizontal(|ui| {
                        let c = ui.label(text("Within the last", 13.0, 500));
                        node(ui, &c, Role::GenericContainer, true, "Within the last", None, parts::DAYSCAPTION);
                        if icon_button(ui, look, parts::DAYSFEWER, "Fewer days", "remove", None, true).clicked() {
                            self.days = self.days.saturating_sub(1).max(1);
                            self.days_text = self.days.to_string();
                        }
                        let tid = ui.scope_id().with("days");
                        let resp = ui.add(egui::TextEdit::singleline(&mut self.days_text).id(tid).desired_width(44.0));
                        if self.focus_days {
                            resp.request_focus();
                            self.focus_days = false;
                        }
                        part(ui, &resp, parts::DAYSINPUT);
                        if resp.changed() {
                            let typed = self.days_text.trim().to_string();
                            if typed.is_empty() {
                                self.days = 1;
                                self.days_text = "1".into();
                            } else if let Ok(n) = typed.parse::<u32>() {
                                let c = n.clamp(1, 365);
                                self.days = c;
                                if c != n {
                                    self.days_text = c.to_string();
                                }
                            }
                        }
                        if icon_button(ui, look, parts::DAYSMORE, "More days", "add", None, true).clicked() {
                            self.days = (self.days + 1).min(365);
                            self.days_text = self.days.to_string();
                        }
                        let u = ui.label(text("days", 13.0, 500));
                        node(ui, &u, Role::GenericContainer, true, "days", None, parts::DAYSUNIT);
                    });
                }
            });
            });
        });

        // Summary, then the footer buttons.
        let (rule, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
        ui.painter().rect_filled(rule, CornerRadius::ZERO, t.edge_1);
        ui.horizontal(|ui| {
            ui.set_height(29.0);
            ui.add_space(12.0);
            let (dr, _) = ui.allocate_exact_size(vec2(6.0, 6.0), Sense::hover());
            ui.painter().circle_filled(dr.center(), 3.0, if view_on { t.accent_500 } else { t.edge_3 });
            let s = ui.label(text(summary.clone(), 12.0, 500));
            node(ui, &s, Role::Status, true, &summary, None, parts::SUMMARY);
        });
        ui.horizontal(|ui| {
            ui.set_height(44.0);
            ui.add_space(12.0);
            if pill(ui, look, parts::RESETBUTTON, "Reset", "restart_alt", "", 84.0, Style::Ghost, view_on).clicked() {
                reset_all(app);
                self.updated = false;
                self.days = 7;
                self.days_text = "7".into();
            }
            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(12.0);
                if pill(ui, look, parts::DONEBUTTON, "Done", "", "", 84.0, Style::Primary, true).clicked() {
                    app.ui.aside = Aside::Preview;
                }
            });
        });
    }

    /// The model of the pane at index i: its own rows, or in a whole-project or all-projects search, the matches grouped by list.
    fn model_for(&self, app: &App, i: usize) -> Model {
        let Some(list) = app.ui.panes.get(i).cloned().flatten() else {
            return Model::default();
        };
        let view = app.ui.view(&list);
        let query = self.queries.get(i).cloned().unwrap_or_default();
        let show_c = app.data.settings.show_completed;
        if self.scope == Scope::List || query.trim().is_empty() {
            return model(app, &list, &view, &query, self.fields, &self.collapsed, show_c);
        }
        let mut lists: Vec<(String, String, String)> = vec![];
        match self.scope {
            Scope::Project => {
                if let Some(p) = app.ui.project.clone() {
                    for l in app.data.lists_of(&p) {
                        lists.push((l.id.clone(), l.name.clone(), String::new()));
                    }
                }
            }
            _ => {
                for p in &app.data.projects {
                    for l in app.data.lists_of(&p.id) {
                        lists.push((l.id.clone(), l.name.clone(), p.name.clone()));
                    }
                }
            }
        }
        let own = model(app, &list, &view, &query, self.fields, &self.collapsed, show_c);
        let mut out = Model { sections: vec![], visible: own.visible, total: own.total, done: own.done, matches: 0 };
        for (id, name, project) in lists {
            let m = model(app, &id, &view, &query, self.fields, &self.collapsed, show_c);
            out.matches += m.matches;
            let rows: Vec<Row> = m.sections.into_iter().flat_map(|s| s.rows).collect();
            if !rows.is_empty() {
                let heading = if project.is_empty() { name } else { format!("{project} · {name}") };
                out.sections.push(Section { heading: Some(heading), rows });
            }
        }
        out
    }

    /// "Showing 8 of 8 tasks" for one list, "Errands 8/8 · Reading 3/3" when split, "Nothing to filter" with no list.
    fn summary(&self, app: &App) -> String {
        let mut shown: Vec<(String, usize, usize)> = vec![];
        for i in 0..app.ui.panes.len() {
            if let Some(name) = app.ui.panes[i].as_deref().and_then(|id| app.data.list(id)).map(|l| l.name.clone()) {
                let m = self.model_for(app, i);
                shown.push((name, m.visible, m.total));
            }
        }
        match shown.len() {
            0 => "Nothing to filter".into(),
            1 => format!("Showing {} of {} tasks", shown[0].1, shown[0].2),
            _ => shown.iter().map(|(n, v, t)| format!("{n} {v}/{t}")).collect::<Vec<_>>().join(" · "),
        }
    }
}

/// The footer of a pane: Sort & filter, the done count and the percentage.
fn footer(app: &mut App, ui: &mut egui::Ui, look: Look, primary: bool, m: &Model, active: bool) {
    let t = look.t;
    let pct = if m.total > 0 { (m.done as f32 / m.total as f32 * 100.0).round() as usize } else { 0 };
    ui.horizontal(|ui| {
        ui.set_height(32.0);
        ui.add_space(4.0);
        let label = if active { "Filtered" } else { "Sort & filter" };
        let w = text_w(ui, label, 13.0, 500) + 40.0;
        if pill(ui, look, if primary { parts::SORTANDFILTER } else { "" }, label, "tune", "", w, Style::Ghost, true).clicked() {
            app.ui.aside = if app.ui.aside == Aside::Filter { Aside::Preview } else { Aside::Filter };
        }
        let (sep, _) = ui.allocate_exact_size(vec2(1.0, 16.0), Sense::hover());
        ui.painter().rect_filled(sep, CornerRadius::ZERO, t.edge_2);
        let dt = format!("{}/{} done", m.done, m.total);
        let dr = ui.label(text(dt.to_uppercase(), 11.0, 600));
        node(ui, &dr, Role::GenericContainer, true, &dt, None, if primary { parts::DONECOUNT } else { "" });
        let frac = if m.total > 0 { m.done as f32 / m.total as f32 } else { 0.0 };
        let (bar, _) = ui.allocate_exact_size(vec2(80.0, 4.0), Sense::hover());
        ui.painter().rect_filled(bar, CornerRadius::same(2), t.edge_2);
        let fill = Rect::from_min_size(bar.min, vec2(bar.width() * frac, bar.height()));
        ui.painter().rect_filled(fill, CornerRadius::same(2), t.accent_500);
        let br = ui.interact(bar, ui.scope_id().with("progress"), Sense::hover());
        node(ui, &br, Role::GenericContainer, true, "Done progress", None, if primary { parts::PROGRESSBAR } else { "" });
        let pt = format!("{pct}%");
        let pr = ui.label(text(pt.clone(), 11.0, 600));
        node(ui, &pr, Role::GenericContainer, true, &pt, None, if primary { parts::DONEPERCENT } else { "" });
    });
}

/// Builds one list's rows and counts from the view, the query and the show setting.
fn model(app: &App, list: &str, view: &View, query: &str, fields: Fields, collapsed: &HashSet<String>, show_c: bool) -> Model {
    let ts: Vec<Item> = app
        .data
        .tasks_of(list)
        .into_iter()
        .map(|t| Item { id: t.id.clone(), title: t.title.clone(), notes: t.notes.clone().unwrap_or_default(), done: t.done, parent: t.parent_id.clone(), has_att: !t.attachments.is_empty() })
        .collect();
    let q = query.trim().to_lowercase();
    let searching = !q.is_empty();
    let mut preds = vec![];
    if searching {
        preds.push(Pred::Query(q.clone(), fields));
    }
    preds.push(Pred::Show(view.show.clone(), show_c));
    for s in &view.refine {
        if let Some((k, w)) = s.split_once(':') {
            preds.push(Pred::Refine(k.to_string(), w == "with"));
        }
    }
    let vis: Vec<bool> = (0..ts.len()).map(|i| (0..preds.len()).all(|p| kept(&preds, &ts, i, p))).collect();
    let ids: HashSet<&str> = ts.iter().map(|t| t.id.as_str()).collect();
    let roots: Vec<usize> = (0..ts.len()).filter(|&i| vis[i] && ts[i].parent.as_deref().is_none_or(|p| !ids.contains(p))).collect();
    let mut rows = vec![];
    for r in order(&ts, view, roots) {
        walk(&ts, &vis, view, r, 0, searching, collapsed, &mut rows);
    }
    let matches = if searching { ts.iter().filter(|t| hit(t, &q, fields)).count() } else { 0 };
    Model {
        sections: vec![Section { heading: None, rows }],
        visible: vis.iter().filter(|v| **v).count(),
        total: ts.len(),
        done: ts.iter().filter(|t| t.done).count(),
        matches,
    }
}

fn kids_of(ts: &[Item], id: &str) -> Vec<usize> {
    (0..ts.len()).filter(|&k| ts[k].parent.as_deref() == Some(id)).collect()
}

/// A task is kept by a predicate when it passes it, or one of its subtasks does (F05-030, F05-039).
fn kept(preds: &[Pred], ts: &[Item], i: usize, p: usize) -> bool {
    let kids = kids_of(ts, &ts[i].id);
    pass(&preds[p], &ts[i], !kids.is_empty()) || kids.iter().any(|&k| kept(preds, ts, k, p))
}

fn pass(p: &Pred, t: &Item, has_kids: bool) -> bool {
    match p {
        Pred::Query(q, f) => hit(t, q, *f),
        Pred::Show(s, sc) => match s.as_str() {
            "active" => !t.done,
            "done" => t.done,
            _ => *sc || !t.done,
        },
        Pred::Refine(k, want) => {
            let has = match k.as_str() {
                "notes" => !t.notes.trim().is_empty(),
                "attachments" => t.has_att,
                _ => has_kids,
            };
            has == *want
        }
    }
}

fn hit(t: &Item, q: &str, f: Fields) -> bool {
    (f != Fields::Notes && t.title.to_lowercase().contains(q)) || (f != Fields::Title && t.notes.to_lowercase().contains(q))
}

/// The stored order is the default; sort by title or done, and descending reverses it.
fn order(ts: &[Item], view: &View, mut xs: Vec<usize>) -> Vec<usize> {
    xs.sort_by(|&a, &b| {
        let base = match view.sort.as_str() {
            "title" => ts[a].title.to_lowercase().cmp(&ts[b].title.to_lowercase()),
            "done" => ts[a].done.cmp(&ts[b].done),
            _ => std::cmp::Ordering::Equal,
        };
        base.then(a.cmp(&b))
    });
    if view.descending {
        xs.reverse();
    }
    xs
}

#[allow(clippy::too_many_arguments)]
fn walk(ts: &[Item], vis: &[bool], view: &View, i: usize, depth: usize, searching: bool, collapsed: &HashSet<String>, out: &mut Vec<Row>) {
    let kids = kids_of(ts, &ts[i].id);
    let shown = order(ts, view, kids.iter().copied().filter(|&k| vis[k]).collect());
    let expanded = searching || !collapsed.contains(&ts[i].id);
    let t = &ts[i];
    out.push(Row {
        id: t.id.clone(),
        title: t.title.clone(),
        note: t.notes.trim().to_string(),
        done: t.done,
        depth,
        has_kids: !kids.is_empty(),
        expanded,
        kids_visible: shown.len(),
        kids_done: shown.iter().filter(|&&k| ts[k].done).count(),
    });
    if expanded {
        for k in shown {
            walk(ts, vis, view, k, depth + 1, searching, collapsed, out);
        }
    }
}

fn targets(app: &App) -> Vec<String> {
    app.ui.panes.iter().flatten().filter(|id| app.data.list(id).is_some()).cloned().collect()
}

fn patch(app: &mut App, f: impl Fn(&mut View)) {
    for id in targets(app) {
        f(app.ui.views.entry(id).or_default());
    }
}

fn reset_all(app: &mut App) {
    for id in targets(app) {
        app.ui.views.remove(&id);
    }
}

fn view_active(v: &View) -> bool {
    v.sort != "manual" || v.descending || v.show != "all" || !v.refine.is_empty() || v.date != "any"
}

fn toggle_split(app: &mut App) {
    if app.ui.split_on() {
        app.ui.split(false);
        return;
    }
    let Some(project) = app.ui.project.clone() else { return };
    let ids: Vec<String> = app.data.lists_of(&project).iter().map(|l| l.id.clone()).collect();
    let cur = app.ui.panes[0].clone();
    let at = ids.iter().position(|i| Some(i) == cur.as_ref());
    let next = at.and_then(|a| ids.get(a + 1)).or_else(|| ids.iter().find(|i| Some(*i) != cur.as_ref())).cloned();
    if let Some(n) = next {
        app.ui.split(true);
        app.ui.open_list(Some(n), Some(1));
    }
}

fn sort_label(s: &str) -> &'static str {
    match s {
        "created" => "Date created",
        "updated" => "Date updated",
        "title" => "Alphabetical",
        "done" => "Done last",
        _ => "Manual",
    }
}

fn range_label(s: &str) -> &'static str {
    match s {
        "today" => "Last 24 hours",
        "week" => "Last 7 days",
        "month" => "Last 30 days",
        "custom" => "Custom…",
        _ => "Any time",
    }
}

fn group_part(heading: &str) -> &'static str {
    match heading {
        "Errands" => parts::GROUPHEADINGERRANDS,
        "Reading" => parts::GROUPHEADINGREADING,
        _ => "",
    }
}

fn row_part(title: &str) -> &'static str {
    ROW_PARTS.iter().find(|(t, _)| *t == title).map(|(_, p)| *p).unwrap_or("")
}

fn tri_parts(key: &str) -> [&'static str; 3] {
    match key {
        "notes" => [parts::NOTESANY, parts::NOTESWITH, parts::NOTESWITHOUT],
        "attachments" => [parts::ATTACHMENTSANY, parts::ATTACHMENTSWITH, parts::ATTACHMENTSWITHOUT],
        _ => [parts::SUBTASKSANY, parts::SUBTASKSWITH, parts::SUBTASKSWITHOUT],
    }
}

/// A refine row's value: "any", "with" or "without", stored in the view as "key:with".
fn tri_of(v: &View, key: &str) -> &'static str {
    for s in &v.refine {
        if let Some((k, w)) = s.split_once(':') {
            if k == key {
                return if w == "with" { "with" } else { "without" };
            }
        }
    }
    "any"
}

fn set_tri(v: &mut View, key: &str, value: &str) {
    let prefix = format!("{key}:");
    v.refine.retain(|s| !s.starts_with(&prefix));
    if value != "any" {
        v.refine.push(format!("{key}:{value}"));
    }
}

fn text_w(ui: &egui::Ui, s: &str, size: f32, w: u16) -> f32 {
    ui.painter().layout_no_wrap(s.to_owned(), FontId::new(size, weight(w)), Color32::WHITE).size().x
}

/// Draws a Material Symbols glyph centred on a point.
fn glyph(ui: &egui::Ui, at: Pos2, name: &str, px: f32, color: Color32) {
    ui.painter().text(at, Align2::CENTER_CENTER, icon(name, px).text(), FontId::new(px, FontFamily::Name("Icons".into())), color);
}

/// Marks a response as a node with a role, a label, an optional selected state, and an optional part id ("" for none).
fn node(ui: &egui::Ui, r: &Response, role: Role, enabled: bool, label: &str, on: Option<bool>, id: &str) {
    r.widget_info(|| match on {
        Some(s) => WidgetInfo::selected(role, enabled, s, label),
        None => WidgetInfo::labeled(role, enabled, label),
    });
    if let (Some(s), Role::Button) = (on, role) {
        ui.ctx().accesskit_node_builder(r.id, |n| n.set_selected(s));
    }
    if !id.is_empty() {
        part(ui, r, id);
    }
}

fn icon_button(ui: &mut egui::Ui, look: Look, part_id: &str, name: &str, glyph_name: &str, on: Option<bool>, enabled: bool) -> Response {
    let t = look.t;
    let (rect, _) = ui.allocate_exact_size(vec2(24.0, 24.0), Sense::hover());
    let r = ui.interact(rect, ui.scope_id().with((part_id, name)), if enabled { Sense::click() } else { Sense::hover() });
    let hot = enabled && r.hovered();
    if on == Some(true) {
        ui.painter().rect_filled(rect, CornerRadius::same(7), t.wash_2);
    } else if hot {
        ui.painter().rect_filled(rect, CornerRadius::same(7), t.wash_1);
    }
    let col = if !enabled {
        t.content_3
    } else if hot || on == Some(true) {
        t.content_1
    } else {
        t.content_2
    };
    glyph(ui, rect.center(), glyph_name, 16.0, col);
    if r.has_focus() && enabled {
        ui.painter().rect_stroke(rect, CornerRadius::same(7), Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
    }
    node(ui, &r, Role::Button, enabled, name, on, part_id);
    r
}

#[allow(clippy::too_many_arguments)]
fn pill(ui: &mut egui::Ui, look: Look, part_id: &str, label: &str, lead: &str, trail: &str, width: f32, style: Style, enabled: bool) -> Response {
    let t = look.t;
    let (rect, _) = ui.allocate_exact_size(vec2(width, 27.0), Sense::hover());
    let r = ui.interact(rect, ui.scope_id().with((part_id, label)), if enabled { Sense::click() } else { Sense::hover() });
    let hot = enabled && r.hovered();
    let (fill, fg) = match style {
        Style::Primary => (if hot { t.accent_600 } else { t.accent_500 }, t.on_accent),
        Style::Neutral => (if hot { t.wash_2 } else { t.surface_0 }, t.content_1),
        Style::Ghost => (if hot { t.wash_1 } else { Color32::TRANSPARENT }, t.content_2),
    };
    let a = if enabled { 1.0 } else { 0.45 };
    ui.painter().rect_filled(rect, CornerRadius::same(7), fill.gamma_multiply(a));
    let mut x = rect.left() + 10.0;
    if !lead.is_empty() {
        glyph(ui, pos2(x + 8.0, rect.center().y), lead, 16.0, fg.gamma_multiply(a));
        x += 22.0;
    }
    ui.painter().text(pos2(x, rect.center().y), Align2::LEFT_CENTER, label, FontId::new(13.0, weight(500)), fg.gamma_multiply(a));
    if !trail.is_empty() {
        glyph(ui, pos2(rect.right() - 10.0, rect.center().y), trail, 16.0, fg.gamma_multiply(a));
    }
    if r.has_focus() && enabled {
        ui.painter().rect_stroke(rect, CornerRadius::same(7), Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
    }
    node(ui, &r, Role::Button, enabled, label, None, part_id);
    r
}

fn tab(ui: &mut egui::Ui, look: Look, part_id: &str, label: &str, width: f32, on: bool) -> Response {
    let t = look.t;
    let (rect, _) = ui.allocate_exact_size(vec2(width, 26.0), Sense::hover());
    let r = ui.interact(rect, ui.scope_id().with((part_id, label)), Sense::click());
    let fill = if on {
        t.wash_2
    } else if r.hovered() {
        t.wash_1
    } else {
        Color32::TRANSPARENT
    };
    ui.painter().rect_filled(rect, CornerRadius::same(7), fill);
    let fg = if on { t.content_1 } else { t.content_2 };
    ui.painter().text(rect.center(), Align2::CENTER_CENTER, label, FontId::new(13.0, weight(if on { 600 } else { 500 })), fg);
    node(ui, &r, Role::Button, true, label, Some(on), part_id);
    r
}

/// A track of choices, one of which is on. Returns the index clicked.
fn segments(ui: &mut egui::Ui, look: Look, opts: &[(&str, &str, bool)]) -> Option<usize> {
    let t = look.t;
    let avail = ui.available_width();
    let (grp, _) = ui.allocate_exact_size(vec2(avail, 24.0), Sense::hover());
    ui.painter().rect_filled(grp, CornerRadius::same(7), t.wash_2);
    let w = grp.width() / opts.len() as f32;
    let mut clicked = None;
    for (i, (part_id, label, on)) in opts.iter().enumerate() {
        let r = Rect::from_min_size(grp.min + vec2(i as f32 * w, 0.0), vec2(w, 24.0)).shrink(1.5);
        let resp = ui.interact(r, ui.scope_id().with((*part_id, *label)), Sense::click());
        if *on {
            ui.painter().rect_filled(r, CornerRadius::same(6), t.accent_500);
        } else if resp.hovered() {
            ui.painter().rect_filled(r, CornerRadius::same(6), t.wash_1);
        }
        let fg = if *on { t.on_accent } else { t.content_2 };
        ui.painter().text(r.center(), Align2::CENTER_CENTER, *label, FontId::new(12.0, weight(if *on { 600 } else { 500 })), fg);
        node(ui, &resp, Role::Button, true, label, Some(*on), part_id);
        if resp.clicked() {
            clicked = Some(i);
        }
    }
    clicked
}

fn menu_item(ui: &mut egui::Ui, look: Look, part_id: &str, label: &str) -> bool {
    let t = look.t;
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 26.0), Sense::hover());
    let r = ui.interact(rect, ui.scope_id().with((part_id, label)), Sense::click());
    if r.hovered() {
        ui.painter().rect_filled(rect, CornerRadius::same(6), t.wash_1);
    }
    ui.painter().text(rect.left_center() + vec2(10.0, 0.0), Align2::LEFT_CENTER, label, FontId::new(13.0, weight(500)), t.content_1);
    node(ui, &r, Role::Button, true, label, None, part_id);
    r.clicked()
}

fn popup(ui: &egui::Ui, key: &str, at: Pos2, add: impl FnOnce(&mut egui::Ui)) {
    egui::Area::new(egui::Id::unique(key)).order(egui::Order::Foreground).fixed_pos(at).show(ui.ctx(), |ui| {
        egui::Frame::popup(ui.style()).show(ui, |ui| {
            ui.set_width(180.0);
            add(ui);
        });
    });
}

/// A section title: the uppercase heading, a dot while the group is non-default, a rule, and the clear control.
fn section_title(ui: &mut egui::Ui, look: Look, label: &str, part_id: &str, on: bool, clear_part: &str, clear_name: &str) -> bool {
    let t = look.t;
    let galley = ui.painter().layout_no_wrap(label.to_uppercase(), FontId::new(12.0, weight(600)), t.content_2);
    let mut cleared = false;
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(galley.size(), Sense::hover());
        ui.painter().galley(rect.min, galley.clone(), t.content_2);
        let h = ui.interact(rect, ui.scope_id().with(part_id), Sense::hover());
        node(ui, &h, Role::Heading, true, label, None, part_id);
        if on {
            let (d, _) = ui.allocate_exact_size(vec2(6.0, 6.0), Sense::hover());
            ui.painter().circle_filled(d.center(), 3.0, t.content_1);
        }
        let w = (ui.available_width() - if on { 28.0 } else { 0.0 }).max(4.0);
        let (line, _) = ui.allocate_exact_size(vec2(w, 1.0), Sense::hover());
        ui.painter().rect_filled(line, CornerRadius::ZERO, t.edge_2);
        if on {
            cleared = icon_button(ui, look, clear_part, clear_name, "restart_alt", None, true).clicked();
        }
    });
    cleared
}

fn center_text(ui: &mut egui::Ui, s: &str, size: f32, w: u16, color: Color32) -> Response {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), size * 2.4), Sense::hover());
    ui.painter().text(rect.center(), Align2::CENTER_CENTER, s, FontId::new(size, weight(w)), color);
    ui.interact(rect, ui.scope_id().with(s), Sense::hover())
}

/// The test's mount: the app on the props' data set, put where the props say the screen starts.
struct Mount {
    app: App,
    screen: FilterPanel,
}

impl taskscape_ui::Mounted for Mount {
    fn frame(&mut self, ui: &mut egui::Ui, look: Look, emit: &mut dyn FnMut(&'static str, serde_json::Value)) {
        for e in self.screen.show(&mut self.app, ui, look) {
            let (n, f) = e.to_json();
            emit(n, f);
        }
    }
}

pub fn mount(mut app: App, props: serde_json::Value) -> Result<Box<dyn taskscape_ui::Mounted>, String> {
    let p: FilterpanelProps = serde_json::from_value(props).map_err(|e| e.to_string())?;
    if !p.showCompleted {
        let _ = app.data.set_setting("show_completed", "false");
    }
    app.open_named(Some("Home"), Some("Errands"));
    app.ui.aside = if p.panel == "closed" { Aside::Preview } else { Aside::Filter };
    Ok(Box::new(Mount { app, screen: FilterPanel::new() }))
}
