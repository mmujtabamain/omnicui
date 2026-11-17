//! contract: omnicui/contracts/disclosurebutton.contract.yaml (id: ui.disclosurebutton). The expand or collapse control on a task row: a chevron that turns when the row's subtasks show, revealed on row hover or keyboard focus while expanded.
use crate::omnicui::disclosurebutton::{parts, DisclosurebuttonEvent, DisclosurebuttonProps};
use crate::{part, rem, weight, Look};
use egui::text::{LayoutJob, TextFormat};
use egui::{pos2, vec2, CornerRadius, FontId, Galley, Rect, Role, Sense, Stroke, StrokeKind, WidgetInfo};
use std::sync::Arc;

/// The 18px box (the web's 1.2857rem). Its hit area pads to the 24px target minimum.
const BOX: f32 = rem(1.2857);
/// How far the hit area reaches past the box on each side (the web's before pseudo-element).
const PAD_HIT: f32 = 3.0;
/// The row is py-1.5 around the box.
const ROW_H: f32 = BOX + 2.0 * rem(0.375);
/// The row's px-2 and gap-2.
const PAD: f32 = 8.0;
const GAP: f32 = 8.0;
/// The web's chevron turn and reveal both use the fast duration.
const FAST: f32 = 0.18;

pub struct DisclosureButton {
    p: DisclosurebuttonProps,
    /// The pointer is over the task row the control sits in. The row sets it before each frame.
    row_hot: bool,
}

impl DisclosureButton {
    pub fn new(p: DisclosurebuttonProps) -> Self {
        Self { p, row_hot: false }
    }

    /// One frame at the cursor: draws the control and returns the events it raised in this frame.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<DisclosurebuttonEvent> {
        if self.p.hidden {
            return vec![];
        }
        let (rect, _) = ui.allocate_exact_size(vec2(BOX, BOX), Sense::hover());
        self.draw_at(ui, look, rect)
    }

    /// Draws the control in the given 18px box, as the row places it, and returns the events it raised.
    fn draw_at(&mut self, ui: &mut egui::Ui, look: Look, rect: Rect) -> Vec<DisclosurebuttonEvent> {
        let mut events = vec![];
        if self.p.hidden {
            return events;
        }
        let t = look.t;
        // The 24px hit area is registered after the row, so it sits on top of the row's own click.
        let mut response = ui.interact(rect.expand(PAD_HIT), ui.scope_id().with("disclosure"), Sense::click());
        // A click, or Enter or Space on the focused control, toggles it. egui turns those keys into a click.
        if response.clicked() {
            self.p.expanded = !self.p.expanded;
            response.mark_changed();
            events.push(DisclosurebuttonEvent::ExpandedChanged { expanded: self.p.expanded });
        }
        let expanded = self.p.expanded;
        let label = if expanded { "Collapse" } else { "Expand" };
        response.widget_info(|| WidgetInfo::labeled(Role::Button, true, label));
        ui.ctx().accesskit_node_builder(response.id, |node| node.set_expanded(expanded));
        part(ui, &response, parts::DISCLOSURE);

        let id = response.id;
        let hot = response.hovered();
        // Expanded: shown on row hover, or while the control has keyboard focus. Collapsed: always shown.
        let shown = hot || self.row_hot || response.has_focus();
        let alpha = if expanded { ui.ctx().animate_bool_with_time(id.with("reveal"), shown, FAST) } else { 1.0 };
        // The chevron turns -90 degrees when expanded (the web's -rotate-90).
        let turn = ui.ctx().animate_bool_with_time(id.with("chevron"), expanded, FAST);

        let painter = ui.painter();
        let cr = CornerRadius::same(4);
        if hot {
            painter.rect_filled(rect, cr, t.wash_2.gamma_multiply(alpha));
        }
        if response.has_focus() {
            painter.rect_stroke(rect, cr, Stroke::new(2.0, t.focus_1.gamma_multiply(alpha)), StrokeKind::Outside);
        }
        // The chevron_right glyph as a thin stroke, rotated about the box centre by the turn.
        let ink = (if hot { t.content_1 } else { t.content_2 }).gamma_multiply(alpha);
        let c = rect.center();
        let (s, co) = (-turn * std::f32::consts::FRAC_PI_2).sin_cos();
        let at = |x: f32, y: f32| pos2(c.x + x * co - y * s, c.y + x * s + y * co);
        let stroke = Stroke::new(1.25, ink);
        let (p0, p1, p2) = (at(-2.5, -4.0), at(2.5, 0.0), at(-2.5, 4.0));
        painter.line_segment([p0, p1], stroke);
        painter.line_segment([p1, p2], stroke);
        events
    }
}

/// The task row the control sits in, as the web's harness mounts it: the control left of the title,
/// and the subtask counter at the right. A click on the row, outside the control, raises rowOpened.
pub struct Row {
    disc: DisclosureButton,
}

/// A label laid out on one line in a colour, with an ellipsis when it is wider than max_w.
fn laid_out(ui: &egui::Ui, s: &str, font: FontId, color: egui::Color32, max_w: f32) -> Arc<Galley> {
    let mut job = LayoutJob::single_section(s.to_owned(), TextFormat { font_id: font, color, ..Default::default() });
    job.wrap.max_width = max_w;
    job.wrap.max_rows = 1;
    job.wrap.break_anywhere = true;
    job.wrap.overflow_character = Some('…');
    ui.painter().layout_job(job)
}

impl crate::Mounted for Row {
    fn frame(&mut self, ui: &mut egui::Ui, look: Look, emit: &mut dyn FnMut(&'static str, serde_json::Value)) {
        let t = look.t;
        // The row is registered first, so the control (registered after it) takes clicks over it.
        let (row, row_resp) = ui.allocate_exact_size(vec2(rem(20.0), ROW_H), Sense::click());
        row_resp.widget_info(|| WidgetInfo::new(Role::GenericContainer));
        part(ui, &row_resp, parts::ROW);
        let hot = ui.rect_contains_pointer(row);
        self.disc.row_hot = hot;
        if hot {
            ui.painter().rect_filled(row, CornerRadius::same(4), t.wash_1);
        }

        let cy = row.center().y;
        let x0 = row.left() + PAD;
        let box_rect = Rect::from_min_size(pos2(x0, cy - BOX / 2.0), vec2(BOX, BOX));
        let disc_shown = !self.disc.p.hidden;
        let mut events = vec![];
        let mut x = x0;
        if disc_shown {
            events = self.disc.draw_at(ui, look, box_rect);
            x = x0 + BOX + GAP;
        }
        // A click on the control itself is not a row open; the control never causes rowOpened.
        let over_disc = disc_shown && ui.rect_contains_pointer(box_rect.expand(PAD_HIT));
        if row_resp.clicked() && !over_disc {
            events.push(DisclosurebuttonEvent::RowOpened {});
        }

        let counter = laid_out(ui, "2/5", FontId::new(rem(0.857), weight(400)), t.content_3, f32::INFINITY);
        let cx = row.right() - PAD - counter.size().x;
        let title = laid_out(ui, "Write the quarterly report", FontId::new(rem(0.929), weight(400)), t.content_1, (cx - GAP - x).max(0.0));
        let painter = ui.painter();
        painter.galley(pos2(x, cy - title.size().y / 2.0), title, t.content_1);
        painter.galley(pos2(cx, cy - counter.size().y / 2.0), counter, t.content_3);

        for e in events {
            let (name, fields) = e.to_json();
            emit(name, fields);
        }
    }
}

/// Builds the task row from a test's props.
pub fn mount(props: serde_json::Value) -> Result<Box<dyn crate::Mounted>, String> {
    let p: DisclosurebuttonProps = serde_json::from_value(props).map_err(|e| format!("props: {e}"))?;
    Ok(Box::new(Row { disc: DisclosureButton::new(p) }))
}
