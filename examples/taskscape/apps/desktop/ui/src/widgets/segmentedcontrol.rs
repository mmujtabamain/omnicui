//! contract: omnicui/contracts/segmentedcontrol.contract.yaml (id: ui.segmentedcontrol). Mutually exclusive choice shown as joined segments, icon-only or icon and text.
use crate::omnicui::segmentedcontrol::{parts, SegmentedcontrolEvent, SegmentedcontrolProps};
use crate::{icon, part, rem, weight, Look};
use egui::{Align2, CornerRadius, FontFamily, FontId, Key, Rect, Response, Role, Sense, Stroke, StrokeKind, WidgetInfo};

const PARTS: [&str; 3] = [parts::FIRSTSEGMENT, parts::SECONDSEGMENT, parts::THIRDSEGMENT];

pub struct Segmentedcontrol {
    p: SegmentedcontrolProps,
    /// The segment that held the keyboard focus at the end of the last frame, if any.
    focus_prev: Option<usize>,
}

/// The "value=Label" pairs in order, at most the three segments the parts name.
fn parse_segments(s: &str) -> Vec<(String, String)> {
    s.split(',')
        .filter_map(|pair| pair.split_once('='))
        .map(|(value, label)| (value.trim().to_owned(), label.trim().to_owned()))
        .take(PARTS.len())
        .collect()
}

impl Segmentedcontrol {
    pub fn new(p: SegmentedcontrolProps) -> Self {
        Self { p, focus_prev: None }
    }

    /// Makes `index` the selection; reports a change only when it differs from the current value.
    fn select(&mut self, index: usize, segs: &[(String, String)], events: &mut Vec<SegmentedcontrolEvent>) {
        let Some((value, _)) = segs.get(index) else { return };
        if *value != self.p.value {
            self.p.value = value.clone();
            events.push(SegmentedcontrolEvent::ValueChanged { value: value.clone() });
        }
    }

    /// One frame: draws the group and returns the events it raised in this frame.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<SegmentedcontrolEvent> {
        let mut events = vec![];
        let t = look.t;
        let segs = parse_segments(&self.p.segments);
        let n = segs.len();
        if n == 0 {
            return events;
        }
        let names: Vec<String> = self.p.icons.split(',').map(|s| s.trim().to_owned()).collect();

        let disabled = self.p.disabled;
        let alpha = if disabled { 0.4 } else { 1.0 };
        let surface = self.p.tone == "surface";
        let track = surface || self.p.variant == "surfaceThumb";
        let show_icon = self.p.shape == "icon" || self.p.variant == "surfaceThumb";
        let show_label = self.p.shape == "text";
        let md = self.p.size != "sm";
        let height = if md { rem(1.643) } else { rem(1.357) };
        let pad = if track { rem(0.143) } else { 0.0 };
        let label_size = if md { 12.0 } else { 11.0 };
        let label_pad = if md { rem(0.286) } else { rem(0.143) };
        let fixed_w = if md { rem(1.786) } else { rem(1.357) };
        let icon_box = rem(1.071);
        let gap = rem(0.286);
        let radius = CornerRadius::same(4);
        let label_font = FontId::new(label_size, weight(600));
        let icon_font = FontId::new(15.0, FontFamily::Name("Icons".into()));

        // Segment widths: fixed for icon-only, otherwise the content plus the label padding.
        let mut widths = Vec::with_capacity(n);
        for (_, label) in &segs {
            let w = if self.p.shape == "icon" {
                fixed_w
            } else {
                let mut w = 2.0 * label_pad;
                if show_icon {
                    w += icon_box;
                }
                if show_label {
                    if show_icon {
                        w += gap;
                    }
                    w += ui.painter().layout_no_wrap(label.clone(), label_font.clone(), t.content_1).size().x;
                }
                w
            };
            widths.push(w);
        }

        // The group: a track for the surface tone and the surfaceThumb variant.
        let total_w = widths.iter().sum::<f32>() + 2.0 * pad;
        let (group_rect, _) = ui.allocate_exact_size(egui::vec2(total_w, height + 2.0 * pad), Sense::hover());
        let group = ui.interact(group_rect, ui.scope_id().with("group"), Sense::hover());
        let group_label = self.p.groupName.clone();
        group.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, group_label.clone()));
        part(ui, &group, parts::GROUP);
        if track {
            ui.painter().rect_filled(group_rect, radius, t.wash_2.gamma_multiply(alpha));
        }

        // The segments: one interaction per segment, laid out left to right inside the track.
        let base = ui.scope_id().with("segment");
        let sense = if disabled { Sense::hover() } else { Sense::click() };
        let mut x = group_rect.left() + pad;
        let mut rects: Vec<Rect> = Vec::with_capacity(n);
        let mut resps: Vec<Response> = Vec::with_capacity(n);
        for (i, w) in widths.iter().enumerate() {
            let rect = Rect::from_min_size(egui::pos2(x, group_rect.top() + pad), egui::vec2(*w, height));
            x += w;
            resps.push(ui.interact(rect, base.with(i), sense));
            rects.push(rect);
        }

        // egui moves focus between widgets on the arrow keys by itself, after this frame's code has
        // moved it; the segments lock the arrows so that only the arrows handled below move the focus.
        for r in &resps {
            let arrows = egui::EventFilter { horizontal_arrows: true, vertical_arrows: true, ..Default::default() };
            ui.ctx().memory_mut(|m| m.set_focus_lock_filter(r.id, arrows));
        }

        // Keys: arrows move focus and select the segment reached (wrapping); Space selects;
        // Enter does nothing (Radix prevents it on radio items); Tab leaves the group.
        let focused = resps.iter().position(|r| r.has_focus());
        let (tab, space, enter, next, prev) = ui.input(|i| {
            (
                i.key_pressed(Key::Tab),
                i.key_pressed(Key::Space),
                i.key_pressed(Key::Enter),
                i.key_pressed(Key::ArrowRight) || i.key_pressed(Key::ArrowDown),
                i.key_pressed(Key::ArrowLeft) || i.key_pressed(Key::ArrowUp),
            )
        });

        if !disabled {
            if let Some(f) = focused {
                let target = if next {
                    Some((f + 1) % n)
                } else if prev {
                    Some((f + n - 1) % n)
                } else {
                    None
                };
                if let Some(to) = target {
                    ui.ctx().memory_mut(|m| m.request_focus(resps[to].id));
                    self.select(to, &segs, &mut events);
                    ui.ctx().request_repaint();
                }
            }
            // Tab moved focus from one segment to the next: the group is one tab stop, so drop it here.
            if tab && self.focus_prev.is_some() {
                if let Some(f) = focused {
                    ui.ctx().memory_mut(|m| m.surrender_focus(resps[f].id));
                }
            }
        }

        // Presses: a click, or Space on the focused segment. Choosing the selected segment changes nothing.
        if !disabled {
            let mut pressed = None;
            for (i, r) in resps.iter().enumerate() {
                let has_focus = ui.ctx().memory(|m| m.has_focus(r.id));
                if (r.clicked() && !(has_focus && enter)) || (space && has_focus) {
                    pressed = Some(i);
                }
            }
            if let Some(i) = pressed {
                self.select(i, &segs, &mut events);
            }
        }

        // Draw each segment for the selection as it is now.
        let sel = segs.iter().position(|(v, _)| *v == self.p.value);
        let painter = ui.painter().clone();
        for i in 0..n {
            let r = &resps[i];
            let rect = rects[i];
            let is_sel = sel == Some(i);
            let has_focus = ui.ctx().memory(|m| m.has_focus(r.id));
            let hovered = r.hovered() && !disabled;

            let (fill, fg) = match (is_sel, surface) {
                (true, true) => (Some(t.surface_3), t.content_1),
                (true, false) => (Some(t.accent_500), t.on_accent),
                (false, _) => (if hovered { Some(t.wash_1) } else { None }, t.content_2),
            };
            if is_sel && !surface {
                // The lift shadow, as a short offset under the accent fill.
                painter.rect_filled(rect.translate(egui::vec2(0.0, 1.0)), radius, t.content_1.gamma_multiply(0.12 * alpha));
            }
            if let Some(f) = fill {
                painter.rect_filled(rect, radius, f.gamma_multiply(alpha));
            }
            if has_focus {
                painter.rect_stroke(rect.expand(2.0), radius, Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
            }

            // The node the contract reads: a button whose selected state is the segment's choice.
            let label = segs[i].1.clone();
            r.widget_info(|| WidgetInfo::labeled(Role::Button, !disabled, label.clone()));
            // The chosen segment is read through the node's selected state (the chosen tab, segment or option).
            ui.ctx().accesskit_node_builder(r.id, |node| node.set_selected(is_sel));
            part(ui, r, PARTS[i]);

            // Content, centred: the icon (a fixed box), then the label.
            let fg = fg.gamma_multiply(alpha);
            let name = names.get(i).map(String::as_str).unwrap_or("");
            let label_w = if show_label {
                painter.layout_no_wrap(segs[i].1.clone(), label_font.clone(), fg).size().x
            } else {
                0.0
            };
            let content_w = (if show_icon { icon_box } else { 0.0 })
                + (if show_icon && show_label { gap } else { 0.0 })
                + label_w;
            let mut cx = rect.center().x - content_w / 2.0;
            if show_icon {
                let glyph = icon(name, 15.0);
                painter.text(
                    egui::pos2(cx + icon_box / 2.0, rect.center().y),
                    Align2::CENTER_CENTER,
                    glyph.text(),
                    icon_font.clone(),
                    fg,
                );
                cx += icon_box + gap;
            }
            if show_label {
                painter.text(egui::pos2(cx, rect.center().y), Align2::LEFT_CENTER, segs[i].1.as_str(), label_font.clone(), fg);
            }
        }

        self.focus_prev = (0..n).find(|&i| ui.ctx().memory(|m| m.has_focus(resps[i].id)));
        events
    }
}

crate::mounted!(Segmentedcontrol, SegmentedcontrolProps);
