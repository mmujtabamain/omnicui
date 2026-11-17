//! contract: omnicui/contracts/text.contract.yaml (id: ui.text). The one general text label: a run of Inter at the web's md size (13px, regular) in the primary content colour, wrapping to its width.
use crate::omnicui::text::{parts, TextEvent, TextProps};
use crate::{part, rem, weight, Look};
use egui::{FontId, Sense, Role, WidgetInfo};

pub struct Text {
    p: TextProps,
}

impl Text {
    pub fn new(p: TextProps) -> Self {
        Self { p }
    }

    /// One frame: draws the text and returns the events it raised in this frame (none: text has no events).
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<TextEvent> {
        let events: Vec<TextEvent> = vec![];
        // The web's defaults: size md (0.929rem), weight regular, tone primary, overflow wrap, align start.
        let font = FontId::new(rem(0.929), weight(400));
        let color = look.t.content_1;
        let content = self.p.content.clone();
        let wrap_width = ui.available_width().max(0.0);
        let galley = ui.painter().layout(content.clone(), font, color, wrap_width);
        let (rect, response) = ui.allocate_exact_size(galley.size(), Sense::hover());
        ui.painter().galley(rect.min, galley, color);
        response.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, &content));
        part(ui, &response, parts::TEXT);
        events
    }
}

crate::mounted!(Text, TextProps);
