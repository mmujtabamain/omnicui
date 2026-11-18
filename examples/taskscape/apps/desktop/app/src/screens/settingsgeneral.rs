//! contract: omnicui/contracts/settingsgeneral.contract.yaml (id: app.settingsgeneral). The Settings page on the desktop: its heading and Discard all, the section list with the search, and the General pane (Appearance and Tasks) with its group discards.
use crate::data::Settings;
use crate::App;
use std::sync::Arc;
use taskscape_ui::omnicui::settingsgeneral::{parts, SettingsgeneralEvent};
use taskscape_ui::{icon, part, rem, weight, Look, Mounted, Tokens};
use egui::{pos2, vec2, Align, Align2, Color32, CornerRadius, FontFamily, FontId, Galley, Key, Layout, Modifiers, Rangef, Rect, Response, Role, RichText, Sense, Shape, Stroke, StrokeKind, TextEdit, UiBuilder, WidgetInfo};

/// Horizontal and vertical padding of a setting row (the web's px-2 and py-1 at 14px rem), and the gap to its control.
const PAD_X: f32 = 8.0;
const PAD_Y: f32 = 7.0;
const GAP: f32 = 10.5;

/// A setting the page writes. The value is the stored string ("true" and "false" for the switch).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Field {
    Theme,
    Motion,
    Show,
}

const APPEARANCE: &[Field] = &[Field::Theme, Field::Motion];
const TASKS: &[Field] = &[Field::Show];
const ALL: &[Field] = &[Field::Theme, Field::Motion, Field::Show];

/// The sections, in tab order: General, Shortcuts, About.
const PANES: [&str; 3] = ["general", "shortcuts", "about"];
const TAB_PARTS: [&str; 3] = [parts::GENERALTAB, parts::SHORTCUTSTAB, parts::ABOUTTAB];
const TAB_NAMES: [&str; 3] = ["General", "Shortcuts", "About"];
const TAB_ICONS: [&str; 3] = ["tune", "keyboard", "info"];
/// The desktop Settings window's sections (decision 19): Capture joins them, after General.
const WINDOW_PANES: [&str; 4] = ["general", "capture", "shortcuts", "about"];
const WINDOW_TAB_PARTS: [&str; 4] = [parts::GENERALTAB, CAPTURE_TAB, parts::SHORTCUTSTAB, parts::ABOUTTAB];
const WINDOW_TAB_NAMES: [&str; 4] = ["General", "Capture", "Shortcuts", "About"];
// The reference's own glyphs (its recording: tune, screenshot_monitor, keyboard, info).
const WINDOW_TAB_ICONS: [&str; 4] = ["tune", "screenshot_monitor", "keyboard", "info"];
/// Parts the page has only in the window (its contract has no Capture section and no row containers).
pub const CAPTURE_TAB: &str = "app.settingsgeneral.captureTab";
pub const SIDEBAR: &str = "app.settingsgeneral.sidebar";
pub const THEME_ROW: &str = "app.settingsgeneral.themeRow";
pub const REDUCED_ROW: &str = "app.settingsgeneral.reducedRow";
pub const SHOW_ROW: &str = "app.settingsgeneral.showCompletedRow";

/// One searchable row of the General pane: its title, description and keywords make its text.
struct Row {
    id: &'static str,
    group: &'static str,
    title: &'static str,
    description: &'static str,
    keywords: &'static str,
}

const ROWS: [Row; 3] = [
    Row {
        id: "theme",
        group: "appearance",
        title: "Theme",
        description: "Follow the system appearance, or pin one.",
        keywords: "dark light color colour mode",
    },
    Row {
        id: "reducedMotion",
        group: "appearance",
        title: "Reduced motion",
        description: "Cuts animations and transitions to the minimum.",
        keywords: "animation accessibility reduce transition",
    },
    Row {
        id: "showCompleted",
        group: "tasks",
        title: "Show completed tasks",
        description: "Keep finished tasks in the list instead of hiding them.",
        keywords: "done finished hidden",
    },
];

/// One segment of a segmented choice: its stored value, its label, an optional glyph and its part.
struct Seg {
    value: &'static str,
    label: &'static str,
    icon: &'static str,
    part: &'static str,
}

const THEME_SEGS: [Seg; 3] = [
    Seg { value: "system", label: "System", icon: "contrast", part: parts::THEMESYSTEM },
    Seg { value: "light", label: "Light", icon: "light_mode", part: parts::THEMELIGHT },
    Seg { value: "dark", label: "Dark", icon: "dark_mode", part: parts::THEMEDARK },
];

const MOTION_SEGS: [Seg; 3] = [
    Seg { value: "system", label: "System", icon: "", part: parts::REDUCEDMOTIONSYSTEM },
    Seg { value: "on", label: "On", icon: "", part: parts::REDUCEDMOTIONON },
    Seg { value: "off", label: "Off", icon: "", part: parts::REDUCEDMOTIONOFF },
];

/// The General pane's settings page: the Settings heading, the section list with the search, and the
/// General pane. The screen keeps the query and the open-time settings; the pane is `app.settings_window`.
pub struct SettingsGeneral {
    /// The settings as they were when this page opened: what discard writes back.
    open: Option<Settings>,
    /// The search query, as typed.
    query: String,
    /// The search is limited to the General pane (a click on a tab that had matches).
    scoped: bool,
    /// The content the fade last started for, and when it started (seconds).
    fade_key: String,
    fade_from: f64,
    /// Drawn as the desktop Settings window: four sections, and the window's own keys (Escape and the
    /// close chord close it). The window draws the panes other than General into `content`.
    pub window: bool,
    /// The pane's rectangle in the last frame, for the window to draw another section's page into.
    pub content: Option<Rect>,
    /// Room kept in the header for the window buttons: at the left for the OS's (macOS), at the right for
    /// the app's own (Windows, Linux), in a real window.
    pub lead: f32,
    pub trail: f32,
    /// Each setting written in the last frame, as (key, value).
    pub writes: Vec<(&'static str, String)>,
    /// Set when the window asked to close in the last frame: "escape" or "shortcut".
    pub closed: Option<&'static str>,
}

impl Default for SettingsGeneral {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingsGeneral {
    pub fn new() -> Self {
        Self {
            open: None,
            query: String::new(),
            scoped: false,
            fade_key: String::new(),
            fade_from: 0.0,
            window: false,
            lead: 0.0,
            trail: 0.0,
            content: None,
            writes: vec![],
            closed: None,
        }
    }

    /// One frame: draws the page on the app and returns the events it raised (the page declares none).
    pub fn show(&mut self, app: &mut App, ui: &mut egui::Ui, look: Look) -> Vec<SettingsgeneralEvent> {
        ui.scope(|ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
            self.draw(app, ui, look);
        });
        vec![]
    }

    fn draw(&mut self, app: &mut App, ui: &mut egui::Ui, look: Look) {
        let t = look.t;
        let p = ui.painter().clone();
        let now = ui.input(|i| i.time);
        let current = app.data.settings.clone();
        let open = self.open.get_or_insert_with(|| current.clone()).clone();
        let w = ui.available_width();
        let mut actions: Vec<(Field, String)> = Vec::new();

        // Header: the heading, and Discard all while any setting differs from when the page opened.
        let (header, _) = ui.allocate_exact_size(vec2(w, rem(3.286)), Sense::hover());
        p.rect_filled(header, 0.0, t.surface_1);
        p.hline(Rangef::new(header.left(), header.right()), header.bottom() - 0.5, Stroke::new(1.0, t.edge_1));
        let title = lay(ui, "Settings", 15.0, 600, t.content_1, f32::INFINITY);
        let ts = title.size();
        let at = pos2(header.left() + rem(1.286) + self.lead, header.center().y - ts.y / 2.0);
        node_text(ui, Rect::from_min_size(at, ts), title, t.content_1, parts::HEADING, "Settings", Role::Heading);
        if ALL.iter().any(|f| differs(&current, &open, *f)) {
            let b = rem(2.0);
            let r = Rect::from_min_size(pos2(header.right() - rem(1.286) - self.trail - b, header.center().y - b / 2.0), vec2(b, b));
            if icon_button(ui, &t, r, parts::DISCARDALL, "Discard all changes", "undo", 19.0, t.content_1) {
                discard(&current, &open, ALL, &mut actions);
            }
        }

        // Body: the section list on the left, the pane on the right.
        let body_h = if ui.available_height().is_finite() { ui.available_height().max(0.0) } else { rem(40.0) };
        let (body, _) = ui.allocate_exact_size(vec2(w, body_h), Sense::hover());
        p.rect_filled(body, 0.0, t.surface_2);
        let nav = Rect::from_min_size(body.min, vec2(rem(11.0), body.height()));
        p.rect_filled(nav, 0.0, t.surface_1);
        p.vline(nav.right() - 0.5, Rangef::new(nav.top(), nav.bottom()), Stroke::new(1.0, t.edge_1));
        let content = Rect::from_min_max(pos2(nav.right(), body.top()), body.max);

        // The search: a well with the field and, while the query is not empty, a clear button.
        let side = rem(0.571);
        let inner_l = nav.left() + side;
        let inner_w = nav.width() - 2.0 * side;
        // The nav's own layout: the search well is allocated in it, and the field is built in a child of the well (as the input widget does).
        let mut nv = ui.new_child(
            UiBuilder::new()
                .max_rect(Rect::from_min_max(pos2(inner_l, nav.top() + side), pos2(nav.right() - side, nav.bottom())))
                .layout(Layout::top_down(Align::Min)),
        );
        let (search_rect, _) = nv.allocate_exact_size(vec2(inner_w, rem(1.75)), Sense::hover());
        // A fixed id: the field keeps its focus and its typed text from frame to frame.
        let search_id = egui::Id::unique("app.settingsgeneral.search");
        let focused = ui.memory(|m| m.has_focus(search_id));
        self.writes.clear();
        self.closed = None;
        if self.window {
            // The window's keys: the search chord focuses the field with its text selected; the close chord closes.
            let (find, close) = ui.input(|i| {
                let only = (i.modifiers.command || i.modifiers.ctrl) && !i.modifiers.shift && !i.modifiers.alt;
                (only && i.key_pressed(Key::F), only && i.key_pressed(Key::W))
            });
            if close {
                self.closed = Some("shortcut");
            }
            if find {
                ui.memory_mut(|m| m.request_focus(search_id));
                let mut state = egui::text_edit::TextEditState::load(ui.ctx(), search_id).unwrap_or_default();
                let all = egui::text::CCursorRange::two(egui::text::CCursor::new(0), egui::text::CCursor::new(self.query.chars().count()));
                state.cursor.set_char_range(Some(all));
                state.store(ui.ctx(), search_id);
            }
            // Escape with an empty search (and no menu or field taking it) closes the window.
            if self.query.is_empty() && ui.input(|i| i.key_pressed(Key::Escape)) {
                self.closed = Some("escape");
            }
        }
        // Escape clears a non-empty search; with an empty search it is left for the window (it closes it).
        if !self.query.is_empty() && ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
            self.query.clear();
            self.scoped = false;
        }
        let has_q = !self.query.is_empty();
        let box_px = rem(1.5);
        // The reference's InputWell: surface-0 (#12365).
        p.rect_filled(search_rect, CornerRadius::same(5), t.surface_0);
        // The reference's InputWell shows focus as a 1 px ring (`focus-within:ring-1`) (#12488).
        if focused {
            p.rect_stroke(search_rect, CornerRadius::same(5), Stroke::new(1.0, t.focus_1), StrokeKind::Outside);
        }
        glyph(&p, "search", pos2(search_rect.left() + rem(0.643) + 7.0, search_rect.center().y), 14.0, t.content_3);
        let edit_left = search_rect.left() + rem(0.643) + 14.0 + 6.0;
        let edit_right = search_rect.right() - 3.5 - if has_q { box_px } else { 0.0 };
        let edit_rect = Rect::from_min_max(pos2(edit_left, search_rect.top()), pos2(edit_right, search_rect.bottom()));
        let mut edit_ui = nv.new_child(UiBuilder::new().max_rect(edit_rect).layout(Layout::left_to_right(Align::Center)));
        let field = TextEdit::singleline(&mut self.query)
            .id(search_id)
            .frame(egui::Frame::NONE)
            .margin(egui::Margin::ZERO)
            .font(FontId::new(12.0, weight(400)))
            .text_color(t.content_1)
            // The placeholder at the field's own 12 px: a hint without a font is drawn at egui's body size (#12488).
            .hint_text(RichText::new("Search").font(FontId::new(12.0, weight(400))).color(t.content_3))
            .desired_width(edit_rect.width())
            .show(&mut edit_ui)
            .response;
        if field.changed() {
            self.scoped = false;
        }
        let value = self.query.clone();
        ui.ctx().accesskit_node_builder(field.id, |n| n.set_value(value));
        field.widget_info(|| WidgetInfo::labeled(Role::TextInput, true, "Search settings"));
        part(ui, &field, parts::SEARCH);
        if has_q {
            let r = Rect::from_center_size(pos2(search_rect.right() - 3.5 - box_px / 2.0, search_rect.center().y), vec2(box_px, box_px));
            if icon_button(ui, &t, r, parts::CLEARSEARCH, "Clear search", "close", 14.0, t.content_3) {
                self.query.clear();
                self.scoped = false;
                ui.memory_mut(|m| m.request_focus(search_id));
            }
        }

        // What the search matches now. The General tab's count and the pane follow it.
        let tokens = tokens_of(&self.query);
        let searching = !tokens.is_empty();
        let general_matches = if searching { ROWS.iter().filter(|r| row_matches(r, &tokens)).count() } else { 0 };
        let n_tabs = if self.window { 4 } else { 3 };
        let (panes, tab_parts, tab_names, tab_icons): (&[&str], &[&str], &[&str], &[&str]) = if self.window {
            (&WINDOW_PANES, &WINDOW_TAB_PARTS, &WINDOW_TAB_NAMES, &WINDOW_TAB_ICONS)
        } else {
            (&PANES, &TAB_PARTS, &TAB_NAMES, &TAB_ICONS)
        };
        let mut counts = vec![0; n_tabs];
        counts[0] = general_matches;
        let any_match = general_matches > 0;

        // The section tabs: a click or Enter or Space selects; the arrows move focus and select.
        let tab_h = rem(2.143);
        let tops = search_rect.bottom() + rem(0.5);
        let tab_rects: Vec<Rect> = (0..n_tabs)
            .map(|i| Rect::from_min_size(pos2(inner_l, tops + i as f32 * (tab_h + 2.0)), vec2(inner_w, tab_h)))
            .collect();
        let tab_resps: Vec<Response> = (0..n_tabs).map(|i| ui.interact(tab_rects[i], ui.scope_id().with(tab_parts[i]), Sense::click())).collect();
        // The section list is one node: a generic container named "Settings sections", over the three tabs.
        let list_rect = Rect::from_min_max(tab_rects[0].min, tab_rects[n_tabs - 1].max);
        let list = ui.interact(list_rect, ui.scope_id().with(parts::SECTIONNAV), Sense::hover());
        list.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, "Settings sections"));
        part(ui, &list, parts::SECTIONNAV);
        // The focused tab keeps the arrows, so that egui's own focus moves do not take them from the list.
        for r in tab_resps.iter().filter(|r| r.has_focus()) {
            ui.memory_mut(|m| {
                m.set_focus_lock_filter(r.id, egui::EventFilter { horizontal_arrows: true, vertical_arrows: true, ..Default::default() })
            });
        }
        let (down, up) = ui.input(|i| (i.key_pressed(Key::ArrowDown), i.key_pressed(Key::ArrowUp)));
        let mut moved: Option<usize> = None;
        if let Some(c) = tab_resps.iter().position(|r| r.has_focus()) {
            if down {
                moved = Some((c + 1) % n_tabs);
            } else if up {
                moved = Some((c + n_tabs - 1) % n_tabs);
            }
        }
        if let Some(n) = moved {
            tab_resps[n].request_focus();
            choose(app, &mut self.query, &mut self.scoped, searching, &counts, panes, n);
            ui.ctx().request_repaint();
        } else {
            for i in 0..n_tabs {
                if pressed(ui, &tab_resps[i]) {
                    choose(app, &mut self.query, &mut self.scoped, searching, &counts, panes, i);
                    // During a search the field keeps the keyboard, so typing carries on in it; otherwise the tab takes it.
                    if searching {
                        ui.memory_mut(|m| m.request_focus(search_id));
                    } else {
                        tab_resps[i].request_focus();
                    }
                }
            }
        }
        let pane = panes.iter().position(|p| Some(*p) == app.settings_window.as_deref()).unwrap_or(0);

        for i in 0..n_tabs {
            let r = tab_rects[i];
            let resp = &tab_resps[i];
            let selected = pane == i;
            let dim = if searching && counts[i] == 0 { 0.4 } else { 1.0 };
            if selected && !(searching && !self.scoped) {
                p.rect_filled(r, CornerRadius::same(5), t.accent_500.gamma_multiply(0.1));
            }
            let base = if selected || resp.hovered() { t.content_1 } else { t.content_2 };
            let fg = base.gamma_multiply(dim);
            glyph(&p, tab_icons[i], pos2(r.left() + rem(0.643) + 8.0, r.center().y), 16.0, t.content_3.gamma_multiply(dim));
            let label = lay(ui, tab_names[i], 13.0, 400, fg, f32::INFINITY);
            let ly = r.center().y - label.size().y / 2.0;
            p.galley(pos2(r.left() + rem(0.643) + 16.0 + rem(0.5), ly), label, fg);
            if i == 0 && searching && general_matches > 0 {
                let count = general_matches.to_string();
                // The reference's recessed badge (as recorded): 18 px tall, 10.5 px semibold, an edge-2 border on
                // surface-0, the field's corners.
                let g = lay(ui, &count, 10.5, 600, t.content_2, f32::INFINITY);
                let gs = g.size();
                let bw = (gs.x + 8.0).max(16.0);
                let bh = 18.0;
                let br = Rect::from_min_size(pos2(r.right() - rem(0.643) - bw, r.center().y - bh / 2.0), vec2(bw, bh));
                p.rect(br, CornerRadius::same(4), t.surface_0, Stroke::new(1.0, t.edge_2), StrokeKind::Inside);
                node_text(ui, Rect::from_center_size(br.center(), gs), g, t.content_2, parts::GENERALMATCHES, &count, Role::GenericContainer);
            }
            resp.widget_info(|| WidgetInfo::selected(Role::Button, true, selected, tab_names[i]));
            ui.ctx().accesskit_node_builder(resp.id, |n| n.set_selected(selected));
            part(ui, resp, tab_parts[i]);
            if resp.has_focus() {
                p.rect_stroke(r, CornerRadius::same(5), Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
            }
        }

        // The pane: its content fades in on each change of pane or search (0.28 s, as the web's pane frame).
        let key = if searching { "search".to_owned() } else { panes[pane].to_owned() };
        if self.fade_key != key {
            self.fade_key = key;
            self.fade_from = now;
        }
        let fade = (((now - self.fade_from) / 0.28) as f32).clamp(0.0, 1.0);
        if fade < 1.0 {
            ui.ctx().request_repaint();
        }
        if self.window {
            let side = ui.interact(nav, ui.scope_id().with(SIDEBAR), Sense::hover());
            side.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, "Settings"));
            part(ui, &side, SIDEBAR);
        }
        self.content = (!searching && pane != 0).then_some(content);
        let panel = ui.interact(content, ui.scope_id().with(parts::CONTENTPANEL), Sense::hover());
        panel.widget_info(|| WidgetInfo::new(Role::GenericContainer));
        part(ui, &panel, parts::CONTENTPANEL);
        let pad = rem(0.857);
        let area = Rect::from_min_max(
            pos2(content.left() + rem(1.286), content.top() + pad),
            pos2(content.right() - rem(1.286), content.bottom() - pad),
        );
        let mut cu = ui.new_child(UiBuilder::new().max_rect(area).layout(Layout::top_down(Align::Min)));
        cu.multiply_opacity(fade);
        let scoped = self.scoped;
        let query = self.query.trim().to_owned();
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(&mut cu, |ui| {
            if searching && !any_match {
                empty_state(ui, &t, &query);
            } else if searching || pane == 0 {
                general_page(ui, &t, &tokens, searching, scoped && !self.window, &current, &open, &mut actions);
            }
        });

        // Every change is written at once; the store reloads, and the next frame shows it.
        if !actions.is_empty() {
            ui.ctx().request_repaint();
        }
        for (f, v) in actions {
            if let Err(e) = app.data.set_setting(key_of(f), &v) {
                app.refused(e);
            } else {
                self.writes.push((key_of(f), v));
            }
        }
    }
}

/// The General pane's content: the Appearance and Tasks groups, filtered by the search.
#[allow(clippy::too_many_arguments)]
fn general_page(
    ui: &mut egui::Ui,
    t: &Tokens,
    tokens: &[String],
    searching: bool,
    scoped: bool,
    cur: &Settings,
    open: &Settings,
    actions: &mut Vec<(Field, String)>,
) {
    // A search across the sections names the group with its section; a search scoped to the pane does not.
    let heading = |name: &str| if searching && !scoped { format!("General · {name}") } else { name.to_owned() };
    let shown = |id: &str| shown_row(id, tokens, searching);
    if group_shown(tokens, searching, "appearance") {
        let changed = differs(cur, open, Field::Theme) || differs(cur, open, Field::Motion);
        let pressed = group(
            ui,
            t,
            &heading("Appearance"),
            parts::APPEARANCEHEADER,
            parts::APPEARANCEGROUP,
            parts::APPEARANCEDISCARD,
            changed,
            actions,
            |ui, actions| {
                let mut first = true;
                if shown("theme") {
                    theme_row(ui, t, &cur.theme, actions);
                    first = false;
                }
                if shown("reducedMotion") {
                    if !first {
                        divider(ui, t);
                    }
                    motion_row(ui, t, &cur.reduced_motion, actions);
                }
            },
        );
        if pressed {
            discard(cur, open, APPEARANCE, actions);
        }
    }
    if group_shown(tokens, searching, "tasks") {
        let changed = differs(cur, open, Field::Show);
        let pressed = group(
            ui,
            t,
            &heading("Tasks"),
            parts::TASKSHEADER,
            parts::TASKSGROUP,
            parts::TASKSDISCARD,
            changed,
            actions,
            |ui, actions| {
                if shown("showCompleted") {
                    show_row(ui, t, cur.show_completed, actions);
                }
            },
        );
        if pressed {
            discard(cur, open, TASKS, actions);
        }
    }
}

/// Shown when a search matches nothing: the title and the quoted, trimmed query.
fn empty_state(ui: &mut egui::Ui, t: &Tokens, query: &str) {
    let w = ui.available_width();
    spacer(ui, rem(4.5));
    let (ir, _) = ui.allocate_exact_size(vec2(w, rem(2.143)), Sense::hover());
    let p = ui.painter().clone();
    glyph(&p, "search", ir.center(), 30.0, t.content_3);
    spacer(ui, rem(0.571));
    let title = lay(ui, "No settings match", 15.0, 500, t.content_2, w);
    let ts = title.size();
    let (tr, _) = ui.allocate_exact_size(vec2(w, ts.y), Sense::hover());
    node_text(ui, Rect::from_min_size(pos2(tr.center().x - ts.x / 2.0, tr.top()), ts), title, t.content_2, parts::EMPTYTITLE, "No settings match", Role::GenericContainer);
    spacer(ui, rem(0.286));
    let subtitle = format!("Nothing here for “{query}”.");
    let sub = lay(ui, &subtitle, 11.0, 400, t.content_3, w);
    let ss = sub.size();
    let (sr, _) = ui.allocate_exact_size(vec2(w, ss.y), Sense::hover());
    node_text(ui, Rect::from_min_size(pos2(sr.center().x - ss.x / 2.0, sr.top()), ss), sub, t.content_3, parts::EMPTYSUBTITLE, &subtitle, Role::GenericContainer);
}

/// A group of rows: its eyebrow label (with a dot while changed, the hairline and the group's discard),
/// then the card. Returns whether its discard was pressed.
#[allow(clippy::too_many_arguments)]
fn group(
    ui: &mut egui::Ui,
    t: &Tokens,
    label: &str,
    label_part: &'static str,
    group_part: &'static str,
    discard_part: &'static str,
    changed: bool,
    actions: &mut Vec<(Field, String)>,
    rows: impl FnOnce(&mut egui::Ui, &mut Vec<(Field, String)>),
) -> bool {
    let w = ui.available_width();
    let p = ui.painter().clone();
    let mut pressed = false;

    let lh = if changed { rem(1.5) } else { rem(1.143) };
    let (row, _) = ui.allocate_exact_size(vec2(w, lh), Sense::hover());
    let caps = taskscape_ui::eyebrow(ui.painter(), label, t.content_3);
    let cs = caps.size();
    let cy = row.center().y;
    let mut x = row.left();
    node_text(ui, Rect::from_min_size(pos2(x, cy - cs.y / 2.0), cs), caps, t.content_3, label_part, label, Role::GenericContainer);
    x += cs.x + rem(0.5);
    if changed {
        p.circle_filled(pos2(x + 3.0, cy), 3.0, t.content_1);
        x += 6.0 + rem(0.5);
    }
    let end = if changed { row.right() - rem(1.5) - rem(0.5) } else { row.right() };
    if end > x {
        p.hline(Rangef::new(x, end), cy, Stroke::new(1.0, t.edge_2));
    }
    if changed {
        let r = Rect::from_min_size(pos2(row.right() - rem(1.5), cy - rem(1.5) / 2.0), vec2(rem(1.5), rem(1.5)));
        pressed = icon_button(ui, t, r, discard_part, "Discard changes in this group", "undo", 14.0, t.content_3);
    }

    // The card: its rows, a hairline between them, and the card's border and fill behind them.
    spacer(ui, rem(0.5));
    let top = ui.cursor().top();
    let left = ui.cursor().left();
    let back = p.add(Shape::Noop);
    rows(ui, actions);
    let card = Rect::from_min_max(pos2(left, top), pos2(left + w, ui.cursor().top()));
    p.set(
        back,
        // The reference's group is a Surface at step 3, over the pane's surface-2 (#12365).
        Shape::Rect(egui::epaint::RectShape::new(card, CornerRadius::same(10), t.surface_3, Stroke::new(1.0, t.edge_2), StrokeKind::Inside)),
    );
    let card_resp = ui.interact(card, ui.scope_id().with(group_part), Sense::hover());
    card_resp.widget_info(|| WidgetInfo::new(Role::GenericContainer));
    part(ui, &card_resp, group_part);
    spacer(ui, rem(1.286));
    pressed
}

/// The theme row: its title and description, and the Theme segments under them.
fn theme_row(ui: &mut egui::Ui, t: &Tokens, value: &str, actions: &mut Vec<(Field, String)>) {
    let tray = setting_row(
        ui,
        t,
        true,
        "Theme",
        parts::THEMETITLE,
        "Follow the system appearance, or pin one.",
        parts::THEMEDESCRIPTION,
        0.0,
        rem(1.857),
        THEME_ROW,
    );
    segments(ui, t, tray, &THEME_SEGS, value, true, Field::Theme, actions);
}

/// The reduced motion row: its title and description, and the three-way choice on the right.
fn motion_row(ui: &mut egui::Ui, t: &Tokens, value: &str, actions: &mut Vec<(Field, String)>) {
    let cw = segments_width(ui, &MOTION_SEGS);
    let tray = setting_row(
        ui,
        t,
        false,
        "Reduced motion",
        parts::REDUCEDMOTIONTITLE,
        "Cuts animations and transitions to the minimum.",
        parts::REDUCEDMOTIONDESCRIPTION,
        cw,
        rem(1.857),
        REDUCED_ROW,
    );
    segments(ui, t, tray, &MOTION_SEGS, value, false, Field::Motion, actions);
}

/// The show completed row: its title and description, and the switch on the right.
fn show_row(ui: &mut egui::Ui, t: &Tokens, on: bool, actions: &mut Vec<(Field, String)>) {
    let sw = setting_row(
        ui,
        t,
        false,
        "Show completed tasks",
        parts::SHOWCOMPLETEDTITLE,
        "Keep finished tasks in the list instead of hiding them.",
        parts::SHOWCOMPLETEDDESCRIPTION,
        // The reference's switch: 30 × 18.
        30.0,
        18.0,
        SHOW_ROW,
    );
    switch(ui, t, sw, on, actions);
}

/// Lays out one setting row: its title and description, and the control's rectangle, which the caller
/// draws. Stacked, the control spans the row under the text; otherwise it sits at the right.
#[allow(clippy::too_many_arguments)]
fn setting_row(
    ui: &mut egui::Ui,
    t: &Tokens,
    stacked: bool,
    title: &str,
    title_part: &'static str,
    desc: &str,
    desc_part: &'static str,
    control_w: f32,
    control_h: f32,
    row_part: &'static str,
) -> Rect {
    let w = ui.available_width();
    let text_w = if stacked { w - 2.0 * PAD_X } else { (w - 2.0 * PAD_X - control_w - GAP).max(40.0) };
    // The reference's rows, measured from its recording: a 13 px regular title on a 19.5 px line, the
    // description on a 17.25 px line 1 px under it, a stacked control 7 px below (ours had 16 and 14 px
    // lines, so every row sat higher than the last).
    let tg = lay(ui, title, 13.0, 400, t.content_1, text_w);
    let dg = lay(ui, desc, 11.5, 400, t.content_3, text_w);
    let rows = |g: &Galley, line: f32| (g.rows.len().max(1) as f32) * line;
    let (title_h, desc_h) = (rows(&tg, 19.5), rows(&dg, 17.25));
    let ts = vec2(tg.size().x, title_h);
    let ds = vec2(dg.size().x, desc_h);
    let text_h = ts.y + 1.0 + ds.y;
    let body_h = if stacked { text_h + 7.0 + control_h } else { text_h.max(control_h) };
    let (row, row_resp) = ui.allocate_exact_size(vec2(w, 2.0 * PAD_Y + body_h), Sense::hover());
    row_resp.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, title));
    part(ui, &row_resp, row_part);
    let x = row.left() + PAD_X;
    let y = if stacked { row.top() + PAD_Y } else { row.center().y - text_h / 2.0 };
    node_text(ui, Rect::from_min_size(pos2(x, y), ts), tg, t.content_1, title_part, title, Role::GenericContainer);
    node_text(ui, Rect::from_min_size(pos2(x, y + ts.y + 1.0), ds), dg, t.content_3, desc_part, desc, Role::GenericContainer);
    if stacked {
        Rect::from_min_size(pos2(x, y + text_h + 7.0), vec2(w - 2.0 * PAD_X, control_h))
    } else {
        Rect::from_min_size(pos2(row.right() - PAD_X - control_w, row.center().y - control_h / 2.0), vec2(control_w, control_h))
    }
}

/// The switch: a track and a thumb, toggled by a click, Space or Enter.
fn switch(ui: &mut egui::Ui, t: &Tokens, rect: Rect, checked: bool, actions: &mut Vec<(Field, String)>) {
    let resp = ui.interact(rect, ui.scope_id().with(parts::SHOWCOMPLETEDSWITCH), Sense::click());
    if pressed(ui, &resp) {
        actions.push((Field::Show, (!checked).to_string()));
    }
    let on = ui.ctx().animate_bool_with_time(resp.id.with("on"), checked, 0.18);
    let track = match (resp.hovered(), checked) {
        (true, true) => t.accent_600,
        (true, false) => t.content_3,
        _ => t.edge_3.lerp_to_gamma(t.accent_500, on),
    };
    let p = ui.painter().clone();
    let radius = CornerRadius::same((rect.height() / 2.0) as u8);
    p.rect_filled(rect, radius, track);
    if resp.has_focus() {
        p.rect_stroke(rect.expand(1.0), radius, Stroke::new(2.0, t.focus_1), StrokeKind::Middle);
    }
    let d = rem(1.0);
    let x = rect.left() + rem(0.1) + 1.0 + rem(1.157) * on;
    let thumb = Rect::from_min_size(pos2(x, rect.center().y - d / 2.0), vec2(d, d));
    p.circle_filled(thumb.center(), d / 2.0, t.on_accent);
    resp.widget_info(|| WidgetInfo::selected(Role::Switch, true, checked, "Show completed tasks"));
    part(ui, &resp, parts::SHOWCOMPLETEDSWITCH);
}

/// A segmented choice: the stored value's segment is raised and selected. Stacked, the segments share the
/// tray's width; otherwise each takes its label's width.
#[allow(clippy::too_many_arguments)]
fn segments(
    ui: &mut egui::Ui,
    t: &Tokens,
    tray: Rect,
    segs: &[Seg],
    value: &str,
    stacked: bool,
    field: Field,
    actions: &mut Vec<(Field, String)>,
) {
    let p = ui.painter().clone();
    // The reference's Segmented (surface thumb): a surface-0 track, the chosen segment raised on surface-3
    // with an edge-2 border (#12365).
    p.rect_filled(tray, CornerRadius::same(5), t.surface_0);
    let n = segs.len();
    let gaps = 2.0 * (n - 1) as f32;
    let inner = tray.width() - 4.0 - gaps;
    let mut x = tray.left() + 2.0;
    for s in segs {
        let w = if stacked { inner / n as f32 } else { seg_width(ui, s) };
        let r = Rect::from_min_size(pos2(x, tray.top() + 2.0), vec2(w, tray.height() - 4.0));
        x += w + 2.0;
        let resp = ui.interact(r, ui.scope_id().with(s.part), Sense::click());
        let selected = s.value == value;
        let fg = if selected || resp.hovered() { t.content_1 } else { t.content_2 };
        if selected {
            p.rect(r, CornerRadius::same(4), t.surface_3, Stroke::new(1.0, t.edge_2), StrokeKind::Inside);
        }
        let label = lay(ui, s.label, 12.0, 600, fg, f32::INFINITY);
        let icon_w = if s.icon.is_empty() { 0.0 } else { 15.0 + rem(0.429) };
        let left = r.center().x - (icon_w + label.size().x) / 2.0;
        if !s.icon.is_empty() {
            glyph(&p, s.icon, pos2(left + 7.5, r.center().y), 15.0, fg);
        }
        let ly = r.center().y - label.size().y / 2.0;
        p.galley(pos2(left + icon_w, ly), label, fg);
        if resp.has_focus() {
            p.rect_stroke(r.expand(2.0), CornerRadius::same(4), Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
        }
        resp.widget_info(|| WidgetInfo::labeled(Role::Button, true, s.label));
        ui.ctx().accesskit_node_builder(resp.id, |n| n.set_selected(selected));
        part(ui, &resp, s.part);
        if pressed(ui, &resp) && !selected {
            actions.push((field, s.value.to_owned()));
        }
    }
}

/// The width of one natural (non-stacked) segment: its icon, its label and the label padding.
fn seg_width(ui: &egui::Ui, s: &Seg) -> f32 {
    let label = lay(ui, s.label, 12.0, 600, Color32::WHITE, f32::INFINITY).size().x;
    let icon_w = if s.icon.is_empty() { 0.0 } else { 15.0 + rem(0.429) };
    icon_w + label + 2.0 * 8.75
}

/// The width of a natural segmented tray: its segments, their gaps and the tray's padding.
fn segments_width(ui: &egui::Ui, segs: &[Seg]) -> f32 {
    let n = segs.len();
    segs.iter().map(|s| seg_width(ui, s)).sum::<f32>() + 2.0 * (n - 1) as f32 + 4.0
}

/// An icon button with no label of its own; a press is a click, Space or Enter. Returns the press.
#[allow(clippy::too_many_arguments)]
fn icon_button(
    ui: &mut egui::Ui,
    t: &Tokens,
    rect: Rect,
    part_id: &'static str,
    label: &str,
    glyph_name: &str,
    px: f32,
    color: Color32,
) -> bool {
    let resp = ui.interact(rect, ui.scope_id().with(part_id), Sense::click());
    let p = ui.painter().clone();
    let hot = resp.hovered();
    if hot {
        p.rect_filled(rect, CornerRadius::same(4), t.wash_2);
    }
    let fg = if hot { t.content_1 } else { color };
    glyph(&p, glyph_name, rect.center(), px, fg);
    if resp.has_focus() {
        p.rect_stroke(rect, CornerRadius::same(4), Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
    }
    resp.widget_info(|| WidgetInfo::labeled(Role::Button, true, label));
    part(ui, &resp, part_id);
    pressed(ui, &resp)
}

/// A text node: a rectangle the text is laid in, tagged with its part and read as its label.
fn node_text(ui: &mut egui::Ui, rect: Rect, galley: Arc<Galley>, color: Color32, part_id: &'static str, label: &str, role: Role) {
    let resp = ui.interact(rect, ui.scope_id().with(part_id), Sense::hover());
    resp.widget_info(|| WidgetInfo::labeled(role, true, label));
    part(ui, &resp, part_id);
    // The text centred in its line box, as a browser sets it in its line-height.
    let top = rect.top() + ((rect.height() - galley.size().y) / 2.0).max(0.0);
    ui.painter().galley(pos2(rect.left(), top), galley, color);
}

/// A hairline between two rows of a card.
fn divider(ui: &mut egui::Ui, t: &Tokens) {
    let w = ui.available_width();
    let (r, _) = ui.allocate_exact_size(vec2(w, 1.0), Sense::hover());
    ui.painter().hline(Rangef::new(r.left(), r.right()), r.top(), Stroke::new(1.0, t.edge_1));
}

/// Empty vertical space of a height, across the full width.
fn spacer(ui: &mut egui::Ui, h: f32) {
    let w = ui.available_width();
    let _ = ui.allocate_exact_size(vec2(w, h), Sense::hover());
}

/// Whether a press happened on this response this frame: a click, or Space or Enter while it has focus.
fn pressed(ui: &egui::Ui, resp: &Response) -> bool {
    let key = ui.input(|i| i.key_pressed(Key::Space) || i.key_pressed(Key::Enter));
    resp.clicked() || (resp.has_focus() && key)
}

/// Text laid out in Inter at a px size and a weight, in a colour, wrapped to a width.
fn lay(ui: &egui::Ui, s: &str, px: f32, w: u16, color: Color32, wrap: f32) -> Arc<Galley> {
    ui.painter().layout(s.to_owned(), FontId::new(px, weight(w)), color, wrap)
}

/// A Material Symbols glyph centred on a point.
fn glyph(p: &egui::Painter, name: &str, at: egui::Pos2, px: f32, color: Color32) {
    let g = icon(name, px);
    p.text(at, Align2::CENTER_CENTER, g.text(), FontId::new(px, FontFamily::Name("Icons".into())), color);
}

/// Selects a section: the pane, and the search's scope. A tab with matches keeps the search scoped to
/// itself; one without matches clears it. Outside a search it only changes the pane.
fn choose(app: &mut App, query: &mut String, scoped: &mut bool, searching: bool, counts: &[usize], panes: &[&str], to: usize) {
    app.settings_window = Some(panes[to].to_owned());
    if !searching {
        return;
    }
    if counts[to] > 0 {
        *scoped = true;
    } else {
        query.clear();
        *scoped = false;
    }
}

/// Whether the General row with this id is shown: every row is, unless a search narrows them.
fn shown_row(id: &str, tokens: &[String], searching: bool) -> bool {
    !searching || ROWS.iter().any(|r| r.id == id && row_matches(r, tokens))
}

/// Whether a group of the General pane is shown: it is, unless no row of it matches a search.
fn group_shown(tokens: &[String], searching: bool, group: &str) -> bool {
    !searching || ROWS.iter().any(|r| r.group == group && row_matches(r, tokens))
}

/// The search words: the query, trimmed, lower-cased and split on spaces.
fn tokens_of(query: &str) -> Vec<String> {
    query.trim().to_lowercase().split_whitespace().map(str::to_owned).collect()
}

/// A row matches when every word is part of its title, description or keywords (ignoring case).
fn row_matches(r: &Row, tokens: &[String]) -> bool {
    let hay = format!("{} {} {}", r.title, r.description, r.keywords).to_lowercase();
    tokens.iter().all(|t| hay.contains(t.as_str()))
}

/// The stored value of a setting, as the store keeps it.
fn value_of(s: &Settings, f: Field) -> String {
    match f {
        Field::Theme => s.theme.clone(),
        Field::Motion => s.reduced_motion.clone(),
        Field::Show => if s.show_completed { "true" } else { "false" }.to_owned(),
    }
}

/// The key the store keeps a setting under.
fn key_of(f: Field) -> &'static str {
    match f {
        Field::Theme => "theme",
        Field::Motion => "reduced_motion",
        Field::Show => "show_completed",
    }
}

/// Whether a setting differs between two snapshots.
fn differs(a: &Settings, b: &Settings, f: Field) -> bool {
    value_of(a, f) != value_of(b, f)
}

/// The test's mount: the app on the props' data set, with the page on its General pane.
struct Mount {
    app: App,
    screen: SettingsGeneral,
}

impl Mounted for Mount {
    fn frame(&mut self, ui: &mut egui::Ui, look: Look, emit: &mut dyn FnMut(&'static str, serde_json::Value)) {
        for e in self.screen.show(&mut self.app, ui, look) {
            let (n, f) = e.to_json();
            emit(n, f);
        }
    }
}

pub fn mount(app: App, props: serde_json::Value) -> Result<Box<dyn Mounted>, String> {
    let _p: taskscape_ui::omnicui::settingsgeneral::SettingsgeneralProps = serde_json::from_value(props).map_err(|e| e.to_string())?;
    Ok(Box::new(Mount { app, screen: SettingsGeneral::new() }))
}

/// Writes back the open-time value of each listed setting that differs now.
fn discard(cur: &Settings, open: &Settings, fields: &[Field], actions: &mut Vec<(Field, String)>) {
    for f in fields {
        if differs(cur, open, *f) {
            actions.push((*f, value_of(open, *f)));
        }
    }
}
