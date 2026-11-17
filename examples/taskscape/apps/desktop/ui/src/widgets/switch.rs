//! contract: omnicui/contracts/switch.contract.yaml (id: ui.switch). An on/off setting that applies at once, named by its row title; forced on (checked and disabled) shows its reason.
use crate::omnicui::switch::{parts, SwitchEvent, SwitchProps};
use crate::{part, rem, weight, Look};
use egui::{Align, CornerRadius, FontId, Key, Layout, Role, Sense, Stroke, StrokeKind, WidgetInfo};

pub struct Switch {
    p: SwitchProps,
}

impl Switch {
    pub fn new(p: SwitchProps) -> Self {
        Self { p }
    }

    /// One frame: draws the switch (and its reason while forced on) and returns the events raised.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<SwitchEvent> {
        let mut events = vec![];
        // The row's switch sits at the right edge, the way the web's items-end column places it.
        ui.with_layout(Layout::top_down(Align::Max), |ui| self.body(ui, look, &mut events));
        events
    }

    fn body(&mut self, ui: &mut egui::Ui, look: Look, events: &mut Vec<SwitchEvent>) {
        let t = look.t;
        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
        let enabled = !self.p.disabled;
        // Disabled is not focusable and takes no press: the web's pointer-events-none.
        let sense = if enabled { Sense::click() } else { Sense::hover() };
        let (rect, mut response) = ui.allocate_exact_size(egui::vec2(rem(2.5), rem(1.357)), sense);

        // Space and Enter both toggle a focused switch. egui may also report a keyboard activation as a
        // click, so the two are OR-ed: one press is one toggle.
        let (space, enter) = ui.input(|i| (i.key_pressed(Key::Space), i.key_pressed(Key::Enter)));
        let key = enabled && response.has_focus() && (space || enter);
        if enabled && (response.clicked() || key) {
            self.p.checked = !self.p.checked;
            response.mark_changed();
            events.push(SwitchEvent::Changed { checked: self.p.checked });
        }

        let (checked, label) = (self.p.checked, self.p.ariaLabel.clone());
        response.widget_info(|| WidgetInfo::selected(Role::Switch, enabled, checked, &label));
        part(ui, &response, parts::SWITCH);
        let focused = response.has_focus();

        // Track: edge_3 off, accent_500 on, eased over the fast duration; hover tints as the web does.
        let hover = enabled && response.hovered();
        let on = ui.ctx().animate_bool_with_time(response.id.with("on"), checked, 0.18);
        let base = t.edge_3.lerp_to_gamma(t.accent_500, on);
        let track = match (hover, checked) {
            (true, true) => t.accent_600,
            (true, false) => t.content_3,
            _ => base,
        };
        let alpha = if enabled { 1.0 } else { 0.45 };
        let radius = CornerRadius::same((rect.height() / 2.0) as u8);
        ui.painter().rect_filled(rect, radius, track.gamma_multiply(alpha));
        if focused {
            ui.painter().rect_stroke(rect.expand(1.0), radius, Stroke::new(2.0, t.focus_1), StrokeKind::Middle);
        }

        // Thumb: 1rem round, inset by the 1px border and 0.1rem padding, travelling 1.157rem when on.
        let d = rem(1.0);
        let inset = rem(0.1) + 1.0;
        let x = rect.left() + inset + rem(1.157) * on;
        let thumb = egui::Rect::from_min_size(egui::pos2(x, rect.center().y - d / 2.0), egui::vec2(d, d));
        ui.painter().circle_filled(thumb.center(), d / 2.0, t.on_accent.gamma_multiply(alpha));
        let thumb_resp = ui.interact(thumb, response.id.with("thumb"), Sense::hover());
        thumb_resp.widget_info(|| WidgetInfo::new(Role::GenericContainer));
        part(ui, &thumb_resp, parts::THUMB);

        // Reason: shown only while forced on (checked and disabled), 0.35rem below the track, right-aligned.
        let forced = checked && !enabled;
        if forced && !self.p.reason.is_empty() {
            ui.add_space(rem(0.35));
            let reason = self.p.reason.clone();
            let galley = ui.painter().layout(reason.clone(), FontId::new(11.0, weight(400)), t.content_2, rem(18.0));
            let (r, reason_resp) = ui.allocate_exact_size(galley.size(), Sense::hover());
            reason_resp.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, &reason));
            part(ui, &reason_resp, parts::REASON);
            ui.painter().galley(r.min, galley, t.content_2);
        }
    }
}

crate::mounted!(Switch, SwitchProps);
