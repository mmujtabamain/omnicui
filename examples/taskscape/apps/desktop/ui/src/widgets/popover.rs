//! contract: omnicui/contracts/popover.contract.yaml (id: ui.popover). The @-mention list: an editor whose @ query shows the matching names in a floating list at the caret, with the highlight kept in the editor.
use crate::omnicui::popover::{parts, PopoverEvent, PopoverProps};
use crate::{part, rem, weight, Look, Mounted, Tokens};
use egui::{Align2, Area, CornerRadius, FontId, Frame, Key, Margin, Modifiers, Order, Pos2, Rect, RichText, Role, ScrollArea, Sense, Stroke, StrokeKind, TextEdit, WidgetInfo};

/// The names the fixture can mention; the @ query shows the ones that contain it.
const ROSTER: &[&str] = &["alan", "alice", "albert", "alexandra", "amara", "alina", "bartholomew", "gabriel"];

/// The part ids of the rows, row1 to row6. A row past the sixth has no part.
const ROW_PARTS: [&str; 6] = [parts::ROW1, parts::ROW2, parts::ROW3, parts::ROW4, parts::ROW5, parts::ROW6];

/// The id of the @ field. Fixed, so the mount can focus it before the widget draws.
#[allow(deprecated)]
fn editor_key() -> egui::Id {
    egui::Id::new("ui.popover.editor")
}

/// The @ query at the end of the text: where its @ is, and the query in lower case.
fn mention_query(text: &str) -> Option<(usize, String)> {
    let at = text.rfind('@')?;
    let query = &text[at + 1..];
    query
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
        .then(|| (at, query.to_lowercase()))
}

/// The popover: the @ field and the list of matches under it.
pub struct Popover {
    p: PopoverProps,
    text: String,
    /// The highlighted row.
    active: usize,
    /// Escape or an outside press closed the list; typing opens it again.
    dismissed: bool,
    /// A key moved the highlight, so the list scrolls to it this frame.
    scroll: bool,
    /// Where the list was last frame, to tell an outside press.
    list_rect: Option<Rect>,
    /// The field had focus at the end of the last frame.
    was_focused: bool,
}

impl Popover {
    pub fn new(p: PopoverProps) -> Self {
        Self { p, text: String::new(), active: 0, dismissed: false, scroll: false, list_rect: None, was_focused: false }
    }

    /// The rows for the current text: the names that contain the query, capped at maxItems.
    fn rows(&self, roster: &[&'static str]) -> Vec<&'static str> {
        let Some((_, query)) = mention_query(&self.text) else {
            return Vec::new();
        };
        let cap = self.p.maxItems.max(0) as usize;
        roster.iter().copied().filter(|name| name.contains(query.as_str())).take(cap).collect()
    }

    /// Picks a row: the @ query becomes the name and a space.
    fn pick(&mut self, value: &str) -> PopoverEvent {
        if let Some((at, _)) = mention_query(&self.text) {
            self.text = format!("{}@{} ", &self.text[..at], value);
        }
        self.active = 0;
        PopoverEvent::Picked { value: value.to_owned() }
    }

    /// One row of the list. Returns true when it was clicked.
    fn row(&mut self, ui: &mut egui::Ui, t: Tokens, i: usize, active: bool, name: &str, scroll: bool) -> bool {
        let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), rem(1.75)), Sense::click());
        // Moving the pointer over a row highlights it (the web's mouseenter); a resting pointer does not.
        if resp.hovered() && ui.input(|inp| inp.pointer.delta() != egui::Vec2::ZERO) {
            self.active = i;
        }
        if active {
            ui.painter().rect_filled(rect, CornerRadius::same(4), t.accent_500);
        }
        let color = if active { t.on_accent } else { t.content_1 };
        ui.painter().text(
            rect.left_center() + egui::vec2(8.0, 0.0),
            Align2::LEFT_CENTER,
            name,
            FontId::new(13.0, weight(400)),
            color,
        );
        resp.widget_info(|| WidgetInfo::labeled(Role::Button, true, name));
        if let Some(&id) = ROW_PARTS.get(i) {
            part(ui, &resp, id);
        }
        if scroll && active {
            resp.scroll_to_me(None);
        }
        resp.clicked()
    }

    /// One frame: the editor, and the list of matches under it while the @ query has any.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look, roster: &[&'static str]) -> Vec<PopoverEvent> {
        let t = look.t;
        let mut events = Vec::new();
        let editor_id = editor_key();

        // Keys act on the rows as they stood at the start of the frame. Arrows and Tab act only while
        // the list is open. Enter and Escape are always taken here, so they never reach the field and
        // focus stays in it. Escape can take focus away before this frame, so a press right after the
        // field had focus still counts.
        let before = self.rows(roster);
        let open_before = !before.is_empty() && !self.dismissed;
        let last = before.len().saturating_sub(1);
        let hl = self.active.min(last);
        let escape_pressed = ui.input(|i| i.key_pressed(Key::Escape));
        let focused = ui.memory(|m| m.has_focus(editor_id)) || (self.was_focused && escape_pressed);
        let mut refocus = false;
        if focused {
            let (down, up, tab, enter, escape) = ui.input_mut(|i| {
                (
                    open_before && i.consume_key(Modifiers::NONE, Key::ArrowDown),
                    open_before && i.consume_key(Modifiers::NONE, Key::ArrowUp),
                    open_before && i.consume_key(Modifiers::NONE, Key::Tab),
                    i.consume_key(Modifiers::NONE, Key::Enter),
                    i.consume_key(Modifiers::NONE, Key::Escape),
                )
            });
            refocus = down || up || tab || enter || escape;
            if down {
                self.active = if hl >= last { 0 } else { hl + 1 };
                self.scroll = true;
            }
            if up {
                self.active = if hl == 0 { last } else { hl - 1 };
                self.scroll = true;
            }
            if open_before && (enter || tab) {
                events.push(self.pick(before[hl]));
            }
            if open_before && escape {
                self.dismissed = true;
                events.push(PopoverEvent::Escaped {});
            }
        }

        if refocus {
            ui.memory_mut(|m| m.request_focus(editor_id));
        }

        // The field: 210 px wide, 24.5 px high, its text inset 8 px.
        let (rect, _) = ui.allocate_exact_size(egui::vec2(rem(15.0), rem(1.75)), Sense::hover());
        ui.painter().rect(rect, CornerRadius::same(4), t.surface_3, Stroke::new(1.0, t.edge_2), StrokeKind::Inside);
        let inner = rect.shrink2(egui::vec2(8.0, 3.5));
        let edit = ui.put(
            inner,
            TextEdit::singleline(&mut self.text)
                .id(editor_id)
                .lock_focus(true)
                .frame(Frame::new())
                .font(FontId::new(13.0, weight(400)))
                .text_color(t.content_1)
                .hint_text(RichText::new("Type @ to mention").color(t.content_3))
                .desired_width(inner.width()),
        );
        part(ui, &edit, parts::EDITOR);
        if edit.changed() {
            self.active = 0;
            self.dismissed = false;
        }
        if edit.has_focus() {
            ui.painter().rect_stroke(rect.expand(2.0), CornerRadius::same(6), Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
        }
        self.was_focused = edit.has_focus();

        // An outside press dismisses the list, as Radix's dismissal does on the web.
        let press = ui.input(|i| if i.pointer.any_pressed() { i.pointer.interact_pos() } else { None });
        if press.is_some_and(|pos| !rect.contains(pos) && !self.list_rect.is_some_and(|r| r.contains(pos))) {
            self.dismissed = true;
        }

        let rows = self.rows(roster);
        if rows.is_empty() || self.dismissed {
            self.list_rect = None;
            return events;
        }
        let hl = self.active.min(rows.len() - 1);
        let scroll = std::mem::take(&mut self.scroll);
        // The caret: the anchor point when one is given, else the field's bottom-left. The list sits 4 px below it.
        let caret = if self.p.anchorX != 0.0 || self.p.anchorY != 0.0 {
            Pos2::new(self.p.anchorX as f32, self.p.anchorY as f32)
        } else {
            rect.left_bottom()
        };
        let list_id = ui.scope_id().with("list");
        let mut clicked = None;
        let area = Area::new(list_id)
            .order(Order::Foreground)
            .fixed_pos(caret + egui::vec2(0.0, 4.0))
            .show(ui.ctx(), |ui| {
                let frame = Frame::new()
                    .fill(t.surface_3)
                    .stroke(Stroke::new(1.0, t.edge_2))
                    .corner_radius(CornerRadius::same(11))
                    .inner_margin(Margin::same(3))
                    .show(ui, |ui| {
                        ui.set_width(rem(15.0) - 8.0);
                        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
                        ScrollArea::vertical().max_height(rem(14.0)).auto_shrink([false, true]).show(ui, |ui| {
                            for (i, name) in rows.iter().enumerate() {
                                if self.row(ui, t, i, i == hl, name, scroll) {
                                    clicked = Some(i);
                                }
                            }
                        });
                    });
                frame.response.widget_info(|| WidgetInfo::new(Role::GenericContainer));
                part(ui, &frame.response, parts::LIST);
                frame.response.rect
            });
        self.list_rect = Some(area.inner);

        if let Some(i) = clicked {
            events.push(self.pick(rows[i]));
            ui.ctx().memory_mut(|m| m.request_focus(editor_id));
        }
        events
    }
}

/// The harness's page around the widget (16 px of padding), with the names the fixture can mention.
/// The field takes focus on the mount's first frame, so typing reaches it without a click.
pub struct PopoverMount {
    pop: Popover,
    focused: bool,
}

impl Mounted for PopoverMount {
    fn frame(&mut self, ui: &mut egui::Ui, look: Look, emit: &mut dyn FnMut(&'static str, serde_json::Value)) {
        let mut events = Vec::new();
        if !self.focused {
            self.focused = true;
            ui.ctx().memory_mut(|m| m.request_focus(editor_key()));
        }
        ui.add_space(16.0);
        ui.horizontal(|ui| {
            ui.add_space(16.0);
            events = self.pop.ui(ui, look, ROSTER);
        });
        for e in events {
            let (name, fields) = e.to_json();
            emit(name, fields);
        }
    }
}

/// Builds the widget from a test's props.
pub fn mount(props: serde_json::Value) -> Result<Box<dyn Mounted>, String> {
    let p: PopoverProps = serde_json::from_value(props).map_err(|e| format!("props: {e}"))?;
    Ok(Box::new(PopoverMount { pop: Popover::new(p), focused: false }))
}
