//! contract: omnicui/contracts/emptystate.contract.yaml (id: ui.emptystate). A centred message shown in place of content: a title, an optional muted subtitle, an optional icon, a loading status and an optional action button.
use crate::omnicui::emptystate::{parts, EmptystateEvent, EmptystateProps};
use crate::{icon, part, weight, Look};
use egui::text::LayoutJob;
use egui::{Align, Align2, Color32, CornerRadius, FontFamily, FontId, Galley, Pos2, Rect, Role, Sense, Stroke, StrokeKind, Vec2, WidgetInfo};
use std::sync::Arc;

/// The web's Tailwind spacing unit in px: px-2 is 8 px.
const UNIT: f32 = 4.0;
/// The outline button's height (h-8) and its label size: 13 px, medium.
const BUTTON_H: f32 = 8.0 * UNIT;
const BUTTON_PX: f32 = 13.0;
/// Title: 1.071rem (15 px), medium. Subtitle: 0.893rem (12.5 px).
const TITLE_PX: f32 = 15.0;
const SUBTITLE_PX: f32 = 12.5;

pub struct Emptystate {
    p: EmptystateProps,
}

/// Lays text out in Inter, centred on its lines and wrapped at `wrap` px.
fn layout(ui: &egui::Ui, s: &str, px: f32, w: u16, color: Color32, wrap: f32) -> Arc<Galley> {
    let mut job = LayoutJob::simple(s.to_owned(), FontId::new(px, weight(w)), color, wrap);
    job.halign = Align::Center;
    ui.painter().layout_job(job)
}

/// Where a laid-out block sits when centred on `cx` with its top at `top`: the position to draw
/// it at, and its visible rect (the rect the part's node covers).
fn centred(g: &Galley, cx: f32, top: f32) -> (Pos2, Rect) {
    let size = g.rect.size();
    let rect = Rect::from_min_size(Pos2::new(cx - size.x / 2.0, top), size);
    (rect.min - g.rect.min.to_vec2(), rect)
}

impl Emptystate {
    pub fn new(p: EmptystateProps) -> Self {
        Self { p }
    }

    /// One frame: draws the message. It raises no events; the action button is the caller's own.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<EmptystateEvent> {
        let t = look.t;
        let pane = self.p.scope == "pane";
        let card = self.p.surface == "card";
        let loading = self.p.loading;
        let show_icon = !self.p.icon.is_empty() && !pane && !loading;
        let show_subtitle = !self.p.subtitle.is_empty() && !loading;
        let show_action = !self.p.actionLabel.is_empty();
        let title_text = self.p.title.clone();
        let subtitle_text = self.p.subtitle.clone();
        let action_text = self.p.actionLabel.clone();
        let icon_name = self.p.icon.clone();
        let icon_px = self.p.iconSize as f32;

        // The web's padding by scope and surface: list pt-24 pb-10 px-4; card px-10 py-12; pane px-4,
        // filling the task pane and centring its content vertically.
        let (pad_x, pad_top, pad_bottom) = if card {
            (10.0 * UNIT, 12.0 * UNIT, 12.0 * UNIT)
        } else if pane {
            (4.0 * UNIT, 0.0, 0.0)
        } else {
            (4.0 * UNIT, 24.0 * UNIT, 10.0 * UNIT)
        };
        let avail_w = ui.available_width();
        let inner_w = (avail_w - 2.0 * pad_x).max(0.0);

        let title = layout(ui, &title_text, TITLE_PX, 500, t.content_2, inner_w);
        let subtitle = if show_subtitle {
            Some(layout(ui, &subtitle_text, SUBTITLE_PX, 400, t.content_3, inner_w))
        } else {
            None
        };
        let action = if show_action {
            Some(layout(ui, &action_text, BUTTON_PX, 500, t.content_1, 1000.0))
        } else {
            None
        };

        // Stacked: icon (mb-2), title, subtitle (mt-1), action (mt-4).
        let icon_h = if show_icon { icon_px + 2.0 * UNIT } else { 0.0 };
        let sub_h = subtitle.as_ref().map_or(0.0, |g| UNIT + g.rect.height());
        let action_h = action.as_ref().map_or(0.0, |_| 4.0 * UNIT + BUTTON_H);
        let content_h = icon_h + title.rect.height() + sub_h + action_h;
        let avail_h = ui.available_height();
        let height = if pane && avail_h.is_finite() {
            avail_h.max(content_h)
        } else {
            pad_top + content_h + pad_bottom
        };

        let (root_rect, root) = ui.allocate_exact_size(Vec2::new(avail_w, height), Sense::hover());
        root.widget_info(|| WidgetInfo::new(Role::GenericContainer));
        part(ui, &root, parts::ROOT);
        if pane {
            ui.painter().rect_filled(root_rect, CornerRadius::same(0), t.surface_2);
        } else if card {
            ui.painter().rect_filled(root_rect, CornerRadius::same(10), t.surface_2);
        }

        let cx = root_rect.center().x;
        let mut y = if pane {
            root_rect.top() + (height - content_h) / 2.0
        } else {
            root_rect.top() + pad_top
        };

        if show_icon {
            let rect = Rect::from_min_size(Pos2::new(cx - icon_px / 2.0, y), Vec2::splat(icon_px));
            let resp = ui.interact(rect, root.id.with("icon"), Sense::hover());
            resp.widget_info(|| WidgetInfo::new(Role::GenericContainer));
            part(ui, &resp, parts::ICON);
            let glyph = icon(&icon_name, icon_px);
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                glyph.text(),
                FontId::new(icon_px, FontFamily::Name("Icons".into())),
                t.content_3,
            );
            y += icon_px + 2.0 * UNIT;
        }

        let (pos, rect) = centred(&title, cx, y);
        let resp = ui.interact(rect, root.id.with("title"), Sense::hover());
        resp.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, title_text.as_str()));
        part(ui, &resp, parts::TITLE);
        ui.painter().galley(pos, title.clone(), t.content_2);
        if loading {
            let status = ui.interact(rect, root.id.with("loadingStatus"), Sense::hover());
            status.widget_info(|| WidgetInfo::labeled(Role::Status, true, title_text.as_str()));
            part(ui, &status, parts::LOADINGSTATUS);
        }
        y += title.rect.height();

        if let Some(sub) = &subtitle {
            y += UNIT;
            let (pos, rect) = centred(sub, cx, y);
            let resp = ui.interact(rect, root.id.with("subtitle"), Sense::hover());
            resp.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, subtitle_text.as_str()));
            part(ui, &resp, parts::SUBTITLE);
            ui.painter().galley(pos, sub.clone(), t.content_3);
            y += sub.rect.height();
        }

        if let Some(label) = &action {
            y += 4.0 * UNIT;
            let w = label.rect.width() + 2.0 * 4.0 * UNIT;
            let rect = Rect::from_min_size(Pos2::new(cx - w / 2.0, y), Vec2::new(w, BUTTON_H));
            let resp = ui.interact(rect, root.id.with("action"), Sense::click());
            resp.widget_info(|| WidgetInfo::labeled(Role::Button, true, action_text.as_str()));
            part(ui, &resp, parts::ACTION);
            let painter = ui.painter();
            let radius = CornerRadius::same(7);
            painter.rect_filled(rect, radius, t.surface_3);
            if resp.hovered() {
                painter.rect_filled(rect, radius, t.wash_2);
            }
            painter.rect_stroke(rect, radius, Stroke::new(1.0, t.edge_2), StrokeKind::Inside);
            if resp.has_focus() {
                painter.rect_stroke(rect.expand(2.0), radius, Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
            }
            let (pos, _) = centred(label, rect.center().x, rect.center().y - label.rect.height() / 2.0);
            painter.galley(pos, label.clone(), t.content_1);
        }

        Vec::new()
    }
}

crate::mounted!(Emptystate, EmptystateProps);
