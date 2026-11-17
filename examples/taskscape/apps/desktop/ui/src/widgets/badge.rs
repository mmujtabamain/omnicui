//! contract: omnicui/contracts/badge.contract.yaml (id: ui.badge). A read-only inline mark: a pill or circle with a count or label, or a 6px dot or 2px bar with no text.
use crate::omnicui::badge::{parts, BadgeEvent, BadgeProps};
use crate::{icon, part, rem, weight, Look};
use egui::{Align2, CornerRadius, FontFamily, FontId, Role, Sense, Stroke, StrokeKind, Vec2, WidgetInfo};

pub struct Badge {
    p: BadgeProps,
}

impl Badge {
    pub fn new(p: BadgeProps) -> Self {
        Self { p }
    }

    /// One frame. A badge is read-only: it never raises an event and has no hover, focus or press state.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<BadgeEvent> {
        // visible: false takes the mark out of the layout, as the web's `hidden` class does.
        if !self.p.visible {
            return vec![];
        }
        let t = look.t;
        let lg = self.p.size == "lg";
        // A round badge is valid only at size lg; round with md renders as a pill.
        let shape = match self.p.shape.as_str() {
            "round" if !lg => "pill",
            "round" => "round",
            "dot" => "dot",
            "bar" => "bar",
            _ => "pill",
        };
        let decorative = shape == "dot" || shape == "bar";
        let (fill, text_color, bordered) = match self.p.tone.as_str() {
            "recessed" => (Some(t.wash_2), t.content_2, false),
            "accent" => (Some(t.accent_500), t.on_accent, false),
            "neutral" => (Some(t.edge_3), t.content_3, false),
            _ => (None, t.content_2, true),
        };
        let border = if bordered { 1.0 } else { 0.0 };
        let has_icon = self.p.icon && !decorative;
        let glyph_px = if lg { 13.0 } else { 11.0 };
        // The icon sits before the text with a 4px gap (gap-[0.286rem]).
        let icon_slot = if has_icon { glyph_px + rem(0.286) } else { 0.0 };
        let font = FontId::new(if lg { rem(0.929) } else { rem(0.786) }, weight(600));
        let text = self.p.children.clone();

        let text_w = if decorative {
            0.0
        } else {
            ui.painter().layout_no_wrap(text.clone(), font.clone(), text_color).size().x
        };

        // Measured box, as the web's Tailwind classes give it (border-box).
        let (w, h) = match shape {
            "dot" => (rem(0.429), rem(0.429)),
            "bar" => (rem(0.143), rem(1.143)),
            _ => {
                let h = if lg { rem(2.0) } else { rem(1.286) };
                let inner = icon_slot + text_w;
                if shape == "round" {
                    // Round is a circle at least 2rem wide, with no horizontal padding.
                    ((inner + 2.0 * border).max(h), h)
                } else {
                    let pad = if lg { 7.0 } else { 3.5 };
                    (inner + 2.0 * (pad + border), h)
                }
            }
        };

        let (rect, response) = ui.allocate_exact_size(Vec2::new(w, h), Sense::hover());
        // title is a hover tooltip only; keyboard and screen reader users get srLabel.
        let response = if self.p.title.is_empty() { response } else { response.on_hover_text(self.p.title.clone()) };

        // The visible count is the value; a screen reader speaks srLabel when it is set.
        let label = if self.p.srLabel.is_empty() { text.clone() } else { self.p.srLabel.clone() };
        let value = text.clone();
        response.widget_info(|| {
            let mut info = WidgetInfo::labeled(Role::GenericContainer, true, label.clone());
            info.current_text_value = Some(value.clone());
            info
        });
        part(ui, &response, parts::BADGE);

        let cy = rect.center().y;
        let group_left = rect.center().x - (icon_slot + text_w) / 2.0;
        if has_icon {
            let icon_rect = egui::Rect::from_center_size(
                egui::pos2(group_left + glyph_px / 2.0, cy),
                Vec2::splat(glyph_px),
            );
            let inner = ui.interact(icon_rect, response.id.with("icon"), Sense::hover());
            inner.widget_info(|| WidgetInfo::new(Role::GenericContainer));
            part(ui, &inner, parts::ICON);
        }

        let painter = ui.painter();
        let radius = CornerRadius::same((rect.width().min(rect.height()) / 2.0).round() as u8);
        if let Some(fill) = fill {
            painter.rect_filled(rect, radius, fill);
        }
        if bordered {
            painter.rect_stroke(rect, radius, Stroke::new(1.0, t.edge_2), StrokeKind::Inside);
        }
        if has_icon {
            let glyph = icon("attach_file", glyph_px);
            painter.text(
                egui::pos2(group_left + glyph_px / 2.0, cy),
                Align2::CENTER_CENTER,
                glyph.text(),
                FontId::new(glyph_px, FontFamily::Name("Icons".into())),
                text_color,
            );
        }
        if !decorative && !text.is_empty() {
            painter.text(
                egui::pos2(group_left + icon_slot + text_w / 2.0, cy),
                Align2::CENTER_CENTER,
                text,
                font,
                text_color,
            );
        }

        vec![]
    }
}

crate::mounted!(Badge, BadgeProps);
