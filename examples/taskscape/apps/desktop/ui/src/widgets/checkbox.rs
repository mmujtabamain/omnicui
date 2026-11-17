//! contract: omnicui/contracts/checkbox.contract.yaml (id: ui.checkbox). A binary mark on a task: intent
//! "done" fills with the done lamp and draws a check, "select" fills with the accent and draws a dot.
use crate::omnicui::checkbox::{parts, CheckboxEvent, CheckboxProps};
use crate::{icon, part, Look};
use egui::{Align2, CornerRadius, Role, Sense, Stroke, StrokeKind, WidgetInfo};

pub struct Checkbox {
    p: CheckboxProps,
}

impl Checkbox {
    pub fn new(p: CheckboxProps) -> Self {
        Self { p }
    }

    pub fn checked(&self) -> bool {
        self.p.checked
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<CheckboxEvent> {
        let mut events = vec![];
        let md = self.p.size == "md";
        let side = if md { 18.0 } else { 16.0 };
        let sense = if self.p.disabled { Sense::hover() } else { Sense::click() };
        let (rect, mut response) = ui.allocate_exact_size(egui::vec2(side, side), sense);
        // A press or Space toggles it (egui clicks a focused widget on Space and Enter; Enter is kept
        // for the row, so only Space counts here).
        let space = response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Space));
        let enter = response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        if !self.p.disabled && ((response.clicked() && !enter) || space) {
            self.p.checked = !self.p.checked;
            response.mark_changed();
            events.push(CheckboxEvent::CheckedChange { checked: self.p.checked });
        }
        let (checked, label) = (self.p.checked, self.p.ariaLabel.clone());
        response.widget_info(|| WidgetInfo::selected(Role::CheckBox, !self.p.disabled, checked, &label));
        part(ui, &response, parts::CONTROL);

        let t = look.t;
        let radius = if self.p.shape == "round" { CornerRadius::same((side / 2.0) as u8) } else { CornerRadius::same(4) };
        let fill = match (checked, self.p.intent.as_str()) {
            (true, "select") => t.accent_500,
            (true, _) => t.done_lamp_1,
            (false, _) => t.surface_3,
        };
        let edge = if checked { fill } else { t.content_1 };
        let alpha = if self.p.disabled { 0.45 } else { 1.0 };
        let painter = ui.painter();
        painter.rect(rect, radius, fill.gamma_multiply(alpha), Stroke::new(1.0, edge.gamma_multiply(alpha)), StrokeKind::Inside);
        if response.has_focus() {
            painter.rect_stroke(rect.expand(2.0), radius, Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
        }
        if checked {
            let inner = ui.interact(rect.shrink(3.0), response.id.with("indicator"), Sense::hover());
            inner.widget_info(|| WidgetInfo::new(Role::GenericContainer));
            part(ui, &inner, parts::INDICATOR);
            if self.p.intent == "select" {
                ui.painter().circle_filled(rect.center(), 3.0, t.on_accent);
            } else {
                let glyph = icon("check", if md { 14.0 } else { 12.0 });
                ui.painter().text(rect.center(), Align2::CENTER_CENTER, glyph.text(), egui::FontId::new(if md { 14.0 } else { 12.0 }, egui::FontFamily::Name("Icons".into())), t.on_accent);
            }
        }
        events
    }
}

crate::mounted!(Checkbox, CheckboxProps);
