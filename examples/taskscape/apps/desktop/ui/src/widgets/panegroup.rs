//! contract: omnicui/contracts/panegroup.contract.yaml (id: ui.panegroup). A resizable window body row: the task pane, an optional split pane and an optional aside, with handles between them that drag, step, collapse and reset the sizes.
use crate::omnicui::panegroup::{parts, PanegroupEvent, PanegroupProps};
use crate::{part, rem, weight, Look, Tokens};
use egui::{
    pos2, vec2, Align2, Color32, CornerRadius, CursorIcon, FontId, Id, Key, PointerButton, Pos2, Rect, Response, Role,
    Sense, Stroke, StrokeKind, WidgetInfo,
};

/// Limits from the contract. Sizes are percent of the group length; the aside is in px in a row.
const TASK_MIN: f64 = 25.0;
const SPLIT_MIN: f64 = 20.0;
const ASIDE_MIN_PX: f64 = 280.0;
const ASIDE_MAX_PX: f64 = 420.0;
const ASIDE_SHOW_PX: f64 = 600.0;
const ASIDE_DEFAULT: f64 = 30.0;
/// Two presses on one handle this close (seconds), with no drag between, reset the sizes.
const DOUBLE_PRESS_S: f64 = 0.5;
/// The pane heading band: py-2, a 12 px line of 16 px, py-2, and its 1 px bottom border.
const HEAD_H: f32 = rem(0.5) * 2.0 + rem(1.143) + 1.0;
/// One row of a demo list: py-2, a 13 px line, py-2, and a 1 px border.
const ROW_H: f32 = rem(0.5) * 2.0 + 16.0 + 1.0;
/// The separator's hit area is 0.5 rem wide and straddles the pane edge.
const HANDLE_HALF: f32 = rem(0.5) / 2.0;

/// The stored layout. None means the default for the current configuration.
#[derive(Clone, Copy, Debug, Default)]
struct Stored {
    task: Option<f64>,
    aside: Option<f64>,
    /// The task size before Enter collapsed it; None when expanded.
    collapsed: Option<f64>,
}

#[derive(Clone, Copy, Debug)]
struct Sizes {
    task: f64,
    split: f64,
    aside: f64,
}

#[derive(Clone, Copy, Debug)]
struct Cfg {
    row: bool,
    length: f64,
    has_split: bool,
    has_aside: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Task,
    Aside,
}

impl Kind {
    fn index(self) -> usize {
        match self {
            Kind::Task => 0,
            Kind::Aside => 1,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Press {
    time: f64,
    moved: bool,
}

#[derive(Clone, Copy, Debug)]
struct Drag {
    kind: Kind,
    origin: Pos2,
    start: Sizes,
    base: Stored,
    draft: Stored,
}

fn clamp(v: f64, lo: f64, hi: f64) -> f64 {
    v.min(hi).max(lo)
}

fn round3(v: f64) -> f64 {
    (v * 1000.0).round() / 1000.0
}

fn same_sizes(a: Sizes, b: Sizes) -> bool {
    (a.task - b.task).abs() < 1e-6 && (a.aside - b.aside).abs() < 1e-6
}

/// Aside size bounds in percent. Column has no pixel limits; the task and split floors always apply.
fn aside_bounds(c: Cfg) -> (f64, f64) {
    let ceiling = 100.0 - TASK_MIN - if c.has_split { SPLIT_MIN } else { 0.0 };
    if !c.row || c.length <= 0.0 {
        return (0.0, ceiling);
    }
    ((ASIDE_MIN_PX / c.length) * 100.0, ceiling.min((ASIDE_MAX_PX / c.length) * 100.0))
}

/// The aside handle's position (percent from the start edge) and its reach, given the task size.
fn aside_reach(c: Cfg, task: f64) -> (f64, f64) {
    let (lo, hi) = aside_bounds(c);
    let top = if c.has_split { hi.min(100.0 - task - SPLIT_MIN) } else { hi };
    (100.0 - top, 100.0 - lo)
}

/// The task size's reach, given the aside size.
fn task_reach(aside: f64) -> (f64, f64) {
    (TASK_MIN, 100.0 - aside - SPLIT_MIN)
}

/// Turns the stored layout into sizes for the current group length and visible panes.
fn resolve(c: Cfg, s: Stored) -> Sizes {
    let mut aside = 0.0;
    if c.has_aside {
        let (lo, hi) = aside_bounds(c);
        aside = clamp(s.aside.unwrap_or(ASIDE_DEFAULT), lo, hi);
    }
    let rest = 100.0 - aside;
    if !c.has_split {
        return Sizes { task: rest, split: 0.0, aside };
    }
    if s.collapsed.is_some() {
        return Sizes { task: 0.0, split: rest, aside };
    }
    let fallback = if c.has_aside { 40.0 } else { 50.0 };
    let task = clamp(s.task.unwrap_or(fallback), TASK_MIN, rest - SPLIT_MIN);
    Sizes { task, split: rest - task, aside }
}

fn step_task(c: Cfg, cur: Sizes, base: Stored, target: f64) -> Stored {
    let (lo, hi) = task_reach(cur.aside);
    Stored {
        task: Some(clamp(target, lo, hi)),
        aside: if c.has_aside { Some(cur.aside) } else { base.aside },
        collapsed: None,
    }
}

fn step_aside(c: Cfg, cur: Sizes, base: Stored, target: f64) -> Stored {
    let (min_pos, max_pos) = aside_reach(c, cur.task);
    Stored {
        task: if c.has_split { Some(cur.task) } else { base.task },
        aside: Some(100.0 - clamp(target, min_pos, max_pos)),
        collapsed: base.collapsed,
    }
}

/// A rect across the whole group, between two positions along its length (x for a row, y for a column).
fn band_rect(rect: Rect, row: bool, a: f32, b: f32) -> Rect {
    if row {
        Rect::from_min_max(pos2(a, rect.top()), pos2(b, rect.bottom()))
    } else {
        Rect::from_min_max(pos2(rect.left(), a), pos2(rect.right(), b))
    }
}

/// Paints one pane: its tone, the heading band with its bottom border, and the demo body.
fn paint_pane(painter: &egui::Painter, t: Tokens, r: Rect, fill: Color32, heading: &str, lines: &[String], rows: bool) {
    let painter = painter.with_clip_rect(r);
    painter.rect_filled(r, CornerRadius::same(0), fill);
    let x = r.left() + rem(0.75);
    painter.text(
        pos2(x, r.top() + rem(0.5) + rem(1.143) * 0.5),
        Align2::LEFT_CENTER,
        heading,
        FontId::new(12.0, weight(600)),
        t.content_2,
    );
    painter.hline(r.x_range(), r.top() + HEAD_H - 0.5, Stroke::new(1.0, t.edge_1));
    let body_top = r.top() + HEAD_H;
    for (i, line) in lines.iter().enumerate() {
        if rows {
            let top = body_top + i as f32 * ROW_H;
            if top >= r.bottom() {
                break;
            }
            painter.text(
                pos2(x, top + ROW_H * 0.5 - 0.5),
                Align2::LEFT_CENTER,
                line,
                FontId::new(13.0, weight(400)),
                t.content_1,
            );
            painter.hline(r.x_range(), top + ROW_H - 0.5, Stroke::new(1.0, t.edge_1));
        } else {
            painter.text(
                pos2(x, body_top + rem(0.5) + 8.0),
                Align2::LEFT_CENTER,
                line,
                FontId::new(13.0, weight(400)),
                t.content_2,
            );
        }
    }
}

/// The accessible node of a pane: a region labelled by its heading.
fn pane_node(ui: &egui::Ui, r: Rect, name: &str, heading: &str, part_id: &str) {
    let resp = ui.interact(r, ui.scope_id().with(name), Sense::hover());
    resp.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, heading));
    part(ui, &resp, part_id);
}

pub struct PaneGroup {
    p: PanegroupProps,
    /// The committed layout, loaded once from memory when autoSaveId is set.
    stored: Stored,
    loaded: bool,
    /// The memory key of the saved layout, under this widget's id; None when autoSaveId is "".
    key: Option<Id>,
    /// The draft layout while a handle is dragged.
    live: Option<Stored>,
    drag: Option<Drag>,
    drag_kind: Option<Kind>,
    presses: [Option<Press>; 2],
}

impl PaneGroup {
    pub fn new(p: PanegroupProps) -> Self {
        Self {
            p,
            stored: Stored::default(),
            loaded: false,
            key: None,
            live: None,
            drag: None,
            drag_kind: None,
            presses: [None, None],
        }
    }

    /// Stores the layout (per window, in the context's memory) and clears the draft.
    fn commit(&mut self, ctx: &egui::Context, next: Stored) {
        self.stored = next;
        self.live = None;
        self.drag_kind = None;
        if let Some(id) = self.key {
            ctx.data_mut(|d| d.insert_temp(id, next));
        }
    }

    fn report(&mut self, ctx: &egui::Context, c: Cfg, next: Stored, events: &mut Vec<PanegroupEvent>) {
        self.commit(ctx, next);
        let s = resolve(c, next);
        events.push(PanegroupEvent::LayoutChanged {
            task: round3(s.task),
            split: round3(s.split),
            aside: round3(s.aside),
        });
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<PanegroupEvent> {
        let mut events: Vec<PanegroupEvent> = Vec::new();
        let t = look.t;

        self.key = (!self.p.autoSaveId.is_empty()).then(|| ui.scope_id().with(("panegroup.layout", self.p.autoSaveId.as_str())));
        if !self.loaded {
            self.loaded = true;
            if let Some(id) = self.key {
                self.stored = ui.ctx().data_mut(|d| d.get_temp::<Stored>(id)).unwrap_or_default();
            }
        }

        // The group takes the width its container gives it (or the width prop, in px), and at most 36 rem tall.
        let avail = ui.available_size();
        let width = if self.p.width > 0 { (self.p.width as f32).min(avail.x) } else { avail.x };
        let (rect, _) = ui.allocate_exact_size(vec2(width, avail.y.min(rem(36.0))), Sense::hover());

        let row = self.p.direction != "column";
        let (start, extent) = if row { (rect.left(), rect.width()) } else { (rect.top(), rect.height()) };
        let has_split = self.p.split && !self.p.noLists;
        let length = f64::from(extent);
        let has_aside = !self.p.asideHidden && !self.p.noLists && (!row || length >= ASIDE_SHOW_PX);
        let c = Cfg { row, length, has_split, has_aside };
        let view = self.live.unwrap_or(self.stored);
        let sizes = resolve(c, view);
        let collapsed = has_split && view.collapsed.is_some();

        let at = |pct: f64| start + (pct / 100.0) as f32 * extent;
        let task_end = at(sizes.task);
        let split_end = at(sizes.task + sizes.split);
        let aside_start = at(100.0 - sizes.aside);

        let task_rows: Vec<String> = (1..=12).map(|i| format!("Task {i}")).collect();
        let note_rows: Vec<String> = (1..=8).map(|i| format!("Note {i}")).collect();

        let painter = ui.painter();
        painter.rect_filled(rect, CornerRadius::same(0), t.surface_1);

        if !collapsed {
            let r = band_rect(rect, row, start, task_end);
            paint_pane(painter, t, r, t.surface_2, "Tasks", &task_rows, true);
            pane_node(ui, r, "taskPane", "Tasks", parts::TASKPANE);
        }
        if has_split {
            let r = band_rect(rect, row, task_end, split_end);
            paint_pane(painter, t, r, t.surface_2, "Notes", &note_rows, true);
            pane_node(ui, r, "splitPane", "Notes", parts::SPLITPANE);
        }
        if has_aside {
            let r = band_rect(rect, row, aside_start, start + extent);
            paint_pane(painter, t, r, t.surface_1, "Preview", &["Preview of the selected task.".to_owned()], false);
            pane_node(ui, r, "aside", "Preview", parts::ASIDE);
        }

        if has_split {
            self.handle(ui, t, rect, Kind::Task, task_end, c, sizes, &mut events);
        }
        if has_aside {
            self.handle(ui, t, rect, Kind::Aside, aside_start, c, sizes, &mut events);
        }

        if !events.is_empty() || self.drag.is_some() {
            ui.ctx().request_repaint();
        }
        events
    }

    /// One separator: its hit area, its pointer and key handling, and its line.
    #[allow(clippy::too_many_arguments)]
    fn handle(
        &mut self,
        ui: &egui::Ui,
        t: Tokens,
        rect: Rect,
        kind: Kind,
        boundary: f32,
        c: Cfg,
        sizes: Sizes,
        events: &mut Vec<PanegroupEvent>,
    ) {
        let (name, part_id, label, value) = match kind {
            Kind::Task => ("taskResizer", parts::TASKRESIZER, "Resize panes", sizes.task),
            Kind::Aside => ("asideResizer", parts::ASIDERESIZER, "Resize preview", 100.0 - sizes.aside),
        };
        let hit = band_rect(rect, c.row, boundary - HANDLE_HALF, boundary + HANDLE_HALF);
        let resp = ui
            .interact(hit, ui.scope_id().with(name), Sense::click_and_drag())
            .on_hover_cursor(if c.row { CursorIcon::ResizeHorizontal } else { CursorIcon::ResizeVertical });
        resp.widget_info(|| WidgetInfo::slider(true, round3(value), label));
        part(ui, &resp, part_id);

        self.pointer(ui, &resp, kind, c, sizes, events);
        if resp.has_focus() {
            self.keys(ui, kind, c, sizes, events);
        }

        let dragging = self.drag_kind == Some(kind);
        let line = if dragging {
            t.accent_500
        } else if resp.has_focus() {
            t.focus_1
        } else if resp.hovered() {
            t.accent_500
        } else {
            t.edge_2
        };
        let painter = ui.painter();
        painter.rect_filled(band_rect(rect, c.row, boundary - 0.5, boundary + 0.5), CornerRadius::same(0), line);
        if resp.has_focus() {
            painter.rect_stroke(hit.shrink(1.0), CornerRadius::same(0), Stroke::new(2.0, t.focus_1), StrokeKind::Inside);
        }
    }

    /// Presses and drags on a handle. A second press with no drag between resets the sizes; a drag
    /// moves the draft and reports once on release.
    fn pointer(&mut self, ui: &egui::Ui, resp: &Response, kind: Kind, c: Cfg, sizes: Sizes, events: &mut Vec<PanegroupEvent>) {
        let k = kind.index();
        let now = ui.input(|i| i.time);
        if resp.is_pointer_button_down_on() && ui.input(|i| i.pointer.button_pressed(PointerButton::Primary)) {
            let double = matches!(self.presses[k], Some(p) if !p.moved && now - p.time < DOUBLE_PRESS_S);
            if double {
                self.presses[k] = None;
                self.drag = None;
                self.report(ui.ctx(), c, Stored::default(), events);
            } else {
                let origin = resp.interact_pointer_pos().unwrap_or(Pos2::ZERO);
                self.presses[k] = Some(Press { time: now, moved: false });
                self.drag = Some(Drag { kind, origin, start: sizes, base: self.stored, draft: self.stored });
            }
        }

        let Some(d) = self.drag else {
            return;
        };
        if d.kind != kind {
            return;
        }
        if ui.input(|i| i.pointer.button_down(PointerButton::Primary)) {
            let pos = ui.input(|i| i.pointer.latest_pos()).unwrap_or(d.origin);
            let px = f64::from(if c.row { pos.x - d.origin.x } else { pos.y - d.origin.y });
            if px == 0.0 || c.length <= 0.0 {
                return;
            }
            if let Some(p) = self.presses[k].as_mut() {
                p.moved = true;
            }
            let pct = px / c.length * 100.0;
            let draft = match kind {
                Kind::Task => step_task(c, d.start, d.base, d.start.task + pct),
                Kind::Aside => step_aside(c, d.start, d.base, 100.0 - d.start.aside + pct),
            };
            if let Some(dr) = self.drag.as_mut() {
                dr.draft = draft;
            }
            self.live = Some(draft);
            self.drag_kind = Some(kind);
        } else {
            self.drag = None;
            if same_sizes(resolve(c, d.draft), d.start) {
                self.live = None;
                self.drag_kind = None;
            } else {
                self.report(ui.ctx(), c, d.draft, events);
            }
        }
    }

    /// Keys on a focused handle: arrows step by 5 percent (10 with Shift), Home and End go to the
    /// ends, and Enter on the task handle collapses or restores the task pane.
    fn keys(&mut self, ui: &egui::Ui, kind: Kind, c: Cfg, sizes: Sizes, events: &mut Vec<PanegroupEvent>) {
        let (back_key, fwd_key) = if c.row { (Key::ArrowLeft, Key::ArrowRight) } else { (Key::ArrowUp, Key::ArrowDown) };
        let (back, fwd, home, end, enter, shift) = ui.input(|i| {
            (
                i.key_pressed(back_key),
                i.key_pressed(fwd_key),
                i.key_pressed(Key::Home),
                i.key_pressed(Key::End),
                i.key_pressed(Key::Enter),
                i.modifiers.shift,
            )
        });

        if kind == Kind::Task && enter {
            let stored = self.stored;
            let aside = if c.has_aside { Some(sizes.aside) } else { stored.aside };
            let next = match stored.collapsed {
                Some(prev) => Stored { task: Some(prev), aside, collapsed: None },
                None => Stored { task: Some(sizes.task), aside, collapsed: Some(sizes.task) },
            };
            self.report(ui.ctx(), c, next, events);
            return;
        }

        let amount = if shift { 10.0 } else { 5.0 };
        let next = match kind {
            Kind::Task => {
                let (_, hi) = task_reach(sizes.aside);
                let target = if home {
                    Some(TASK_MIN)
                } else if end {
                    Some(hi)
                } else if back {
                    Some(sizes.task - amount)
                } else if fwd {
                    Some(sizes.task + amount)
                } else {
                    None
                };
                target.map(|v| step_task(c, sizes, self.stored, v))
            }
            Kind::Aside => {
                let (min_pos, max_pos) = aside_reach(c, sizes.task);
                let pos = 100.0 - sizes.aside;
                let target = if home {
                    Some(min_pos)
                } else if end {
                    Some(max_pos)
                } else if back {
                    Some(pos - amount)
                } else if fwd {
                    Some(pos + amount)
                } else {
                    None
                };
                target.map(|v| step_aside(c, sizes, self.stored, v))
            }
        };
        if let Some(n) = next {
            if !same_sizes(resolve(c, n), sizes) {
                self.report(ui.ctx(), c, n, events);
            }
        }
    }
}

crate::mounted!(PaneGroup, PanegroupProps);
