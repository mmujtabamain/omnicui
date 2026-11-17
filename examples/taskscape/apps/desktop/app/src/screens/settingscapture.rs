//! contract: omnicui/contracts/settingscapture.contract.yaml (id: app.settingscapture). The Settings window: its section list and search, and the Capture pane with the capture mode and the folder screenshots are saved to.
use crate::App;
use egui::text::{CCursor, CCursorRange, LayoutJob, TextFormat};
use egui::{pos2, vec2, Align, Align2, Color32, CornerRadius, FontFamily, FontId, Galley, Key, Layout, Painter, Pos2, Rangef, Rect, Response, Role, Sense, Stroke, StrokeKind, TextEdit, UiBuilder, WidgetInfo};
use std::path::PathBuf;
use std::sync::Arc;
use taskscape_ui::omnicui::settingscapture::{parts, SettingscaptureEvent, SettingscaptureProps};
use taskscape_ui::{icon, part, weight, Look, Mounted, Tokens};

const MODE_KEY: &str = "screenshot_mode";
const TITLE_H: f32 = 46.0;
const SIDEBAR_W: f32 = 153.0;
const TAB_Y: f32 = 86.0;
const TAB_H: f32 = 30.0;
const TAB_STEP: f32 = 31.67;
const PAD: f32 = 18.0;
const GROUP_TOP: f32 = 58.0;
const HEAD_H: f32 = 16.0;
const GROUP_GAP: f32 = 20.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Section {
    General,
    Capture,
    Shortcuts,
    About,
}

const SECTIONS: [Section; 4] = [Section::General, Section::Capture, Section::Shortcuts, Section::About];

impl Section {
    fn label(self) -> &'static str {
        match self {
            Self::General => "General",
            Self::Capture => "Capture",
            Self::Shortcuts => "Shortcuts",
            Self::About => "About",
        }
    }
    fn icon(self) -> &'static str {
        match self {
            Self::General => "tune",
            Self::Capture => "screenshot_monitor",
            Self::Shortcuts => "keyboard",
            Self::About => "info",
        }
    }
    fn index(self) -> usize {
        SECTIONS.iter().position(|s| *s == self).unwrap_or(0)
    }
    fn named(name: &str) -> Self {
        match name {
            "general" => Self::General,
            "shortcuts" => Self::Shortcuts,
            "about" => Self::About,
            _ => Self::Capture,
        }
    }
}

fn tab_part(s: Section) -> &'static str {
    match s {
        Section::General => parts::SECTIONGENERAL,
        Section::Capture => parts::SECTIONCAPTURE,
        Section::Shortcuts => parts::SECTIONSHORTCUTS,
        Section::About => parts::SECTIONABOUT,
    }
}

/// The screenshot capture mode. Any stored value other than "region" is Full screen (F10-032).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Full,
    Region,
}

impl Mode {
    fn of(stored: Option<String>) -> Self {
        if stored.as_deref() == Some("region") {
            Self::Region
        } else {
            Self::Full
        }
    }
    fn key(self) -> &'static str {
        match self {
            Self::Full => "fullscreen",
            Self::Region => "region",
        }
    }
    fn hint(self) -> &'static str {
        match self {
            Self::Full => "Capture the entire screen instantly.",
            Self::Region => "Drag to select an area, or press Space to grab a window. Esc cancels.",
        }
    }
}

#[derive(Clone, Copy)]
enum Control {
    Segmented,
    Path,
    Text(&'static str),
    Accel(&'static str),
    Nothing,
}

#[derive(Clone, Copy)]
struct RowParts {
    row: &'static str,
    title: &'static str,
    desc: &'static str,
    foot: &'static str,
    control: &'static str,
}

const NO_PARTS: RowParts = RowParts { row: "", title: "", desc: "", foot: "", control: "" };

#[derive(Clone, Copy)]
struct Row {
    title: &'static str,
    desc: &'static str,
    keywords: &'static str,
    control: Control,
    inline: bool,
    parts: RowParts,
}

#[derive(Clone, Copy)]
struct Group {
    id: &'static str,
    label: &'static str,
    rows: &'static [Row],
}

const MODE_ROW: Row = Row {
    title: "Capture mode",
    desc: "",
    keywords: "screenshot capture region area window full screen",
    control: Control::Segmented,
    inline: false,
    parts: RowParts {
        row: parts::CAPTUREMODEROW,
        title: parts::CAPTUREMODELABEL,
        desc: "",
        foot: parts::CAPTUREHINT,
        control: parts::CAPTUREMODECONTROL,
    },
};

const PATH_ROW: Row = Row {
    title: "Saved to",
    desc: "Click to open the folder.",
    keywords: "folder path directory location disk screenshots finder",
    control: Control::Path,
    inline: false,
    parts: RowParts {
        row: parts::SAVEDTOROW,
        title: parts::SAVEDTOLABEL,
        desc: parts::SAVEDTODESCRIPTION,
        foot: "",
        control: "",
    },
};

const VERSION_ROW: Row = Row {
    title: "Taskscape",
    desc: "A macOS task manager with a menu-bar capture bar.",
    keywords: "version build number release about",
    control: Control::Text(env!("CARGO_PKG_VERSION")),
    inline: true,
    parts: NO_PARTS,
};

const CAPTURE_BAR_ROW: Row = Row {
    title: "Show or hide the capture bar",
    desc: "",
    keywords: "shortcut hotkey key binding global capture bar",
    control: Control::Accel("Mod+Return"),
    inline: true,
    parts: NO_PARTS,
};

const RESTORE_ROW: Row = Row {
    title: "Restore defaults",
    desc: "Puts appearance, capture and task preferences, and every keyboard shortcut, back the way they shipped. Your tasks are untouched.",
    keywords: "reset clear defaults factory revert erase preferences",
    control: Control::Nothing,
    inline: false,
    parts: NO_PARTS,
};

const CAPTURE_GROUPS: &[Group] = &[Group { id: "screenshots", label: "Screenshots", rows: &[MODE_ROW, PATH_ROW] }];
const SHORTCUT_GROUPS: &[Group] = &[Group { id: "global", label: "Global", rows: &[CAPTURE_BAR_ROW] }];
const ABOUT_GROUPS: &[Group] = &[
    Group { id: "application", label: "Application", rows: &[VERSION_ROW] },
    Group { id: "reset", label: "Reset", rows: &[RESTORE_ROW] },
];

fn groups(s: Section) -> &'static [Group] {
    match s {
        Section::General => &[],
        Section::Capture => CAPTURE_GROUPS,
        Section::Shortcuts => SHORTCUT_GROUPS,
        Section::About => ABOUT_GROUPS,
    }
}

fn words_of(query: &str) -> Vec<String> {
    query.trim().to_lowercase().split_whitespace().map(str::to_owned).collect()
}

fn row_matches(row: &Row, words: &[String]) -> bool {
    let hay = format!("{} {} {}", row.title, row.desc, row.keywords).to_lowercase();
    words.iter().all(|w| hay.contains(w.as_str()))
}

fn count_matches(s: Section, words: &[String]) -> usize {
    groups(s).iter().flat_map(|g| g.rows.iter()).filter(|r| row_matches(r, words)).count()
}

fn heading_text(s: Section, g: &Group, searching: bool, scope: Option<Section>) -> String {
    if searching && scope.is_none() {
        format!("{} · {}", s.label(), g.label)
    } else {
        g.label.to_owned()
    }
}

/// The platform's accelerator text: "Mod+Return" is ⌘Return on macOS, Ctrl+Return elsewhere.
fn accel(s: &str) -> String {
    let mac = cfg!(target_os = "macos");
    let keys: Vec<String> = s
        .split('+')
        .map(str::trim)
        .map(|k| match k {
            "Mod" => if mac { "⌘".to_owned() } else { "Ctrl".to_owned() },
            other => other.to_owned(),
        })
        .collect();
    if mac {
        keys.concat()
    } else {
        keys.join("+")
    }
}

/// The folder screenshots are saved to: the data folder's screenshots directory.
fn screenshots_dir() -> String {
    let base = std::env::var_os("TASKSCAPE_DATA_DIR").map(PathBuf::from).or_else(|| {
        std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .map(|h| PathBuf::from(h).join(".taskscape"))
    });
    base.unwrap_or_default().join("screenshots").display().to_string()
}

fn rect_at(o: Pos2, x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect::from_min_size(pos2(o.x + x, o.y + y), vec2(w, h))
}

fn pos_at(o: Pos2, x: f32, y: f32) -> Pos2 {
    pos2(o.x + x, o.y + y)
}

/// Lays text out in Inter at a px size and weight, wrapped to `w`, or one truncated line.
fn job(ui: &egui::Ui, s: &str, size: f32, w: u16, color: Color32, max_w: f32, one_line: bool) -> Arc<Galley> {
    let mut j = LayoutJob::single_section(s.to_owned(), TextFormat { font_id: FontId::new(size, weight(w)), color, ..Default::default() });
    j.wrap.max_width = max_w;
    if one_line {
        j.wrap.max_rows = 1;
        j.wrap.break_anywhere = true;
        j.wrap.overflow_character = Some('…');
    }
    ui.painter().layout_job(j)
}

fn draw_icon(painter: &Painter, name: &str, center: Pos2, px: f32, color: Color32) {
    let glyph = icon(name, px);
    painter.text(center, Align2::CENTER_CENTER, glyph.text(), FontId::new(px, FontFamily::Name("Icons".into())), color);
}

/// A part with no widget of its own: an invisible hover node with its label.
fn node(ui: &egui::Ui, rect: Rect, key: &str, part_id: &str, info: impl Fn() -> WidgetInfo) -> Response {
    let r = ui.interact(rect, ui.scope_id().with(key), Sense::hover());
    r.widget_info(info);
    part(ui, &r, part_id);
    r
}

/// A chosen tab or segment reads as `selected`, which WidgetInfo does not carry for a button.
fn mark_selected(ui: &egui::Ui, r: &Response, selected: bool) {
    if selected {
        ui.ctx().accesskit_node_builder(r.id, |n| n.set_selected(true));
    }
}

fn tag(ui: &egui::Ui, rect: Rect, key: &str, part_id: &str, label: &str) {
    if part_id.is_empty() {
        return;
    }
    node(ui, rect, key, part_id, || WidgetInfo::labeled(Role::GenericContainer, true, label));
}

fn icon_button(ui: &egui::Ui, painter: &Painter, rect: Rect, key: &str, glyph: &str, part_id: &str, label: &str, t: Tokens) -> Response {
    let r = ui.interact(rect, ui.scope_id().with(key), Sense::click());
    if r.hovered() {
        painter.rect_filled(rect, CornerRadius::same(4), t.wash_2);
    }
    let color = if r.hovered() { t.content_1 } else { t.content_2 };
    draw_icon(painter, glyph, rect.center(), 16.0, color);
    if r.has_focus() {
        painter.rect_stroke(rect, CornerRadius::same(4), Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
    }
    r.widget_info(|| WidgetInfo::labeled(Role::Button, true, label));
    part(ui, &r, part_id);
    r
}

/// The Surface track with its two segments; returns the mode a click chose, if it is a new one.
fn segmented(ui: &egui::Ui, painter: &Painter, rect: Rect, mode: Mode, t: Tokens) -> Option<Mode> {
    painter.rect_filled(rect, CornerRadius::same(7), t.surface_0);
    let mut chosen = None;
    for (i, m) in [Mode::Full, Mode::Region].into_iter().enumerate() {
        let (key, part_id, label, glyph) = match m {
            Mode::Full => ("segment-full", parts::SEGMENTFULLSCREEN, "Full screen", "fullscreen"),
            Mode::Region => ("segment-region", parts::SEGMENTREGION, "Region", "crop_free"),
        };
        let seg = Rect::from_min_size(pos2(rect.left() + i as f32 * rect.width() / 2.0, rect.top()), vec2(rect.width() / 2.0, rect.height()));
        let selected = mode == m;
        let r = ui.interact(seg, ui.scope_id().with(key), Sense::click());
        if selected {
            painter.rect(seg.shrink(2.0), CornerRadius::same(4), t.surface_3, Stroke::new(1.0, t.edge_2), StrokeKind::Inside);
        }
        let fg = if selected { t.content_1 } else { t.content_2 };
        let text = job(ui, label, 12.0, if selected { 600 } else { 500 }, fg, f32::INFINITY, false);
        let total = 15.0 + 6.0 + text.size().x;
        let x0 = seg.center().x - total / 2.0;
        draw_icon(painter, glyph, pos2(x0 + 7.5, seg.center().y), 15.0, fg);
        painter.galley(pos2(x0 + 21.0, seg.center().y - text.size().y / 2.0), text, fg);
        if r.has_focus() {
            painter.rect_stroke(seg, CornerRadius::same(4), Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
        }
        r.widget_info(|| WidgetInfo::selected(Role::Button, true, selected, label));
        mark_selected(ui, &r, selected);
        part(ui, &r, part_id);
        if r.clicked() && !selected {
            chosen = Some(m);
        }
    }
    chosen
}

/// A row laid out before it is drawn, so the card's height is known.
struct Laid {
    row: Row,
    title: Arc<Galley>,
    desc: Option<Arc<Galley>>,
    foot: Option<Arc<Galley>>,
    ctrl: Option<Arc<Galley>>,
    loading: bool,
    h: f32,
}

fn lay(ui: &egui::Ui, row: Row, mode: Mode, loading: bool, t: Tokens, w: f32, path: &str) -> Laid {
    let title = job(ui, row.title, 13.0, 400, t.content_1, w, false);
    let desc = (!row.desc.is_empty()).then(|| job(ui, row.desc, 11.5, 400, t.content_3, w, false));
    let foot = matches!(row.control, Control::Segmented).then(|| job(ui, mode.hint(), 11.0, 400, t.content_3, w, false));
    let ctrl = match row.control {
        Control::Path if loading => Some(job(ui, "—", 11.5, 400, t.content_3, w, false)),
        Control::Path => Some(job(ui, path, 11.5, 400, t.content_2, w - 28.0, true)),
        Control::Text(s) => Some(job(ui, s, 11.5, 400, t.content_2, w, true)),
        Control::Accel(s) => Some(job(ui, &accel(s), 11.5, 400, t.content_2, w, true)),
        Control::Segmented | Control::Nothing => None,
    };
    let desc_h = desc.as_ref().map_or(0.0, |d| 2.0 + d.size().y.max(17.25));
    let h = if row.inline {
        let text_h = 19.5 + desc_h;
        16.0 + text_h.max(if ctrl.is_some() { 21.0 } else { 0.0 })
    } else {
        let mut h = 7.0 + 19.5 + desc_h;
        match row.control {
            Control::Segmented => h += 4.0 + 26.0 + 6.0 + 16.5,
            Control::Path => h += 5.0 + 21.0,
            _ => {}
        }
        h + 8.0
    };
    Laid { row, title, desc, foot, ctrl, loading, h }
}

pub struct SettingsCapture {
    section: Section,
    /// Drawn inside the Settings window, which has its own sidebar: the page stays on its own section and
    /// ignores its sidebar, which sits out of view.
    pub embedded: bool,
    query: String,
    /// While searching: the section the results are narrowed to (None: every section).
    scope: Option<Section>,
    mode: Option<Mode>,
    /// The mode the window opened with: the baseline discard returns to.
    opened: Option<Mode>,
    path_loading: bool,
    want_focus: Option<Section>,
    focus_search: bool,
}

impl SettingsCapture {
    pub fn new() -> Self {
        Self {
            section: Section::Capture,
            embedded: false,
            query: String::new(),
            scope: None,
            mode: None,
            opened: None,
            path_loading: false,
            want_focus: None,
            focus_search: false,
        }
    }

    fn set_mode(&mut self, app: &mut App, to: Mode, events: &mut Vec<SettingscaptureEvent>) {
        match app.data.set_setting(MODE_KEY, to.key()) {
            Ok(()) => {
                self.mode = Some(to);
                events.push(SettingscaptureEvent::SettingSaved { key: MODE_KEY.into(), value: to.key().into() });
            }
            Err(e) => app.refused(e),
        }
    }

    /// A section was picked. While searching, a section with matches narrows the results to it;
    /// one without matches clears the search and shows the section.
    fn select(&mut self, s: Section, words: &[String]) {
        if words.is_empty() {
            self.section = s;
            self.scope = None;
        } else if count_matches(s, words) > 0 {
            self.section = s;
            self.scope = Some(s);
        } else {
            self.query.clear();
            self.section = s;
            self.scope = None;
        }
    }

    fn paint_row(&mut self, ui: &egui::Ui, painter: &Painter, app: &mut App, events: &mut Vec<SettingscaptureEvent>, l: &Laid, r: Rect, mode: Mode, t: Tokens, path: &str) {
        let row = l.row;
        let x = r.left() + 9.0;
        let w = r.width() - 18.0;
        if row.inline {
            let desc_h = l.desc.as_ref().map_or(0.0, |d| 2.0 + d.size().y.max(17.25));
            let top = r.top() + (r.height() - (19.5 + desc_h)) / 2.0;
            let title_at = pos2(x, top + (19.5 - l.title.size().y) / 2.0);
            painter.galley(title_at, l.title.clone(), t.content_1);
            tag(ui, Rect::from_min_size(title_at, l.title.size()), &format!("title-{}", row.title), row.parts.title, row.title);
            if let Some(d) = &l.desc {
                let at = pos2(x, top + 21.5);
                painter.galley(at, d.clone(), t.content_3);
                tag(ui, Rect::from_min_size(at, d.size()), &format!("desc-{}", row.title), row.parts.desc, row.desc);
            }
            if let Some(c) = &l.ctrl {
                let at = pos2(r.right() - 9.0 - c.size().x, r.center().y - c.size().y / 2.0);
                painter.galley(at, c.clone(), t.content_2);
            }
        } else {
            let mut yy = r.top() + 7.0;
            let title_at = pos2(x, yy + (19.5 - l.title.size().y) / 2.0);
            painter.galley(title_at, l.title.clone(), t.content_1);
            tag(ui, Rect::from_min_size(title_at, l.title.size()), &format!("title-{}", row.title), row.parts.title, row.title);
            yy += 19.5;
            if let Some(d) = &l.desc {
                let at = pos2(x, yy + 2.0);
                painter.galley(at, d.clone(), t.content_3);
                tag(ui, Rect::from_min_size(at, d.size()), &format!("desc-{}", row.title), row.parts.desc, row.desc);
                yy += 2.0 + d.size().y.max(17.25);
            }
            match row.control {
                Control::Segmented => {
                    let track = Rect::from_min_size(pos2(x, yy + 4.0), vec2(w, 26.0));
                    tag(ui, track, "mode-control", row.parts.control, row.title);
                    if let Some(next) = segmented(ui, painter, track, mode, t) {
                        self.set_mode(app, next, events);
                        ui.ctx().request_repaint();
                    }
                    yy = track.bottom() + 6.0;
                    if let Some(f) = &l.foot {
                        let at = pos2(x, yy);
                        painter.galley(at, f.clone(), t.content_3);
                        tag(ui, Rect::from_min_size(at, f.size()), "mode-hint", row.parts.foot, mode.hint());
                    }
                }
                Control::Path => {
                    let at = pos2(x, yy + 5.0);
                    let bh = 21.0;
                    if let Some(c) = &l.ctrl {
                        if l.loading {
                            let ph = Rect::from_min_size(pos2(x, at.y + (bh - c.size().y) / 2.0), c.size());
                            painter.galley(ph.min, c.clone(), t.content_3);
                            tag(ui, ph, "path-placeholder", parts::PATHPLACEHOLDER, "—");
                        } else {
                            let br = Rect::from_min_size(at, vec2((c.size().x + 28.0).min(w), bh));
                            let resp = ui.interact(br, ui.scope_id().with("path-button"), Sense::click());
                            let hot = resp.hovered();
                            if hot {
                                painter.rect_filled(br, CornerRadius::same(4), t.wash_1);
                            }
                            let fg = if hot { t.content_1 } else { t.content_2 };
                            draw_icon(painter, "folder_open", pos2(br.left() + 12.0, br.center().y), 14.0, fg);
                            painter.galley(pos2(br.left() + 23.0, br.center().y - c.size().y / 2.0), c.clone(), fg);
                            if resp.has_focus() {
                                painter.rect_stroke(br, CornerRadius::same(4), Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
                            }
                            let aria = format!("Open {path} in the file manager");
                            resp.widget_info(|| WidgetInfo::labeled(Role::Button, true, &aria));
                            ui.ctx().accesskit_node_builder(resp.id, |n| n.set_value(path));
                            part(ui, &resp, parts::PATHBUTTON);
                            if resp.clicked() {
                                events.push(SettingscaptureEvent::FolderOpenRequested { folder: "screenshots".into() });
                            }
                        }
                    }
                }
                Control::Text(_) | Control::Accel(_) | Control::Nothing => {}
            }
        }
        tag(ui, r, &format!("row-{}", row.title), row.parts.row, row.title);
    }

    /// One frame: the title strip, the sections and search, and the pane for what is shown.
    pub fn show(&mut self, app: &mut App, ui: &mut egui::Ui, look: Look) -> Vec<SettingscaptureEvent> {
        if self.embedded {
            self.section = Section::Capture;
        }
        let mut events = vec![];
        let ctx = ui.ctx().clone();
        let t = look.t;

        if self.mode.is_none() {
            let m = Mode::of(app.data.get_setting(MODE_KEY));
            self.mode = Some(m);
            self.opened = Some(m);
        }
        let mode = self.mode.unwrap_or(Mode::Full);
        let opened = self.opened.unwrap_or(Mode::Full);

        if let Some(s) = self.want_focus.take() {
            ctx.memory_mut(|m| m.request_focus(ui.scope_id().with(("section", s.index()))));
        }

        let (esc, find) = ui.input(|i| (i.key_pressed(Key::Escape), (i.modifiers.command || i.modifiers.mac_cmd || i.modifiers.ctrl) && i.key_pressed(Key::F)));
        if find {
            self.focus_search = true;
            ctx.request_repaint();
        }
        if esc {
            if self.query.is_empty() {
                events.push(SettingscaptureEvent::WindowClosed { reason: "escape".into() });
            } else {
                self.query.clear();
                self.scope = None;
                ctx.request_repaint();
            }
        }

        let whole = ui.available_rect_before_wrap();
        ui.allocate_rect(whole, Sense::hover());
        let o = whole.min;
        let (ww, wh) = (whole.width(), whole.height());
        let painter = ui.painter_at(whole);
        let title_rect = rect_at(o, 0.0, 0.0, ww, TITLE_H);
        let side_rect = rect_at(o, 0.0, TITLE_H, SIDEBAR_W, wh - TITLE_H);
        let pane_rect = rect_at(o, SIDEBAR_W + 1.0, TITLE_H, ww - SIDEBAR_W - 1.0, wh - TITLE_H);
        painter.rect_filled(title_rect, CornerRadius::same(0), t.surface_1);
        painter.rect_filled(side_rect, CornerRadius::same(0), t.surface_1);
        painter.rect_filled(pane_rect, CornerRadius::same(0), t.surface_2);
        painter.hline(Rangef::new(o.x, o.x + ww), o.y + TITLE_H - 0.5, Stroke::new(1.0, t.edge_1));
        painter.vline(o.x + SIDEBAR_W + 0.5, Rangef::new(o.y + TITLE_H, o.y + wh), Stroke::new(1.0, t.edge_1));

        // Title strip.
        node(ui, title_rect, "titlebar", parts::TITLEBAR, || WidgetInfo::new(Role::GenericContainer));
        let title = job(ui, "Settings", 13.5, 600, t.content_1, f32::INFINITY, false);
        let title_at = pos_at(o, 16.0, (TITLE_H - title.size().y) / 2.0);
        let title_box = Rect::from_min_size(title_at, title.size());
        painter.galley(title_at, title, t.content_1);
        node(ui, title_box, "title", parts::TITLE, || WidgetInfo::labeled(Role::Heading, true, "Settings"));
        if mode != opened {
            let r = icon_button(ui, &painter, rect_at(o, ww - 36.0, 11.0, 24.0, 24.0), "discard-all", "undo", parts::DISCARDALL, "Discard changes", t);
            if r.clicked() {
                self.set_mode(app, opened, &mut events);
                ctx.request_repaint();
            }
        }

        // Sidebar: the search field and the section tabs.
        node(ui, side_rect, "sidebar", parts::SIDEBAR, || WidgetInfo::new(Role::GenericContainer));
        let field = rect_at(o, 7.0, 53.0, 139.0, 25.0);
        painter.rect_filled(field, CornerRadius::same(7), t.surface_0);
        draw_icon(&painter, "search", pos_at(o, 23.0, 65.5), 14.0, t.content_3);
        let clear_on = !self.query.is_empty();
        let edit_rect = rect_at(o, 35.0, 53.0, if clear_on { 84.0 } else { 102.0 }, 25.0);
        let search_id = ui.scope_id().with("search-field");
        let edit = ui
            .scope_builder(UiBuilder::new().max_rect(edit_rect).layout(Layout::left_to_right(Align::Center)), |ui| {
                ui.add(
                    TextEdit::singleline(&mut self.query)
                        .id(search_id)
                        .hint_text("Search")
                        .font(FontId::new(12.0, weight(400)))
                        .frame(egui::Frame::NONE)
                        .text_color(t.content_1)
                        .desired_width(edit_rect.width()),
                )
            })
            .inner;
        part(ui, &edit, parts::SEARCHFIELD);
        if edit.changed() {
            self.scope = None;
        }
        if std::mem::take(&mut self.focus_search) {
            edit.request_focus();
            if let Some(mut st) = TextEdit::load_state(&ctx, search_id) {
                let end = self.query.chars().count();
                st.cursor.set_char_range(Some(CCursorRange::two(CCursor::new(0), CCursor::new(end))));
                st.store(&ctx, search_id);
            }
        }
        if clear_on {
            let r = icon_button(ui, &painter, rect_at(o, 122.0, 56.5, 18.0, 18.0), "clear-search", "close", parts::CLEARSEARCH, "Clear search", t);
            if r.clicked() {
                self.query.clear();
                self.scope = None;
                ctx.memory_mut(|m| m.request_focus(search_id));
                ctx.request_repaint();
            }
        }

        let words = words_of(&self.query);
        let searching = !words.is_empty();
        let capture_hits = if searching { count_matches(Section::Capture, &words) } else { 0 };

        node(ui, rect_at(o, 7.0, TAB_Y, 139.0, 3.0 * TAB_STEP + TAB_H), "sections", parts::SECTIONS, || {
            WidgetInfo::labeled(Role::GenericContainer, true, "Settings sections")
        });
        let mut click_pick = None;
        let mut key_pick = None;
        // The selected tab is drawn first, so it is the one the Tab key reaches from the search field (F10-007).
        let selected_index = self.section.index();
        let order = std::iter::once(selected_index).chain((0..SECTIONS.len()).filter(|i| *i != selected_index));
        for i in order {
            let s = SECTIONS[i];
            let rect = rect_at(o, 7.0, TAB_Y + i as f32 * TAB_STEP, 139.0, TAB_H);
            let selected = self.section == s;
            let r = ui.interact(rect, ui.scope_id().with(("section", i)), Sense::click());
            mark_selected(ui, &r, selected);
            if r.clicked() {
                click_pick = Some(s);
            }
            if r.has_focus() {
                let dir = ui.input(|inp| if inp.key_pressed(Key::ArrowUp) { -1 } else if inp.key_pressed(Key::ArrowDown) { 1 } else { 0 });
                if dir != 0 {
                    key_pick = Some(SECTIONS[(i as i32 + dir).rem_euclid(4) as usize]);
                }
            }
            if selected {
                painter.rect_filled(rect, CornerRadius::same(4), t.wash_2);
            } else if r.hovered() {
                painter.rect_filled(rect, CornerRadius::same(4), t.wash_1);
            }
            draw_icon(&painter, s.icon(), pos2(rect.left() + 17.0, rect.center().y), 16.0, if selected { t.content_1 } else { t.content_3 });
            let label = job(ui, s.label(), 13.0, 400, t.content_1, f32::INFINITY, false);
            let label_at = pos2(rect.left() + 32.0, rect.center().y - label.size().y / 2.0);
            painter.galley(label_at, label, t.content_1);
            if r.has_focus() {
                painter.rect_stroke(rect, CornerRadius::same(4), Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
            }
            r.widget_info(|| WidgetInfo::selected(Role::Button, true, selected, s.label()));
            part(ui, &r, tab_part(s));
            if s == Section::Capture && searching && capture_hits > 0 {
                let count = capture_hits.to_string();
                let g = job(ui, &count, 10.5, 600, t.content_2, f32::INFINITY, false);
                let bw = g.size().x + 10.0;
                let badge = Rect::from_min_size(pos2(rect.right() - 8.0 - bw, rect.center().y - 8.0), vec2(bw, 16.0));
                painter.rect_filled(badge, CornerRadius::same(8), t.surface_0);
                painter.galley(pos2(badge.center().x - g.size().x / 2.0, badge.center().y - g.size().y / 2.0), g, t.content_2);
                node(ui, badge, "match-count", parts::CAPTUREMATCHCOUNT, || WidgetInfo::labeled(Role::GenericContainer, true, &count));
            }
        }
        if let Some(s) = key_pick.or(click_pick) {
            self.select(s, &words);
            if key_pick.is_some() {
                self.want_focus = Some(s);
            }
            ctx.request_repaint();
        }

        // Content: the groups of the shown section, or of every section that matches.
        let words = words_of(&self.query);
        let searching = !words.is_empty();
        let scope = if searching { self.scope } else { None };
        let shown: Vec<Section> = if searching {
            SECTIONS.into_iter().filter(|s| scope.is_none_or(|sc| sc == *s)).collect()
        } else {
            vec![self.section]
        };
        let mut blocks: Vec<(Section, Group, Vec<Row>)> = vec![];
        for s in shown {
            for g in groups(s) {
                let rows: Vec<Row> = g.rows.iter().copied().filter(|r| !searching || row_matches(r, &words)).collect();
                if !rows.is_empty() {
                    blocks.push((s, *g, rows));
                }
            }
        }
        let any = !blocks.is_empty();

        let path = screenshots_dir();
        let content_x = o.x + SIDEBAR_W + 1.0 + PAD;
        let content_r = o.x + ww - PAD;
        let card_w = content_r - content_x;
        let mut y = o.y + GROUP_TOP;
        let mut label_parts: Vec<String> = vec![];
        for (s, g, rows) in blocks {
            let heading = heading_text(s, &g, searching, scope);
            let laid: Vec<Laid> = rows.iter().map(|r| lay(ui, *r, mode, self.path_loading, t, card_w - 18.0, &path)).collect();
            let card_h = laid.iter().map(|l| l.h).sum::<f32>() + laid.len().saturating_sub(1) as f32;
            let card = Rect::from_min_size(pos2(content_x, y + HEAD_H + 7.0), vec2(card_w, card_h));
            let shots = g.id == "screenshots";
            if shots {
                node(ui, Rect::from_min_max(pos2(content_x, y), pos2(content_r, card.bottom())), "group", parts::SCREENSHOTSGROUP, || {
                    WidgetInfo::new(Role::GenericContainer)
                });
            }

            let caps = heading.to_uppercase();
            let hg = taskscape_ui::eyebrow(&painter, &caps, t.content_3);
            let head = Rect::from_min_size(pos2(content_x, y), vec2(hg.size().x, HEAD_H));
            painter.galley(pos2(head.left(), y + (HEAD_H - hg.size().y) / 2.0), hg, t.content_3);
            if shots {
                node(ui, head, "heading", parts::SCREENSHOTSHEADING, || WidgetInfo::labeled(Role::Heading, true, &heading));
            }
            label_parts.push(heading.clone());
            let line_end = if shots && mode != opened { content_r - 24.0 } else { content_r };
            if line_end > head.right() + 8.0 {
                painter.hline(Rangef::new(head.right() + 8.0, line_end), y + HEAD_H / 2.0, Stroke::new(1.0, t.edge_1));
            }
            if shots && mode != opened {
                let r = icon_button(ui, &painter, Rect::from_min_size(pos2(content_r - 20.0, y - 2.0), vec2(20.0, 20.0)), "group-discard", "undo", parts::SCREENSHOTSDISCARD, "Discard changes", t);
                if r.clicked() {
                    self.set_mode(app, opened, &mut events);
                    ctx.request_repaint();
                }
            }

            painter.rect(card, CornerRadius::same(10), t.surface_3, Stroke::new(1.0, t.edge_2), StrokeKind::Inside);
            let mut ry = card.top();
            for (i, l) in laid.iter().enumerate() {
                if i > 0 {
                    painter.hline(Rangef::new(card.left(), card.right()), ry, Stroke::new(1.0, t.edge_1));
                    ry += 1.0;
                }
                let row_rect = Rect::from_min_size(pos2(card.left(), ry), vec2(card.width(), l.h));
                label_parts.push(l.row.title.to_owned());
                self.paint_row(ui, &painter, app, &mut events, l, row_rect, mode, t, &path);
                ry += l.h;
            }
            y = card.bottom() + GROUP_GAP;
        }

        if searching && !any {
            let title_text = "No settings match";
            let sub_text = format!("Nothing here for “{}”.", self.query.trim());
            let tg = job(ui, title_text, 13.5, 600, t.content_1, f32::INFINITY, false);
            let sg = job(ui, &sub_text, 12.0, 400, t.content_3, f32::INFINITY, false);
            let cx = pane_rect.center().x;
            let ty = o.y + 150.0;
            let tr = Rect::from_min_size(pos2(cx - tg.size().x / 2.0, ty), tg.size());
            let sr = Rect::from_min_size(pos2(cx - sg.size().x / 2.0, ty + tg.size().y + 4.0), sg.size());
            node(ui, tr, "empty-title", parts::EMPTYTITLE, || WidgetInfo::labeled(Role::GenericContainer, true, title_text));
            node(ui, sr, "empty-subtitle", parts::EMPTYSUBTITLE, || WidgetInfo::labeled(Role::GenericContainer, true, &sub_text));
            node(ui, tr.union(sr), "empty", parts::SEARCHEMPTY, || {
                WidgetInfo::labeled(Role::GenericContainer, true, format!("{title_text} {sub_text}"))
            });
            painter.galley(tr.min, tg, t.content_1);
            painter.galley(sr.min, sg, t.content_3);
            label_parts.push(title_text.to_owned());
            label_parts.push(sub_text);
        }

        let pane_label = label_parts.join(" ");
        node(ui, pane_rect, "pane", parts::CONTENTPANE, || WidgetInfo::labeled(Role::GenericContainer, true, &pane_label));

        events
    }
}

struct Mount {
    app: App,
    screen: SettingsCapture,
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
    let p: SettingscaptureProps = serde_json::from_value(props).map_err(|e| e.to_string())?;
    // The harness stores the mode before the window opens; the screen itself writes nothing here.
    app.data.set_setting(MODE_KEY, &p.storedScreenshotMode).map_err(|e| e.to_string())?;
    let mut screen = SettingsCapture::new();
    screen.section = Section::named(&p.open);
    screen.path_loading = p.pathLoading;
    Ok(Box::new(Mount { app, screen }))
}
