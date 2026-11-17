//! contract: omnicui/contracts/tasklist.contract.yaml (id: app.tasklist). The task list pane: the search bar, the task tree of the active list (or the matches grouped by list), the empty states, and the footer with the done stats or the bulk bar.
use crate::app::Sheet;
use crate::state::Aside;
use crate::App;
use egui::text::{CCursor, LayoutJob, TextFormat};
use egui::text_selection::CCursorRange;
use egui::{pos2, vec2, Align, Align2, Color32, CornerRadius, EventFilter, FontFamily, FontId, Id, Key, Layout, Margin, Modifiers, Order, Pos2, Response, Rect, RichText, Role, Sense, Stroke, StrokeKind, TextEdit, UiBuilder, WidgetInfo};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use taskscape_storage::Task;
use taskscape_ui::omnicui::badge::{parts as badge_parts, BadgeProps};
use taskscape_ui::omnicui::button::{parts as button_parts, ButtonProps};
use taskscape_ui::omnicui::heading::parts as heading_parts;
use taskscape_ui::omnicui::tasklist::{parts, TasklistEvent, TasklistProps};
use taskscape_ui::widgets::badge::Badge;
use taskscape_ui::widgets::button::Button;
use taskscape_ui::widgets::emptystate::Emptystate;
use taskscape_ui::widgets::heading::Heading;
use taskscape_ui::widgets::progressbar::ProgressBar;
use taskscape_ui::widgets::separator::Separator;
use taskscape_ui::{as_part, icon, part, rem, weight, Look, Mounted};

pub type TaskId = String;

// Geometry, in px (the web's rem is 14 px).
const SEARCH_TOP: f32 = 10.0;
const WELL_H: f32 = 35.0;
const SEARCH_H: f32 = 52.0;
const FOOTER_H: f32 = 32.0;
const INDENT0: f32 = 20.0;
const INDENT_STEP: f32 = 24.0;
const CHECK: f32 = 16.0;
const GAP: f32 = 11.0;
const PAD_R: f32 = 9.0;
const MENU_W: f32 = 152.0;
const MENU_ITEM: f32 = 30.0;
const SEARCH_MENU_W: f32 = 176.0;

const SPLIT_SEARCH: &str = "app.mainwindow.splitSearch";
const SPLIT_CLOSE: &str = "app.mainwindow.splitClose";
const SPLIT_ROW: &str = "app.mainwindow.splitRow";
const SPLIT_DONE: &str = "app.mainwindow.splitFooterDoneCount";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Scope {
    List,
    Project,
    All,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Fields {
    Both,
    Title,
    Notes,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Zone {
    Before,
    After,
    Nest,
}

/// The side effects a frame asks for. They run after the frame is drawn, on the app.
enum Act {
    Select(Vec<TaskId>),
    TogglePick(TaskId),
    Preview(Option<TaskId>),
    SetDone(Vec<TaskId>, bool),
    SetDoneTree(TaskId, bool),
    MoveTo(Vec<TaskId>, TaskId),
    Promote(TaskId),
    Nest(TaskId),
    Delete(TaskId),
    AddSubtask(TaskId, String),
    Rename(TaskId, String),
    Drop(Vec<TaskId>, Option<TaskId>, Zone, TaskId),
    OpenMatch { id: TaskId, list: TaskId, project: TaskId, title: String, list_name: String },
    Draft(TaskId, String),
    CloseSplit(String),
    ToggleFilter,
    Emit(TasklistEvent),
}

/// Where a row sits, for the rows of one list or a match in another.
#[derive(Clone)]
struct Where {
    list: TaskId,
    list_name: String,
    project: TaskId,
    project_name: String,
}

#[derive(Clone)]
struct Row {
    id: TaskId,
    title: String,
    note: String,
    done: bool,
    depth: usize,
    kids: usize,
    kids_done: usize,
    expanded: bool,
    attach: usize,
    parent: bool,
    list: TaskId,
    list_name: String,
    project: TaskId,
}

impl Row {
    fn of(t: &Task, depth: usize, kids: usize, kids_done: usize, expanded: bool, w: &Where) -> Row {
        Row {
            id: t.id.clone(),
            title: t.title.clone(),
            note: plain(t.notes.as_deref().unwrap_or("")),
            done: t.done,
            depth,
            kids,
            kids_done,
            expanded,
            attach: t.attachments.len(),
            parent: t.parent_id.is_some(),
            list: w.list.clone(),
            list_name: w.list_name.clone(),
            project: w.project.clone(),
        }
    }
}

struct Group {
    heading: Option<String>,
    list_name: String,
    rows: Vec<Row>,
}

/// The screen's model for one frame, built from the app and owning its strings.
struct Shown {
    list: Option<TaskId>,
    list_name: String,
    searching: bool,
    show_headings: bool,
    groups: Vec<Group>,
    total: usize,
    done: usize,
    selection: Vec<TaskId>,
    preview: Option<TaskId>,
    split: bool,
    project_lists: Vec<(TaskId, String)>,
    first_root: Option<TaskId>,
    first_sub: Option<TaskId>,
    first_match: Option<TaskId>,
    empty: Option<(String, String, String)>,
    done_of: HashMap<TaskId, bool>,
    q: String,
}

impl Shown {
    fn flat(&self) -> Vec<&Row> {
        self.groups.iter().flat_map(|g| g.rows.iter()).collect()
    }
}

#[derive(Default, Clone, Copy)]
struct Parts {
    row: Option<&'static str>,
    check: Option<&'static str>,
    more: Option<&'static str>,
    disclosure: Option<&'static str>,
    badge: Option<&'static str>,
    attach: Option<&'static str>,
    subtask: Option<&'static str>,
    mark: Option<&'static str>,
}

impl Parts {
    /// `top` wins where it names a part.
    fn over(base: Parts, top: Parts) -> Parts {
        Parts {
            row: top.row.or(base.row),
            check: top.check.or(base.check),
            more: top.more.or(base.more),
            disclosure: top.disclosure.or(base.disclosure),
            badge: top.badge.or(base.badge),
            attach: top.attach.or(base.attach),
            subtask: top.subtask.or(base.subtask),
            mark: top.mark.or(base.mark),
        }
    }
}

/// The contract parts a row of the web's titles carries (the web's TITLE_PARTS).
fn title_parts(title: &str) -> Parts {
    let mut p = Parts::default();
    match title {
        "Post the parcel to Mum" => {
            p.row = Some(parts::ROWPOSTPARCEL);
            p.check = Some(parts::CHECKPOST);
        }
        "Book bike service" => {
            p.row = Some(parts::ROWBOOKBIKE);
            p.check = Some(parts::CHECKBIKE);
            p.badge = Some(parts::BADGEBIKE);
        }
        "Cut a spare key" => p.check = Some(parts::CHECKCUT),
        "Buy groceries for the week" => {
            p.row = Some(parts::ROWGROCERIES);
            p.check = Some(parts::CHECKGROCERIES);
            p.badge = Some(parts::BADGEGROCERIES);
            p.disclosure = Some(parts::DISCLOSURE);
        }
        "Sourdough loaf" => {
            p.check = Some(parts::CHECKSOURDOUGH);
            p.more = Some(parts::MORESOURDOUGH);
        }
        "Oat milk" => p.row = Some(parts::ROWOATMILK),
        "Buy stamps" => p.row = Some(parts::ROWNEWSUBTASK),
        "Errand number 40" => p.row = Some(parts::ROWLASTERRAND),
        "Rewrite the hero section" => p.attach = Some(parts::ATTACHMENTBADGE),
        "Return library books" => p.row = Some(parts::ROWRETURNBOOKS),
        "Trim the hedge" => p.row = Some(parts::MATCHTRIMHEDGE),
        _ => {}
    }
    p
}

/// Removes the tags of a note: the row shows its words only.
fn plain(s: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' if in_tag => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.trim().to_owned()
}

/// Sibling order: the sort order, then the id.
fn order(a: &&Task, b: &&Task) -> Ordering {
    (a.sort_order as f64).partial_cmp(&(b.sort_order as f64)).unwrap_or(Ordering::Equal).then_with(|| a.id.cmp(&b.id))
}

type Kids<'a> = HashMap<String, Vec<&'a Task>>;

/// The roots and the children of one list. A task whose parent is not in the list is a root.
fn tree<'a>(tasks: &[&'a Task]) -> (Vec<&'a Task>, Kids<'a>) {
    let ids: HashSet<String> = tasks.iter().map(|t| t.id.clone()).collect();
    let mut sorted: Vec<&'a Task> = tasks.to_vec();
    sorted.sort_by(order);
    let mut roots: Vec<&'a Task> = Vec::new();
    let mut kids: Kids<'a> = HashMap::new();
    for t in sorted {
        match t.parent_id.clone() {
            Some(p) if ids.contains(&p) => kids.entry(p).or_default().push(t),
            _ => roots.push(t),
        }
    }
    (roots, kids)
}

fn push_tree(t: &Task, depth: usize, kids: &HashMap<String, Vec<&Task>>, collapsed: &HashSet<TaskId>, show_done: bool, w: &Where, out: &mut Vec<Row>) {
    if !show_done && t.done {
        return;
    }
    let children: Vec<&Task> = kids.get(&t.id).cloned().unwrap_or_default();
    let expanded = !collapsed.contains(&t.id);
    let kids_done = children.iter().filter(|c| c.done).count();
    out.push(Row::of(t, depth, children.len(), kids_done, expanded, w));
    if expanded {
        for c in children {
            push_tree(c, depth + 1, kids, collapsed, show_done, w, out);
        }
    }
}

fn matches(t: &Task, q: &str, fields: Fields) -> bool {
    let in_title = t.title.to_lowercase().contains(q);
    let in_notes = plain(t.notes.as_deref().unwrap_or("")).to_lowercase().contains(q);
    match fields {
        Fields::Both => in_title || in_notes,
        Fields::Title => in_title,
        Fields::Notes => in_notes,
    }
}

/// The matches of a search, flattened in tree order.
fn match_walk(t: &Task, kids: &HashMap<String, Vec<&Task>>, q: &str, fields: Fields, show_done: bool, w: &Where, out: &mut Vec<Row>) {
    if (show_done || !t.done) && matches(t, q, fields) {
        out.push(Row::of(t, 0, 0, 0, false, w));
    }
    if let Some(cs) = kids.get(&t.id) {
        for c in cs {
            match_walk(c, kids, q, fields, show_done, w, out);
        }
    }
}

/// The task and everything under it.
fn subtree(all: &[Task], id: &str) -> Vec<TaskId> {
    let mut out = vec![id.to_owned()];
    let mut i = 0;
    while i < out.len() {
        let cur = out[i].clone();
        for t in all {
            if t.parent_id.as_deref() == Some(cur.as_str()) {
                out.push(t.id.clone());
            }
        }
        i += 1;
    }
    out
}

/// The tasks under one parent in one list, in order, leaving out the dragged ones.
fn siblings_of<'a>(all: &'a [Task], dragged: &HashSet<&str>, parent: Option<&str>, in_list: &str) -> Vec<&'a Task> {
    let mut v: Vec<&'a Task> = all.iter().filter(|t| t.list_id == in_list && t.parent_id.as_deref() == parent && !dragged.contains(t.id.as_str())).collect();
    v.sort_by(order);
    v
}

/// The moves a drop makes: (task, new parent, new list, new sort order). Dragged tasks whose parent is dragged too ride
/// along with it. Empty when the drop would put a task inside itself.
fn plan_drop(all: &[Task], ids: &[TaskId], target: Option<&Task>, zone: Zone, list: &str) -> Vec<(TaskId, Option<TaskId>, TaskId, f64)> {
    let dragged: HashSet<&str> = ids.iter().map(|s| s.as_str()).collect();
    let mut roots: Vec<TaskId> = Vec::new();
    for id in ids {
        let Some(t) = all.iter().find(|t| t.id == *id) else { continue };
        let parent_dragged = t.parent_id.as_deref().is_some_and(|p| dragged.contains(p));
        if !parent_dragged {
            roots.push(id.clone());
        }
    }
    if roots.is_empty() {
        return vec![];
    }
    if let Some(tg) = target {
        for r in &roots {
            if subtree(all, r).iter().any(|x| *x == tg.id) {
                return vec![];
            }
        }
    }
    let mut out: Vec<(TaskId, Option<TaskId>, TaskId, f64)> = Vec::new();
    match target {
        None => {
            let base = siblings_of(all, &dragged, None, list).last().map(|t| t.sort_order as f64).unwrap_or(0.0) + 1.0;
            for (i, id) in roots.iter().enumerate() {
                out.push((id.clone(), None, list.to_owned(), base + i as f64));
            }
        }
        Some(tg) if zone == Zone::Nest => {
            let last = siblings_of(all, &dragged, Some(tg.id.as_str()), tg.list_id.as_str()).last().map(|t| t.sort_order as f64).unwrap_or(0.0);
            for (i, id) in roots.iter().enumerate() {
                out.push((id.clone(), Some(tg.id.clone()), tg.list_id.clone(), last + 1.0 + i as f64));
            }
        }
        Some(tg) => {
            let sib = siblings_of(all, &dragged, tg.parent_id.as_deref(), tg.list_id.as_str());
            let idx = sib.iter().position(|t| t.id == tg.id).unwrap_or(sib.len());
            let at = (idx + usize::from(zone == Zone::After)).min(sib.len());
            let mut seq: Vec<TaskId> = sib[..at].iter().map(|t| t.id.clone()).collect();
            seq.extend(roots.iter().cloned());
            seq.extend(sib[at..].iter().map(|t| t.id.clone()));
            for (i, id) in seq.iter().enumerate() {
                let to_order = (i + 1) as f64;
                let needs = match all.iter().find(|t| t.id == *id) {
                    None => true,
                    Some(t) => dragged.contains(id.as_str()) || (t.sort_order as f64) != to_order,
                };
                if needs {
                    out.push((id.clone(), tg.parent_id.clone(), tg.list_id.clone(), to_order));
                }
            }
        }
    }
    out
}

/// Moves a task and, when its list changes, the subtasks under it.
pub fn move_tree(app: &mut App, id: &str, parent: Option<&str>, list: &str, order: Option<f64>) {
    if let Err(e) = app.data.move_task(id, parent, Some(list), order) {
        app.refused(e);
        return;
    }
    let kids: Vec<TaskId> = app.data.children_of(id).iter().map(|t| t.id.clone()).collect();
    for k in kids {
        move_tree(app, &k, Some(id), list, None);
    }
}

fn letter_of(key: Key, shift: bool) -> Option<char> {
    const LETTERS: [Key; 26] = [
        Key::A, Key::B, Key::C, Key::D, Key::E, Key::F, Key::G, Key::H, Key::I, Key::J, Key::K, Key::L, Key::M, Key::N, Key::O, Key::P, Key::Q, Key::R, Key::S,
        Key::T, Key::U, Key::V, Key::W, Key::X, Key::Y, Key::Z,
    ];
    LETTERS.iter().position(|k| *k == key).map(|i| {
        let c = (b'a' + i as u8) as char;
        if shift {
            c.to_ascii_uppercase()
        } else {
            c
        }
    })
}

/// The platform modifier: Command on macOS, Control elsewhere. Some event sources set only the Control flag on Linux
/// and Windows, so either flag counts there.
fn platform(m: Modifiers) -> bool {
    if cfg!(target_os = "macos") {
        m.command
    } else {
        m.command || m.ctrl
    }
}

#[derive(Default)]
struct Keys {
    m: Modifiers,
    plat: bool,
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    tab: bool,
    esc: bool,
    f2: bool,
    a: bool,
    n: bool,
    f: bool,
    enter: bool,
    space: bool,
    letter: Option<char>,
}

fn read_keys(ui: &egui::Ui) -> Keys {
    ui.input(|i| {
        let typed = i.events.iter().find_map(|e| match e {
            egui::Event::Text(t) => t.chars().next().filter(|c| c.is_alphanumeric()),
            _ => None,
        });
        let letter = typed.or_else(|| {
            i.events.iter().find_map(|e| match e {
                egui::Event::Key { key, pressed: true, modifiers, .. } if !modifiers.command && !modifiers.alt && !modifiers.ctrl => letter_of(*key, modifiers.shift),
                _ => None,
            })
        });
        Keys {
            m: i.modifiers,
            plat: platform(i.modifiers),
            up: i.key_pressed(Key::ArrowUp),
            down: i.key_pressed(Key::ArrowDown),
            left: i.key_pressed(Key::ArrowLeft),
            right: i.key_pressed(Key::ArrowRight),
            tab: i.key_pressed(Key::Tab),
            esc: i.key_pressed(Key::Escape),
            f2: i.key_pressed(Key::F2),
            a: i.key_pressed(Key::A),
            n: i.key_pressed(Key::N),
            f: i.key_pressed(Key::F),
            enter: i.key_pressed(Key::Enter),
            space: i.key_pressed(Key::Space),
            letter,
        }
    })
}

fn body_id(id: &str) -> Id {
    Id::new(("tasklist.body", id))
}
fn check_id(id: &str) -> Id {
    Id::new(("tasklist.check", id))
}
pub fn search_id() -> Id {
    Id::new("tasklist.search")
}
fn rename_id() -> Id {
    Id::new("tasklist.rename")
}
fn composer_id() -> Id {
    Id::new("tasklist.composer")
}

fn info(role: Role, label: &str, value: &str) -> WidgetInfo {
    let mut i = WidgetInfo::labeled(role, true, label);
    i.current_text_value = Some(value.to_owned());
    i
}

fn paint_icon(p: &egui::Painter, at: Pos2, name: &str, px: f32, color: Color32) {
    p.text(at, Align2::CENTER_CENTER, icon(name, px).text(), FontId::new(px, FontFamily::Name("Icons".into())), color);
}

fn select_all(ctx: &egui::Context, id: Id, text: &str) {
    if let Some(mut state) = TextEdit::load_state(ctx, id) {
        state.cursor.set_char_range(Some(CCursorRange::two(CCursor::new(0), CCursor::new(text.chars().count()))));
        state.store(ctx, id);
    }
}

/// A bare single-line field at a rect, named and valued for the accessibility tree.
#[allow(clippy::too_many_arguments)]
fn text_field(ui: &mut egui::Ui, rect: Rect, id: Id, text: &mut String, hint: &str, label: &str, part_id: Option<&str>, px: f32, w: u16, color: Color32, hint_color: Color32) -> Response {
    let mut child = ui.new_child(UiBuilder::new().max_rect(rect).layout(Layout::left_to_right(Align::Center)));
    let resp = child.add(
        TextEdit::singleline(&mut *text)
            .id(id)
            .frame(egui::Frame::NONE)
            .margin(Margin::ZERO)
            .font(FontId::new(px, weight(w)))
            .text_color(color)
            .hint_text(RichText::new(hint).color(hint_color))
            .desired_width(rect.width().max(1.0)),
    );
    let value = text.clone();
    resp.widget_info(|| info(Role::TextInput, label, &value));
    ui.ctx().accesskit_node_builder(resp.id, |node| node.set_value(value.clone()));
    if let Some(p) = part_id {
        part(&child, &resp, p);
    }
    resp
}

/// A library button, or a plain one when the part has no name in this pane.
fn button(ui: &mut egui::Ui, look: Look, part_id: Option<&str>, props: serde_json::Value) -> (bool, Rect) {
    let p: ButtonProps = serde_json::from_value(props).expect("button props");
    let out = ui.scope(|ui| match part_id {
        Some(id) => as_part(ui, &[(button_parts::BUTTON, id)], |ui| Button::new(p).ui(ui, look)),
        None => Button::new(p).ui(ui, look),
    });
    (!out.inner.is_empty(), out.response.rect)
}

/// A library badge with a count, tagged with a part when the row names one.
fn badge(ui: &mut egui::Ui, look: Look, part_id: Option<&str>, value: &str, glyph: bool) {
    let p: BadgeProps = serde_json::from_value(serde_json::json!({
        "tone": "outline",
        "size": "md",
        "shape": "pill",
        "icon": glyph,
        "children": value,
        "visible": true,
    }))
    .expect("badge props");
    let _ = ui.scope(|ui| match part_id {
        Some(id) => as_part(ui, &[(badge_parts::BADGE, id)], |ui| Badge::new(p).ui(ui, look)),
        None => Badge::new(p).ui(ui, look),
    });
}

/// A text run whose value is the text a test reads (the uppercase of the web is a style, shown here).
fn status(ui: &mut egui::Ui, part_id: Option<&str>, value: &str, shown: &str, px: f32, w: u16, color: Color32) {
    let galley = ui.painter().layout_no_wrap(shown.to_owned(), FontId::new(px, weight(w)), color);
    let (rect, resp) = ui.allocate_exact_size(galley.size(), Sense::hover());
    resp.widget_info(|| info(Role::Status, value, value));
    if let Some(p) = part_id {
        part(ui, &resp, p);
    }
    ui.painter().galley(rect.min, galley, color);
}

/// A status that is out of sight: it has no size (the part reads as not visible) but keeps its text, as the web keeps
/// both footers in the page.
fn hidden_status(ui: &mut egui::Ui, at: Pos2, part_id: &str, value: &str) {
    let resp = ui.interact(Rect::from_min_size(at, vec2(0.0, 0.0)), Id::new(("tasklist.hidden", part_id)), Sense::hover());
    resp.widget_info(|| info(Role::Status, value, value));
    part(ui, &resp, part_id);
}

/// A word-wrapped-free title or note line, with the query's matches filled. Returns the first match's span in chars.
fn marked_job(text: &str, q: &str, font: &FontId, color: Color32, strike: bool, hl: Color32) -> (LayoutJob, Option<(usize, usize, String)>) {
    let mut job = LayoutJob::default();
    let mut first: Option<(usize, usize, String)> = None;
    let lower = text.to_lowercase();
    let mut spans: Vec<(usize, usize)> = Vec::new();
    if !q.is_empty() && lower.len() == text.len() {
        spans = lower.match_indices(q).map(|(i, m)| (i, i + m.len())).collect();
    }
    let fmt = |bg: Option<Color32>| TextFormat {
        font_id: font.clone(),
        color,
        strikethrough: if strike { Stroke::new(1.0, color) } else { Stroke::NONE },
        background: bg.unwrap_or(Color32::TRANSPARENT),
        ..Default::default()
    };
    let mut at = 0usize;
    let mut chars = 0usize;
    for (s, e) in spans {
        if s > at {
            let piece = &text[at..s];
            job.append(piece, 0.0, fmt(None));
            chars += piece.chars().count();
        }
        let piece = &text[s..e];
        if first.is_none() {
            first = Some((chars, chars + piece.chars().count(), piece.to_owned()));
        }
        job.append(piece, 0.0, fmt(Some(hl)));
        chars += piece.chars().count();
        at = e;
    }
    if at < text.len() {
        job.append(&text[at..], 0.0, fmt(None));
    }
    (job, first)
}

fn mark_rect(g: &egui::Galley, start: usize, end: usize) -> Rect {
    let a = g.pos_from_cursor(CCursor::new(start));
    let b = g.pos_from_cursor(CCursor::new(end));
    Rect::from_min_max(pos2(a.left(), a.top()), pos2(b.left().max(a.left() + 1.0), a.bottom()))
}

/// One row of a menu: a hover fill, an optional check, icon, label and submenu chevron, as the web's MenuItem.
#[allow(clippy::too_many_arguments)]
fn menu_row(ui: &mut egui::Ui, look: Look, rect: Rect, key: &str, label: &str, icon_name: Option<&str>, danger: bool, checked: Option<bool>, submenu: bool, part_id: Option<&str>) -> Response {
    let t = look.t;
    let resp = ui.interact(rect, Id::new(("tasklist.menurow", key)), Sense::click());
    let painter = ui.painter().clone();
    if resp.hovered() {
        painter.rect_filled(rect.shrink2(vec2(4.0, 0.0)), CornerRadius::same(4), t.wash_2);
    }
    let ink = if danger { t.danger_500 } else { t.content_1 };
    let cy = rect.center().y;
    let mut x = rect.left() + 8.75;
    if let Some(on) = checked {
        if on {
            paint_icon(&painter, pos2(x + 7.0, cy), "check", 14.0, t.content_1);
        }
        x += 14.0 + 7.0;
    }
    if let Some(name) = icon_name {
        paint_icon(&painter, pos2(x + 7.5, cy), name, 15.0, if danger { t.danger_500 } else { t.content_2 });
        x += 15.0 + 7.0;
    }
    painter.text(pos2(x, cy), Align2::LEFT_CENTER, label, FontId::new(13.0, weight(400)), ink);
    if submenu {
        paint_icon(&painter, pos2(rect.right() - 12.0, cy), "chevron_right", 15.0, t.content_2);
    }
    match checked {
        Some(on) => {
            resp.widget_info(|| WidgetInfo::selected(Role::CheckBox, true, on, label));
            ui.ctx().accesskit_node_builder(resp.id, |node| {
                node.set_role(Role::Button);
                node.set_selected(on);
            });
        }
        None => {
            resp.widget_info(|| WidgetInfo::labeled(Role::Button, true, label));
        }
    }
    if let Some(p) = part_id {
        part(ui, &resp, p);
    }
    resp
}

fn zone_of(p: Pos2, r: Rect) -> Zone {
    let y = (p.y - r.top()) / r.height().max(1.0);
    if y < 0.3 {
        Zone::Before
    } else if y > 0.7 {
        Zone::After
    } else {
        Zone::Nest
    }
}

/// The task list pane (contract app.tasklist): one pane of the window, showing the list in `app.ui.panes[pane]`.
pub struct TaskList {
    /// Set by a window around the pane: the rows a test names by their text get these parts on their
    /// title button, as (text the title or note contains, part).
    pub claims: Vec<(&'static str, &'static str)>,
    /// Set by a window around the pane: the first root row's whole box carries this part, and title
    /// buttons read as the title alone (the window's tests name the title, the row, and its parts apart).
    pub row_box: Option<(&'static str, &'static str)>,
    pane: usize,
    query: String,
    scope: Scope,
    fields: Fields,
    options_open: bool,
    collapsed: HashSet<TaskId>,
    renaming: Option<TaskId>,
    rename_text: String,
    rename_pending: bool,
    rename_armed: bool,
    composing: Option<TaskId>,
    composer_text: String,
    composer_pending: bool,
    composer_armed: bool,
    menu_for: Option<TaskId>,
    menu_at: Option<Pos2>,
    submenu_open: bool,
    bulk_move_open: bool,
    anchor: Option<TaskId>,
    match_at: Option<usize>,
    dragging: Vec<TaskId>,
    drop_at: Option<(TaskId, Zone)>,
    rows_now: Vec<(TaskId, Rect)>,
    rows_prev: Vec<(TaskId, Rect)>,
    /// Each parent's disclosure button box, this frame and the last: the arrows on a focused disclosure
    /// collapse and expand its row.
    disclosures_now: Vec<(TaskId, Rect)>,
    disclosures_prev: Vec<(TaskId, Rect)>,
    more_rects: Vec<(TaskId, Rect)>,
    options_btn: Option<Rect>,
    bulk_btn: Option<Rect>,
    open_panels: Vec<Rect>,
    /// The pane's rect this frame: a submenu opens on the other side when the right has no room.
    pane_rect: Rect,
    /// Whether the search field had the focus at the end of the last frame.
    search_had_focus: bool,
}

impl TaskList {
    pub fn new(pane: usize) -> Self {
        Self {
            claims: vec![],
            row_box: None,
            pane,
            query: String::new(),
            scope: Scope::List,
            fields: Fields::Both,
            options_open: false,
            collapsed: HashSet::new(),
            renaming: None,
            rename_text: String::new(),
            rename_pending: false,
            rename_armed: false,
            composing: None,
            composer_text: String::new(),
            composer_pending: false,
            composer_armed: false,
            menu_for: None,
            menu_at: None,
            submenu_open: false,
            bulk_move_open: false,
            anchor: None,
            match_at: None,
            dragging: Vec::new(),
            drop_at: None,
            rows_now: Vec::new(),
            rows_prev: Vec::new(),
            disclosures_now: Vec::new(),
            disclosures_prev: Vec::new(),
            more_rects: Vec::new(),
            options_btn: None,
            bulk_btn: None,
            open_panels: Vec::new(),
            pane_rect: Rect::ZERO,
            search_had_focus: false,
        }
    }

    /// One frame: draws the pane and returns the events it raised.
    pub fn show(&mut self, app: &mut App, ui: &mut egui::Ui, look: Look) -> Vec<TasklistEvent> {
        let ctx = ui.ctx().clone();
        let full = ui.available_rect_before_wrap();
        self.pane_rect = full;
        ui.allocate_rect(full, Sense::hover());
        if ui.input(|i| i.pointer.primary_pressed() && i.pointer.interact_pos().is_some_and(|p| full.contains(p))) {
            app.ui.active_pane = self.pane;
        }
        self.rows_prev = std::mem::take(&mut self.rows_now);
        self.disclosures_prev = std::mem::take(&mut self.disclosures_now);
        self.more_rects.clear();
        self.open_panels.clear();
        self.options_btn = None;
        self.bulk_btn = None;
        let s = self.build(app);
        let mut acts: Vec<Act> = Vec::new();

        // Escape closes an open menu before anything else reads it.
        if (self.menu_for.is_some() || self.options_open || self.bulk_move_open) && ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
            self.close_menus();
        }
        // The search field: Enter and Shift+Enter step through the matches, Escape clears it and leaves it. Escape can
        // take the focus away before the frame runs, so the field that had it last frame still gets that Escape.
        let escape_now = ui.input(|i| i.key_pressed(Key::Escape));
        if ctx.memory(|m| m.has_focus(search_id())) || (self.search_had_focus && escape_now) {
            let shift_enter = ui.input_mut(|i| i.consume_key(Modifiers::SHIFT, Key::Enter));
            let enter = ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Enter));
            let esc = ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape));
            if shift_enter {
                self.step(&s, -1, &ctx, &mut acts);
            } else if enter {
                self.step(&s, 1, &ctx, &mut acts);
            }
            if esc {
                self.query.clear();
                self.match_at = None;
                ctx.memory_mut(|m| m.surrender_focus(search_id()));
            }
        }

        let search_rect = Rect::from_min_size(full.min, vec2(full.width(), SEARCH_H));
        let footer_rect = Rect::from_min_max(pos2(full.left(), full.bottom() - FOOTER_H), full.max);
        let body_rect = Rect::from_min_max(pos2(full.left(), search_rect.bottom()), pos2(full.right(), footer_rect.top()));
        ui.painter().rect_filled(full, CornerRadius::same(0), look.t.surface_2);
        self.search_bar(ui, look, &s, search_rect, &mut acts);
        self.body(ui, look, &s, body_rect, &mut acts);
        self.footer(ui, look, &s, footer_rect, &mut acts);
        self.row_keys(ui, &s, &mut acts);
        self.drag_update(ui, body_rect, &s, &mut acts);
        self.menus(&ctx, look, &s, &mut acts);

        // A press outside the open menus, and outside what opens them, closes them.
        if let Some(p) = ui.input(|i| if i.pointer.primary_pressed() { i.pointer.interact_pos() } else { None }) {
            let inside = self.open_panels.iter().any(|r| r.contains(p));
            let trigger = self.more_rects.iter().any(|(_, r)| r.contains(p)) || self.options_btn.is_some_and(|r| r.contains(p)) || self.bulk_btn.is_some_and(|r| r.contains(p));
            if !inside && !trigger {
                self.close_menus();
            }
        }

        if !acts.is_empty() {
            ctx.request_repaint();
        }
        self.search_had_focus = ctx.memory(|m| m.has_focus(search_id()));
        let mut events = Vec::new();
        self.apply(app, acts, &mut events);
        events
    }

    fn close_menus(&mut self) {
        self.menu_for = None;
        self.menu_at = None;
        self.options_open = false;
        self.bulk_move_open = false;
        self.submenu_open = false;
    }

    /// The part for this pane: the main part on the left (or only) pane, the split part on the right.
    fn pp(&self, main: &'static str, split: Option<&'static str>) -> Option<&'static str> {
        if self.pane == 0 {
            Some(main)
        } else {
            split
        }
    }

    /// Builds the frame's model from the app: the rows of the list, or the matches grouped by list.
    fn build(&self, app: &App) -> Shown {
        let list = app.ui.panes.get(self.pane).cloned().flatten();
        let project = app.ui.project.clone();
        let list_rec = list.as_deref().and_then(|l| app.data.list(l));
        let list_name = list_rec.map(|l| l.name.clone()).unwrap_or_default();
        let project_name = project.as_deref().and_then(|p| app.data.project(p)).map(|p| p.name.clone()).unwrap_or_default();
        let show_done = app.data.settings.show_completed;
        let q = self.query.trim().to_lowercase();
        let searching = !q.is_empty();
        let list_tasks: Vec<&Task> = list.as_deref().map(|l| app.data.tasks_of(l)).unwrap_or_default();
        let here = Where {
            list: list.clone().unwrap_or_default(),
            list_name: list_name.clone(),
            project: project.clone().unwrap_or_default(),
            project_name: project_name.clone(),
        };
        let (roots, kids) = tree(&list_tasks);
        let mut groups: Vec<Group> = Vec::new();
        if !searching {
            let mut rows = Vec::new();
            for r in &roots {
                push_tree(r, 0, &kids, &self.collapsed, show_done, &here, &mut rows);
            }
            groups.push(Group { heading: None, list_name: list_name.clone(), rows });
        } else if self.scope == Scope::List {
            let mut rows = Vec::new();
            for r in &roots {
                match_walk(r, &kids, &q, self.fields, show_done, &here, &mut rows);
            }
            groups.push(Group { heading: None, list_name: list_name.clone(), rows });
        } else {
            let mut projects: Vec<(TaskId, String)> = Vec::new();
            if let Some(p) = &project {
                projects.push((p.clone(), project_name.clone()));
            }
            if self.scope == Scope::All {
                for p in &app.data.projects {
                    if Some(&p.id) != project.as_ref() {
                        projects.push((p.id.clone(), p.name.clone()));
                    }
                }
            }
            for (pid, pname) in projects {
                for l in app.data.lists_of(&pid) {
                    let tasks = app.data.tasks_of(&l.id);
                    let (lr, lk) = tree(&tasks);
                    let w = Where { list: l.id.clone(), list_name: l.name.clone(), project: pid.clone(), project_name: pname.clone() };
                    let mut rows = Vec::new();
                    for r in &lr {
                        match_walk(r, &lk, &q, self.fields, show_done, &w, &mut rows);
                    }
                    if !rows.is_empty() {
                        let heading = if self.scope == Scope::All { format!("{} · {}", pname, l.name) } else { l.name.clone() };
                        groups.push(Group { heading: Some(heading), list_name: l.name.clone(), rows });
                    }
                }
            }
        }

        let total = list_tasks.len();
        let done = list_tasks.iter().filter(|t| t.done).count();
        let (first_root, first_sub, first_match, visible_count) = {
            let visible: Vec<&Row> = groups.iter().flat_map(|g| g.rows.iter()).collect();
            (
                visible.iter().find(|r| r.depth == 0).map(|r| r.id.clone()),
                visible.iter().find(|r| r.depth > 0).map(|r| r.id.clone()),
                if searching { visible.first().map(|r| r.id.clone()) } else { None },
                visible.len(),
            )
        };
        let empty = if visible_count == 0 {
            if searching {
                Some(("search_off".to_owned(), format!("No matches in {list_name}"), "Try a different search or scope".to_owned()))
            } else if list_rec.is_some() && total == 0 {
                let hint = if cfg!(target_os = "macos") { "Press ⌘N to add a task" } else { "Press Ctrl+N to add a task" };
                Some(("landscape".to_owned(), "Nothing here yet".to_owned(), hint.to_owned()))
            } else {
                None
            }
        } else {
            None
        };
        let project_lists: Vec<(TaskId, String)> = project.as_deref().map(|p| app.data.lists_of(p).iter().map(|l| (l.id.clone(), l.name.clone())).collect()).unwrap_or_default();
        let done_of: HashMap<TaskId, bool> = app.data.tasks.iter().map(|t| (t.id.clone(), t.done)).collect();
        Shown {
            list,
            list_name,
            searching,
            show_headings: searching && self.scope != Scope::List,
            groups,
            total,
            done,
            selection: app.ui.selection.clone(),
            preview: app.ui.preview.clone(),
            split: app.ui.split_on(),
            project_lists,
            first_root,
            first_sub,
            first_match,
            empty,
            done_of,
            q,
        }
    }

    /// The parts a row carries: the first root's row, check and More; the title's own; the first subtask's and the first match's.
    fn parts_of(&self, row: &Row, s: &Shown) -> Parts {
        let is_root = s.first_root.as_deref() == Some(row.id.as_str());
        let first = if is_root {
            Parts {
                row: self.pp(parts::ROW, Some(SPLIT_ROW)),
                check: self.pp(parts::CHECK, None),
                more: self.pp(parts::MORE, None),
                ..Default::default()
            }
        } else {
            Parts::default()
        };
        let title = if self.pane == 0 { title_parts(&row.title) } else { Parts::default() };
        let mut out = if s.searching { Parts::over(first, title) } else { Parts::over(title, first) };
        if self.pane == 0 && s.first_sub.as_deref() == Some(row.id.as_str()) {
            out.subtask = Some(parts::SUBTASK);
        }
        if self.pane == 0 && s.first_match.as_deref() == Some(row.id.as_str()) {
            out.mark = Some(parts::MATCHMARK);
        }
        out
    }

    /// Steps the focus through the matches, in tree order, wrapping; each step previews the match.
    fn step(&mut self, s: &Shown, dir: i32, ctx: &egui::Context, acts: &mut Vec<Act>) {
        let flat = s.flat();
        let n = flat.len();
        if n == 0 {
            return;
        }
        let at = if dir > 0 {
            self.match_at.map_or(0, |m| (m + 1) % n)
        } else {
            match self.match_at {
                Some(m) if m > 0 => m - 1,
                _ => n - 1,
            }
        };
        self.match_at = Some(at);
        let row = flat[at];
        ctx.memory_mut(|m| m.request_focus(body_id(&row.id)));
        acts.push(Act::Preview(Some(row.id.clone())));
        acts.push(Act::Emit(TasklistEvent::TaskPreviewed { title: row.title.clone() }));
    }

    /// Starts a search with a typed character, with the search field taking the keyboard (a printable key
    /// pressed while the window's focus is outside a field).
    /// The tasks being dragged out of this pane, if a drag is on.
    pub fn dragged(&self) -> &[TaskId] {
        &self.dragging
    }

    /// Hands a drag to this pane (the pointer moved onto it from the other pane), or ends it (empty).
    pub fn set_dragged(&mut self, ids: Vec<TaskId>) {
        self.dragging = ids;
    }

    pub fn start_search(&mut self, ctx: &egui::Context, c: char) {
        self.query = c.to_string();
        self.match_at = None;
        ctx.memory_mut(|m| m.request_focus(search_id()));
    }

    fn start_rename(&mut self, id: &str, title: &str) {
        self.renaming = Some(id.to_owned());
        self.rename_text = title.to_owned();
        self.rename_pending = true;
        self.rename_armed = false;
    }

    fn search_bar(&mut self, ui: &mut egui::Ui, look: Look, s: &Shown, rect: Rect, acts: &mut Vec<Act>) {
        let t = look.t;
        let ctx = ui.ctx().clone();
        let well = Rect::from_min_max(pos2(rect.left() + 14.0, rect.top() + SEARCH_TOP), pos2(rect.right() - 14.0, rect.top() + SEARCH_TOP + WELL_H));
        ui.painter().rect_filled(well, CornerRadius::same(7), t.surface_0);
        let focused = ctx.memory(|m| m.has_focus(search_id()));
        if focused {
            ui.painter().rect_stroke(well.expand(2.0), CornerRadius::same(9), Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
        }
        paint_icon(ui.painter(), pos2(well.left() + 18.0, well.center().y), "search", 18.0, t.content_2);

        let inner = Rect::from_min_max(pos2(well.left() + 9.0 + 18.0 + 5.0, well.top()), pos2(well.right() - 9.0, well.bottom()));
        let mut ch = ui.new_child(UiBuilder::new().max_rect(inner).layout(Layout::right_to_left(Align::Center)));
        ch.spacing_mut().item_spacing.x = 5.0;

        if s.split {
            let (pressed, _) = button(
                &mut ch,
                look,
                self.pp(parts::CLOSESPLIT, Some(SPLIT_CLOSE)),
                serde_json::json!({"variant": "ghost", "size": "sm", "tone": "muted", "shape": "icon", "icon": "close", "iconSize": 16, "ariaLabel": "Close split", "width": "content", "label": ""}),
            );
            if pressed {
                acts.push(Act::CloseSplit(s.list_name.clone()));
            }
        }
        let (pressed, _) = button(
            &mut ch,
            look,
            self.pp(parts::NEWTASK, None),
            serde_json::json!({"variant": "ghost", "size": "sm", "tone": "muted", "shape": "icon", "icon": "add", "iconSize": 18, "ariaLabel": "New task", "width": "content", "label": ""}),
        );
        if pressed {
            acts.push(Act::Draft(s.list.clone().unwrap_or_default(), s.list_name.clone()));
        }
        let sep = ch.allocate_exact_size(vec2(1.0, 16.0), Sense::hover()).0;
        ch.painter().rect_filled(sep, CornerRadius::same(0), t.edge_2);
        let (pressed, opt_rect) = button(
            &mut ch,
            look,
            self.pp(parts::SEARCHOPTIONS, None),
            serde_json::json!({"variant": "ghost", "size": "sm", "tone": "muted", "shape": "icon", "icon": "tune", "iconSize": 16, "ariaLabel": "Search options", "width": "content", "label": ""}),
        );
        if pressed {
            self.options_open = !self.options_open;
        }
        self.options_btn = Some(opt_rect);
        if s.searching {
            let (pressed, _) = button(
                &mut ch,
                look,
                self.pp(parts::CLEARSEARCH, None),
                serde_json::json!({"variant": "ghost", "size": "sm", "tone": "muted", "shape": "icon", "icon": "close", "iconSize": 14, "ariaLabel": "Clear search", "width": "content", "label": ""}),
            );
            if pressed {
                self.query.clear();
                self.match_at = None;
                ctx.memory_mut(|m| m.request_focus(search_id()));
            }
            let n = s.flat().len().to_string();
            status(&mut ch, self.pp(parts::MATCHCOUNT, None), &n, &n, 12.0, 600, t.content_2);
        }

        // The field takes the well's room left of the buttons, measured from the buttons that were drawn.
        let used = ch.min_rect();
        let field_rect = Rect::from_min_max(inner.min, pos2((used.left() - 5.0).max(inner.left() + 20.0), inner.bottom()));
        let placeholder = format!("Search {}", s.list_name);
        let search_part = self.pp(parts::SEARCH, Some(SPLIT_SEARCH));
        let resp = text_field(
            ui,
            field_rect,
            search_id(),
            &mut self.query,
            &placeholder,
            &placeholder,
            search_part,
            14.0,
            400,
            t.content_1,
            t.content_3,
        );
        if resp.changed() {
            self.match_at = None;
        }
    }

    fn body(&mut self, ui: &mut egui::Ui, look: Look, s: &Shown, rect: Rect, acts: &mut Vec<Act>) {
        let mut ch = ui.new_child(UiBuilder::new().max_rect(rect).layout(Layout::top_down(Align::Min)));
        let region = ch.interact(rect, Id::new("tasklist.region"), Sense::click());
        region.widget_info(|| WidgetInfo::new(Role::GenericContainer));
        if let Some(p) = self.pp(parts::SCROLLAREA, None) {
            part(&ch, &region, p);
        }
        let blank_click = region.clicked();
        egui::ScrollArea::vertical().id_salt("tasklist.scroll").auto_shrink([false, false]).scroll_source(egui::scroll_area::ScrollSource { drag: egui::scroll_area::DragScroll::Never, ..Default::default() }).show(&mut ch, |ui| {
            for (gi, g) in s.groups.iter().enumerate() {
                if let Some(h) = &g.heading {
                    self.heading(ui, look, gi, g, h);
                }
                for row in &g.rows {
                    self.draw_row(ui, look, s, row, acts);
                    if self.composing.as_deref() == Some(row.id.as_str()) {
                        self.composer(ui, look, row, acts);
                    }
                }
            }
            if let Some((icon_name, title, sub)) = &s.empty {
                self.empty(ui, look, icon_name, title, sub);
            }
            ui.add_space(8.0);
        });
        if blank_click {
            acts.push(Act::Select(vec![]));
            acts.push(Act::Preview(None));
        }
    }

    fn heading(&self, ui: &mut egui::Ui, look: Look, gi: usize, g: &Group, text: &str) {
        let part_id = if gi == 0 {
            self.pp(parts::GROUPHEADING, None)
        } else if g.list_name == "Reading" {
            self.pp(parts::GROUPHEADINGREADING, None)
        } else {
            None
        };
        let renames: Vec<(&str, &str)> = match part_id {
            Some(p) => vec![(heading_parts::TITLE, p)],
            None => vec![],
        };
        ui.add_space(9.0);
        ui.horizontal(|ui| {
            ui.add_space(14.0);
            as_part(ui, &renames, |ui| Heading::new(serde_json::from_value(serde_json::json!({"size": "sm", "overflow": "truncate", "content": text})).expect("heading props")).ui(ui, look));
        });
        ui.add_space(4.0);
    }

    fn empty(&self, ui: &mut egui::Ui, look: Look, icon_name: &str, title: &str, sub: &str) {
        let r = ui
            .scope(|ui| {
                let p: taskscape_ui::omnicui::emptystate::EmptystateProps = serde_json::from_value(serde_json::json!({
                    "scope": "list",
                    "icon": icon_name,
                    "iconSize": 24,
                    "title": title,
                    "subtitle": sub,
                }))
                .expect("emptystate props");
                Emptystate::new(p).ui(ui, look);
            })
            .response
            .rect;
        let resp = ui.interact(r, Id::new("tasklist.empty"), Sense::hover());
        let joined = format!("{title} {sub}");
        resp.widget_info(|| info(Role::GenericContainer, title, &joined));
        if let Some(p) = self.pp(parts::EMPTYSTATE, None) {
            part(ui, &resp, p);
        }
    }

    /// One row of the tree: the round check, the title and note, the counts, the disclosure and the More button.
    fn draw_row(&mut self, ui: &mut egui::Ui, look: Look, s: &Shown, row: &Row, acts: &mut Vec<Act>) {
        let t = look.t;
        let pt = self.parts_of(row, s);
        let picked = s.selection.contains(&row.id);
        let previewed = s.preview.as_deref() == Some(row.id.as_str());
        let tinted = picked || previewed;
        let indent = INDENT0 + INDENT_STEP * row.depth as f32;
        let has_note = !row.note.is_empty();
        let height = if has_note { rem(3.3) } else { rem(2.25) };
        let width = ui.available_width();
        let (rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
        self.rows_now.push((row.id.clone(), rect));
        let drop = match &self.drop_at {
            Some((id, z)) if *id == row.id => Some(*z),
            _ => None,
        };
        let radius = CornerRadius::same(5);
        let painter = ui.painter().clone();
        let hovered = ui.rect_contains_pointer(rect);
        if tinted || drop == Some(Zone::Nest) {
            painter.rect_filled(rect, radius, t.selection_1);
        } else if hovered {
            painter.rect_filled(rect, radius, t.wash_1);
        }
        if drop == Some(Zone::Nest) {
            painter.rect_stroke(rect, radius, Stroke::new(2.0, t.accent_500), StrokeKind::Inside);
        }
        if drop == Some(Zone::Before) {
            painter.rect_filled(Rect::from_min_size(rect.left_top(), vec2(width, 2.0)), CornerRadius::same(0), t.accent_500);
        }
        if drop == Some(Zone::After) {
            painter.rect_filled(Rect::from_min_size(pos2(rect.left(), rect.bottom() - 2.0), vec2(width, 2.0)), CornerRadius::same(0), t.accent_500);
        }

        let value = if row.note.is_empty() { row.title.clone() } else { format!("{} {}", row.title, row.note) };
        let mut pt = pt;
        if let Some((_, p)) = self.claims.iter().find(|(text, _)| value.contains(text)) {
            pt.row = Some(p);
        }
        if let Some((_, p)) = self.row_box.filter(|(title, _)| row.title == *title) {
            let c = ui.interact(rect, Id::new(("tasklist.rowbox", row.id.as_str())), Sense::hover());
            c.widget_info(|| info(Role::GenericContainer, &row.title, &value));
            part(ui, &c, p);
        }
        // Inside a window, a title button reads as its title alone.
        let value = if self.row_box.is_some() { row.title.clone() } else { value };
        if let Some(p) = pt.subtask {
            let c = ui.interact(rect, Id::new(("tasklist.sub", row.id.as_str())), Sense::hover());
            c.widget_info(|| info(Role::GenericContainer, &row.title, &value));
            part(ui, &c, p);
        }

        let (alt, plat, shift, enter_key, space_key) = ui.input(|i| (i.modifiers.alt, platform(i.modifiers), i.modifiers.shift, i.key_pressed(Key::Enter), i.key_pressed(Key::Space)));

        // The round check: a press picks the row, Option (Alt) with a press marks it done.
        let check_rect = Rect::from_min_size(pos2(rect.left() + indent, rect.center().y - CHECK / 2.0), vec2(CHECK, CHECK));
        let check = ui.interact(check_rect, check_id(&row.id), Sense::click());
        let name = if alt || row.done {
            if row.done {
                "Mark not done"
            } else {
                "Mark done"
            }
        } else if picked {
            "Deselect"
        } else {
            "Select"
        };
        check.widget_info(|| WidgetInfo::selected(Role::CheckBox, true, picked, name));
        ui.ctx().accesskit_node_builder(check.id, |node| {
            node.set_role(Role::Button);
            node.set_selected(picked);
        });
        if let Some(p) = pt.check {
            part(ui, &check, p);
        }
        let (fill, edge, round) = if alt {
            if row.done {
                (Some(t.done_lamp_1), t.done_lamp_1, false)
            } else {
                (Some(t.surface_3), t.content_1, false)
            }
        } else if picked {
            (Some(t.accent_500), t.accent_500, true)
        } else {
            (None, t.edge_3, true)
        };
        let cr = if round { CornerRadius::same(8) } else { CornerRadius::same(4) };
        if let Some(f) = fill {
            painter.rect_filled(check_rect, cr, f);
        }
        painter.rect_stroke(check_rect, cr, Stroke::new(1.0, edge), StrokeKind::Inside);
        if alt && row.done {
            paint_icon(&painter, check_rect.center(), "check", 14.0, t.on_accent);
        } else if !alt && picked {
            painter.circle_filled(check_rect.center(), 3.0, t.on_accent);
        }
        if check.has_focus() {
            painter.rect_stroke(check_rect.expand(2.0), cr, Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
        }
        if check.clicked() {
            if alt {
                acts.push(Act::SetDoneTree(row.id.clone(), !row.done));
            } else {
                self.anchor = Some(row.id.clone());
                acts.push(Act::TogglePick(row.id.clone()));
            }
        }

        // The trailing items, laid out from the right: More, the count, the disclosure, the attachments.
        let trail_left = rect.left() + indent + CHECK + GAP;
        let trail_rect = Rect::from_min_max(pos2(trail_left, rect.top()), pos2(rect.right() - PAD_R, rect.bottom()));
        let mut tr = ui.new_child(UiBuilder::new().max_rect(trail_rect).layout(Layout::right_to_left(Align::Center)));
        tr.spacing_mut().item_spacing.x = 8.0;
        let (more_pressed, more_rect) = button(
            &mut tr,
            look,
            pt.more,
            serde_json::json!({"variant": "ghost", "size": "sm", "tone": "muted", "shape": "icon", "icon": "more_horiz", "iconSize": 17, "ariaLabel": "More", "reveal": "always", "width": "content", "label": ""}),
        );
        self.more_rects.push((row.id.clone(), more_rect));
        if more_pressed {
            self.menu_for = if self.menu_for.as_deref() == Some(row.id.as_str()) { None } else { Some(row.id.clone()) };
            self.menu_at = None;
            self.submenu_open = false;
        }
        if row.kids > 0 && !s.searching {
            badge(&mut tr, look, pt.badge, &format!("{}/{}", row.kids_done, row.kids), false);
            let expanded = row.expanded;
            let (pressed, disclosure_rect) = button(
                &mut tr,
                look,
                pt.disclosure,
                serde_json::json!({
                    "variant": "ghost",
                    "size": "sm",
                    "tone": "muted",
                    "shape": "icon",
                    "icon": if expanded { "expand_more" } else { "chevron_left" },
                    "iconSize": 16,
                    "ariaLabel": if expanded { "Collapse" } else { "Expand" },
                    "reveal": if expanded { "row" } else { "always" },
                    "width": "content",
                    "label": ""
                }),
            );
            self.disclosures_now.push((row.id.clone(), disclosure_rect));
            if pressed {
                if expanded {
                    self.collapsed.insert(row.id.clone());
                } else {
                    self.collapsed.remove(&row.id);
                }
            }
        }
        if row.attach > 0 {
            badge(&mut tr, look, pt.attach, &row.attach.to_string(), true);
        }
        let used = tr.min_rect();

        // The title block, or the inline field while the row is renamed.
        let title_left = trail_left;
        let title_right = (used.left() - GAP).max(title_left + 20.0);
        let body_rect = Rect::from_min_max(pos2(title_left, rect.top()), pos2(title_right, rect.bottom()));
        if self.renaming.as_deref() == Some(row.id.as_str()) {
            self.rename_field(ui, look, body_rect, row, acts);
        } else {
            let body = ui.interact(body_rect, body_id(&row.id), Sense::click_and_drag());
            body.widget_info(|| info(Role::Button, &row.title, &value));
            ui.ctx().accesskit_node_builder(body.id, |node| node.set_selected(tinted));
            if let Some(p) = pt.row {
                part(ui, &body, p);
            }
            let chord = (enter_key && plat) || (space_key && alt);
            if body.clicked() && !chord {
                let flat = s.flat();
                let idx = flat.iter().position(|r| r.id == row.id).unwrap_or(0);
                if shift {
                    let from = self.anchor.as_ref().and_then(|a| flat.iter().position(|r| &r.id == a)).unwrap_or(idx);
                    let (lo, hi) = (from.min(idx), from.max(idx));
                    acts.push(Act::Select(flat[lo..=hi].iter().map(|r| r.id.clone()).collect()));
                } else if plat {
                    self.anchor = Some(row.id.clone());
                    acts.push(Act::TogglePick(row.id.clone()));
                } else if s.searching && s.list.as_deref() != Some(row.list.as_str()) {
                    acts.push(Act::OpenMatch {
                        id: row.id.clone(),
                        list: row.list.clone(),
                        project: row.project.clone(),
                        title: row.title.clone(),
                        list_name: row.list_name.clone(),
                    });
                } else {
                    self.anchor = Some(row.id.clone());
                    acts.push(Act::Select(vec![]));
                    acts.push(Act::Preview(Some(row.id.clone())));
                }
            }
            if body.double_clicked() {
                acts.push(Act::SetDoneTree(row.id.clone(), !row.done));
            }
            if body.secondary_clicked() {
                self.menu_for = Some(row.id.clone());
                self.menu_at = ui.input(|i| i.pointer.interact_pos());
                self.submenu_open = false;
            }
            if body.drag_started() {
                self.dragging = if picked && s.selection.len() > 1 { s.selection.clone() } else { vec![row.id.clone()] };
            }
            if body.has_focus() {
                painter.rect_stroke(body_rect, radius, Stroke::new(2.0, t.focus_1), StrokeKind::Inside);
            }

            // The title and the note, with the query's matches filled.
            let q = s.q.clone();
            let title_hit = !q.is_empty() && row.title.to_lowercase().contains(&q);
            let note_hit = !q.is_empty() && row.note.to_lowercase().contains(&q);
            let hl = t.accent_500.gamma_multiply(if look.dark { 0.30 } else { 0.25 });
            let ink = if row.done { t.content_3 } else { t.content_1 };
            let (tjob, tmatch) = marked_job(&row.title, &q, &FontId::new(rem(1.0), weight(500)), ink, row.done, hl);
            let mut tjob = tjob;
            tjob.wrap.max_width = body_rect.width().max(1.0);
            tjob.wrap.max_rows = 1;
            tjob.wrap.break_anywhere = true;
            tjob.wrap.overflow_character = Some('…');
            let tgal = ui.painter().layout_job(tjob);
            let title_y = if has_note { body_rect.top() + 6.0 } else { body_rect.center().y - tgal.size().y / 2.0 };
            let title_pos = pos2(body_rect.left(), title_y);
            painter.galley(title_pos, tgal.clone(), ink);
            if title_hit {
                if let (Some(p), Some((cs, ce, m))) = (pt.mark, tmatch) {
                    let r = mark_rect(&tgal, cs, ce).translate(title_pos.to_vec2());
                    let mk = ui.interact(r, Id::new(("tasklist.mark", row.id.as_str())), Sense::hover());
                    mk.widget_info(|| info(Role::GenericContainer, &m, &m));
                    part(ui, &mk, p);
                }
            }
            if has_note {
                let ncol = t.content_2.gamma_multiply(if row.done { 0.55 } else { 1.0 });
                let (njob, nmatch) = marked_job(&row.note, &q, &FontId::new(rem(0.893), weight(400)), ncol, false, hl);
                let mut njob = njob;
                njob.wrap.max_width = body_rect.width().max(1.0);
                njob.wrap.max_rows = 1;
                njob.wrap.break_anywhere = true;
                njob.wrap.overflow_character = Some('…');
                let ngal = ui.painter().layout_job(njob);
                let note_pos = pos2(body_rect.left(), body_rect.top() + 25.0);
                painter.galley(note_pos, ngal.clone(), ncol);
                if note_hit && !title_hit {
                    if let (Some(p), Some((cs, ce, m))) = (pt.mark, nmatch) {
                        let r = mark_rect(&ngal, cs, ce).translate(note_pos.to_vec2());
                        let mk = ui.interact(r, Id::new(("tasklist.mark-note", row.id.as_str())), Sense::hover());
                        mk.widget_info(|| info(Role::GenericContainer, &m, &m));
                        part(ui, &mk, p);
                    }
                }
            }
        }

        // The hairline sits at this row's bottom edge: the separator measures the rect it is drawn in.
        let mut rule = ui.new_child(UiBuilder::new().max_rect(rect));
        separator_row(&mut rule, look, row.depth);
    }

    /// The inline title field of a row being renamed: the whole title is selected when it opens.
    fn rename_field(&mut self, ui: &mut egui::Ui, look: Look, rect: Rect, row: &Row, acts: &mut Vec<Act>) {
        let ctx = ui.ctx().clone();
        let id = rename_id();
        let had_focus = ctx.memory(|m| m.has_focus(id));
        let (enter, esc) = if had_focus {
            (ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Enter)), ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)))
        } else {
            (false, false)
        };
        if self.rename_pending && !had_focus {
            ctx.memory_mut(|m| m.request_focus(id));
            ctx.request_repaint();
        }
        let rename_part = self.pp(parts::RENAMEFIELD, None);
        let resp = text_field(ui, rect, id, &mut self.rename_text, "", "Task title", rename_part, 14.0, 500, look.t.content_1, look.t.content_3);
        let focused = resp.has_focus();
        if focused && self.rename_pending {
            self.rename_pending = false;
            select_all(&ctx, id, &self.rename_text);
        }
        if esc {
            self.renaming = None;
            self.rename_armed = false;
            self.rename_pending = false;
            return;
        }
        if focused {
            self.rename_armed = true;
        }
        if enter || (self.rename_armed && !focused) {
            let title = self.rename_text.trim().to_owned();
            if !title.is_empty() && title != row.title {
                acts.push(Act::Rename(row.id.clone(), title));
            }
            self.renaming = None;
            self.rename_armed = false;
            self.rename_pending = false;
        }
    }

    /// The inline field under a parent row: Enter adds a subtask, an empty Enter, Escape or leaving the field dismisses it.
    fn composer(&mut self, ui: &mut egui::Ui, look: Look, row: &Row, acts: &mut Vec<Act>) {
        let t = look.t;
        let ctx = ui.ctx().clone();
        let id = composer_id();
        let indent = INDENT0 + INDENT_STEP * (row.depth + 1) as f32;
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), rem(2.571)), Sense::hover());
        let had_focus = ctx.memory(|m| m.has_focus(id));
        let (enter, esc) = if had_focus {
            (ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Enter)), ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)))
        } else {
            (false, false)
        };
        paint_icon(ui.painter(), pos2(rect.left() + indent + 7.5, rect.center().y), "subdirectory_arrow_right", 15.0, t.content_3);
        if self.composer_pending && !had_focus {
            ctx.memory_mut(|m| m.request_focus(id));
            ctx.request_repaint();
        }
        let field = Rect::from_min_max(pos2(rect.left() + indent + 21.0, rect.top()), pos2(rect.right() - 8.0, rect.bottom()));
        let composer_part = self.pp(parts::SUBTASKCOMPOSER, None);
        let resp = text_field(ui, field, id, &mut self.composer_text, "Subtask title — Enter to add", "Subtask title", composer_part, 14.0, 400, t.content_1, t.content_3);
        let focused = resp.has_focus();
        if focused {
            self.composer_pending = false;
            self.composer_armed = true;
        }
        if esc {
            self.composing = None;
            self.composer_text.clear();
            self.composer_armed = false;
            return;
        }
        if enter {
            let title = self.composer_text.trim().to_owned();
            if title.is_empty() {
                self.composing = None;
                self.composer_armed = false;
            } else {
                self.composer_text.clear();
                acts.push(Act::AddSubtask(row.id.clone(), title));
            }
            return;
        }
        if self.composer_armed && !focused {
            self.composing = None;
            self.composer_text.clear();
            self.composer_armed = false;
        }
    }

    /// Keys on the focused row or its check: the arrows move and nest, F2 renames, the chord marks it done, a letter starts a search.
    fn row_keys(&mut self, ui: &mut egui::Ui, s: &Shown, acts: &mut Vec<Act>) {
        let ctx = ui.ctx().clone();
        let Some(focus) = ctx.memory(|m| m.focused()) else { return };
        // A focused disclosure: ArrowLeft collapses its row, ArrowRight expands it.
        if let Some(at) = ctx.read_response(focus).map(|r| r.rect) {
            if let Some((id, _)) = self.disclosures_prev.iter().find(|(_, r)| r.contains(at.center())) {
                ctx.memory_mut(|m| m.set_focus_lock_filter(focus, EventFilter { horizontal_arrows: true, ..Default::default() }));
                let (left, right) = ui.input(|i| (i.key_pressed(Key::ArrowLeft), i.key_pressed(Key::ArrowRight)));
                if left {
                    self.collapsed.insert(id.clone());
                } else if right {
                    self.collapsed.remove(id);
                }
                return;
            }
        }
        let flat = s.flat();
        let Some(idx) = flat.iter().position(|r| body_id(&r.id) == focus || check_id(&r.id) == focus) else { return };
        ctx.memory_mut(|m| m.set_focus_lock_filter(focus, EventFilter { tab: true, horizontal_arrows: true, vertical_arrows: true, escape: true }));
        let row = flat[idx];
        let k = read_keys(ui);
        let mac = cfg!(target_os = "macos");
        let go = |id: &str| ctx.memory_mut(|m| m.request_focus(body_id(id)));

        if k.esc && !s.selection.is_empty() {
            acts.push(Act::Select(vec![]));
        }
        if k.up || k.down {
            let dir: isize = if k.down { 1 } else { -1 };
            // In the main window the arrows wrap and the preview follows the active task (the web's window).
            let window = self.row_box.is_some();
            let n = flat.len() as isize;
            let next = if window && !k.m.shift { (idx as isize + dir).rem_euclid(n) as usize } else { (idx as isize + dir).clamp(0, n - 1) as usize };
            if window && !k.m.shift && next != idx {
                acts.push(Act::Preview(Some(flat[next].id.clone())));
            }
            if k.m.shift {
                let from = self.anchor.as_ref().and_then(|a| flat.iter().position(|r| &r.id == a)).unwrap_or(idx);
                let (lo, hi) = (from.min(next), from.max(next));
                acts.push(Act::Select(flat[lo..=hi].iter().map(|r| r.id.clone()).collect()));
            }
            if next != idx {
                go(flat[next].id.as_str());
            }
        }
        if k.right {
            if row.kids > 0 && !row.expanded {
                self.collapsed.remove(&row.id);
            } else if row.kids > 0 {
                if let Some(nx) = flat.get(idx + 1) {
                    if nx.depth > row.depth {
                        go(nx.id.as_str());
                    }
                }
            }
        }
        if k.left {
            if row.kids > 0 && row.expanded {
                self.collapsed.insert(row.id.clone());
            } else if row.depth > 0 {
                if let Some(p) = flat[..idx].iter().rev().find(|r| r.depth < row.depth) {
                    go(p.id.as_str());
                }
            }
        }
        if k.tab {
            if k.m.shift {
                if row.parent {
                    acts.push(Act::Promote(row.id.clone()));
                }
            } else {
                acts.push(Act::Nest(row.id.clone()));
            }
        }
        if k.f2 {
            self.start_rename(&row.id, &row.title);
        }
        // Delete moves the task to the Trash, asking first (decision 10).
        if ui.input(|i| i.key_pressed(Key::Delete) || (cfg!(target_os = "macos") && i.modifiers.mac_cmd && i.key_pressed(Key::Backspace))) {
            acts.push(Act::Delete(row.id.clone()));
        }
        let chord = if mac { k.space && k.m.alt } else { k.enter && k.plat };
        if chord {
            if s.selection.contains(&row.id) {
                let all_done = s.selection.iter().all(|x| s.done_of.get(x).copied().unwrap_or(false));
                acts.push(Act::SetDone(s.selection.clone(), !all_done));
            } else {
                acts.push(Act::SetDoneTree(row.id.clone(), !row.done));
            }
        }
        if k.a && k.plat {
            acts.push(Act::Select(flat.iter().map(|r| r.id.clone()).collect()));
        }
        if k.n && k.plat {
            acts.push(Act::Draft(s.list.clone().unwrap_or_default(), s.list_name.clone()));
        }
        if k.f && k.plat {
            ctx.memory_mut(|m| m.request_focus(search_id()));
        }
        if let Some(c) = k.letter {
            if !k.plat && !k.m.alt {
                self.query = c.to_string();
                self.match_at = None;
                ctx.memory_mut(|m| m.request_focus(search_id()));
            }
        }
    }

    /// The drag of rows: the zone under the pointer (last frame's rows) shows the drop; the release drops.
    fn drag_update(&mut self, ui: &mut egui::Ui, body: Rect, s: &Shown, acts: &mut Vec<Act>) {
        if self.dragging.is_empty() {
            self.drop_at = None;
            return;
        }
        let pointer = ui.input(|i| i.pointer.interact_pos());
        let target = pointer.and_then(|p| self.rows_prev.iter().find(|(id, r)| r.contains(p) && !self.dragging.contains(id)).map(|(id, r)| (id.clone(), zone_of(p, *r))));
        self.drop_at = target.clone();
        if ui.input(|i| i.pointer.primary_released()) {
            let ids = std::mem::take(&mut self.dragging);
            match target {
                Some((id, zone)) => {
                    if let Some(list) = s.list.clone() {
                        acts.push(Act::Drop(ids, Some(id), zone, list));
                    }
                }
                None => {
                    if let (Some(p), Some(list)) = (pointer, s.list.clone()) {
                        if body.contains(p) {
                            acts.push(Act::Drop(ids, None, Zone::After, list));
                        }
                    }
                }
            }
            self.drop_at = None;
        }
    }

    fn footer(&mut self, ui: &mut egui::Ui, look: Look, s: &Shown, rect: Rect, acts: &mut Vec<Act>) {
        let t = look.t;
        let painter = ui.painter().clone();
        painter.rect_filled(rect, CornerRadius::same(0), t.surface_2);
        painter.line_segment([rect.left_top(), rect.right_top()], Stroke::new(1.0, t.edge_2));
        let inner = Rect::from_min_max(pos2(rect.left() + 12.0, rect.top()), pos2(rect.right() - 12.0, rect.bottom()));
        let mut ch = ui.new_child(UiBuilder::new().max_rect(inner).layout(Layout::left_to_right(Align::Center)));
        ch.spacing_mut().item_spacing.x = 8.0;
        let pct = if s.total > 0 { ((s.done as f64 / s.total as f64) * 100.0).round() as usize } else { 0 };
        let total_for_bar = s.total.max(1) as f64;

        if s.selection.is_empty() {
            let (pressed, _) = button(
                &mut ch,
                look,
                self.pp(parts::SORTFILTER, None),
                serde_json::json!({"variant": "ghost", "size": "footer", "tone": "muted", "shape": "text", "icon": "tune", "label": "", "ariaLabel": "Sort & filter", "width": "content"}),
            );
            if pressed {
                acts.push(Act::ToggleFilter);
            }
            let sep = ch.allocate_exact_size(vec2(1.0, 16.0), Sense::hover()).0;
            ch.painter().rect_filled(sep, CornerRadius::same(0), t.edge_2);
            let value = format!("{}/{} done", s.done, s.total);
            let shown = format!("{}/{} DONE", s.done, s.total);
            status(&mut ch, self.pp(parts::DONECOUNT, Some(SPLIT_DONE)), &value, &shown, 12.0, 600, t.content_2);
            ch.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let p = format!("{pct}%");
                status(ui, self.pp(parts::PERCENT, None), &p, &p, 12.0, 600, t.content_2);
                let bar: taskscape_ui::omnicui::progressbar::ProgressbarProps = serde_json::from_value(serde_json::json!({
                    "value": s.done as f64,
                    "max": total_for_bar,
                    "ariaLabel": "Done progress",
                }))
                .expect("progress props");
                ProgressBar::new(bar).ui(ui, look);
            });
        } else {
            // The bar as a whole carries the bulkBar part. It is added before its buttons, so they stay on top.
            let bar = ui.interact(rect, Id::new("tasklist.bulkbar"), Sense::hover());
            bar.widget_info(|| WidgetInfo::new(Role::GenericContainer));
            part(ui, &bar, parts::BULKBAR);
            // The stats footer is out of sight while picked, but its counts keep their text (the web keeps both in the page).
            if let Some(p) = self.pp(parts::DONECOUNT, Some(SPLIT_DONE)) {
                hidden_status(ui, rect.min, p, &format!("{}/{} done", s.done, s.total));
            }
            if let Some(p) = self.pp(parts::PERCENT, None) {
                hidden_status(ui, rect.min, p, &format!("{pct}%"));
            }
            let n = s.selection.len();
            let value = format!("{n} selected");
            let shown = format!("{n} SELECTED");
            status(&mut ch, self.pp(parts::SELECTIONCOUNT, None), &value, &shown, 12.0, 600, t.accent_500);
            let sep = ch.allocate_exact_size(vec2(1.0, 16.0), Sense::hover()).0;
            ch.painter().rect_filled(sep, CornerRadius::same(0), t.edge_2);
            let (pressed, _) = button(
                &mut ch,
                look,
                self.pp(parts::BULKDONE, None),
                serde_json::json!({"variant": "ghost", "size": "footer", "tone": "muted", "shape": "text", "icon": "task_alt", "label": "Mark done", "ariaLabel": "Mark done", "width": "content"}),
            );
            if pressed {
                acts.push(Act::SetDone(s.selection.clone(), true));
            }
            let (pressed, _) = button(
                &mut ch,
                look,
                self.pp(parts::BULKUNDONE, None),
                serde_json::json!({"variant": "ghost", "size": "footer", "tone": "muted", "shape": "text", "icon": "radio_button_unchecked", "label": "Mark not done", "ariaLabel": "Mark not done", "width": "content"}),
            );
            if pressed {
                acts.push(Act::SetDone(s.selection.clone(), false));
            }
            let no_targets = s.project_lists.iter().all(|(id, _)| Some(id.as_str()) == s.list.as_deref());
            let (pressed, move_rect) = button(
                &mut ch,
                look,
                self.pp(parts::BULKMOVE, None),
                serde_json::json!({"variant": "ghost", "size": "footer", "tone": "muted", "shape": "text", "icon": "arrow_forward", "label": "Move to list", "ariaLabel": "Move to list", "width": "content", "disabled": no_targets}),
            );
            if pressed && !no_targets {
                self.bulk_move_open = !self.bulk_move_open;
            }
            self.bulk_btn = Some(move_rect);
            let (pressed, _) = button(
                &mut ch,
                look,
                self.pp(parts::BULKCOPY, None),
                serde_json::json!({"variant": "ghost", "size": "footer", "tone": "muted", "shape": "text", "icon": "content_copy", "label": "Copy", "ariaLabel": "Copy", "width": "content"}),
            );
            if pressed {
                let lines: Vec<String> = s
                    .selection
                    .iter()
                    .filter_map(|id| s.flat().into_iter().find(|r| &r.id == id).map(|r| format!("- [{}] {}", if r.done { "x" } else { " " }, r.title)))
                    .collect();
                ch.ctx().copy_text(lines.join("\n"));
                acts.push(Act::Emit(TasklistEvent::Copied { count: n as i64 }));
            }
            let (pressed, _) = button(
                &mut ch,
                look,
                self.pp(parts::BULKDELETE, None),
                serde_json::json!({"variant": "ghost", "size": "footer", "tone": "danger", "shape": "text", "icon": "delete", "label": "Delete", "ariaLabel": "Delete", "width": "content", "disabled": n != 1}),
            );
            if pressed && n == 1 {
                acts.push(Act::Delete(s.selection[0].clone()));
            }
            ch.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let (pressed, _) = button(
                    ui,
                    look,
                    self.pp(parts::BULKCLEAR, None),
                    serde_json::json!({"variant": "ghost", "size": "footer", "tone": "muted", "shape": "text", "icon": "close", "label": "Clear", "ariaLabel": "Clear", "width": "content"}),
                );
                if pressed {
                    acts.push(Act::Select(vec![]));
                }
            });
        }
    }

    fn menus(&mut self, ctx: &egui::Context, look: Look, s: &Shown, acts: &mut Vec<Act>) {
        self.row_menu(ctx, look, s, acts);
        self.options_menu(ctx, look);
        self.bulk_menu(ctx, look, s, acts);
    }

    /// The More menu of a row: the single-task actions, Move to list with its submenu.
    fn row_menu(&mut self, ctx: &egui::Context, look: Look, s: &Shown, acts: &mut Vec<Act>) {
        let t = look.t;
        let Some(id) = self.menu_for.clone() else { return };
        let flat = s.flat();
        let Some(row) = flat.iter().copied().find(|r| r.id == id) else {
            self.menu_for = None;
            return;
        };
        let anchor = match self.menu_at {
            Some(p) => p,
            None => match self.more_rects.iter().find(|(i, _)| *i == id).map(|(_, r)| *r) {
                Some(r) => pos2(r.right() - MENU_W, r.bottom() + 4.0),
                None => {
                    self.menu_for = None;
                    return;
                }
            },
        };
        let targets: Vec<(TaskId, String)> = s.project_lists.iter().filter(|(l, _)| *l != row.list).cloned().collect();
        let pane0 = self.pane == 0;
        // Items (Copy, Add subtask, Rename, Move to list, Promote on a subtask, Delete) and the 9 px rule before Delete.
        let count = if row.parent { 6.0 } else { 5.0 };
        let height = count * MENU_ITEM + 8.0 + 9.0;
        let panel = Rect::from_min_size(anchor, vec2(MENU_W, height));
        self.open_panels.push(panel);
        let move_rect = Rect::from_min_size(pos2(panel.left() + 4.0, panel.top() + 4.0 + 3.0 * MENU_ITEM), vec2(MENU_W - 8.0, MENU_ITEM));
        let row_id = row.id.clone();
        let row_title = row.title.clone();
        let row_done = row.done;
        let has_parent = row.parent;
        let mut close = false;
        let mut hovered_move = false;
        let mut hovered_other = false;
        let pick = |main: &'static str| if pane0 { Some(main) } else { None };

        egui::Area::new(Id::new("tasklist.rowmenu")).order(Order::Foreground).fixed_pos(panel.min).show(ctx, |ui| {
            ui.painter().rect_filled(panel, CornerRadius::same(7), t.surface_3);
            ui.painter().rect_stroke(panel, CornerRadius::same(7), Stroke::new(1.0, t.edge_2), StrokeKind::Inside);
            let mr = Rect::from_min_size(panel.min, panel.size());
            let menu_resp = ui.interact(mr, Id::new("tasklist.rowmenu.panel"), Sense::hover());
            if let Some(p) = pick(parts::ROWMENU) {
                part(ui, &menu_resp, p);
            }
            menu_resp.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, "Task menu"));
            let mut y = panel.top() + 4.0;
            let mut next = |y: &mut f32| {
                let r = Rect::from_min_size(pos2(panel.left() + 4.0, *y), vec2(MENU_W - 8.0, MENU_ITEM));
                *y += MENU_ITEM;
                r
            };

            let r = next(&mut y);
            let resp = menu_row(ui, look, r, "copy", "Copy", Some("content_copy"), false, None, false, pick(parts::MENUCOPY));
            if resp.hovered() {
                hovered_other = true;
            }
            if resp.clicked() {
                let text = format!("- [{}] {}", if row_done { "x" } else { " " }, row_title);
                ui.ctx().copy_text(text);
                acts.push(Act::Emit(TasklistEvent::Copied { count: 1 }));
                close = true;
            }

            let r = next(&mut y);
            let resp = menu_row(ui, look, r, "add", "Add subtask", Some("subdirectory_arrow_right"), false, None, false, pick(parts::MENUADDSUBTASK));
            if resp.hovered() {
                hovered_other = true;
            }
            if resp.clicked() {
                self.composing = Some(row_id.clone());
                self.composer_text.clear();
                self.composer_pending = true;
                self.composer_armed = false;
                close = true;
            }

            let r = next(&mut y);
            let resp = menu_row(ui, look, r, "rename", "Rename", Some("edit"), false, None, false, pick(parts::MENURENAME));
            if resp.hovered() {
                hovered_other = true;
            }
            if resp.clicked() {
                self.start_rename(&row_id, &row_title);
                close = true;
            }

            let r = next(&mut y);
            let resp = menu_row(ui, look, r, "move", "Move to list", Some("arrow_forward"), false, None, true, pick(parts::MENUMOVETOLIST));
            if resp.hovered() {
                hovered_move = true;
            }
            if resp.clicked() {
                self.submenu_open = !self.submenu_open;
            }

            if has_parent {
                let r = next(&mut y);
                let resp = menu_row(ui, look, r, "promote", "Promote to top level", Some("north_west"), false, None, false, pick(parts::MENUPROMOTE));
                if resp.hovered() {
                    hovered_other = true;
                }
                if resp.clicked() {
                    acts.push(Act::Promote(row_id.clone()));
                    close = true;
                }
            }

            // The rule sits in its own 9 px band, drawn where it falls in the panel.
            ui.painter().line_segment([pos2(panel.left() + 8.0, y + 4.5), pos2(panel.right() - 8.0, y + 4.5)], Stroke::new(1.0, t.edge_2));
            y += 9.0;
            let r = next(&mut y);
            let resp = menu_row(ui, look, r, "delete", "Delete task…", Some("delete"), true, None, false, pick(parts::MENUDELETE));
            if resp.hovered() {
                hovered_other = true;
            }
            if resp.clicked() {
                acts.push(Act::Delete(row_id.clone()));
                close = true;
            }
        });

        if hovered_move {
            self.submenu_open = true;
        } else if hovered_other {
            self.submenu_open = false;
        }
        if close {
            self.close_menus();
            return;
        }
        if self.submenu_open && !targets.is_empty() {
            // Opens to the right of the menu; on the left when the pane has no room there (the pane is 860 px at most).
            let sub_x = if panel.right() + 2.0 + MENU_W > self.pane_rect.right() { panel.left() - 2.0 - MENU_W } else { panel.right() + 2.0 };
            let sub = Rect::from_min_size(pos2(sub_x, move_rect.top() - 4.0), vec2(MENU_W, targets.len() as f32 * MENU_ITEM + 8.0));
            self.open_panels.push(sub);
            let mut picked: Option<TaskId> = None;
            egui::Area::new(Id::new("tasklist.movemenu")).order(Order::Foreground).fixed_pos(sub.min).show(ctx, |ui| {
                ui.painter().rect_filled(sub, CornerRadius::same(7), t.surface_3);
                ui.painter().rect_stroke(sub, CornerRadius::same(7), Stroke::new(1.0, t.edge_2), StrokeKind::Inside);
                let mut y = sub.top() + 4.0;
                for (tid, name) in &targets {
                    let r = Rect::from_min_size(pos2(sub.left() + 4.0, y), vec2(MENU_W - 8.0, MENU_ITEM));
                    y += MENU_ITEM;
                    let part_id = if name == "Reading" { pick(parts::MOVETARGETREADING) } else { None };
                    let resp = menu_row(ui, look, r, &format!("move-{tid}"), name, None, false, None, false, part_id);
                    if resp.clicked() {
                        picked = Some(tid.clone());
                    }
                }
            });
            if let Some(tid) = picked {
                acts.push(Act::MoveTo(vec![row_id.clone()], tid));
                self.close_menus();
            }
        }
    }

    /// The scope and fields menu of the search bar: radio rows for the scope, then for the fields.
    fn options_menu(&mut self, ctx: &egui::Context, look: Look) {
        let t = look.t;
        if !self.options_open {
            return;
        }
        let Some(btn) = self.options_btn else {
            self.options_open = false;
            return;
        };
        let pane0 = self.pane == 0;
        let panel = Rect::from_min_size(pos2(btn.right() - SEARCH_MENU_W, btn.bottom() + 4.0), vec2(SEARCH_MENU_W, 6.0 * MENU_ITEM + 8.0 + 9.0));
        self.open_panels.push(panel);
        let (scope, fields) = (self.scope, self.fields);
        let mut next_scope: Option<Scope> = None;
        let mut next_fields: Option<Fields> = None;
        egui::Area::new(Id::new("tasklist.searchmenu")).order(Order::Foreground).fixed_pos(panel.min).show(ctx, |ui| {
            ui.painter().rect_filled(panel, CornerRadius::same(7), t.surface_3);
            ui.painter().rect_stroke(panel, CornerRadius::same(7), Stroke::new(1.0, t.edge_2), StrokeKind::Inside);
            let mr = Rect::from_min_size(panel.min, panel.size());
            let menu_resp = ui.interact(mr, Id::new("tasklist.searchmenu.panel"), Sense::hover());
            if pane0 {
                part(ui, &menu_resp, parts::SEARCHMENU);
            }
            menu_resp.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, "Search options"));
            let mut y = panel.top() + 4.0;
            let mut next = |y: &mut f32| {
                let r = Rect::from_min_size(pos2(panel.left() + 4.0, *y), vec2(SEARCH_MENU_W - 8.0, MENU_ITEM));
                *y += MENU_ITEM;
                r
            };
            let pick = |main: &'static str| if pane0 { Some(main) } else { None };
            let r = next(&mut y);
            if menu_row(ui, look, r, "scope-list", "This list", None, false, Some(scope == Scope::List), false, pick(parts::SCOPETHISLIST)).clicked() {
                next_scope = Some(Scope::List);
            }
            let r = next(&mut y);
            if menu_row(ui, look, r, "scope-project", "Whole project", None, false, Some(scope == Scope::Project), false, pick(parts::SCOPEWHOLEPROJECT)).clicked() {
                next_scope = Some(Scope::Project);
            }
            let r = next(&mut y);
            if menu_row(ui, look, r, "scope-all", "All projects", None, false, Some(scope == Scope::All), false, pick(parts::SCOPEALLPROJECTS)).clicked() {
                next_scope = Some(Scope::All);
            }
            y += 9.0;
            let r = next(&mut y);
            if menu_row(ui, look, r, "fields-both", "Title & notes", None, false, Some(fields == Fields::Both), false, pick(parts::FIELDSTITLEANDNOTES)).clicked() {
                next_fields = Some(Fields::Both);
            }
            let r = next(&mut y);
            if menu_row(ui, look, r, "fields-title", "Title only", None, false, Some(fields == Fields::Title), false, pick(parts::FIELDSTITLEONLY)).clicked() {
                next_fields = Some(Fields::Title);
            }
            let r = next(&mut y);
            if menu_row(ui, look, r, "fields-notes", "Notes only", None, false, Some(fields == Fields::Notes), false, pick(parts::FIELDSNOTESONLY)).clicked() {
                next_fields = Some(Fields::Notes);
            }
        });
        if let Some(sc) = next_scope {
            self.scope = sc;
            self.options_open = false;
        }
        if let Some(f) = next_fields {
            self.fields = f;
            self.options_open = false;
        }
    }

    /// The bulk Move menu: the other lists of the project, opened above its footer button.
    fn bulk_menu(&mut self, ctx: &egui::Context, look: Look, s: &Shown, acts: &mut Vec<Act>) {
        let t = look.t;
        if !self.bulk_move_open {
            return;
        }
        let Some(btn) = self.bulk_btn else {
            self.bulk_move_open = false;
            return;
        };
        let targets: Vec<(TaskId, String)> = s.project_lists.iter().filter(|(l, _)| Some(l.as_str()) != s.list.as_deref()).cloned().collect();
        let pane0 = self.pane == 0;
        let h = targets.len() as f32 * MENU_ITEM + 8.0;
        let panel = Rect::from_min_size(pos2(btn.left(), btn.top() - 4.0 - h), vec2(MENU_W, h));
        self.open_panels.push(panel);
        let mut picked: Option<TaskId> = None;
        egui::Area::new(Id::new("tasklist.bulkmenu")).order(Order::Foreground).fixed_pos(panel.min).show(ctx, |ui| {
            ui.painter().rect_filled(panel, CornerRadius::same(7), t.surface_3);
            ui.painter().rect_stroke(panel, CornerRadius::same(7), Stroke::new(1.0, t.edge_2), StrokeKind::Inside);
            let mut y = panel.top() + 4.0;
            for (tid, name) in &targets {
                let r = Rect::from_min_size(pos2(panel.left() + 4.0, y), vec2(MENU_W - 8.0, MENU_ITEM));
                y += MENU_ITEM;
                let part_id = if pane0 && name == "Reading" { Some(parts::MOVETARGETREADING) } else { None };
                if menu_row(ui, look, r, &format!("bulk-{tid}"), name, None, false, None, false, part_id).clicked() {
                    picked = Some(tid.clone());
                }
            }
        });
        if let Some(tid) = picked {
            acts.push(Act::MoveTo(s.selection.clone(), tid));
            self.bulk_move_open = false;
        }
    }

    fn apply(&mut self, app: &mut App, acts: Vec<Act>, events: &mut Vec<TasklistEvent>) {
        for act in acts {
            match act {
                Act::Select(ids) => app.ui.selection = ids,
                Act::TogglePick(id) => {
                    if let Some(i) = app.ui.selection.iter().position(|x| *x == id) {
                        app.ui.selection.remove(i);
                    } else {
                        app.ui.selection.push(id);
                    }
                }
                Act::Preview(id) => app.ui.preview = id,
                Act::SetDone(ids, done) => {
                    for id in ids {
                        if let Err(e) = app.data.update_task(&id, None, None, Some(done)) {
                            app.refused(e);
                        }
                    }
                }
                Act::SetDoneTree(id, done) => {
                    for x in subtree(&app.data.tasks, &id) {
                        if let Err(e) = app.data.update_task(&x, None, None, Some(done)) {
                            app.refused(e);
                        }
                    }
                }
                Act::MoveTo(ids, list) => {
                    for id in &ids {
                        move_tree(app, id, None, &list, None);
                    }
                    app.ui.selection.retain(|x| !ids.contains(x));
                }
                Act::Promote(id) => {
                    if app.data.task(&id).is_some_and(|t| t.parent_id.is_some()) {
                        if let Err(e) = app.data.move_task(&id, None, None, None) {
                            app.refused(e);
                        }
                    }
                }
                Act::Nest(id) => {
                    let prev: Option<TaskId> = match app.data.task(&id) {
                        None => None,
                        Some(t) => {
                            let mut sibs: Vec<&Task> = app.data.tasks.iter().filter(|x| x.list_id == t.list_id && x.parent_id == t.parent_id).collect();
                            sibs.sort_by(order);
                            sibs.iter().position(|x| x.id == id).and_then(|i| if i > 0 { Some(sibs[i - 1].id.clone()) } else { None })
                        }
                    };
                    if let Some(p) = prev {
                        if let Err(e) = app.data.move_task(&id, Some(&p), None, None) {
                            app.refused(e);
                        }
                    }
                }
                Act::Delete(id) => app.sheet = Some(Sheet::DeleteTask(id)),
                Act::AddSubtask(parent, title) => {
                    let list = app.data.task(&parent).map(|t| t.list_id.clone());
                    if let Some(list) = list {
                        if let Err(e) = app.data.create_task(&list, &title, None, Some(&parent)) {
                            app.refused(e);
                        }
                        self.collapsed.remove(&parent);
                    }
                }
                Act::Rename(id, title) => {
                    if let Err(e) = app.data.update_task(&id, Some(&title), None, None) {
                        app.refused(e);
                    }
                }
                Act::Drop(ids, target, zone, list) => {
                    let plan = {
                        let tg = target.as_ref().and_then(|id| app.data.task(id));
                        plan_drop(&app.data.tasks, &ids, tg, zone, &list)
                    };
                    for (id, parent, to_list, order_at) in plan {
                        move_tree(app, &id, parent.as_deref(), &to_list, Some(order_at));
                    }
                }
                Act::OpenMatch { id, list, project, title, list_name } => {
                    if app.ui.project.as_deref() != Some(project.as_str()) {
                        app.ui.set_project(Some(project));
                    }
                    app.ui.open_list(Some(list), Some(self.pane));
                    app.ui.selection.clear();
                    app.ui.preview = Some(id);
                    self.query.clear();
                    self.scope = Scope::List;
                    self.match_at = None;
                    events.push(TasklistEvent::ListOpened { list: list_name });
                    events.push(TasklistEvent::TaskPreviewed { title });
                }
                Act::Draft(list, name) => {
                    app.ui.draft_list = Some(list);
                    events.push(TasklistEvent::DraftStarted { list: name });
                }
                Act::CloseSplit(name) => {
                    // Close split removes the right pane: the left pane's list stays.
                    app.ui.active_pane = 0;
                    app.ui.split(false);
                    events.push(TasklistEvent::SplitClosed { list: name });
                }
                Act::ToggleFilter => {
                    app.ui.aside = if app.ui.aside == Aside::Filter { Aside::Preview } else { Aside::Filter };
                }
                Act::Emit(e) => events.push(e),
            }
        }
    }
}

/// The hairline under a row, inset to its depth (the web's Separator inset row).
fn separator_row(ui: &mut egui::Ui, look: Look, depth: usize) {
    let props: taskscape_ui::omnicui::separator::SeparatorProps = serde_json::from_value(serde_json::json!({
        "orientation": "horizontal",
        "inset": "row",
        "depth": depth as i64,
    }))
    .expect("separator props");
    Separator::new(props).ui(ui, look);
}

pub struct TaskListMount {
    app: App,
    screen: TaskList,
}

impl Mounted for TaskListMount {
    fn frame(&mut self, ui: &mut egui::Ui, look: Look, emit: &mut dyn FnMut(&'static str, serde_json::Value)) {
        for e in self.screen.show(&mut self.app, ui, look) {
            let (n, f) = e.to_json();
            emit(n, f);
        }
    }
}

/// The test's mount: the app on the props' data set, put where the props say the screen starts.
pub fn mount(mut app: App, props: serde_json::Value) -> Result<Box<dyn Mounted>, String> {
    let p: TasklistProps = serde_json::from_value(props).map_err(|e| e.to_string())?;
    if !p.showCompleted {
        app.data.set_setting("show_completed", "false").map_err(|e| e.to_string())?;
    }
    app.open_named(Some(&p.project), Some(&p.list));
    if p.split {
        app.ui.split(true);
    }
    Ok(Box::new(TaskListMount { app, screen: TaskList::new(0) }))
}
