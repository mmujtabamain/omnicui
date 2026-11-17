//! contract: omnicui/contracts/mentionchip.contract.yaml (id: ui.mentionchip). An inline @mention chip for an attachment in a note body: a focusable pill that opens the attachment on a press, Enter or Space, and leaves the body as one unit on Backspace or Delete.
use crate::omnicui::mentionchip::{parts, MentionchipEvent, MentionchipProps};
use crate::{part, rem, weight, Look};
use egui::{Align2, CornerRadius, FontId, Key, Role, Sense, Stroke, StrokeKind, WidgetInfo};

/// The web's sm button: text-sm (0.875rem) in medium weight, a 1.714rem tall pill, px-2 padding.
const TEXT: f32 = rem(0.875);
const HEIGHT: f32 = rem(1.714);
const PAD_X: f32 = 8.0;

pub struct MentionChip {
    p: MentionchipProps,
    /// Backspace or Delete removed the chip: it has left the note body, so it is no longer drawn.
    removed: bool,
}

impl MentionChip {
    pub fn new(p: MentionchipProps) -> Self {
        Self { p, removed: false }
    }

    /// One frame: draws the chip and returns the events it raised in this frame.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<MentionchipEvent> {
        let mut events = vec![];
        if self.removed {
            return events;
        }
        let t = look.t;
        let label = format!("@{}", self.p.name);
        let font = FontId::new(TEXT, weight(500));
        // max-w-full: the chip takes its text's width, but never more than the line has.
        let natural = ui.painter().layout_no_wrap(label.clone(), font.clone(), t.accent_600).size().x + 2.0 * PAD_X;
        let width = natural.min(ui.available_width());
        let (rect, response) = ui.allocate_exact_size(egui::vec2(width, HEIGHT), Sense::click());

        // Backspace or Delete on the focused chip removes it as one unit. The chip is not tagged
        // in this frame, so it has no node and the target reports it as not visible.
        let remove = response.has_focus() && ui.input(|i| i.key_pressed(Key::Backspace) || i.key_pressed(Key::Delete));
        if remove {
            self.removed = true;
            events.push(MentionchipEvent::Removed { name: self.p.name.clone() });
            return events;
        }

        // A press, Enter or Space opens it: egui clicks a focused widget on Enter and Space. The press
        // also takes keyboard focus, as the web's button does.
        if response.clicked() {
            response.request_focus();
            events.push(MentionchipEvent::Opened { source: self.p.source.clone(), name: self.p.name.clone() });
        }
        response.widget_info(|| WidgetInfo::labeled(Role::Button, true, &label));
        part(ui, &response, parts::CHIP);

        // Local and remote look the same: the source only decides where the press goes.
        let radius = CornerRadius::same((HEIGHT / 2.0) as u8);
        let painter = ui.painter();
        painter.rect(rect, radius, t.selection_1, Stroke::NONE, StrokeKind::Inside);
        if response.has_focus() {
            painter.rect_stroke(rect.expand(2.0), radius, Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
        }
        painter
            .with_clip_rect(rect)
            .text(rect.left_center() + egui::vec2(PAD_X, 0.0), Align2::LEFT_CENTER, label, font, t.accent_600);
        events
    }
}

crate::mounted!(MentionChip, MentionchipProps);
