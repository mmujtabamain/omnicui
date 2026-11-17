//! contract: omnicui/contracts/sectionheader.contract.yaml (id: ui.sectionheader). An eyebrow heading for a content block: an 11px semibold caps label, an optional hairline or trailing rule, a count, and an optional restore control with a changed dot.
use crate::omnicui::sectionheader::{parts, SectionheaderEvent, SectionheaderProps};
use crate::{icon, part, rem, weight, Look};
use egui::text::{LayoutJob, TextFormat};
use egui::{pos2, vec2, Align2, Color32, CornerRadius, FontFamily, FontId, Galley, Rect, Role, Sense, Stroke, StrokeKind, WidgetInfo};
use std::sync::Arc;

/// The label's size in px (text xs). Its lowercase letters are small capitals at 80% of it, and it tracks by 0.08em.
const LABEL_PX: f32 = 11.0;
/// The restore control: an icon button at size sm, 21 px square, with a 14 px glyph.
const RESTORE_PX: f32 = 21.0;
const GLYPH_PX: f32 = 14.0;

pub struct SectionHeader {
    p: SectionheaderProps,
}

/// The eyebrow label as the web draws it: font-variant caps, so lowercase letters are small capitals
/// and every letter tracks. The text itself stays as written (the node's label is the label).
fn title_galley(ui: &egui::Ui, label: &str, color: Color32, max_w: f32) -> Arc<Galley> {
    // Runs of capitals and runs of small capitals, each run uppercased.
    let mut runs: Vec<(bool, String)> = vec![];
    for c in label.chars() {
        let small = c.is_lowercase();
        let extended = match runs.last_mut() {
            Some((prev, text)) if *prev == small => {
                text.extend(c.to_uppercase());
                true
            }
            _ => false,
        };
        if !extended {
            runs.push((small, c.to_uppercase().collect()));
        }
    }
    let mut job = LayoutJob::default();
    for (small, text) in &runs {
        let size = if *small { LABEL_PX * 0.8 } else { LABEL_PX };
        job.append(
            text,
            0.0,
            TextFormat {
                font_id: FontId::new(size, weight(600)),
                color,
                extra_letter_spacing: LABEL_PX * 0.08,
                ..Default::default()
            },
        );
    }
    // Truncated with an ellipsis on one line when it is wider than the room the header gives it.
    job.wrap.max_width = max_w;
    job.wrap.max_rows = 1;
    job.wrap.break_anywhere = true;
    job.wrap.overflow_character = Some('…');
    ui.painter().layout_job(job)
}

impl SectionHeader {
    pub fn new(p: SectionheaderProps) -> Self {
        Self { p }
    }

    /// One frame: draws the header and returns the events it raised in this frame.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<SectionheaderEvent> {
        let mut events = vec![];
        let p = &self.p;
        let t = look.t;

        let list = p.inset == "listGroup";
        let heading = !list && (p.r#as == "h2" || p.r#as == "h3");
        let show_rule = !list && p.rule != "none";
        let hairline = p.rule == "hairline";
        let has_trailing = !p.trailing.is_empty();
        let show_restore = p.active && p.restore != "none";
        let has_group = has_trailing || show_restore;

        // The web's spacing, in rem (1rem = 14px): the listGroup's px-3, mt-2 and mb-1; the root's gap-2; gap-1.
        let pad = if list { rem(0.75) } else { 0.0 };
        let gap = rem(0.5);
        let gap_small = rem(0.25);
        let dot_px = rem(0.429);
        if list {
            ui.add_space(rem(0.5));
        }

        let avail = if ui.available_width().is_finite() { ui.available_width() } else { 240.0 };
        let inner = (avail - 2.0 * pad).max(0.0);

        // The fixed-width items: the count text, then the restore control after a gap-1.
        let count = has_trailing.then(|| ui.painter().layout_no_wrap(p.trailing.clone(), FontId::new(LABEL_PX, weight(400)), t.content_2));
        let count_w = count.as_ref().map_or(0.0, |c| c.size().x);
        let count_h = count.as_ref().map_or(0.0, |c| c.size().y);
        let sep = if has_trailing && show_restore { gap_small } else { 0.0 };
        let restore_w = if show_restore { RESTORE_PX } else { 0.0 };
        let group_w = count_w + sep + restore_w;
        let dot_w = if p.active { dot_px } else { 0.0 };

        // The items in the row after the title: the dot, the rule, the group. Each is one flex gap apart.
        let n_other = p.active as usize + has_group as usize + show_rule as usize;
        let gaps = gap * n_other as f32;
        // A trailing rule keeps its minimum (min-w-4) so the title leaves room for it.
        let rule_min = if show_rule && !hairline { rem(1.0) } else { 0.0 };
        let title_max = (inner - dot_w - group_w - gaps - rule_min).max(0.0);
        let title = title_galley(ui, &p.label, t.content_2, title_max);
        let title_w = title.size().x;
        let title_h = title.size().y;

        // The rule: a hairline runs a fixed 17.9rem and shrinks; a trailing rule fills the rest.
        let room = inner - title_w - dot_w - group_w - gaps;
        let rule_w = if !show_rule {
            0.0
        } else if hairline {
            room.clamp(0.0, rem(17.9))
        } else {
            room.max(rule_min)
        };

        let row_h = title_h.max(count_h).max(restore_w);
        let (rect, root) = ui.allocate_exact_size(vec2(avail, row_h), Sense::hover());

        let painter = ui.painter();
        let cy = rect.center().y;
        let mut x = rect.left() + pad;

        // The title: a heading for block headings, a plain region for a listGroup.
        let title_rect = Rect::from_min_size(pos2(x, cy - title_h / 2.0), vec2(title_w, title_h));
        painter.galley(title_rect.min, title, t.content_2);
        let role = if heading { Role::Heading } else { Role::GenericContainer };
        let title_resp = ui.interact(title_rect, root.id.with("title"), Sense::hover());
        title_resp.widget_info(|| WidgetInfo::labeled(role, true, &p.label));
        part(ui, &title_resp, parts::TITLE);
        x += title_w;

        // The changed dot: a round 6px mark in primary ink, decorative.
        if p.active {
            x += gap;
            let dot_rect = Rect::from_min_size(pos2(x, cy - dot_w / 2.0), vec2(dot_w, dot_w));
            painter.circle_filled(dot_rect.center(), dot_w / 2.0, t.content_1);
            let dot = ui.interact(dot_rect, root.id.with("changedDot"), Sense::hover());
            dot.widget_info(|| WidgetInfo::new(Role::GenericContainer));
            part(ui, &dot, parts::CHANGEDDOT);
        }

        // The rule: a 1px line in edge_2, decorative.
        if show_rule {
            x += gap;
            let rule_rect = Rect::from_min_size(pos2(x, cy - 0.5), vec2(rule_w, 1.0));
            painter.rect_filled(rule_rect, CornerRadius::same(0), t.edge_2);
            let rule = ui.interact(rule_rect, root.id.with("rule"), Sense::hover());
            rule.widget_info(|| WidgetInfo::new(Role::GenericContainer));
            part(ui, &rule, parts::RULE);
        }

        // The group sits at the trailing edge: the count, then the restore control.
        if has_group {
            let mut gx = rect.right() - pad - group_w;
            if let Some(c) = count {
                let cr = Rect::from_min_size(pos2(gx, cy - count_h / 2.0), vec2(count_w, count_h));
                painter.galley(cr.min, c, t.content_2);
                let trailing = ui.interact(cr, root.id.with("trailing"), Sense::hover());
                trailing.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, &p.trailing));
                part(ui, &trailing, parts::TRAILING);
                gx += count_w + sep;
            }
            if show_restore {
                let rr = Rect::from_min_size(pos2(gx, cy - RESTORE_PX / 2.0), vec2(RESTORE_PX, RESTORE_PX));
                let resp = ui.interact(rr, root.id.with("restore"), Sense::click());
                resp.widget_info(|| WidgetInfo::labeled(Role::Button, true, &p.restoreLabel));
                part(ui, &resp, parts::RESTORE);
                // A press, or Enter or Space on the focused control, runs the restore once.
                if resp.clicked() {
                    events.push(SectionheaderEvent::RestoreRequested { kind: p.restore.clone() });
                }
                // The ghost icon button: a wash on hover, muted glyph that darkens on hover.
                let hot = ui.rect_contains_pointer(rr);
                if hot {
                    painter.rect_filled(rr, CornerRadius::same(4), t.wash_1);
                }
                if resp.has_focus() {
                    painter.rect_stroke(rr.expand(2.0), CornerRadius::same(6), Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
                }
                let name = if p.restore == "reset" { "restart_alt" } else { "undo" };
                let glyph = icon(name, GLYPH_PX);
                let color = if hot { t.content_2 } else { t.content_3 };
                painter.text(rr.center(), Align2::CENTER_CENTER, glyph.text(), FontId::new(GLYPH_PX, FontFamily::Name("Icons".into())), color);
                if !p.restoreLabel.is_empty() {
                    let _ = resp.on_hover_text(p.restoreLabel.clone());
                }
            }
        }

        if list {
            ui.add_space(rem(0.25));
        }
        events
    }
}

crate::mounted!(SectionHeader, SectionheaderProps);
