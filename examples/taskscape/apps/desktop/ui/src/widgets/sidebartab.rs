//! contract: omnicui/contracts/sidebartab.contract.yaml (id: ui.sidebartab). One pane tab in the settings sidebar's vertical tablist: a 16 px icon, the pane label and a trailing match count, filled when it is the selected pane.
use crate::omnicui::sidebartab::{parts, SidebartabEvent, SidebartabProps};
use crate::omnicui::text::TextProps;
use crate::widgets::text::Text;
use crate::{icon, part, rem, weight, Look};
use egui::{pos2, vec2, Align, Align2, Color32, CornerRadius, FontFamily, FontId, Layout, Rect, Role, Sense, Stroke, StrokeKind, UiBuilder, WidgetInfo};

pub struct Sidebartab {
    p: SidebartabProps,
}

impl Sidebartab {
    pub fn new(p: SidebartabProps) -> Self {
        Self { p }
    }

    /// One frame: draws the tab and returns the events it raised in this frame.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<SidebartabEvent> {
        let mut events = vec![];
        // The web's box: h-[2.143rem] w-[9.929rem] max-w-full, px-2 and gap-2 (8 px), a 16 px icon,
        // rounded-(--radius-field) (4 px here: egui's corner radius is whole pixels).
        let width = rem(9.929).min(ui.available_width()).max(0.0);
        let height = rem(2.143);
        let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::click());

        // A press selects an unselected tab once and focuses it (egui turns Enter and Space on a
        // focused widget into a click too). The selected tab does nothing.
        if response.clicked() {
            response.request_focus();
            if !self.p.selected {
                self.p.selected = true;
                events.push(SidebartabEvent::Select { pane: self.p.label.clone() });
            }
        }
        let selected = self.p.selected;
        let label_name = self.p.label.clone();
        response.widget_info(|| WidgetInfo::selected(Role::Button, true, selected, &label_name));
        // The tab is a button with aria-selected: set AccessKit's selected on its node directly, since
        // the WidgetInfo selection is not carried to a button.
        ui.ctx().accesskit_node_builder(response.id, |node| node.set_selected(selected));
        part(ui, &response, parts::TAB);

        let t = look.t;
        // The web's opacity-40 on the whole tab: every colour below is multiplied by this alpha.
        let a = if self.p.dimmed { 0.4 } else { 1.0 };
        let painter = ui.painter().clone();
        let radius = CornerRadius::same(4);
        // Selected: accent at 10 percent when the selection is scoped to this pane, the neutral wash when not.
        let fill = if selected {
            if self.p.scoped {
                t.accent_500.gamma_multiply(0.10)
            } else {
                t.wash_2
            }
        } else if response.hovered() {
            t.wash_1
        } else {
            Color32::TRANSPARENT
        };
        painter.rect_filled(rect, radius, fill.gamma_multiply(a));
        if response.has_focus() {
            // The 2 px focus ring sits outside the tab (the web's ring-2 with no offset).
            painter.rect_stroke(rect, radius, Stroke::new(2.0, t.focus_1.gamma_multiply(a)), StrokeKind::Outside);
        }

        // The icon: 16 px, 8 px in from the left edge, content-3 (the web's text-content-3).
        let icon_rect = Rect::from_center_size(pos2(rect.left() + 16.0, rect.center().y), vec2(16.0, 16.0));
        let icon_response = ui.interact(icon_rect, response.id.with("icon"), Sense::hover());
        icon_response.widget_info(|| WidgetInfo::new(Role::GenericContainer));
        part(ui, &icon_response, parts::ICON);
        let glyph = icon(&self.p.icon, 16.0);
        painter.text(
            icon_rect.center(),
            Align2::CENTER_CENTER,
            glyph.text(),
            FontId::new(16.0, FontFamily::Name("Icons".into())),
            t.content_3.gamma_multiply(a),
        );

        // The badge: the match count, trailing, present only when it is above 0.
        let count = self.p.badge;
        let badge = (count > 0).then(|| {
            painter.layout_no_wrap(count.to_string(), FontId::new(rem(0.786), weight(400)), t.content_2.gamma_multiply(a))
        });

        // The label fills the space between the icon (8 px gap) and the badge (8 px gap) or the right padding.
        let left = rect.left() + 32.0;
        let right = match &badge {
            Some(g) => rect.right() - 8.0 - g.size().x - 8.0,
            None => rect.right() - 8.0,
        };
        let label_rect = Rect::from_min_max(pos2(left, rect.top()), pos2(right.max(left), rect.bottom()));
        let label_response = ui.interact(label_rect, response.id.with("label"), Sense::hover());
        label_response.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, &label_name));
        part(ui, &label_response, parts::LABEL);
        // The label is the shared text widget (13 px, regular, primary), dimmed with the tab.
        let mut text_look = look;
        text_look.t.content_1 = t.content_1.gamma_multiply(a);
        let mut child = ui.new_child(UiBuilder::new().max_rect(label_rect).layout(Layout::left_to_right(Align::Center)));
        let mut text = Text::new(TextProps { content: self.p.label.clone(), ..Default::default() });
        let _ = text.ui(&mut child, text_look);

        if let Some(g) = badge {
            let size = g.size();
            let badge_rect = Rect::from_min_size(pos2(rect.right() - 8.0 - size.x, rect.center().y - size.y / 2.0), size);
            let badge_response = ui.interact(badge_rect, response.id.with("badge"), Sense::hover());
            let count_text = count.to_string();
            badge_response.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, &count_text));
            part(ui, &badge_response, parts::BADGE);
            painter.galley(badge_rect.min, g, t.content_2.gamma_multiply(a));
        }
        events
    }
}

crate::mounted!(Sidebartab, SidebartabProps);
