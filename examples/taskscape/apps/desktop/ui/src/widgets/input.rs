//! contract: omnicui/contracts/input.contract.yaml (id: ui.input). Single-line text entry with no chrome of its own: an inline title that reads as text until it is clicked, a draft or bare field, and a multiline title that wraps to two lines.
use crate::omnicui::input::{parts, InputEvent, InputProps};
use crate::{part, rem, weight, Look};
use egui::text::CCursor;
use egui::text_selection::CCursorRange;
use egui::{pos2, vec2, Align, CornerRadius, FontId, Id, Key, Layout, Margin, Modifiers, Rect, Role, RichText, ScrollArea, Sense, Stroke, StrokeKind, TextEdit, UiBuilder, WidgetInfo};

/// The gap between a field's edge and its text (the web's px-space-4).
const PAD: f32 = 5.25;

pub struct Input {
    p: InputProps,
    /// The text as the field holds it now (raw, untrimmed).
    text: String,
    /// Inline: an editing session is open. The other modes are always editing.
    editing: bool,
    /// Commit has already fired for this session, or the session was cancelled.
    settled: bool,
    /// The text when the session opened; Escape reverts to it.
    session_start: String,
    seen_request: i64,
    want_focus: bool,
    select_all: bool,
    was_focused: bool,
}

impl Input {
    pub fn new(p: InputProps) -> Self {
        let text = p.value.clone();
        let want_focus = p.autoFocus && p.mode != "inline" && !p.disabled;
        let seen_request = p.editRequest;
        Self {
            p,
            session_start: text.clone(),
            text,
            editing: false,
            settled: false,
            seen_request,
            want_focus,
            select_all: false,
            was_focused: false,
        }
    }

    /// One frame: draws the field and returns the events it raised in this frame.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<InputEvent> {
        let mut events = vec![];
        // A changed editRequest asks an inline field to enter editing; a value present at mount does not.
        if self.p.editRequest != self.seen_request {
            self.seen_request = self.p.editRequest;
            if self.p.mode == "inline" && !self.p.disabled {
                self.start_edit();
            }
        }
        if self.editable() {
            self.keys(ui, &mut events);
        }
        if self.editable() {
            self.edit(ui, look, &mut events);
        } else {
            self.rest(ui, look);
        }
        events
    }

    /// Whether the field is showing its text editor this frame.
    fn editable(&self) -> bool {
        !self.p.disabled && (self.p.mode != "inline" || self.editing)
    }

    fn edit_id(&self) -> Id {
        Id::unique("ui.input.field")
    }

    /// Font size in px, line height in px and weight, from the size axis.
    fn metrics(&self) -> (f32, f32, u16) {
        match self.p.size.as_str() {
            "inline" => (14.0, 21.0, 500),
            "sm" => (12.0, 18.0, 400),
            "display" => (18.0, 21.0, 600),
            _ => (14.0, 21.0, 400),
        }
    }

    /// Horizontal padding: the bare mode has none, the others use px-space-4.
    fn pad(&self) -> f32 {
        if self.p.mode == "bare" {
            0.0
        } else {
            PAD
        }
    }

    /// The field's width: full width of its host, except inline, which caps at 10rem.
    fn width(&self, ui: &egui::Ui) -> f32 {
        let full = ui.available_width();
        let full = if full.is_finite() { full } else { rem(20.0) };
        if self.p.size == "inline" {
            full.min(rem(10.0))
        } else {
            full
        }
    }

    fn start_edit(&mut self) {
        self.editing = true;
        self.settled = false;
        self.session_start = self.text.clone();
        self.want_focus = true;
        self.select_all = self.p.selectOnOpen && !self.text.is_empty();
    }

    fn commit_once(&mut self, text: String, events: &mut Vec<InputEvent>) {
        if self.p.mode == "bare" || self.settled {
            return;
        }
        self.settled = true;
        events.push(InputEvent::Commit { value: text });
    }

    /// Reads the keys before the text editor sees them, so Enter never adds a line and Escape never
    /// leaves the field to the editor. Keys bubble (as propagated events) unless stopKeyPropagation.
    fn keys(&mut self, ui: &mut egui::Ui, events: &mut Vec<InputEvent>) {
        let id = self.edit_id();
        let active = self.was_focused || ui.memory(|m| m.has_focus(id));
        if !active {
            return;
        }
        let (shift_enter, enter, escape, tab) = ui.input_mut(|i| {
            let shift_enter = i.consume_key(Modifiers::SHIFT, Key::Enter);
            let enter = i.consume_key(Modifiers::NONE, Key::Enter);
            let escape = i.consume_key(Modifiers::NONE, Key::Escape);
            (shift_enter, enter, escape, i.key_pressed(Key::Tab))
        });
        let stop = self.p.stopKeyPropagation;

        if enter || shift_enter {
            // Enter submits. Shift+Enter behaves as Enter on a multiline field and does nothing on a single line.
            let key = if enter { "Enter" } else { "Shift+Enter" };
            if enter || self.p.multiline {
                let text = self.text.trim().to_owned();
                events.push(InputEvent::Submit { value: text.clone() });
                self.commit_once(text, events);
            }
            if !stop {
                events.push(InputEvent::Propagated { key: key.to_owned() });
            }
        }

        if escape {
            self.settled = true;
            events.push(InputEvent::Cancel {});
            if self.p.mode == "inline" {
                // Inline reverts the text and leaves editing without committing.
                self.text = self.session_start.clone();
                self.editing = false;
                ui.memory_mut(|m| m.surrender_focus(id));
            }
            if !stop {
                events.push(InputEvent::Propagated { key: "Escape".to_owned() });
            }
        }

        // Tab is not handled here: egui moves focus as the browser does, and the blur commits.
        if tab && !stop {
            events.push(InputEvent::Propagated { key: "Tab".to_owned() });
        }
    }

    /// The text editor: the field while it can be typed in.
    fn edit(&mut self, ui: &mut egui::Ui, look: Look, events: &mut Vec<InputEvent>) {
        let t = look.t;
        let (px, line, weight_w) = self.metrics();
        let font = FontId::new(px, weight(weight_w));
        let pad = self.pad();
        let width = self.width(ui);
        let inner_w = (width - 2.0 * pad).max(0.0);
        let multi = self.p.multiline;
        let edit_id = self.edit_id();
        let radius = CornerRadius::same(4);

        // A multiline field is one line (21px) and grows to two (42px), then scrolls. Counted in rows: an
        // 18 px line is taller than 21 px, so a height test made every display title two lines tall.
        let box_h = if multi && ui.painter().layout(self.text.clone(), font.clone(), t.content_1, inner_w).rows.len() > 1 {
            rem(3.0)
        } else {
            line
        };

        let (rect, _) = ui.allocate_exact_size(vec2(width, box_h), Sense::hover());
        if self.p.mode != "bare" {
            ui.painter().rect_filled(rect, radius, t.surface_0);
        }
        let inner = Rect::from_min_size(rect.min + vec2(pad, 0.0), vec2(inner_w, box_h));
        let layout = if multi {
            Layout::top_down(Align::Min)
        } else {
            Layout::left_to_right(Align::Center)
        };
        let mut child = ui.new_child(UiBuilder::new().max_rect(inner).layout(layout));
        child.visuals_mut().selection.bg_fill = t.selection_1;

        let hint = RichText::new(self.p.placeholder.clone()).color(t.content_3);
        let field = if multi {
            // One row at rest (egui's multiline field asks for four, which made the title two lines tall, #12343).
            TextEdit::multiline(&mut self.text).desired_rows(1)
        } else {
            TextEdit::singleline(&mut self.text)
        };
        let field = field
            .id(edit_id)
            .frame(egui::Frame::NONE)
            .margin(Margin::ZERO)
            .font(font)
            .text_color(t.content_1)
            .hint_text(hint)
            .desired_width(inner_w);
        let out = if multi {
            ScrollArea::vertical()
                .id_salt(edit_id.with("scroll"))
                .max_height(box_h)
                .min_scrolled_height(box_h)
                .auto_shrink([false, true])
                .show(&mut child, |ui| field.show(ui))
                .inner
        } else {
            field.show(&mut child)
        };
        let response = out.response;

        // Focus: a fresh session, autoFocus, or a rest field that was clicked asks for the editor.
        if self.want_focus {
            response.request_focus();
            self.want_focus = false;
        }
        let focused = ui.memory(|m| m.has_focus(edit_id));
        // Keyboard focus with a non-empty value selects all; a pointer focus places the caret instead.
        if focused && !self.was_focused && self.p.selectOnFocus && !response.is_pointer_button_down_on() && !self.text.is_empty() {
            self.select_all = true;
        }
        if focused && self.select_all {
            if let Some(mut state) = TextEdit::load_state(ui.ctx(), edit_id) {
                let end = self.text.chars().count();
                state
                    .cursor
                    .set_char_range(Some(CCursorRange::two(CCursor::new(0), CCursor::new(end))));
                state.store(ui.ctx(), edit_id);
            }
            self.select_all = false;
        }

        if response.changed() {
            events.push(InputEvent::ValueChange { value: self.text.clone() });
        }

        // Blur ends an inline session and commits (once per session) with the trimmed text.
        let lost = self.was_focused && !focused;
        self.was_focused = focused;
        if lost {
            if self.p.mode == "inline" {
                self.editing = false;
            }
            let text = self.text.trim().to_owned();
            self.commit_once(text, events);
        }

        // A bare field's host shows the focus (the sheet's name row): a ring here too read as two (#12360).
        if focused && self.p.mode != "bare" {
            ui.painter().rect_stroke(rect.expand(2.0), radius, Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
        }
        let label = self.p.ariaLabel.clone();
        response.widget_info(|| WidgetInfo::labeled(Role::TextInput, true, &label));
        set_value(ui, response.id, self.text.clone());
        part(ui, &response, parts::FIELD);
    }

    /// The field at rest: the text as plain text (inline), or a disabled field. A click on an inline
    /// field starts editing.
    fn rest(&mut self, ui: &mut egui::Ui, look: Look) {
        let t = look.t;
        self.was_focused = false;
        let disabled = self.p.disabled;
        let (px, line, weight_w) = self.metrics();
        let font = FontId::new(px, weight(weight_w));
        let pad = self.pad();
        let width = self.width(ui);
        let inner_w = (width - 2.0 * pad).max(0.0);
        let multi = self.p.multiline;
        let alpha = if disabled { 0.45 } else { 1.0 };
        let (shown, color) = if self.text.is_empty() {
            (self.p.placeholder.clone(), t.content_3)
        } else {
            (self.text.clone(), t.content_1)
        };
        let color = color.gamma_multiply(alpha);
        let galley = ui.painter().layout(shown, font, color, if multi { inner_w } else { f32::INFINITY });
        let box_h = if multi && galley.rows.len() > 1 { rem(3.0) } else { line };

        let sense = if disabled { Sense::hover() } else { Sense::click() };
        let (rect, response) = ui.allocate_exact_size(vec2(width, box_h), sense);
        if response.clicked() && self.p.mode == "inline" {
            self.start_edit();
        }

        let radius = CornerRadius::same(4);
        if self.p.mode == "draft" {
            ui.painter().rect_filled(rect, radius, t.surface_0.gamma_multiply(alpha));
        }
        let pos = if multi {
            rect.min + vec2(pad, 0.0)
        } else {
            pos2(rect.left() + pad, rect.center().y - galley.size().y / 2.0)
        };
        ui.painter().galley(pos, galley, color);
        if response.has_focus() && !disabled {
            ui.painter().rect_stroke(rect.expand(2.0), radius, Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
        }

        let label = self.p.ariaLabel.clone();
        response.widget_info(|| WidgetInfo::labeled(Role::TextInput, !disabled, &label));
        set_value(ui, response.id, self.text.clone());
        part(ui, &response, parts::FIELD);
    }
}

/// The node's value is the field's text, whichever of the two states draws it.
fn set_value(ui: &egui::Ui, id: Id, value: String) {
    ui.ctx().accesskit_node_builder(id, |node| node.set_value(value));
}

crate::mounted!(Input, InputProps);
