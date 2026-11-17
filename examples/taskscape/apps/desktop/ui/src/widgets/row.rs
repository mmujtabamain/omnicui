//! contract: omnicui/contracts/row.contract.yaml (id: ui.row). The background and layout of one list row: the fill per state, the content slot shifted by indent, and the Open control that shows on hover and on keyboard focus.
use crate::omnicui::row::{parts, RowEvent, RowProps};
use crate::{part, rem, text, weight, Look};
use egui::{pos2, vec2, Align, Align2, CornerRadius, FontId, Layout, Rect, Role, Sense, Stroke, StrokeKind, UiBuilder, WidgetInfo};

/// The Web's transition-colors duration (0.18 s fast).
const FAST: f32 = 0.18;

/// The row surface. It paints the fill and lays its content out in the content slot.
pub struct Row {
    p: RowProps,
}

impl Row {
    pub fn new(p: RowProps) -> Self {
        Self { p }
    }

    /// Multi-select rows (inset, md) carry the Open control; other rows do not.
    pub fn has_open(&self) -> bool {
        self.p.shape == "inset" && self.p.density == "md"
    }

    /// Draws the bare surface with no children. Returns the events it raised (none).
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<RowEvent> {
        self.show(ui, look, |_, _| {});
        vec![]
    }

    /// Draws the surface, then the content slot. `content` gets the slot's Ui and whether the row is hovered.
    pub fn show(&self, ui: &mut egui::Ui, look: Look, content: impl FnOnce(&mut egui::Ui, bool)) {
        let t = look.t;
        let inset = self.p.shape != "full-bleed";
        let height = if self.p.density == "sm" { rem(2.0) } else { rem(2.25) };
        let width = ui.available_width();
        let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::hover());

        // A selected row keeps its fill while hovered; the hover wash shows only on unselected rows.
        let hovered = response.contains_pointer();
        let selected = self.p.selected;
        let hover_t = ui.ctx().animate_bool_with_time(response.id.with("wash"), hovered && !selected, FAST);
        let fill = if selected {
            if self.p.tone == "wash" { t.wash_1 } else { t.selection_1 }
        } else {
            t.wash_1.gamma_multiply(hover_t)
        };
        let radius = if inset { CornerRadius::same(4) } else { CornerRadius::ZERO };
        ui.painter().rect_filled(rect, radius, fill);

        response.widget_info(|| WidgetInfo::new(Role::GenericContainer));
        part(ui, &response, parts::ROW);

        // The content slot: inside the 14px side padding, shifted right by the indent.
        let pad = rem(1.0);
        let left = (rect.left() + pad + self.p.indent as f32).min(rect.right() - pad);
        let content_rect = Rect::from_min_max(pos2(left, rect.top()), pos2(rect.right() - pad, rect.bottom()));
        let slot = ui.interact(content_rect, response.id.with("content"), Sense::hover());
        slot.widget_info(|| WidgetInfo::new(Role::GenericContainer));
        part(ui, &slot, parts::CONTENT);

        let gap = if inset { rem(1.25) } else { 0.0 };
        ui.scope_builder(UiBuilder::new().max_rect(content_rect).layout(Layout::left_to_right(Align::Center)), |ui| {
            ui.spacing_mut().item_spacing.x = gap;
            content(ui, hovered);
        });
    }
}

fn open_font() -> FontId {
    FontId::new(12.0, weight(500))
}

/// The Open control's width: its label plus 8px of padding each side.
fn open_width(ui: &egui::Ui) -> f32 {
    ui.painter().layout_no_wrap("Open".to_owned(), open_font(), ui.visuals().text_color()).size().x + 16.0
}

/// The Open control of a multi-select row. It is always laid out, so it can take keyboard focus;
/// it fades in (opacity) while the row is hovered or the control has focus.
fn open_control(ui: &mut egui::Ui, look: Look, reveal: bool, active: bool) -> egui::Response {
    let t = look.t;
    let alpha = ui.ctx().animate_bool_with_time(ui.scope_id().with("open-reveal"), reveal, FAST);
    let font = open_font();
    let (rect, resp) = ui.allocate_exact_size(vec2(open_width(ui), rem(1.714)), Sense::click());
    resp.widget_info(|| WidgetInfo::labeled(Role::Button, true, "Open"));
    part(ui, &resp, parts::OPEN);

    let fill = if active || resp.hovered() { t.wash_2 } else { t.surface_3 };
    let p = ui.painter();
    p.rect(
        rect,
        CornerRadius::same(7),
        fill.gamma_multiply(alpha),
        Stroke::new(1.0, t.edge_2.gamma_multiply(alpha)),
        StrokeKind::Inside,
    );
    p.text(rect.center(), Align2::CENTER_CENTER, "Open", font, t.content_1.gamma_multiply(alpha));
    if resp.has_focus() {
        p.rect_stroke(rect.expand(2.0), CornerRadius::same(9), Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
    }
    resp
}

/// The harness: a title, and the Open control on multi-select rows. A press toggles its active mark.
pub struct RowMount {
    row: Row,
    opened: bool,
    open_focused: bool,
}

impl RowMount {
    pub fn new(p: RowProps) -> Self {
        Self { row: Row::new(p), opened: false, open_focused: false }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<RowEvent> {
        let multi = self.row.has_open();
        let RowMount { row, opened, open_focused } = self;
        row.show(ui, look, |ui, hovered| {
            // The title takes the room left of the Open control (flex-1 min-w-0), then the control.
            let open_room = if multi { open_width(ui) + ui.spacing().item_spacing.x } else { 0.0 };
            let title_w = (ui.available_width() - open_room).max(0.0);
            ui.scope(|ui| {
                ui.set_width(title_w);
                ui.add(egui::Label::new(text("Write the quarterly report", 14.0, 400).color(look.t.content_1)).truncate());
            });
            if multi {
                let resp = open_control(ui, look, hovered || *open_focused, *opened);
                if resp.clicked() {
                    *opened = !*opened;
                }
                *open_focused = resp.has_focus();
            } else {
                *open_focused = false;
            }
        });
        vec![]
    }
}

impl crate::Mounted for RowMount {
    fn frame(&mut self, ui: &mut egui::Ui, look: Look, emit: &mut dyn FnMut(&'static str, serde_json::Value)) {
        for e in self.ui(ui, look) {
            let (name, fields) = e.to_json();
            emit(name, fields);
        }
    }
}

/// Builds the row from a test's props.
pub fn mount(props: serde_json::Value) -> Result<Box<dyn crate::Mounted>, String> {
    let p: RowProps = serde_json::from_value(props).map_err(|e| format!("props: {e}"))?;
    Ok(Box::new(RowMount::new(p)))
}
