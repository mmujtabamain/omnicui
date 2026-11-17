//! contract: omnicui/contracts/toasthost.contract.yaml (id: app.toasthost). The undo notice pinned bottom-centre of the main window: a message, Undo (Try again after a failed Undo) and Dismiss. It draws nothing when no notice is set.
use crate::app::Toast;
use crate::data::Id;
use crate::App;
use egui::text::LayoutJob;
use egui::{pos2, vec2, Align2, Area, Color32, CornerRadius, FontFamily, FontId, Order, Rect, Role, Sense, Shadow, Stroke, StrokeKind, WidgetInfo};
use std::time::Duration;
use taskscape_ui::omnicui::toasthost::{parts, ToasthostEvent, ToasthostProps};
use taskscape_ui::{icon, part, rem, weight, Look, Mounted};

/// The reference timer: 6 s from the notice's start, not paused by hover or focus. A failed Undo has no timer.
const TIMEOUT_SECS: f64 = 6.0;
/// The entry rise: 0.28 s, eased, starting 0.571rem (8 px) lower.
const RISE_SECS: f64 = 0.28;
const RISE_PX: f32 = rem(0.571);
/// The pill is 41 px (2.929rem) tall and sits 17 px (1.214rem) above the window bottom.
const PILL_H: f32 = rem(2.929);
const BOTTOM: f32 = rem(1.214);
/// The pill keeps 2.286rem (32 px) clear of the window edges.
const EDGE: f32 = rem(2.286);
/// Inside padding and gap of the pill: pl-3.5, pr-1.5, gap-2.
const PAD_L: f32 = rem(0.875);
const PAD_R: f32 = rem(0.375);
const GAP: f32 = rem(0.5);
/// The buttons, as the web's Button at size md: 24.5 px tall, 8 px side padding, a 14 px glyph and 6 px between glyph and label.
const BTN_H: f32 = rem(1.75);
const BTN_PAD: f32 = 8.0;
const BTN_ICON: f32 = 14.0;
const BTN_GAP: f32 = 6.0;
/// The label is Inter at the web's md size (13 px), weight medium.
const TEXT_PX: f32 = 13.0;
const PILL_RADIUS: u8 = 7;
const ACTION_RADIUS: u8 = 4;
const CLOSE_RADIUS: u8 = 5;

/// What the notice showed last frame. A newer notice (another message, other Undo ids or an error) restarts the timer.
#[derive(Clone, Debug, PartialEq)]
struct Shown {
    message: String,
    undo: Vec<Id>,
    error: Option<String>,
}

/// The undo notice. The notice itself is `app.toast`; the screen keeps only its timer and entry animation.
pub struct ToastHost {
    shown: Option<Shown>,
    /// When the shown notice appeared, on egui's input clock (seconds).
    since: f64,
}

impl ToastHost {
    pub fn new() -> Self {
        Self { shown: None, since: 0.0 }
    }

    /// One frame: draws the notice at the window's bottom centre and returns the events it raised.
    pub fn show(&mut self, app: &mut App, ui: &mut egui::Ui, look: Look) -> Vec<ToasthostEvent> {
        let mut events = vec![];
        let Some(toast) = app.toast.clone() else {
            self.shown = None;
            return events;
        };
        let now = ui.input(|i| i.time);
        let key = Shown { message: toast.message.clone(), undo: toast.undo.clone(), error: toast.error.clone() };
        if self.shown.as_ref() != Some(&key) {
            self.shown = Some(key);
            self.since = now;
        }

        // The timer closes the notice after 6 s, and reports nothing (only Undo and Dismiss are events).
        let failed = toast.error.is_some();
        if !failed {
            let left = TIMEOUT_SECS - (now - self.since);
            if left <= 0.0 {
                app.toast = None;
                self.shown = None;
                return events;
            }
            ui.ctx().request_repaint_after(Duration::from_secs_f64(left));
        }

        // The entry rise; with reduced motion on (the setting, not the OS), it is at rest at once.
        let reduce = app.data.settings.reduced_motion == "on";
        let progress = if reduce { 1.0 } else { ((now - self.since) / RISE_SECS).clamp(0.0, 1.0) };
        if progress < 1.0 {
            ui.ctx().request_repaint();
        }
        let eased = 1.0 - (1.0 - progress).powi(3);
        let rise = RISE_PX * ((1.0 - eased) as f32);

        let ctx = ui.ctx().clone();
        let screen_w = ctx.content_rect().width();
        let text = toast.error.clone().unwrap_or_else(|| toast.message.clone());
        let (undo_pressed, dismiss_pressed) = Area::new(ui.scope_id().with("app.toasthost.area"))
            .order(Order::Foreground)
            .movable(false)
            .interactable(true)
            .anchor(Align2::CENTER_BOTTOM, vec2(0.0, -BOTTOM + rise))
            .show(&ctx, |ui| pill(ui, look, &text, failed, screen_w))
            .inner;

        if dismiss_pressed {
            app.toast = None;
            self.shown = None;
            events.push(ToasthostEvent::Dismissed { via: "dismiss".to_owned() });
        } else if undo_pressed {
            self.undo(app, &toast, &mut events);
        }
        events
    }

    /// Undo: puts the trashed tasks back. On success the notice closes and `undone` and `dismissed` (via undo) are raised.
    /// On failure the notice stays with the error, and its action reads Try again; no event is raised.
    fn undo(&mut self, app: &mut App, toast: &Toast, events: &mut Vec<ToasthostEvent>) {
        if toast.undo.is_empty() {
            // An error with nothing to restore (a refused save): there is nothing to try again, so the notice closes.
            app.toast = None;
            self.shown = None;
            return;
        }
        // The count and the title are the root tasks restored (subtasks come back with their parent, uncounted).
        // Read from the Trash before the restore moves them out. The title is the first root in list order
        // (sort_order), as the web's bulk notice names it, whatever order the store returns the ids in.
        let mut roots: Vec<(f64, String)> = toast
            .undo
            .iter()
            .filter_map(|id| app.data.trash.iter().find(|t| &t.id == id))
            .filter(|t| t.parent_id.is_none())
            .map(|t| (t.sort_order, t.title.clone()))
            .collect();
        roots.sort_by(|a, b| a.0.total_cmp(&b.0));
        let count = roots.len() as i64;
        let title = roots.first().map(|r| r.1.clone()).unwrap_or_default();
        match app.data.restore_tasks(&toast.undo) {
            Ok(()) => {
                app.toast = None;
                self.shown = None;
                events.push(ToasthostEvent::Undone { count, title });
                events.push(ToasthostEvent::Dismissed { via: "undo".to_owned() });
            }
            Err(e) => {
                app.last_error = Some(e.to_string());
                app.toast = Some(Toast { message: String::new(), undo: toast.undo.clone(), error: Some("Could not undo the delete.".to_owned()) });
            }
        }
    }
}

/// The pill: the notice surface with its message, the action and the close control. Returns whether the action and the close control were pressed this frame.
fn pill(ui: &mut egui::Ui, look: Look, text: &str, failed: bool, screen_w: f32) -> (bool, bool) {
    let t = look.t;
    let base = ui.scope_id().with("app.toasthost");
    let action = if failed { "Try again" } else { "Undo" };

    // Widths: the action and the close control are fixed; the message takes what is left and truncates.
    let action_size = ui.painter().layout_no_wrap(action.to_owned(), FontId::new(TEXT_PX, weight(500)), t.accent_500).size();
    let action_w = BTN_PAD + BTN_ICON + BTN_GAP + action_size.x + BTN_PAD;
    let max_w = (screen_w - EDGE).max(0.0);
    let fixed = PAD_L + PAD_R + GAP * 2.0 + action_w + BTN_H;
    let avail = (max_w - fixed).max(0.0);
    let mut job = LayoutJob::simple_singleline(text.to_owned(), FontId::new(TEXT_PX, weight(500)), t.content_1);
    job.wrap.max_width = avail;
    job.wrap.max_rows = 1;
    job.wrap.break_anywhere = true;
    job.wrap.overflow_character = Some('…');
    let msg = ui.painter().layout_job(job);
    let msg_size = msg.size();
    let msg_w = msg_size.x.min(avail);
    let pill_w = fixed + msg_w;

    let (rect, _) = ui.allocate_exact_size(vec2(pill_w, PILL_H), Sense::hover());
    let cy = rect.center().y;
    let radius = CornerRadius::same(PILL_RADIUS);

    // Parts, in tab order: the notice, its message, the action, the close control.
    let notice = ui.interact(rect, base.with("notice"), Sense::hover());
    notice.widget_info(|| WidgetInfo::new(Role::GenericContainer));
    part(ui, &notice, parts::NOTICE);

    let msg_rect = Rect::from_min_size(pos2(rect.left() + PAD_L, cy - msg_size.y / 2.0), vec2(msg_w, msg_size.y));
    let message = ui.interact(msg_rect, base.with("message"), Sense::hover());
    message.widget_info(|| WidgetInfo::labeled(Role::Status, true, text));
    part(ui, &message, parts::MESSAGE);

    let action_rect = Rect::from_min_size(pos2(msg_rect.right() + GAP, cy - BTN_H / 2.0), vec2(action_w, BTN_H));
    let undo = ui.interact(action_rect, base.with("undo"), Sense::click());
    undo.widget_info(|| WidgetInfo::labeled(Role::Button, true, action));
    part(ui, &undo, parts::UNDO);

    let close_rect = Rect::from_min_size(pos2(rect.right() - PAD_R - BTN_H, cy - BTN_H / 2.0), vec2(BTN_H, BTN_H));
    let dismiss = ui.interact(close_rect, base.with("dismiss"), Sense::click());
    dismiss.widget_info(|| WidgetInfo::labeled(Role::Button, true, "Dismiss"));
    part(ui, &dismiss, parts::DISMISS);
    // The web's Dismiss carries its name as its tooltip.
    let _ = dismiss.clone().on_hover_text("Dismiss");

    // Paint: the shadow and the surface, then the message.
    let shadow = Shadow { offset: [0, 4], blur: 16, spread: 0, color: Color32::from_black_alpha(if look.dark { 110 } else { 40 }) };
    ui.painter().add(shadow.as_shape(rect, radius));
    ui.painter().rect(rect, radius, t.surface_3, Stroke::new(1.0, t.edge_2), StrokeKind::Inside);
    ui.painter().galley(msg_rect.min, msg, t.content_1);

    // The action: its glyph and label in the accent tone (a text hover, no fill), with a focus ring.
    let action_col = if undo.hovered() { t.accent_600 } else { t.accent_500 };
    let action_cr = CornerRadius::same(ACTION_RADIUS);
    if undo.has_focus() {
        ui.painter().rect_stroke(action_rect, action_cr, Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
    }
    let undo_glyph = icon("undo", BTN_ICON);
    ui.painter().text(
        pos2(action_rect.left() + BTN_PAD + BTN_ICON / 2.0, cy),
        Align2::CENTER_CENTER,
        undo_glyph.text(),
        FontId::new(BTN_ICON, FontFamily::Name("Icons".into())),
        action_col,
    );
    ui.painter().text(
        pos2(action_rect.left() + BTN_PAD + BTN_ICON + BTN_GAP, cy),
        Align2::LEFT_CENTER,
        action,
        FontId::new(TEXT_PX, weight(500)),
        action_col,
    );

    // The close control: the close glyph in the muted tone, with a focus ring.
    let close_col = if dismiss.hovered() { t.content_2 } else { t.content_3 };
    if dismiss.has_focus() {
        ui.painter().rect_stroke(close_rect, CornerRadius::same(CLOSE_RADIUS), Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
    }
    let close_glyph = icon("close", BTN_ICON);
    ui.painter().text(
        close_rect.center(),
        Align2::CENTER_CENTER,
        close_glyph.text(),
        FontId::new(BTN_ICON, FontFamily::Name("Icons".into())),
        close_col,
    );

    (undo.clicked(), dismiss.clicked())
}

/// The test's mount: the app on the props' data set, with the deletes the props name run through the app's own Move to Trash.
/// Applied in the contract's order: bulkTrash, then trash, then trashAgain; each replaces the notice.
struct Mount {
    app: App,
    screen: ToastHost,
}

impl Mounted for Mount {
    fn frame(&mut self, ui: &mut egui::Ui, look: Look, emit: &mut dyn FnMut(&'static str, serde_json::Value)) {
        for e in self.screen.show(&mut self.app, ui, look) {
            let (n, f) = e.to_json();
            emit(n, f);
        }
    }
}

/// The Errands list of the Home project (or of any project, if Home has none).
fn errands(app: &App) -> Option<Id> {
    let home = app.data.project_named("Home").map(|p| p.id.clone());
    app.data
        .list_named(home.as_deref(), "Errands")
        .or_else(|| app.data.list_named(None, "Errands"))
        .map(|l| l.id.clone())
}

/// The id of the task of this title in the list.
fn task_titled(app: &App, list: &str, title: &str) -> Option<Id> {
    if title.is_empty() {
        return None;
    }
    app.data.tasks_of(list).into_iter().find(|t| t.title == title).map(|t| t.id.clone())
}

pub fn mount(mut app: App, props: serde_json::Value) -> Result<Box<dyn Mounted>, String> {
    let p: ToasthostProps = serde_json::from_value(props).map_err(|e| e.to_string())?;
    if let Some(list) = errands(&app) {
        if p.bulkTrash > 0 {
            // The first n root tasks of Errands in visual order: a bulk delete of roots only.
            let roots: Vec<Id> = app
                .data
                .tasks_of(&list)
                .into_iter()
                .filter(|t| t.parent_id.is_none())
                .take(p.bulkTrash.max(0) as usize)
                .map(|t| t.id.clone())
                .collect();
            if !roots.is_empty() {
                app.trash(&roots);
                let n = roots.len();
                if let Some(t) = app.toast.as_mut() {
                    t.message = format!("{n} moved to Trash");
                }
            }
        }
        for title in [p.trash.as_str(), p.trashAgain.as_str()] {
            if let Some(id) = task_titled(&app, &list, title) {
                app.trash(&[id]);
            }
        }
    }
    Ok(Box::new(Mount { app, screen: ToastHost::new() }))
}
