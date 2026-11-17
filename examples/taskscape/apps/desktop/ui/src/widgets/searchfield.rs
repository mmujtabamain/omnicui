//! contract: omnicui/contracts/searchfield.contract.yaml (id: ui.searchfield). The one search input for the list and settings search rows: a magnifier, a plain text field and an optional clear button, in a pill well; filter or match navigation, md or sm.
use crate::omnicui::searchfield::{parts, SearchfieldEvent, SearchfieldProps};
use crate::{icon, part, rem, text, weight, Look};
use egui::{pos2, vec2, Align, Align2, Color32, CornerRadius, FontFamily, FontId, Key, Layout, Margin, Modifiers, Painter, Pos2, Rect, Role, Sense, Stroke, StrokeKind, TextEdit, UiBuilder, WidgetInfo};

/// The search well: a pill in the search tone (wash fill, no border). Not built on InputGroup or Input,
/// because the field must stay a plain text input with the searchbox role inferred from the element.
pub struct SearchField {
    p: SearchfieldProps,
    /// The query as the field holds it now.
    value: String,
    /// autoFocus asks for focus once, on the first frame.
    want_focus: bool,
    /// The field had keyboard focus at the end of the last frame. egui surrenders focus on Escape before
    /// the frame's UI runs, so the key handling reads this too.
    was_focused: bool,
}

impl SearchField {
    pub fn new(p: SearchfieldProps) -> Self {
        let value = p.value.clone();
        let want_focus = p.autoFocus && !p.disabled;
        Self { p, value, want_focus, was_focused: false }
    }

    /// One frame: draws the well and returns the events it raised in this frame.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<SearchfieldEvent> {
        let mut events = vec![];
        let t = look.t;
        let disabled = self.p.disabled;
        let enabled = !disabled;
        let md = self.p.size != "sm";
        let matching = self.p.navigation == "match";
        let clearable = self.p.clearable;
        // A disabled well is drawn at the web's opacity-45.
        let alpha = if disabled { 0.45 } else { 1.0 };
        let fade = |c: Color32| c.gamma_multiply(alpha);

        // Sizes, in the web's rem: well 2.5rem (md) or 1.75rem (sm) high, field text 1rem or 0.857rem,
        // icon 18px or 14px, the well's side padding and gap 0.5rem or 0.375rem, the clear button a Button
        // icon of 24.5px (md) or 21px (sm).
        let height = if md { rem(2.5) } else { rem(1.75) };
        let field_px = if md { rem(1.0) } else { rem(0.857) };
        let icon_px = if md { 18.0 } else { 14.0 };
        let pad = if md { rem(0.5) } else { rem(0.375) };
        let gap = pad;
        let btn = if md { 24.5 } else { 21.0 };
        let btn_glyph = if md { 16.0 } else { 14.0 };
        let btn_radius: u8 = if md { 5 } else { 4 };
        let well_radius = CornerRadius::same((height / 2.0) as u8);
        let ring_radius = CornerRadius::same((height / 2.0 + 2.0) as u8);

        let placeholder = self.p.placeholder.clone();
        let label = self.p.ariaLabel.clone();
        let field_id = ui.scope_id().with("searchfield.field");

        // Keys are read before the editor sees them. Match navigation takes Enter, Shift+Enter and Escape
        // (Escape always blurs). Filter takes Escape only to clear a clearable, non-empty query; an empty
        // one is left alone, so it bubbles as on the web.
        let focused_before = self.was_focused || ui.memory(|m| m.has_focus(field_id));
        if focused_before && enabled {
            if matching {
                let shift = ui.input_mut(|i| i.consume_key(Modifiers::SHIFT, Key::Enter));
                let enter = ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Enter));
                if shift || enter {
                    let direction = if shift { "previous" } else { "next" };
                    events.push(SearchfieldEvent::StepMatch { direction: direction.to_owned() });
                }
                if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
                    if !self.value.is_empty() {
                        self.value.clear();
                        events.push(SearchfieldEvent::Changed { value: String::new() });
                        events.push(SearchfieldEvent::Cleared {});
                    }
                    ui.memory_mut(|m| m.surrender_focus(field_id));
                }
            } else if clearable && !self.value.is_empty() && ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
                self.value.clear();
                events.push(SearchfieldEvent::Changed { value: String::new() });
                events.push(SearchfieldEvent::Cleared {});
                // The query is cleared and the field keeps focus, as on the web.
                ui.memory_mut(|m| m.request_focus(field_id));
            }
        }

        // Typed text that arrives while nothing has focus goes to the field, so that the field already has
        // focus when the text is read (as InputGroup does). Nothing is focused at rest, so no ring at mount.
        let typed = ui.input(|i| i.events.iter().any(|e| matches!(e, egui::Event::Text(_))));
        if typed && enabled && ui.memory(|m| m.focused().is_none()) {
            ui.memory_mut(|m| m.request_focus(field_id));
        }

        let showing_clear = clearable && !self.value.is_empty();

        // The well spans the pane's width; its slots sit inside it.
        let avail = ui.available_width();
        let full_w = if avail.is_finite() { avail } else { rem(20.0) };
        let (outer, well) = ui.allocate_exact_size(vec2(full_w, height), Sense::hover());
        well.widget_info(|| WidgetInfo::new(Role::GenericContainer));
        part(ui, &well, parts::WELL);

        let icon_rect = Rect::from_min_size(pos2(outer.left() + pad, outer.center().y - icon_px / 2.0), vec2(icon_px, icon_px));
        let clear_rect = Rect::from_min_size(pos2(outer.right() - pad - btn, outer.center().y - btn / 2.0), vec2(btn, btn));
        let right_edge = if showing_clear { clear_rect.left() - gap } else { outer.right() - pad };
        let field_left = icon_rect.right() + gap;
        let field_rect = Rect::from_min_max(pos2(field_left, outer.top()), pos2(right_edge.max(field_left), outer.bottom()));

        // A press in the well, other than on the clear button, focuses the field.
        if enabled && ui.input(|i| i.pointer.primary_pressed()) {
            if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                if outer.contains(pos) && !(showing_clear && clear_rect.contains(pos)) {
                    ui.memory_mut(|m| m.request_focus(field_id));
                }
            }
        }

        let painter: Painter = ui.painter().clone();
        painter.rect_filled(outer, well_radius, fade(t.wash_2));

        // The leading magnifier: decorative (no name), but a node so the contract can see it.
        let icon_resp = ui.interact(icon_rect, well.id.with("icon"), Sense::hover());
        icon_resp.widget_info(|| WidgetInfo::new(Role::GenericContainer));
        part(ui, &icon_resp, parts::ICON);
        draw_glyph(&painter, "search", icon_rect.center(), icon_px, fade(t.content_3));

        // The native-style field: a plain text input, vertically centred in its slot.
        let mut child = ui.new_child(UiBuilder::new().max_rect(field_rect).layout(Layout::left_to_right(Align::Center)));
        child.visuals_mut().selection.bg_fill = t.selection_1;
        let field_resp = TextEdit::singleline(&mut self.value)
            .id(field_id)
            .frame(egui::Frame::NONE)
            .margin(Margin::ZERO)
            .font(FontId::new(field_px, weight(400)))
            .text_color(fade(t.content_1))
            .hint_text(text(placeholder, field_px, 400).color(fade(t.content_3)))
            .desired_width(field_rect.width())
            .interactive(enabled)
            .show(&mut child)
            .response;

        if self.want_focus {
            field_resp.request_focus();
            self.want_focus = false;
        }
        if field_resp.changed() {
            events.push(SearchfieldEvent::Changed { value: self.value.clone() });
        }

        // The field's name is the aria label the host passes; its value is the query.
        let value = self.value.clone();
        field_resp.widget_info(|| WidgetInfo::labeled(Role::TextInput, enabled, label.as_str()));
        ui.ctx().accesskit_node_builder(field_resp.id, |node| node.set_value(value));
        part(ui, &field_resp, parts::FIELD);

        // The clear button: a muted icon button, present only while the clearable query is non-empty.
        let mut clear_focus = false;
        if showing_clear {
            let sense = if enabled { Sense::click() } else { Sense::hover() };
            let clear = ui.interact(clear_rect, well.id.with("clear"), sense);
            clear.widget_info(|| WidgetInfo::labeled(Role::Button, enabled, "Clear search"));
            part(ui, &clear, parts::CLEARBUTTON);
            clear_focus = clear.has_focus();
            let hot = enabled && clear.hovered();
            if hot {
                painter.rect_filled(clear_rect, CornerRadius::same(btn_radius), fade(t.wash_1));
            }
            let glyph_col = fade(if hot { t.content_2 } else { t.content_3 });
            draw_glyph(&painter, "close", clear_rect.center(), btn_glyph, glyph_col);

            // Enter and Space on the focused button clear the query, as a press does.
            let keyed = clear.has_focus() && ui.input(|i| i.key_pressed(Key::Enter) || i.key_pressed(Key::Space));
            if enabled && (clear.clicked() || keyed) {
                self.value.clear();
                events.push(SearchfieldEvent::Changed { value: String::new() });
                events.push(SearchfieldEvent::Cleared {});
                // The web hands focus back to the field after the clear button unmounts.
                ui.memory_mut(|m| m.request_focus(field_id));
            }
        }

        // The focus ring sits round the well while the field or the clear button has keyboard focus.
        if enabled && (field_resp.has_focus() || clear_focus) {
            painter.rect_stroke(outer.expand(2.0), ring_radius, Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
        }
        self.was_focused = field_resp.has_focus();

        events
    }
}

/// Draws a Material Symbols glyph centred on `center`.
fn draw_glyph(painter: &Painter, name: &str, center: Pos2, px: f32, color: Color32) {
    painter.text(center, Align2::CENTER_CENTER, icon(name, px).text(), FontId::new(px, FontFamily::Name("Icons".into())), color);
}

crate::mounted!(SearchField, SearchfieldProps);
