//! contract: omnicui/contracts/progressbar.contract.yaml (id: ui.progressbar). A thin, read-only bar showing the done ratio of a list or pane, filled with the done lamp on a muted track.
use crate::omnicui::progressbar::{parts, ProgressbarEvent, ProgressbarProps};
use crate::{part, rem, Look};
use egui::{CornerRadius, Rect, Role, Sense, WidgetInfo};

pub struct ProgressBar {
    p: ProgressbarProps,
}

impl ProgressBar {
    pub fn new(p: ProgressbarProps) -> Self {
        Self { p }
    }

    /// One frame: draws the track and, above zero, the fill. The bar is read-only and raises no events.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<ProgressbarEvent> {
        // As the web does: a max that is not above 0 falls back to 1, and the value is clamped to 0..=max.
        // `f64::max` ignores a NaN value, so NaN reads as 0.
        let max = if self.p.max > 0.0 { self.p.max } else { 1.0 };
        let value = self.p.value.max(0.0).min(max);
        let ratio = (value / max) as f32;
        let t = look.t;
        let label = self.p.ariaLabel.clone();

        // The track: 84 px wide (6rem) and about 4 px tall (0.286rem), fully rounded.
        let (rect, response) = ui.allocate_exact_size(egui::vec2(rem(6.0), rem(0.286)), Sense::hover());
        response.widget_info(|| WidgetInfo::slider(true, value, &label));
        ui.ctx().accesskit_node_builder(response.id, |node| {
            node.set_min_numeric_value(0.0);
            node.set_max_numeric_value(max);
        });
        part(ui, &response, parts::BAR);

        let radius = CornerRadius::same((rect.height() / 2.0) as u8);
        ui.painter().rect_filled(rect, radius, t.wash_2);

        // The fill is absent at zero, and otherwise covers the left `ratio` of the track.
        if value > 0.0 {
            let fill = Rect::from_min_size(rect.min, egui::vec2(rect.width() * ratio, rect.height()));
            let fill_response = ui.interact(fill, response.id.with("fill"), Sense::hover());
            fill_response.widget_info(|| WidgetInfo::new(Role::GenericContainer));
            part(ui, &fill_response, parts::FILL);
            ui.painter().rect_filled(fill, radius, t.done_lamp_1);
        }
        Vec::new()
    }
}

crate::mounted!(ProgressBar, ProgressbarProps);
