//! contract: omnicui/contracts/card.contract.yaml (id: ui.card). A rounded container for grouped content: a group heading or search breadcrumb, a hint, a pending-change dot with a discard action, and the note editor whose toolbar shows while focus is within.
use crate::omnicui::card::{parts, CardEvent, CardProps};
use crate::{icon, part, rem, text, weight, Look};
use egui::{vec2, Align, Align2, Color32, CornerRadius, FontFamily, FontId, Frame, Layout, Rect, Role, Sense, Stroke, StrokeKind, WidgetInfo};

pub struct Card {
    p: CardProps,
    /// The note editor's text.
    note: String,
    /// A toolbar button had focus last frame, so the toolbar stays shown.
    toolbar_focus: bool,
}

impl Card {
    pub fn new(p: CardProps) -> Self {
        Self { p, note: String::new(), toolbar_focus: false }
    }

    /// The dot and the discard action show only when a change is pending and a discard handler is given.
    pub fn pending(&self) -> bool {
        self.p.changed && self.p.discardable
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<CardEvent> {
        let mut events = vec![];
        let t = look.t;
        let pending = self.pending();
        let heading = if self.p.breadcrumb.is_empty() { self.p.label.clone() } else { self.p.breadcrumb.clone() };
        let hint = self.p.hint.clone();
        let label = self.p.label.clone();
        let search = self.p.context == "search";
        let note_on = self.p.note;
        let show_header = !heading.is_empty() || !hint.is_empty() || pending;

        let fill = match self.p.tone.as_str() {
            "base" => Color32::TRANSPARENT,
            "surface-3" => t.surface_3,
            _ => t.surface_2,
        };
        let radius: f32 = if self.p.radius == "control" { 7.0 } else { 10.5 };
        let solid = self.p.border == "solid";
        let dashed = self.p.border == "dashed";

        // The discard action is measured first, so the heading column leaves room for it on the right.
        let discard_galley = ui.painter().layout_no_wrap(
            "Discard".to_owned(),
            FontId::new(rem(0.857), weight(500)),
            t.content_2,
        );
        let discard_w = discard_galley.size().x + 16.0;
        let right_w = if pending { discard_w + 6.0 + 6.0 + 12.0 } else { 12.0 };

        let note_buf = &mut self.note;
        let toolbar_was = self.toolbar_focus;
        let mut toolbar_now = false;

        let frame = Frame::new()
            .fill(fill)
            .stroke(if solid { Stroke::new(1.0, t.edge_2) } else { Stroke::NONE })
            .corner_radius(CornerRadius::same(radius.round() as u8))
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                ui.set_min_width(ui.available_width());
                let total = ui.available_width();

                if show_header {
                    ui.add_space(10.0);
                    ui.horizontal_top(|ui| {
                        ui.add_space(12.0);
                        let left_w = (total - 12.0 - 8.0 - right_w).max(0.0);
                        ui.allocate_ui(vec2(left_w, 0.0), |ui| {
                            ui.set_width(left_w);
                            if !heading.is_empty() {
                                let w = if search { 500 } else { 600 };
                                let r = ui.label(text(heading.as_str(), rem(0.857), w).color(t.content_2));
                                r.widget_info(|| WidgetInfo::labeled(Role::Heading, true, &heading));
                                part(ui, &r, parts::HEADING);
                            }
                            if !hint.is_empty() {
                                let r = ui.label(text(hint.as_str(), rem(0.786), 400).color(t.content_3));
                                r.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, &hint));
                                part(ui, &r, parts::HINT);
                            }
                        });
                        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                            ui.add_space(12.0);
                            if pending {
                                let (rect, resp) = ui.allocate_exact_size(vec2(discard_w, 24.0), Sense::click());
                                if resp.hovered() {
                                    let wash = if resp.is_pointer_button_down_on() { t.wash_2 } else { t.wash_1 };
                                    ui.painter().rect_filled(rect, CornerRadius::same(7), wash);
                                }
                                ui.painter().galley(rect.center() - discard_galley.size() / 2.0, discard_galley.clone(), t.content_2);
                                if resp.has_focus() {
                                    ui.painter().rect_stroke(rect.expand(2.0), CornerRadius::same(9), Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
                                }
                                resp.widget_info(|| WidgetInfo::labeled(Role::Button, true, "Discard"));
                                part(ui, &resp, parts::DISCARD);
                                if resp.clicked() {
                                    events.push(CardEvent::Discarded { label: label.clone() });
                                }
                                ui.add_space(6.0);
                                let (dot, dot_resp) = ui.allocate_exact_size(vec2(6.0, 24.0), Sense::hover());
                                ui.painter().circle_filled(dot.center(), 3.0, t.content_3);
                                dot_resp.widget_info(|| WidgetInfo::new(Role::GenericContainer));
                                part(ui, &dot_resp, parts::CHANGEDOT);
                            }
                        });
                    });
                    ui.add_space(6.0);
                }

                if note_on {
                    ui.add_space(10.0);
                    let editor_focus = ui
                        .horizontal_top(|ui| {
                            ui.add_space(12.0);
                            let w = (total - 24.0).max(0.0);
                            let r = ui.add(
                                egui::TextEdit::multiline(note_buf)
                                    .frame(egui::Frame::NONE)
                                    .font(FontId::new(rem(0.929), weight(400)))
                                    .text_color(t.content_1)
                                    .desired_width(w)
                                    .min_size(vec2(w, rem(4.5))),
                            );
                            part(ui, &r, parts::EDITOR);
                            r.has_focus()
                        })
                        .inner;
                    ui.add_space(10.0);

                    // The toolbar shows while focus is within the card: in the editor, or on a toolbar button.
                    if editor_focus || toolbar_was {
                        let (line, _) = ui.allocate_exact_size(vec2(total, 1.0), Sense::hover());
                        ui.painter().rect_filled(line, 0.0, t.edge_1);
                        ui.add_space(4.0);
                        let bar = ui.horizontal_top(|ui| {
                            ui.add_space(8.0);
                            let mut any_focus = false;
                            let buttons = [
                                ("format_bold", "Bold"),
                                ("format_italic", "Italic"),
                                ("format_list_bulleted", "Bulleted list"),
                                ("link", "Link"),
                            ];
                            for (glyph, name) in buttons {
                                let (rect, resp) = ui.allocate_exact_size(vec2(28.0, 28.0), Sense::click());
                                if resp.hovered() {
                                    ui.painter().rect_filled(rect, CornerRadius::same(7), t.wash_1);
                                }
                                ui.painter().text(
                                    rect.center(),
                                    Align2::CENTER_CENTER,
                                    icon(glyph, 16.0).text(),
                                    FontId::new(16.0, FontFamily::Name("Icons".into())),
                                    t.content_2,
                                );
                                if resp.has_focus() {
                                    any_focus = true;
                                    ui.painter().rect_stroke(rect.expand(2.0), CornerRadius::same(9), Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
                                }
                                resp.widget_info(|| WidgetInfo::labeled(Role::Button, true, name));
                                ui.add_space(2.0);
                            }
                            any_focus
                        });
                        bar.response.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, "Formatting"));
                        part(ui, &bar.response, parts::EDITORTOOLBAR);
                        toolbar_now = bar.inner;
                        ui.add_space(4.0);
                    }
                }
            });

        self.toolbar_focus = toolbar_now;
        let rect = frame.response.rect;
        if dashed {
            dashed_rect(ui.painter(), rect, t.edge_2);
        }

        // The container itself: a generic node, named by its heading when there is one.
        let container = ui.interact(rect, frame.response.id.with("container"), Sense::hover());
        if heading.is_empty() {
            container.widget_info(|| WidgetInfo::new(Role::GenericContainer));
        } else {
            container.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, &heading));
        }
        part(ui, &container, parts::CONTAINER);

        events
    }
}

/// A 1px dashed edge around a rect (egui strokes cannot dash): 4px dashes with 3px gaps.
fn dashed_rect(painter: &egui::Painter, rect: Rect, color: Color32) {
    let stroke = Stroke::new(1.0, color);
    let r = rect.shrink(0.5);
    let corners = [r.left_top(), r.right_top(), r.right_bottom(), r.left_bottom()];
    for i in 0..4 {
        let (a, b) = (corners[i], corners[(i + 1) % 4]);
        let len = (b - a).length();
        if len <= 0.0 {
            continue;
        }
        let dir = (b - a) / len;
        let mut d = 0.0;
        while d < len {
            let e = (d + 4.0).min(len);
            painter.line_segment([a + dir * d, a + dir * e], stroke);
            d += 7.0;
        }
    }
}

crate::mounted!(Card, CardProps);
