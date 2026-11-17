//! contract: omnicui/contracts/toast.contract.yaml (id: ui.toast). A transient, non-blocking notice pill at the bottom centre of the window, with an optional Undo action and a close control.
use crate::omnicui::toast::{parts, ToastEvent, ToastProps};
use crate::{icon, part, rem, weight, Look};
use egui::text::LayoutJob;
use egui::{vec2, Align2, Area, Color32, CornerRadius, FontFamily, FontId, Key, Order, Pos2, Rect, Role, Sense, Shadow, Stroke, StrokeKind, Vec2, WidgetInfo};
use std::time::Duration;

/// The entry rise: 0.28 s (the web's base duration), eased, starting 0.571rem (8 px) lower.
const RISE_SECS: f64 = 0.28;
const RISE_PX: f32 = rem(0.571);
/// How far the pill must be dragged in its dismiss direction to dismiss (Radix's swipe threshold).
const SWIPE_PX: f32 = 50.0;

/// The pill is 41 px (2.929rem) tall and sits 20 px (1.429rem) above the window bottom.
const PILL_H: f32 = rem(2.929);
const BOTTOM: f32 = rem(1.429);
/// Inside padding and gap of the pill: pl-3.5, pr-1.5, gap-2.
const PAD_L: f32 = rem(0.875);
const PAD_R: f32 = rem(0.375);
const GAP: f32 = rem(0.5);
/// The action's height and the close control's square: 28 px.
const CONTROL: f32 = rem(2.0);
/// The action's horizontal padding (px-2).
const ACTION_PAD: f32 = 8.0;
/// The pill keeps 2.286rem (32 px) clear of the window edges.
const EDGE: f32 = rem(2.286);
/// Control radius, in px.
const RADIUS: u8 = 7;

pub struct Toast {
    p: ToastProps,
    /// Milliseconds of auto-dismiss time left. It carries across a pause.
    remaining: f64,
    /// Times are egui's input clock, in seconds, so the entry and the timer follow the frames the target drives.
    last_tick: Option<f64>,
    entered: Option<f64>,
    /// The widget has asked to close. It asks once; after that it stops counting and answering.
    asked: bool,
    /// The drag so far while the pill is being swiped.
    swipe: Vec2,
    /// Whether focus was inside the pill last frame. egui drops focus on Escape before the widget reads it.
    focus_held: bool,
}

impl Toast {
    pub fn new(p: ToastProps) -> Self {
        let remaining = p.duration.max(0.0);
        Self { p, remaining, last_tick: None, entered: None, asked: false, swipe: Vec2::ZERO, focus_held: false }
    }

    /// The harness waits on this while awaitTimeout is set, until the toast has asked to close.
    pub fn waiting(&self) -> bool {
        self.p.awaitTimeout && self.p.open && !self.asked
    }

    /// The direction a swipe dismisses in.
    fn dismiss_dir(&self) -> Vec2 {
        match self.p.swipeDirection.as_str() {
            "up" => vec2(0.0, -1.0),
            "left" => vec2(-1.0, 0.0),
            "right" => vec2(1.0, 0.0),
            _ => vec2(0.0, 1.0),
        }
    }

    /// One frame: draws the toast and returns the events it raised in this frame.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<ToastEvent> {
        let mut events = vec![];
        if !self.p.open {
            // Idle: nothing is mounted. The timer, the entry and the ask start over for the next open.
            self.last_tick = None;
            self.entered = None;
            self.remaining = self.p.duration.max(0.0);
            self.asked = false;
            self.swipe = Vec2::ZERO;
            return events;
        }
        let now = ui.input(|i| i.time);
        let entered = *self.entered.get_or_insert(now);
        let progress = if self.p.reducedMotion { 1.0 } else { ((now - entered) / RISE_SECS).clamp(0.0, 1.0) };
        let eased = 1.0 - (1.0 - progress).powi(3);
        let rise = RISE_PX * ((1.0 - eased) as f32);
        if progress < 1.0 {
            ui.ctx().request_repaint();
        }
        let dir = self.dismiss_dir();
        let shift = dir * self.swipe.dot(dir).max(0.0);
        let offset = vec2(shift.x, -BOTTOM + rise + shift.y);

        let ctx = ui.ctx().clone();
        let screen_w = ctx.content_rect().width();
        let _ = Area::new(ui.scope_id().with("ui.toast.area"))
            .order(Order::Foreground)
            .movable(false)
            .interactable(true)
            .anchor(Align2::CENTER_BOTTOM, offset)
            .show(&ctx, |ui| self.pill(ui, look, screen_w, now, &mut events));
        events
    }

    /// Records that the widget asks to close, once per open. As the web harness does on onOpenChange(false),
    /// the toast then hides itself: open is taken down, so the pill is no longer drawn.
    fn dismiss(&mut self, reason: &str, events: &mut Vec<ToastEvent>) {
        events.push(ToastEvent::Dismissed { reason: reason.to_owned() });
        events.push(ToastEvent::OpenChanged { open: false });
        self.asked = true;
        self.p.open = false;
    }

    /// The pill itself, laid out in the window's bottom centre.
    fn pill(&mut self, ui: &mut egui::Ui, look: Look, screen_w: f32, now: f64, events: &mut Vec<ToastEvent>) {
        let t = look.t;
        let base = ui.scope_id().with("ui.toast");
        let failed = self.p.failed;
        let message = if failed { self.p.errorMessage.clone() } else { self.p.message.clone() };
        let label = if failed { "Try again".to_owned() } else { self.p.actionLabel.clone() };
        let show_action = self.p.showAction;
        let max_w = (screen_w - EDGE).max(0.0);

        // Widths: the action and the close control are fixed, the message takes what is left and truncates.
        let action_w = if show_action {
            let g = ui.painter().layout_no_wrap(label.clone(), FontId::new(13.0, weight(500)), t.content_1);
            g.size().x + 2.0 * ACTION_PAD
        } else {
            0.0
        };
        let gaps = if show_action { 2.0 } else { 1.0 };
        let fixed = PAD_L + PAD_R + GAP * gaps + action_w + CONTROL;
        let avail = (max_w - fixed).max(0.0);
        let mut job = LayoutJob::simple_singleline(message.clone(), FontId::new(13.0, weight(400)), t.content_1);
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
        let radius = CornerRadius::same(RADIUS);
        let ring_radius = CornerRadius::same(RADIUS + 2);

        // Parts, in tab order: the list, the notice, its message, the action, the close control.
        let viewport = ui.interact(rect, base.with("viewport"), Sense::focusable_noninteractive());
        viewport.widget_info(|| WidgetInfo::new(Role::GenericContainer));
        part(ui, &viewport, parts::VIEWPORT);

        let root = ui.interact(rect, base.with("toast"), Sense::drag());
        root.widget_info(|| WidgetInfo::new(Role::Status));
        part(ui, &root, parts::TOAST);

        let msg_rect = Rect::from_min_size(Pos2::new(rect.left() + PAD_L, cy - msg_size.y / 2.0), vec2(msg_w, msg_size.y));
        let message_resp = ui.interact(msg_rect, base.with("message"), Sense::hover());
        message_resp.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, &message));
        part(ui, &message_resp, parts::MESSAGE);

        let action_rect = Rect::from_min_size(Pos2::new(rect.left() + PAD_L + msg_w + GAP, cy - CONTROL / 2.0), vec2(action_w, CONTROL));
        let action = if show_action {
            let r = ui.interact(action_rect, base.with("action"), Sense::click());
            r.widget_info(|| WidgetInfo::labeled(Role::Button, true, &label));
            part(ui, &r, parts::ACTION);
            Some(r)
        } else {
            None
        };

        let close_rect = Rect::from_min_size(Pos2::new(rect.right() - PAD_R - CONTROL, cy - CONTROL / 2.0), vec2(CONTROL, CONTROL));
        let close = ui.interact(close_rect, base.with("close"), Sense::click());
        close.widget_info(|| WidgetInfo::labeled(Role::Button, true, "Close"));
        part(ui, &close, parts::CLOSE);

        // Behaviour.
        let focus_inside = viewport.has_focus() || close.has_focus() || action.as_ref().is_some_and(|a| a.has_focus());
        let paused = ui.rect_contains_pointer(rect) || focus_inside;
        let action_pressed = action.as_ref().is_some_and(|a| a.clicked());
        let close_pressed = close.clicked();

        // The list hotkey (F8 by default) moves focus into the list. An empty hotkey parses to no key, so it is off.
        if let Some(key) = Key::from_name(&self.p.focusHotkey) {
            if ui.input(|i| i.key_pressed(key)) {
                viewport.request_focus();
            }
        }

        // Swipe: the drag moves the pill; past the threshold in the dismiss direction it dismisses.
        let dir = self.dismiss_dir();
        if !self.asked {
            if root.dragged() || root.drag_stopped() {
                self.swipe += root.drag_delta();
            }
            if root.drag_stopped() {
                let travel = self.swipe.dot(dir);
                if travel >= SWIPE_PX {
                    // The pill stays where the swipe left it until the app closes it.
                    self.dismiss("swipe", events);
                } else {
                    self.swipe = Vec2::ZERO;
                }
            }
        }

        // Auto-dismiss: counts down while open and not hovered or focused.
        if !self.asked {
            let dt = self.last_tick.map_or(0.0, |at| (now - at) * 1000.0);
            self.last_tick = Some(now);
            if !paused {
                self.remaining -= dt;
            }
            if self.remaining <= 0.0 {
                self.dismiss("timeout", events);
            } else if !paused {
                ui.ctx().request_repaint_after(Duration::from_secs_f64(self.remaining / 1000.0));
            }
        }

        if !self.asked {
            if action_pressed {
                events.push(ToastEvent::Actioned { label: label.clone() });
                self.dismiss("action", events);
            } else if close_pressed {
                self.dismiss("close", events);
            } else if (focus_inside || self.focus_held) && ui.input(|i| i.key_pressed(Key::Escape)) {
                self.dismiss("escape", events);
            }
        }
        self.focus_held = focus_inside;

        // Paint: shadow, pill, focus ring, message, action, close.
        let shadow = Shadow {
            offset: [0, 4],
            blur: 16,
            spread: 0,
            color: Color32::from_black_alpha(if look.dark { 110 } else { 40 }),
        };
        ui.painter().add(shadow.as_shape(rect, radius));
        ui.painter().rect(rect, radius, t.surface_3, Stroke::new(1.0, t.edge_2), StrokeKind::Inside);
        if viewport.has_focus() {
            ui.painter().rect_stroke(rect.expand(2.0), ring_radius, Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
        }
        ui.painter().galley(msg_rect.min, msg, t.content_1);

        if let Some(a) = &action {
            let fill = if a.is_pointer_button_down_on() {
                Some(t.wash_2)
            } else if a.hovered() {
                Some(t.wash_1)
            } else {
                None
            };
            if let Some(f) = fill {
                ui.painter().rect_filled(action_rect, radius, f);
            }
            ui.painter().text(action_rect.center(), Align2::CENTER_CENTER, label.as_str(), FontId::new(13.0, weight(500)), t.content_1);
            if a.has_focus() {
                ui.painter().rect_stroke(action_rect.expand(2.0), ring_radius, Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
            }
        }

        let fill = if close.is_pointer_button_down_on() {
            Some(t.wash_2)
        } else if close.hovered() {
            Some(t.wash_1)
        } else {
            None
        };
        if let Some(f) = fill {
            ui.painter().rect_filled(close_rect, radius, f);
        }
        let glyph = icon("close", 16.0);
        ui.painter().text(
            close_rect.center(),
            Align2::CENTER_CENTER,
            glyph.text(),
            FontId::new(16.0, FontFamily::Name("Icons".into())),
            t.content_2,
        );
        if close.has_focus() {
            ui.painter().rect_stroke(close_rect.expand(2.0), ring_radius, Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
        }
    }
}

impl crate::Mounted for Toast {
    fn frame(&mut self, ui: &mut egui::Ui, look: Look, emit: &mut dyn FnMut(&'static str, serde_json::Value)) {
        for e in self.ui(ui, look) {
            let (name, fields) = e.to_json();
            emit(name, fields);
        }
    }

    fn busy(&self) -> bool {
        self.waiting()
    }
}

/// Builds the widget from a test's props.
pub fn mount(props: serde_json::Value) -> Result<Box<dyn crate::Mounted>, String> {
    let p: ToastProps = serde_json::from_value(props).map_err(|e| format!("props: {e}"))?;
    Ok(Box::new(Toast::new(p)))
}
