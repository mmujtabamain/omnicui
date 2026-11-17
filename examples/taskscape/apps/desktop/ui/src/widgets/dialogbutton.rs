//! contract: omnicui/contracts/dialogbutton.contract.yaml (id: ui.dialogbutton). The action button in a sheet or confirm dialog footer: an accept, a cancel or an instead button, in the web's lg size with a primary, neutral or danger look; a press, Enter or Space activates it.
use crate::omnicui::dialogbutton::{parts, DialogbuttonEvent, DialogbuttonProps};
use crate::{part, rem, weight, Look};
use egui::{vec2, Align2, Color32, CornerRadius, FontId, Role, Sense, Stroke, StrokeKind, WidgetInfo};

pub struct DialogButton {
    p: DialogbuttonProps,
}

impl DialogButton {
    pub fn new(p: DialogbuttonProps) -> Self {
        Self { p }
    }

    /// One frame: draws the button and returns the events it raised in this frame.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<DialogbuttonEvent> {
        let mut events = vec![];
        let t = look.t;
        let p = &self.p;

        // The web's tone: primary and danger are the variant; "" and anything else take neutral.
        let (fill, hot_fill, fg) = match p.variant.as_str() {
            "primary" => (t.accent_500, t.accent_600, t.on_accent),
            "danger" => (t.danger_500, t.danger_600, t.on_accent),
            _ => (t.surface_0, t.wash_2, t.content_1),
        };

        // The lg size in px: 28px high, 10px side padding, Inter Medium 14px, at least 6rem (84px) wide.
        let h = 28.0;
        let pad = 10.0;
        let font = FontId::new(14.0, weight(500));
        let label = p.label.clone();
        let text_w = ui.painter().layout_no_wrap(label.clone(), font.clone(), Color32::WHITE).size().x;
        let w = (text_w + 2.0 * pad).max(rem(6.0));

        let disabled = p.disabled;
        let enabled = !disabled;
        let sense = if disabled { Sense::hover() } else { Sense::click() };
        let (rect, response) = ui.allocate_exact_size(vec2(w, h), sense);
        let hot = !disabled && response.hovered();

        let clicked = enabled && response.clicked();
        response.widget_info(|| WidgetInfo::labeled(Role::Button, enabled, &label));
        part(ui, &response, parts::BUTTON);

        if clicked {
            match p.role.as_str() {
                "accept" => events.push(DialogbuttonEvent::Activate { role: "accept".to_owned() }),
                "cancel" => events.push(DialogbuttonEvent::Dismiss { role: "cancel".to_owned() }),
                "instead" => events.push(DialogbuttonEvent::Resolve { insteadId: p.insteadId.clone() }),
                _ => {}
            }
        }

        // Paint: fill, focus ring, then the label centred. Disabled is 45% opacity.
        let a = if disabled { 0.45 } else { 1.0 };
        let cr = CornerRadius::same(4);
        let painter = ui.painter();
        let fill_now = if hot { hot_fill } else { fill };
        painter.rect_filled(rect, cr, fill_now.gamma_multiply(a));
        if response.has_focus() && !disabled {
            painter.rect_stroke(rect, cr, Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
        }
        painter.text(rect.center(), Align2::CENTER_CENTER, label, font, fg.gamma_multiply(a));

        events
    }
}

/// The contract's fixture: the harness is a sheet footer, a flex row that sits the button at its right edge.
pub struct DialogButtonMount {
    widget: DialogButton,
}

impl crate::Mounted for DialogButtonMount {
    fn frame(&mut self, ui: &mut egui::Ui, look: Look, emit: &mut dyn FnMut(&'static str, serde_json::Value)) {
        let events = ui
            .with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| self.widget.ui(ui, look))
            .inner;
        for e in events {
            let (name, fields) = e.to_json();
            emit(name, fields);
        }
    }
}

/// Builds the widget from a test's props.
pub fn mount(props: serde_json::Value) -> Result<Box<dyn crate::Mounted>, String> {
    let p: DialogbuttonProps = serde_json::from_value(props).map_err(|e| format!("props: {e}"))?;
    Ok(Box::new(DialogButtonMount { widget: DialogButton::new(p) }))
}
