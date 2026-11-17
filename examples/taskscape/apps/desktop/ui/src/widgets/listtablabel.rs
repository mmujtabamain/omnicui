//! contract: omnicui/contracts/listtablabel.contract.yaml (id: ui.listtablabel). A label-only list tab: the name of an open list, active or inactive, with a close x on the active tab when closable.
use crate::omnicui::listtablabel::{parts, ListtablabelEvent, ListtablabelProps};
use crate::{icon, part, rem, weight, Look};
use egui::{pos2, vec2, Align2, Color32, CornerRadius, CursorIcon, FontFamily, FontId, Rect, Role, Sense, Stroke, StrokeKind, WidgetInfo};

/// Tab height: 1.75rem (24.5px).
const TAB_H: f32 = rem(1.75);
/// Text: 0.929rem (13px), font-medium.
const TEXT_PX: f32 = 13.0;
/// Horizontal padding: px-2 (8px). Border: 1px on each side.
const PAD: f32 = 8.0;
const BORDER: f32 = 1.0;
/// The tab's max width, max-w-[11.429rem] (160px), border and padding included.
const MAX_W: f32 = rem(11.429);
/// The split-view marker: size-[0.5rem] (7px), then a gap of 0.357rem (5px) to the name.
const MARKER: f32 = rem(0.5);
const GAP: f32 = rem(0.357);
/// The web's --radius-field (5px).
const FIELD_RADIUS: u8 = 5;

/// Shortens `name` with an ellipsis until it fits in `max_w` pixels.
fn fit(painter: &egui::Painter, name: &str, font: &FontId, color: Color32, max_w: f32) -> String {
    let width = |s: &str| painter.layout_no_wrap(s.to_owned(), font.clone(), color).size().x;
    if width(name) <= max_w {
        return name.to_owned();
    }
    let chars: Vec<char> = name.chars().collect();
    let (mut lo, mut hi) = (0usize, chars.len());
    while lo < hi {
        let mid = (lo + hi + 1) / 2;
        let cand: String = chars[..mid].iter().collect::<String>() + "\u{2026}";
        if width(&cand) <= max_w {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    chars[..lo].iter().collect::<String>() + "\u{2026}"
}

pub struct ListTabLabel {
    p: ListtablabelProps,
}

impl ListTabLabel {
    pub fn new(p: ListtablabelProps) -> Self {
        Self { p }
    }

    /// One frame: draws the tab (and its close x) and returns the events raised this frame.
    /// `selected` is host-controlled: a press never changes it.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<ListtablabelEvent> {
        let mut events = vec![];
        let t = look.t;
        let selected = self.p.selected;
        // The raised surface only applies to the active tab.
        let raised = selected && self.p.surface == "raised";
        let show_close = selected && self.p.closable;
        let name = self.p.name.clone();
        let show_marker = self.p.paneMarker;
        let font = FontId::new(TEXT_PX, weight(500));
        let ink = if selected { t.content_1 } else { t.content_3 };
        let radius = CornerRadius::same(FIELD_RADIUS);

        // Width: the name, truncated so the whole tab stays within max-w (160px).
        let marker_w = if show_marker { MARKER + GAP } else { 0.0 };
        let max_text = MAX_W - 2.0 * BORDER - 2.0 * PAD - marker_w;
        let label = fit(ui.painter(), &name, &font, ink, max_text);
        let text_w = ui.painter().layout_no_wrap(label.clone(), font.clone(), ink).size().x;
        let tab_w = 2.0 * BORDER + 2.0 * PAD + marker_w + text_w;

        ui.horizontal(|ui| {
            // The close x sits directly after the tab, as in the web's inline-flex root.
            ui.spacing_mut().item_spacing.x = 0.0;

            // The tab element. Its label is the list name alone: the marker and the close x are not in it.
            let (rect, response) = ui.allocate_exact_size(vec2(tab_w, TAB_H), Sense::click());
            let response = response.on_hover_cursor(CursorIcon::PointingHand);
            // A press, Enter or Space selects an inactive tab. The web does not emit it on the active tab.
            if response.clicked() && !selected {
                events.push(ListtablabelEvent::Select {});
            }
            response.widget_info(|| WidgetInfo::selected(Role::Button, true, selected, name.clone()));
            part(ui, &response, parts::TAB);
            // The chosen tab: AccessKit's selected flag is set on the node itself, since a Button's
            // WidgetInfo does not carry it through.
            ui.ctx().accesskit_node_builder(response.id, |node| node.set_selected(selected));

            // The split-view marker: decorative, a node of its own, outside the tab's name.
            let marker_center = if show_marker {
                let c = pos2(rect.left() + BORDER + PAD + MARKER / 2.0, rect.center().y);
                let mi = ui.interact(Rect::from_center_size(c, vec2(MARKER, MARKER)), response.id.with("paneMarker"), Sense::hover());
                mi.widget_info(|| WidgetInfo::new(Role::GenericContainer));
                part(ui, &mi, parts::PANEMARKER);
                Some(c)
            } else {
                None
            };

            // The close x: only on the active tab, when closable. A sibling of the tab, not a child.
            let close = if show_close {
                let (crect, cr) = ui.allocate_exact_size(vec2(TAB_H, TAB_H), Sense::click());
                let cr = cr.on_hover_cursor(CursorIcon::PointingHand);
                if cr.clicked() {
                    events.push(ListtablabelEvent::Close {});
                }
                cr.widget_info(|| WidgetInfo::labeled(Role::Button, true, format!("Close {name}")));
                part(ui, &cr, parts::CLOSE);
                Some((crect, cr))
            } else {
                None
            };

            // Drawing.
            let painter = ui.painter();
            if raised {
                // The web's shadow-lift, approximated as a 1px lower edge.
                painter.rect_filled(rect.translate(vec2(0.0, 1.0)), radius, t.edge_2);
                painter.rect_filled(rect, radius, t.surface_3);
            } else if response.hovered() {
                painter.rect_filled(rect, radius, t.wash_1);
            }
            if selected {
                painter.rect_stroke(rect, radius, Stroke::new(BORDER, t.edge_2), StrokeKind::Inside);
            }
            if let Some(c) = marker_center {
                painter.circle_filled(c, MARKER / 2.0, t.accent_500);
            }
            painter.text(
                pos2(rect.left() + BORDER + PAD + marker_w, rect.center().y),
                Align2::LEFT_CENTER,
                label,
                font.clone(),
                ink,
            );
            if response.has_focus() {
                painter.rect_stroke(rect.expand(2.0), CornerRadius::same(FIELD_RADIUS + 2), Stroke::new(2.0, t.focus_1), StrokeKind::Inside);
            }

            if let Some((crect, cr)) = &close {
                if cr.hovered() {
                    painter.rect_filled(*crect, CornerRadius::same(7), t.wash_2);
                }
                let color = if cr.hovered() { t.content_1 } else { t.content_2 };
                let glyph = icon("close", 14.0);
                painter.text(
                    crect.center(),
                    Align2::CENTER_CENTER,
                    glyph.text(),
                    FontId::new(14.0, FontFamily::Name("Icons".into())),
                    color,
                );
                if cr.has_focus() {
                    painter.rect_stroke(crect.expand(2.0), CornerRadius::same(7 + 2), Stroke::new(2.0, t.focus_1), StrokeKind::Inside);
                }
            }
        });

        events
    }
}

crate::mounted!(ListTabLabel, ListtablabelProps);
