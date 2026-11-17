//! contract: omnicui/contracts/subtaskrow.contract.yaml (id: ui.subtaskrow). A subtask row in the task inspector: a 28px row with a square done checkbox and a title button that selects the subtask, indented 18px per depth level from 6px.
use crate::omnicui::button::{ButtonEvent, ButtonProps};
use crate::omnicui::checkbox::{CheckboxEvent, CheckboxProps};
use crate::omnicui::subtaskrow::{parts, SubtaskrowEvent, SubtaskrowProps};
use crate::widgets::button::Button;
use crate::widgets::checkbox::Checkbox;
use crate::{part, rem, Look};
use egui::{pos2, vec2, Align, CornerRadius, Id, Layout, Rect, Role, Sense, Stroke, UiBuilder, WidgetInfo};

pub struct SubtaskRow {
    p: SubtaskrowProps,
    /// The done state. A press writes the next value back, the way the parent does in the web harness.
    done: bool,
}

/// Retags the node egui drew for `id` with a contract part id. The composed widgets tag their own
/// ids (ui.checkbox.control, ui.button.button); the row gives its checkbox and title their part ids.
fn retag(ui: &egui::Ui, id: Id, part_id: &str) {
    ui.ctx().accesskit_node_builder(id, |node| node.set_author_id(part_id.to_owned()));
}

/// egui gives a control keyboard focus on a pointer press. The web shows its focus ring for keys only,
/// so a press hands the focus back.
fn drop_pointer_focus(ui: &egui::Ui, id: Id) {
    if ui.input(|i| i.pointer.any_pressed() || i.pointer.any_released()) {
        ui.ctx().memory_mut(|m| m.surrender_focus(id));
    }
}

impl SubtaskRow {
    pub fn new(p: SubtaskrowProps) -> Self {
        let done = p.done;
        Self { p, done }
    }

    /// One frame: draws the row and returns the events it raised in this frame.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<SubtaskrowEvent> {
        let mut events = vec![];
        let t = look.t;
        let done = self.done;
        let title = self.p.title.clone();

        // The row: 28px tall, a field that washes on hover. Left padding 6px at depth 0 plus 18px per
        // level; 8px on the right (px-2).
        let width = if ui.available_width().is_finite() { ui.available_width() } else { 240.0 };
        let (row, row_resp) = ui.allocate_exact_size(vec2(width, rem(2.0)), Sense::hover());
        row_resp.widget_info(|| WidgetInfo::new(Role::GenericContainer));
        part(ui, &row_resp, parts::ROW);
        if ui.rect_contains_pointer(row) {
            ui.painter().rect_filled(row, CornerRadius::same(4), t.wash_1);
        }
        let indent = rem(0.4286) + self.p.depth.max(0.0) as f32 * rem(1.2857);
        let inner = Rect::from_min_max(pos2(row.left() + indent, row.top()), pos2(row.right() - 8.0, row.bottom()));
        let cy = row.center().y;

        let mut toggled: Option<bool> = None;
        let mut selected = false;
        let _ = ui.scope_builder(UiBuilder::new().max_rect(inner).layout(Layout::left_to_right(Align::Center)), |ui| {
            // No gap between the parts: the title's own 8px left padding is the gap.
            ui.spacing_mut().item_spacing.x = 0.0;

            // The checkbox: controlled by done, a press reports the next value.
            let cb_id = ui.next_auto_id();
            let mut cb = Checkbox::new(CheckboxProps {
                checked: done,
                ariaLabel: title.clone(),
                intent: "done".into(),
                shape: "square".into(),
                size: "sm".into(),
                ..Default::default()
            });
            for e in cb.ui(ui, look) {
                let CheckboxEvent::CheckedChange { checked } = e;
                toggled = Some(checked);
            }
            retag(ui, cb_id, parts::CHECKBOX);
            drop_pointer_focus(ui, cb_id);

            // The title: a ghost button at the row's 28px height; a done title is muted and struck through.
            let title_id = ui.next_auto_id();
            let x0 = ui.cursor().left();
            let tone = if done { "muted" } else { "strong" };
            let props = ButtonProps {
                label: title.clone(),
                variant: "ghost".into(),
                size: "xl".into(),
                shape: "text".into(),
                width: "content".into(),
                tone: tone.into(),
                tabbable: true,
                ..Default::default()
            };
            for e in Button::new(props).ui(ui, look) {
                let ButtonEvent::Activated { .. } = e;
                selected = true;
            }
            retag(ui, title_id, parts::TITLE);
            drop_pointer_focus(ui, title_id);
            if done {
                let x1 = ui.cursor().left();
                ui.painter().line_segment([pos2(x0 + 8.0, cy), pos2(x1 - 8.0, cy)], Stroke::new(1.0, t.content_3));
            }
        });

        if let Some(next) = toggled {
            self.done = next;
            events.push(SubtaskrowEvent::Toggled { done: next });
        }
        if selected {
            events.push(SubtaskrowEvent::Selected {});
        }
        events
    }
}

crate::mounted!(SubtaskRow, SubtaskrowProps);
