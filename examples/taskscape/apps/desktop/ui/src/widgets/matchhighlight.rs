//! contract: omnicui/contracts/matchhighlight.contract.yaml (id: ui.matchhighlight). The inline highlight behind the text that matches a search query: a fill behind the text, drawn only when the query is not blank; the text itself is always shown.
use crate::omnicui::matchhighlight::{parts, MatchhighlightEvent, MatchhighlightProps};
use crate::{part, rem, weight, Look};
use egui::{CornerRadius, FontId, Role, Sense, WidgetInfo};

pub struct MatchHighlight {
    p: MatchhighlightProps,
}

impl MatchHighlight {
    pub fn new(p: MatchhighlightProps) -> Self {
        Self { p }
    }

    /// The fill shows only when the query is non-empty after trimming.
    pub fn highlighted(&self) -> bool {
        !self.p.query.trim().is_empty()
    }

    /// One frame: draws the highlight and returns the events it raised (none: the highlight is not interactive).
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<MatchhighlightEvent> {
        let t = look.t;
        // Text inherits its font and colour from the surrounding text; with no surrounding text here,
        // it takes the web's body size (1rem) and regular weight, in the primary text colour.
        let color = t.content_1;
        let font = FontId::new(rem(1.0), weight(400));
        let galley = ui.painter().layout_no_wrap(self.p.text.clone(), font, color);
        // Not focusable and no hover state: the highlight only reports its node.
        let (rect, response) = ui.allocate_exact_size(galley.size(), Sense::hover());
        let label = self.p.text.clone();
        response.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, &label));
        part(ui, &response, parts::HIGHLIGHT);
        if self.highlighted() {
            // Open product decision: the web fills with accent-500 (25% light, 30% dark) until the
            // non-accent search token is chosen, so the fill is accent_500 at that opacity.
            let alpha = if look.dark { 0.30 } else { 0.25 };
            ui.painter().rect_filled(rect, CornerRadius::same(3), t.accent_500.gamma_multiply(alpha));
        }
        ui.painter().galley(rect.min, galley, color);
        Vec::new()
    }
}

crate::mounted!(MatchHighlight, MatchhighlightProps);
