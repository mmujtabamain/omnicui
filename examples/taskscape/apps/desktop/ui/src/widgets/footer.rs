//! contract: omnicui/contracts/footer.contract.yaml (id: ui.footer). A padded action bar at the foot of a sheet or dialog: the caller's buttons sit right-aligned over an optional fill and hairline.
use crate::omnicui::footer::{parts, FooterEvent, FooterProps};
use crate::{part, rem, weight, Look, Mounted};
use egui::{pos2, vec2, Align2, Color32, CornerRadius, FontId, Rect, Role, Sense, Stroke, StrokeKind, WidgetInfo};

/// The bar. It lays out nothing of its own: it draws the fill, the hairline and the drag region, and
/// hands the content rect to the caller, which places its actions in it.
pub struct Footer {
    p: FooterProps,
}

impl Footer {
    pub fn new(p: FooterProps) -> Self {
        Self { p }
    }

    /// One frame: draws the bar, calls `actions` with the rect its buttons go in, and returns the events
    /// it raised (none: the bar emits nothing of its own).
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look, actions: impl FnOnce(&mut egui::Ui, Rect)) -> Vec<FooterEvent> {
        let row = self.p.density == "row";
        let drag = self.p.drag == "window";
        // The web box is border-box: the hairline sits inside the height, and py-5 pads a row.
        let hairline = if self.p.edge == "none" { 0.0 } else { 1.0 };
        let bar_h = rem(2.25);
        let height = if row { 40.0 + bar_h + hairline } else { bar_h };
        // A drag region takes presses on its background only: the buttons are drawn over it, later, so they win.
        let sense = if drag { Sense::drag() } else { Sense::hover() };
        let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), height), sense);
        if drag && response.drag_started() {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }
        response.widget_info(|| WidgetInfo::new(Role::GenericContainer));
        part(ui, &response, parts::FOOTER);

        let t = look.t;
        let painter = ui.painter();
        if self.p.tone == "raised" {
            painter.rect_filled(rect, CornerRadius::same(0), t.surface_2);
        }
        if self.p.edge == "top" {
            painter.rect_filled(Rect::from_min_size(rect.left_top(), vec2(rect.width(), 1.0)), CornerRadius::same(0), t.edge_2);
        } else if self.p.edge == "bottom" {
            painter.rect_filled(Rect::from_min_size(rect.left_bottom() - vec2(0.0, 1.0), vec2(rect.width(), 1.0)), CornerRadius::same(0), t.edge_2);
        }

        // px-6 is 24px on each side; a row adds py-5 (20px) above and below, inside the hairline.
        let pad_y = if row { 20.0 } else { 0.0 };
        let top = rect.top() + pad_y + (if self.p.edge == "top" { 1.0 } else { 0.0 });
        let bottom = rect.bottom() - pad_y - (if self.p.edge == "bottom" { 1.0 } else { 0.0 });
        let content = Rect::from_min_max(pos2(rect.left() + 24.0, top), pos2(rect.right() - 24.0, bottom));
        actions(ui, content);
        Vec::new()
    }
}

/// The mounted bar: the harness's Cancel and Save buttons, laid out in the bar the way the web harness does.
pub struct FooterMounted {
    footer: Footer,
}

impl Mounted for FooterMounted {
    fn frame(&mut self, ui: &mut egui::Ui, look: Look, emit: &mut dyn FnMut(&'static str, serde_json::Value)) {
        let row = self.footer.p.density == "row";
        for e in self.footer.ui(ui, look, |ui, content| draw_actions(ui, look, content, row)) {
            let (name, fields) = e.to_json();
            emit(name, fields);
        }
    }
}

/// The harness's actions in DOM order: Cancel (outline) then Save (primary), right-aligned in the bar.
fn draw_actions(ui: &mut egui::Ui, look: Look, content: Rect, row: bool) {
    // The web's h-8 bar buttons (px-3, gap-2) and h-9 row buttons (px-4, gap-3).
    let (height, pad_x, gap) = if row { (rem(2.25), 16.0, 12.0) } else { (rem(2.0), 12.0, 8.0) };
    let cancel_w = label_width(ui, "Cancel", look.t.content_1) + 2.0 * pad_x;
    let save_w = label_width(ui, "Save", look.t.on_accent) + 2.0 * pad_x;
    let top = content.center().y - height / 2.0;
    let save = Rect::from_min_size(pos2(content.right() - save_w, top), vec2(save_w, height));
    let cancel = Rect::from_min_size(pos2(save.left() - gap - cancel_w, top), vec2(cancel_w, height));
    button(ui, look, cancel, parts::CANCEL, "Cancel", false);
    button(ui, look, save, parts::ACCEPT, "Save", true);
}

fn label_width(ui: &egui::Ui, label: &str, color: Color32) -> f32 {
    ui.painter().layout_no_wrap(label.to_owned(), FontId::new(13.0, weight(500)), color).size().x
}

/// One button: an outline (Cancel) or primary (Save) control with a 7px radius, a hover wash and a focus ring.
fn button(ui: &mut egui::Ui, look: Look, rect: Rect, part_id: &'static str, label: &str, primary: bool) {
    let id = ui.scope_id().with(part_id);
    let response = ui.interact(rect, id, Sense::click());
    // egui drops focus on Escape before the frame runs; the footer does not take Escape, so the
    // button that had focus keeps it, as the web's Cancel does.
    let kept_key = id.with("kept_focus");
    let was_focused = ui.ctx().data(|d| d.get_temp::<bool>(kept_key)).unwrap_or(false);
    let escape = ui.input(|i| i.key_pressed(egui::Key::Escape));
    if was_focused && escape && !response.has_focus() {
        response.request_focus();
    }
    let kept = response.has_focus() || (was_focused && escape);
    ui.ctx().data_mut(|d| d.insert_temp(kept_key, kept));
    response.widget_info(|| WidgetInfo::labeled(Role::Button, true, label));
    part(ui, &response, part_id);

    let t = look.t;
    let hovered = response.hovered();
    let radius = CornerRadius::same(7);
    let (fill, fg, stroke) = if primary {
        (if hovered { t.accent_600 } else { t.accent_500 }, t.on_accent, Stroke::NONE)
    } else {
        (t.surface_3, t.content_1, Stroke::new(1.0, t.edge_2))
    };
    let painter = ui.painter();
    painter.rect(rect, radius, fill, stroke, StrokeKind::Inside);
    if hovered && !primary {
        painter.rect_filled(rect, radius, t.wash_2);
    }
    if response.has_focus() {
        // 2px of focus_1 just outside the button.
        painter.rect_stroke(rect.expand(2.0), CornerRadius::same(9), Stroke::new(2.0, t.focus_1), StrokeKind::Inside);
    }
    painter.text(rect.center(), Align2::CENTER_CENTER, label, FontId::new(13.0, weight(500)), fg);
}

/// Builds the bar from a test's props.
pub fn mount(props: serde_json::Value) -> Result<Box<dyn Mounted>, String> {
    let p: FooterProps = serde_json::from_value(props).map_err(|e| format!("props: {e}"))?;
    Ok(Box::new(FooterMounted { footer: Footer::new(p) }))
}
