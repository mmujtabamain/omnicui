//! contract: omnicui/contracts/commandinput.contract.yaml (id: ui.commandinput). The input row at the top of the command palette: a bolt glyph, the query input, and a 1px bottom rule that turns to the focus colour.
use crate::omnicui::commandinput::{parts, CommandinputEvent, CommandinputProps};
use crate::{icon, part, rem, text, weight, Look};
use egui::{Align, Align2, CornerRadius, FontFamily, FontId, Key, Layout, Modifiers, Pos2, Role, Sense, Stroke, TextEdit, UiBuilder, Vec2, WidgetInfo};

/// One input row. The row draws the chrome; the inner text input holds the query, which the host filters.
pub struct CommandInput {
    p: CommandinputProps,
    value: String,
    focused_once: bool,
}

/// How many presses of `key` are waiting this frame. Each press is taken, so the input never sees it.
fn take_presses(ui: &egui::Ui, key: Key) -> usize {
    let mut n = 0;
    ui.ctx().input_mut(|i| {
        while i.consume_key(Modifiers::NONE, key) {
            n += 1;
        }
    });
    n
}

impl CommandInput {
    pub fn new(p: CommandinputProps) -> Self {
        let value = p.value.clone();
        Self { p, value, focused_once: false }
    }

    /// One frame: draws the row and returns the events it raised in this frame.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<CommandinputEvent> {
        let t = look.t;
        let mut events = vec![];
        let height = rem(3.143);
        let text_px = rem(1.0);
        let pad = 8.0;
        let gap = 8.0;
        let icon_px = 16.0;
        let font = FontId::new(text_px, weight(400));
        let leading_on = self.p.leading == "icon";

        let full_w = ui.available_width();
        let (outer, row) = ui.allocate_exact_size(Vec2::new(full_w, height), Sense::hover());
        row.widget_info(|| WidgetInfo::new(Role::GenericContainer));
        part(ui, &row, parts::ROW);
        let input_id = row.id.with("input");

        // Takes focus once, when the row first opens (autoFocus).
        if self.p.autoFocus && !self.focused_once {
            self.focused_once = true;
            ui.ctx().memory_mut(|m| m.request_focus(input_id));
        }

        // The arrows and Enter are the host's. They are taken before the input is drawn, so that Enter
        // does not drop focus and the arrows do not move the caret.
        let focused = ui.ctx().memory(|m| m.has_focus(input_id));
        // The characters typed this frame, read before the input takes them: each is one user edit.
        let typed: Vec<char> = if focused {
            ui.input(|i| {
                i.events
                    .iter()
                    .filter_map(|e| match e {
                        egui::Event::Text(s) => Some(s.chars().collect::<Vec<char>>()),
                        _ => None,
                    })
                    .flatten()
                    .collect()
            })
        } else {
            Vec::new()
        };
        let before = self.value.clone();
        if focused {
            for _ in 0..take_presses(ui, Key::ArrowDown) {
                events.push(CommandinputEvent::MoveActive { direction: "down".to_owned() });
            }
            for _ in 0..take_presses(ui, Key::ArrowUp) {
                events.push(CommandinputEvent::MoveActive { direction: "up".to_owned() });
            }
            for _ in 0..take_presses(ui, Key::Enter) {
                events.push(CommandinputEvent::Submit { value: self.value.clone() });
            }
        }

        let painter = ui.painter().clone();
        painter.rect_filled(outer, CornerRadius::same(0), t.surface_1);

        let inner = outer.shrink2(Vec2::new(pad, 0.0));
        let mut value = std::mem::take(&mut self.value);
        let placeholder = self.p.placeholder.clone();
        let label = self.p.ariaLabel.clone();
        let input = ui
            .scope_builder(UiBuilder::new().max_rect(inner).layout(Layout::left_to_right(Align::Center)), |ui| {
                ui.spacing_mut().item_spacing = Vec2::new(gap, 0.0);
                if leading_on {
                    let (r, resp) = ui.allocate_exact_size(Vec2::new(icon_px, height), Sense::hover());
                    resp.widget_info(|| WidgetInfo::new(Role::GenericContainer));
                    part(ui, &resp, parts::LEADING);
                    ui.painter().text(
                        r.center(),
                        Align2::CENTER_CENTER,
                        icon("bolt", icon_px).text(),
                        FontId::new(icon_px, FontFamily::Name("Icons".into())),
                        t.content_3,
                    );
                }
                let width = ui.available_width();
                ui.add(
                    TextEdit::singleline(&mut value)
                        .id(input_id)
                        .frame(egui::Frame::NONE)
                        .desired_width(width)
                        .font(font.clone())
                        .text_color(t.content_1)
                        .hint_text(text(placeholder.as_str(), text_px, 400).color(t.content_3)),
                )
            })
            .inner;
        self.value = value;

        if input.changed() {
            // Several characters in one frame are several keystrokes: each reports its own query.
            if typed.len() > 1 && self.value.starts_with(&before) {
                let mut acc = before.clone();
                for c in &typed[..typed.len() - 1] {
                    acc.push(*c);
                    events.push(CommandinputEvent::QueryChanged { value: acc.clone() });
                }
            }
            events.push(CommandinputEvent::QueryChanged { value: self.value.clone() });
        }
        part(ui, &input, parts::INPUT);
        // The input's name is the host's aria label; the text edit only reports its hint.
        ui.ctx().accesskit_node_builder(input.id, |node| node.set_label(label.as_str()));

        // The 1px bottom rule carries the focus: it turns to the focus colour while the input has focus.
        let rule = if input.has_focus() { t.focus_1 } else { t.edge_1 };
        let y = outer.bottom() - 0.5;
        painter.line_segment([Pos2::new(outer.left(), y), Pos2::new(outer.right(), y)], Stroke::new(1.0, rule));

        events
    }
}

crate::mounted!(CommandInput, CommandinputProps);
