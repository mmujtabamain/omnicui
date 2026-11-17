//! contract: omnicui/contracts/numberfield.contract.yaml (id: ui.numberfield). A narrow day-count field: whole days, clamped to min..max, stepped by the arrow keys.
use crate::omnicui::numberfield::{parts, NumberfieldEvent, NumberfieldProps};
use crate::{part, rem, weight, Look};
use egui::text::{CCursor, CCursorRange};
use egui::text_edit::TextEditState;
use egui::{vec2, Align2, Context, CornerRadius, FontId, Id, Key, Modifiers, Rect, Sense, Stroke, StrokeKind, TextEdit, Ui};

/// Clamps a day count into min..max.
fn clamp(n: i64, min: i64, max: i64) -> i64 {
    n.max(min).min(max)
}

/// The whole days in the text: None while it is empty or not a number. A fraction is truncated.
fn parse_days(text: &str) -> Option<i64> {
    let t = text.trim();
    if t.is_empty() {
        return None;
    }
    let n: f64 = t.parse().ok()?;
    n.is_finite().then(|| n.trunc() as i64)
}

/// Puts the text caret at the end of a field and clears any selection.
fn caret_to_end(ctx: &Context, id: Id, len: usize) {
    let mut state = TextEditState::load(ctx, id).unwrap_or_default();
    state.cursor.set_char_range(Some(CCursorRange::one(CCursor::new(len))));
    state.store(ctx, id);
}

pub struct NumberField {
    p: NumberfieldProps,
    /// The text the field shows. Edits show at once, so this is the field's own state.
    text: String,
    /// The day count last emitted, or seeded from the value prop.
    committed: i64,
    had_focus: bool,
}

impl NumberField {
    pub fn new(p: NumberfieldProps) -> Self {
        let committed = clamp(p.value, p.min, p.max);
        Self { p, text: committed.to_string(), committed, had_focus: false }
    }

    /// Emits a day count, unless it is the one already committed.
    fn commit(&mut self, next: i64, events: &mut Vec<NumberfieldEvent>) {
        if next != self.committed {
            self.committed = next;
            events.push(NumberfieldEvent::Changed { value: next });
        }
    }

    /// One frame: draws the field and returns the events it raised in this frame.
    pub fn ui(&mut self, ui: &mut Ui, look: Look) -> Vec<NumberfieldEvent> {
        let mut events = vec![];
        let (min, max) = (self.p.min, self.p.max);
        let id = ui.make_persistent_id("field");
        let focused = ui.memory(|m| m.has_focus(id));
        let mut move_caret = focused && !self.had_focus;
        self.had_focus = focused;

        // The arrow keys, Home and End step the value before the text field sees them, since the
        // text field would otherwise move its caret with them.
        if focused {
            let current = parse_days(&self.text).unwrap_or(self.committed);
            let stepped = if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowUp)) {
                Some(clamp(current + 1, min, max))
            } else if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowDown)) {
                Some(clamp(current - 1, min, max))
            } else if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Home)) {
                Some(min)
            } else if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::End)) {
                Some(max)
            } else {
                None
            };
            if let Some(next) = stepped {
                self.text = next.to_string();
                self.commit(next, &mut events);
                move_caret = true;
            }
        }
        if move_caret {
            caret_to_end(ui.ctx(), id, self.text.chars().count());
        }

        let t = look.t;
        // w-12 by h-6.5 at 1rem = 14px: 42 x 22.75 px. Radius 4 px. Border 1 px edge_2, edge_3 on hover.
        let (rect, _) = ui.allocate_exact_size(vec2(rem(3.0), rem(1.625)), Sense::hover());
        let edge = if ui.rect_contains_pointer(rect) { t.edge_3 } else { t.edge_2 };
        ui.painter().rect(rect, CornerRadius::same(4), t.surface_3, Stroke::new(1.0, edge), StrokeKind::Inside);

        // The text sits centred in the padded box, one line high.
        let font = FontId::new(rem(0.929), weight(400));
        let line = ui.ctx().fonts_mut(|f| f.row_height(&font));
        let text_rect = Rect::from_center_size(rect.center(), vec2(rect.width() - rem(0.25) * 2.0, line.min(rect.height())));
        let field = TextEdit::singleline(&mut self.text)
            .id(id)
            .frame(egui::Frame::NONE)
            .font(font)
            .text_color(t.content_1)
            .align(Align2::CENTER_CENTER)
            .desired_width(text_rect.width());
        let response = ui.put(text_rect, field);
        part(ui, &response, parts::FIELD);

        if response.gained_focus() {
            // Keep the caret at the end, so Backspace removes the last digit (no select-all on focus).
            caret_to_end(ui.ctx(), id, self.text.chars().count());
        }
        if response.changed() {
            self.text.retain(|c| c.is_ascii_digit());
            match parse_days(&self.text) {
                // Empty while editing: shown, not emitted, not pinned to min.
                None => {}
                Some(days) if days > max => {
                    self.text = max.to_string();
                    self.commit(max, &mut events);
                    caret_to_end(ui.ctx(), id, self.text.chars().count());
                }
                // Below min the typed value is shown and held until blur: more digits may bring it into range.
                Some(days) if days >= min => self.commit(days, &mut events),
                Some(_) => {}
            }
        }
        if response.lost_focus() {
            // On blur an empty field reverts to the last emitted value, and a typed value is clamped.
            let next = parse_days(&self.text).map_or(self.committed, |d| clamp(d, min, max));
            self.text = next.to_string();
            self.commit(next, &mut events);
        }

        // The node's name is the ariaLabel, and its value is the text the field shows.
        let label = self.p.ariaLabel.clone();
        let value = self.text.clone();
        ui.ctx().accesskit_node_builder(response.id, |node| {
            if !label.is_empty() {
                node.set_label(label);
            }
            node.set_value(value);
        });

        // Focus ring: 1 px of the neutral foreground outside the field (the web's ring-1 ring-content-1).
        if response.has_focus() {
            ui.painter().rect_stroke(rect, CornerRadius::same(4), Stroke::new(1.0, t.content_1), StrokeKind::Outside);
        }
        events
    }
}

crate::mounted!(NumberField, NumberfieldProps);
