//! contract: omnicui/contracts/statusindicator.contract.yaml (id: ui.statusindicator). A non-interactive 32px status cell with a 20px glyph, shown while a status is active; its explanation opens as a tooltip on hover, focus or tap, and Escape closes it.
use crate::omnicui::statusindicator::{parts, StatusindicatorEvent, StatusindicatorProps};
use crate::{icon, part, rem, text, Look};
use egui::{Align2, Color32, CornerRadius, FontFamily, FontId, Key, Margin, Order, Rect, Role, Sense, Shadow, Stroke, StrokeKind, WidgetInfo};

/// The gap between the cell and the tooltip (the web's sideOffset 6).
const GAP: f32 = 6.0;
/// The tooltip keeps this far from the screen's edges (the web's collisionPadding 6).
const COLLISION: f32 = 6.0;

pub struct StatusIndicator {
    p: StatusindicatorProps,
    /// The tooltip is drawn.
    open: bool,
    /// Escape closed the tooltip: it stays closed until the pointer enters again or the cell gains focus.
    dismissed: bool,
    /// The pointer was on the cell or on the tooltip last frame.
    was_inside: bool,
    /// Where the tooltip was last frame, so the pointer can cross the gap onto it.
    tip_rect: Option<Rect>,
    /// The tooltip's size as last measured, to place it above or below the cell before it is drawn.
    tip_size: egui::Vec2,
}

impl StatusIndicator {
    pub fn new(p: StatusindicatorProps) -> Self {
        Self { p, open: false, dismissed: false, was_inside: false, tip_rect: None, tip_size: egui::Vec2::ZERO }
    }

    /// One frame: draws the cell and its tooltip (nothing while inactive). Raises no events.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<StatusindicatorEvent> {
        if !self.p.active {
            // Nothing is drawn, and a tooltip open on the hidden cell closes with it.
            self.open = false;
            self.dismissed = false;
            self.was_inside = false;
            self.tip_rect = None;
            return Vec::new();
        }

        let t = look.t;
        let label = self.p.label.clone();
        let (rect, response) = ui.allocate_exact_size(egui::vec2(rem(2.2857), rem(2.2857)), Sense::click());
        response.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, &label));
        part(ui, &response, parts::CELL);

        // The glyph: Icon xl, 20px, centred in the 32px cell.
        let glyph_rect = Rect::from_center_size(rect.center(), egui::vec2(20.0, 20.0));
        let glyph = ui.interact(glyph_rect, response.id.with("glyph"), Sense::hover());
        glyph.widget_info(|| WidgetInfo::new(Role::GenericContainer));
        part(ui, &glyph, parts::GLYPH);
        let name = icon(&self.p.icon, 20.0);
        ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            name.text(),
            FontId::new(20.0, FontFamily::Name("Icons".into())),
            t.content_2,
        );

        // Keyboard focus draws the 2px focus ring just outside the cell.
        if response.has_focus() {
            ui.painter().rect_stroke(rect, CornerRadius::same(7), Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
        }

        // Pointer and focus decide when the tooltip opens and closes. The pointer is judged by its
        // position against the cell and the tooltip, so a tooltip laid over the cell cannot flicker it.
        let pointer = ui.input(|i| i.pointer.hover_pos());
        let over_cell = pointer.is_some_and(|p| rect.contains(p));
        let over_tip = pointer.is_some_and(|p| self.tip_rect.is_some_and(|r| r.contains(p)));
        let inside = over_cell || over_tip;
        let focused = response.has_focus();

        if inside && !self.was_inside {
            // The pointer entered: a tooltip Escape dismissed may open again.
            self.dismissed = false;
        }
        if !inside && !focused {
            // The pointer left both the cell and the tooltip; focus keeps the tooltip open.
            self.open = false;
        }
        if response.gained_focus() {
            self.open = true;
            self.dismissed = false;
        }
        // A tap opens the tooltip too. Enter and Space also click a focused cell; they must not act here.
        let key_click = ui.input(|i| i.key_pressed(Key::Enter) || i.key_pressed(Key::Space));
        if response.clicked() && !key_click {
            self.open = true;
            self.dismissed = false;
        }
        // Escape closes the tooltip whether or not it is drawn yet: a hover in the same frame may open it below.
        if ui.input(|i| i.key_pressed(Key::Escape)) {
            self.open = false;
            self.dismissed = true;
        }
        if over_cell && !self.dismissed {
            self.open = true;
        }
        self.was_inside = inside;

        // The tooltip: above the cell, its bottom centre GAP over the cell's top centre. Below the cell
        // when there is no room above (Radix flips the side). It stays COLLISION inside the screen's sides.
        if self.open {
            let tip = self.p.tooltip.clone();
            let screen = ui.ctx().content_rect();
            let size = self.tip_size;
            let room_above = rect.top() - GAP - size.y - COLLISION >= screen.top();
            let (pivot, y) = if room_above {
                (Align2::CENTER_BOTTOM, rect.top() - GAP)
            } else {
                (Align2::CENTER_TOP, rect.bottom() + GAP)
            };
            let half = size.x / 2.0;
            let (lo, hi) = (screen.left() + COLLISION + half, screen.right() - COLLISION - half);
            let x = rect.center().x.max(lo).min(hi.max(lo));
            let body_id = response.id.with("body");
            let shown = egui::Area::new(response.id.with("tooltip"))
                .order(Order::Tooltip)
                .pivot(pivot)
                .fixed_pos(egui::pos2(x, y))
                .show(ui.ctx(), |ui| {
                    let frame = egui::Frame::new()
                        .fill(t.content_1)
                        .corner_radius(CornerRadius::same(5))
                        .inner_margin(Margin::symmetric(8, 4))
                        .shadow(Shadow { offset: [0, 2], blur: 12, spread: 0, color: Color32::from_black_alpha(64) })
                        .show(ui, |ui| {
                            ui.set_max_width(rem(18.0));
                            ui.label(text(tip.as_str(), 12.0, 400).color(t.surface_2));
                        });
                    let body = ui.interact(frame.response.rect, body_id, Sense::hover());
                    body.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, &tip));
                    part(ui, &body, parts::TOOLTIP);
                    body.rect
                });
            let measured = shown.inner.size();
            if (measured - self.tip_size).length() > 0.5 {
                // Placed from a stale size: draw again so it sits on the right side of the cell.
                self.tip_size = measured;
                ui.ctx().request_repaint();
            }
            self.tip_rect = Some(shown.inner);
        } else {
            self.tip_rect = None;
        }
        Vec::new()
    }
}

crate::mounted!(StatusIndicator, StatusindicatorProps);
