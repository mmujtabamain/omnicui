//! contract: omnicui/contracts/button.contract.yaml (id: ui.button). The one general action button: a label with an optional leading and trailing glyph, in the web's variants, tones, sizes and shapes; a press, Enter or Space activates it.
use crate::omnicui::button::{parts, ButtonEvent, ButtonProps};
use crate::{icon, part, text, weight, Look, Tokens};
use egui::text::{LayoutJob, TextFormat};
use egui::{pos2, vec2, Align2, Color32, CornerRadius, FontFamily, FontId, Galley, Key, Painter, Rect, Role, Sense, Shape, Stroke, StrokeKind, WidgetInfo};
use std::sync::Arc;

pub struct Button {
    p: ButtonProps,
    count: i64,
    /// Escape was pressed while focused: the tooltip stays shut until focus leaves.
    escaped: bool,
    /// The button had keyboard focus at the end of the last frame.
    focused: bool,
}

/// The web's size table in px: box height, horizontal padding, label size, glyph size and corner radius.
struct Dims {
    h: f32,
    px: f32,
    text: f32,
    glyph: f32,
    radius: f32,
}

fn dims(size: &str, media: bool) -> Dims {
    let (h, px, text, glyph, radius) = match size {
        "sm" => (21.0, 6.0, 12.0, 14.0, 4.375),
        "default" => (21.0, 6.0, 13.0, 16.0, 4.375),
        "lg" => (28.0, 10.0, 14.0, 16.0, 4.375),
        "footer" => (19.0, 4.0, 12.0, 14.0, 4.375),
        "xl" => (28.0, 8.0, 14.0, if media { 17.0 } else { 19.0 }, 7.0),
        "step" => (39.0, 12.0, 14.0, 26.0, 19.5),
        "dashedShort" => (35.0, 12.0, 13.0, 16.0, 7.0),
        "dashedTall" => (84.0, 12.0, 13.0, 18.0, 7.0),
        _ => (24.5, 8.0, 13.0, 16.0, 4.375),
    };
    Dims { h, px, text, glyph, radius }
}

/// The text colour a tone gives, with its hover step.
fn tone_text(tone: &str, hot: bool, t: &Tokens) -> Color32 {
    match tone {
        "strong" => t.content_1,
        "accent" if hot => t.accent_600,
        "accent" => t.accent_500,
        "danger" if hot => t.danger_600,
        "danger" => t.danger_500,
        "done" => t.done_lamp_1,
        "muted" if hot => t.content_2,
        "muted" => t.content_3,
        _ if hot => t.content_1,
        _ => t.content_2,
    }
}

/// An accelerator as shown: "Mod+S" is ⌘S on macOS and Ctrl+S elsewhere.
fn accel(s: &str) -> String {
    let mac = cfg!(target_os = "macos");
    let keys: Vec<String> = s
        .split('+')
        .map(str::trim)
        .map(|k| if k == "Mod" { if mac { "⌘".to_owned() } else { "Ctrl".to_owned() } } else { k.to_owned() })
        .collect();
    if mac { keys.concat() } else { keys.join("+") }
}

/// A label laid out on one line (ellipsis when truncated) or wrapped within max_w.
fn label_galley(ui: &egui::Ui, s: String, font: FontId, max_w: f32, truncate: bool) -> Arc<Galley> {
    // PLACEHOLDER colour: the painter fills in the tone colour at paint time (the default is a grey).
    let mut job = LayoutJob::single_section(s, TextFormat { font_id: font, color: egui::Color32::PLACEHOLDER, ..Default::default() });
    job.wrap.max_width = max_w;
    if truncate {
        job.wrap.max_rows = 1;
        job.wrap.break_anywhere = true;
        job.wrap.overflow_character = Some('…');
    }
    ui.painter().layout_job(job)
}

/// Draws a Material Symbols glyph centred in a rect.
fn draw_glyph(painter: &Painter, name: &str, at: Rect, px: f32, color: Color32) {
    let glyph = icon(name, px);
    painter.text(at.center(), Align2::CENTER_CENTER, glyph.text(), FontId::new(px, FontFamily::Name("Icons".into())), color);
}

impl Button {
    pub fn new(p: ButtonProps) -> Self {
        Self { p, count: 0, escaped: false, focused: false }
    }

    /// One frame: draws the button and returns the events it raised in this frame.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<ButtonEvent> {
        let mut events = vec![];
        let p = self.p.clone();
        let t = look.t;
        let media = p.surface == "media";
        let icon_shape = matches!(p.shape.as_str(), "icon" | "square" | "round");
        let stacked = p.layout == "stacked";
        let error = !p.errorText.is_empty();
        let feedback = !p.feedbackLabel.is_empty();
        let disabled = p.disabled || p.busy;

        // Sizes, in px (the web's rem at 14px).
        let d = dims(&p.size, media);
        let h = d.h;
        let glyph_px = if p.iconSize > 0 { p.iconSize as f32 } else { d.glyph };
        let radius = if p.radius > 0.0 {
            p.radius as f32
        } else if p.shape == "round" || p.size == "step" {
            h / 2.0
        } else if p.shape == "icon" && p.size == "md" {
            5.0
        } else {
            d.radius
        };
        let px = if icon_shape || p.variant == "link" { 0.0 } else if p.block { 8.0 } else { d.px };
        let gap = if stacked { 4.0 } else { 6.0 };

        // What is written: the feedback label while set, else the label; the empty-label dash when that is empty.
        let typed = if feedback { p.feedbackLabel.clone() } else { p.label.clone() };
        let show_label = p.shape == "text" && (!typed.is_empty() || !p.emptyLabel.is_empty());
        let label_text = if typed.is_empty() { p.emptyLabel.clone() } else { typed.clone() };

        // The leading glyph: the spinner while busy, else the error glyph, the feedback icon, or the icon.
        let lead_name = if p.busy {
            "progress_activity".to_owned()
        } else if error {
            "error".to_owned()
        } else if feedback && !p.feedbackIcon.is_empty() {
            p.feedbackIcon.clone()
        } else {
            p.icon.clone()
        };
        let has_lead = !lead_name.is_empty();
        let has_trail = !p.trailingIcon.is_empty();
        let lead_w = if has_lead { glyph_px } else { 0.0 };
        let trail_w = if has_trail { glyph_px } else { 0.0 };
        let n_items = has_lead as usize + show_label as usize + has_trail as usize;
        let row_gaps = gap * n_items.saturating_sub(1) as f32;

        // Font: Inter at the size's px, medium (500) by default, semibold for strong, normal for block and mono.
        let strong = p.variant == "strong";
        let weight_w = if p.block || p.mono {
            400
        } else if strong {
            600
        } else {
            500
        };
        // No monospace face is installed (only Inter and the icons), so a mono path is Inter at 11.5px.
        let family = weight(weight_w);
        let font = FontId::new(if p.mono { 11.5 } else { d.text }, family);

        // Width: icon shapes are square; a stacked tile is square at its width; otherwise the width prop, or the label's width.
        let avail = if ui.available_width().is_finite() { ui.available_width() } else { 240.0 };
        let fixed_w = if icon_shape {
            Some(h)
        } else if stacked {
            Some(if p.width == "full" { avail } else { avail * 0.5 })
        } else {
            match p.width.as_str() {
                "content" => None,
                "half" => Some(avail * 0.5),
                _ => Some(avail),
            }
        };
        let label_max = match fixed_w {
            None => f32::INFINITY,
            Some(w) if stacked => (w - 8.0).max(0.0),
            Some(w) => (w - 2.0 * px - lead_w - trail_w - row_gaps).max(0.0),
        };
        let label = if show_label {
            Some(label_galley(ui, label_text.clone(), font, label_max, p.truncate))
        } else {
            None
        };
        let label_w = label.as_ref().map_or(0.0, |g| g.size().x);
        let label_h = label.as_ref().map_or(0.0, |g| g.size().y);
        let (w, box_h) = match fixed_w {
            Some(w) => (w, if stacked { w } else { h }),
            None => (label_w + lead_w + trail_w + row_gaps + 2.0 * px, h),
        };

        // A disabled or busy button takes no pointer and no focus. Without tabbable it is not in the Tab order.
        let sense = if disabled { Sense::hover() } else { Sense::click() };
        let (rect, response) = ui.allocate_exact_size(vec2(w, box_h), sense);
        if !p.tabbable && response.has_focus() {
            ui.ctx().memory_mut(|m| m.surrender_focus(response.id));
        }

        let hovered = !disabled && ui.rect_contains_pointer(rect);
        // Escape does not activate. egui surrenders focus on Escape, so the button takes it back, and
        // the tooltip stays shut until focus leaves.
        let escape = ui.input(|i| i.key_pressed(Key::Escape));
        if escape && (response.has_focus() || self.focused) {
            self.escaped = true;
            response.request_focus();
        } else if !response.has_focus() {
            self.escaped = false;
        }
        self.focused = response.has_focus();
        if !disabled && response.clicked() {
            self.count += 1;
            events.push(ButtonEvent::Activated { count: self.count });
        }

        let name = if p.ariaLabel.is_empty() { label_text.clone() } else { p.ariaLabel.clone() };
        let enabled = !disabled;
        let active = p.active;
        response.widget_info(|| WidgetInfo::selected(Role::Button, enabled, active, &name));
        if active {
            // The contract reads an active button as `selected`, which WidgetInfo does not carry for a button.
            ui.ctx().accesskit_node_builder(response.id, |node| node.set_selected(true));
        }
        part(ui, &response, parts::BUTTON);

        // Opacity: disabled is 45%; reveal row fades in on hover or focus.
        let reveal = if p.reveal == "row" {
            ui.ctx().animate_bool_with_time(response.id.with("reveal"), hovered || response.has_focus(), 0.18)
        } else {
            1.0
        };
        let a = (if disabled { 0.45 } else { 1.0 }) * reveal;

        // Colours: the variant's fill and border, then the tone's text colour.
        let hot = hovered;
        let mut fg = tone_text(&p.tone, hot, &t);
        let mut fill: Option<Color32> = None;
        let mut border: Option<(Color32, bool)> = None;
        let mut underline = false;
        match p.variant.as_str() {
            "primary" => {
                fill = Some(if hot { t.accent_600 } else { t.accent_500 });
                fg = t.on_accent;
            }
            "neutral" => {
                fill = Some(if hot { t.wash_2 } else { t.surface_0 });
                fg = t.content_1;
            }
            "outline" => {
                fill = Some(t.surface_3);
                border = Some((if hot { t.edge_3 } else { t.edge_2 }, false));
            }
            "danger" => {
                fill = Some(if hot { t.danger_600 } else { t.danger_500 });
                fg = t.on_accent;
            }
            "strong" => {
                if matches!(p.tone.as_str(), "default" | "neutral") {
                    fg = t.content_1;
                }
            }
            "filled" => {
                fill = Some(if hot { t.wash_2 } else { t.surface_0 });
            }
            "dashed" => {
                border = Some((if hot { t.edge_3 } else { t.edge_2 }, true));
                if hot {
                    fill = Some(t.wash_1);
                }
            }
            "link" => {
                underline = hot;
            }
            _ => {}
        }
        if media {
            let white = Color32::WHITE;
            fg = if hot { white } else { white.gamma_multiply(0.9) };
            if p.tone == "danger" && hot {
                fill = Some(t.danger_500);
            } else if p.variant == "filled" {
                fill = Some(white.gamma_multiply(if hot { 0.2 } else { 0.1 }));
            } else if p.variant == "ghost" && hot {
                fill = Some(white.gamma_multiply(0.15));
            }
        }
        if hot && p.hover == "wash" && fill.is_none() {
            fill = Some(t.wash_1);
        }
        if p.active {
            fill = Some(t.wash_2);
        }
        if error {
            fg = if hot { t.danger_600 } else { t.danger_500 };
        }

        // Paint: fill, border, focus ring, then the glyphs and the label.
        let painter = ui.painter();
        let cr = CornerRadius::same(radius.round().clamp(0.0, 255.0) as u8);
        if let Some(c) = fill {
            painter.rect_filled(rect, cr, c.gamma_multiply(a));
        }
        if let Some((c, dashed)) = border {
            let stroke = Stroke::new(1.0, c.gamma_multiply(a));
            if dashed {
                let edge = [rect.left_top(), rect.right_top(), rect.right_bottom(), rect.left_bottom(), rect.left_top()];
                painter.extend(Shape::dashed_line(&edge, stroke, 4.0, 3.0));
            } else {
                painter.rect_stroke(rect, cr, stroke, StrokeKind::Inside);
            }
        }
        if response.has_focus() && !disabled {
            painter.rect_stroke(rect, cr, Stroke::new(2.0, t.focus_1.gamma_multiply(a)), StrokeKind::Outside);
        }

        // Placement of the items: a row (centred, or left aligned for block) or a column for stacked.
        let cy = rect.center().y;
        let mut placed: Vec<(u8, Rect)> = vec![];
        if stacked {
            let gap_v = if has_lead && show_label { gap } else { 0.0 };
            let total = lead_w + gap_v + label_h;
            let mut y = cy - total / 2.0;
            if has_lead {
                placed.push((0, Rect::from_min_size(pos2(rect.center().x - lead_w / 2.0, y), vec2(lead_w, lead_w))));
                y += lead_w + gap_v;
            }
            if label.is_some() {
                placed.push((1, Rect::from_min_size(pos2(rect.center().x - label_w / 2.0, y), vec2(label_w, label_h))));
            }
        } else {
            let mut items: Vec<(u8, f32, f32)> = vec![];
            if has_lead {
                items.push((0, lead_w, glyph_px));
            }
            if show_label {
                items.push((1, label_w, label_h));
            }
            if has_trail {
                items.push((2, trail_w, glyph_px));
            }
            let total = items.iter().map(|i| i.1).sum::<f32>() + row_gaps;
            let mut x = if p.block { rect.left() + px } else { rect.center().x - total / 2.0 };
            for (k, iw, ih) in items {
                placed.push((k, Rect::from_min_size(pos2(x, cy - ih / 2.0), vec2(iw, ih))));
                x += iw + gap;
            }
        }

        let glyph_col = fg.gamma_multiply(a);
        let label_col = (if typed.is_empty() { t.content_3 } else { fg }).gamma_multiply(a);
        for (k, r) in &placed {
            match k {
                0 => draw_glyph(painter, &lead_name, *r, glyph_px, glyph_col),
                1 => {
                    if let Some(g) = &label {
                        painter.galley(r.min, g.clone(), label_col);
                    }
                }
                _ => draw_glyph(painter, &p.trailingIcon, *r, glyph_px, glyph_col),
            }
        }
        if underline {
            if let Some((_, r)) = placed.iter().find(|(k, _)| *k == 1) {
                let y = r.bottom() - 1.0;
                painter.line_segment([pos2(r.left(), y), pos2(r.right(), y)], Stroke::new(1.0, fg.gamma_multiply(a)));
            }
        }

        // Parts: the glyphs, the label, the feedback status and the tooltip.
        for (k, r) in &placed {
            let (id, part_id) = match k {
                0 => ("lead", if p.busy { parts::SPINNER } else { parts::ICON }),
                1 => ("label", parts::LABEL),
                _ => ("trailing", parts::TRAILINGICON),
            };
            let node = ui.interact(*r, response.id.with(id), Sense::hover());
            if *k == 1 {
                node.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, !disabled, &label_text));
            } else {
                node.widget_info(|| WidgetInfo::new(Role::GenericContainer));
            }
            part(ui, &node, part_id);
        }
        if feedback {
            let node = ui.interact(rect, response.id.with("feedback"), Sense::hover());
            node.widget_info(|| WidgetInfo::labeled(Role::Status, true, &p.feedbackLabel));
            part(ui, &node, parts::FEEDBACK);
        }

        let mut tip_parts = vec![if error { p.errorText.clone() } else { p.description.clone() }];
        if !p.shortcut.is_empty() {
            tip_parts.push(accel(&p.shortcut));
        }
        let tip = tip_parts.into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join(" ");
        let show_tip = !tip.is_empty() && !self.escaped && (response.has_focus() || hovered);
        if show_tip {
            let tip_id = response.id.with("tooltip");
            let pos = rect.left_bottom() + vec2(0.0, 4.0);
            egui::Area::new(tip_id).order(egui::Order::Tooltip).fixed_pos(pos).show(ui.ctx(), |tui| {
                let frame = egui::Frame::popup(tui.style()).show(tui, |tui| {
                    tui.label(text(tip.clone(), 12.0, 500));
                });
                let node = tui.interact(frame.response.rect, tip_id.with("box"), Sense::hover());
                node.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, &tip));
                part(tui, &node, parts::TOOLTIP);
            });
        }

        events
    }
}

crate::mounted!(Button, ButtonProps);
