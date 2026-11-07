//! contract: omnicui/contracts/counter.contract.yaml (id: ui.counter). Written the egui way: the count is
//! plain state on the struct, and the frame is drawn from it every time.
use crate::omnicui::counter::{parts, CounterEvent, CounterProps};
use crate::{defect, part, Look};
use egui::{Align, Button, CornerRadius, Label, Layout, RichText, Stroke, Role, WidgetInfo};

pub struct Counter {
    count: i64,
}

impl Counter {
    pub fn new(props: CounterProps) -> Self {
        Self { count: props.start } // contract: props.start, default 0
    }

    /// Draws one frame. Returns `changed` when the button was pressed in it.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Option<CounterEvent> {
        let mut event = None;
        let size = if defect("egui.counter.small-text") { 13.0 } else { 16.0 };
        ui.allocate_ui_with_layout(egui::vec2(200.0, 30.0), Layout::left_to_right(Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 12.0;
            ui.spacing_mut().button_padding = egui::vec2(12.0, 0.0);
            let value = ui.add(Label::new(RichText::new(self.count.to_string()).size(size).color(look.fg)));
            part(ui, &value, parts::VALUE);
            let button = ui.add(
                Button::new(RichText::new("+").size(size).color(look.fg))
                    .fill(egui::Color32::TRANSPARENT)
                    .stroke(Stroke::new(1.0, look.fg))
                    .corner_radius(CornerRadius::same(6))
                    .min_size(egui::vec2(0.0, 30.0)),
            );
            // Its name is "Increment", not its visible "+": what a screen reader announces.
            button.widget_info(|| WidgetInfo::labeled(Role::Button, true, "Increment"));
            part(ui, &button, parts::INCREMENT);
            // egui buttons are clicked by the mouse, and by Enter or Space while focused.
            if button.clicked() {
                self.count += 1;
                event = Some(CounterEvent::Changed { count: self.count }); // contract: events.changed
            }
        });
        event
    }
}
