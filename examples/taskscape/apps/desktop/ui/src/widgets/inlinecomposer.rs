//! contract: omnicui/contracts/inlinecomposer.contract.yaml (id: ui.inlinecomposer). A single-line title field inserted under a parent task row to add a subtask: Enter with a title submits it and clears the field, Enter on a blank field, Escape, or leaving the field dismisses it.
use crate::omnicui::inlinecomposer::{parts, InlinecomposerEvent, InlinecomposerProps};
use crate::omnicui::input::{InputEvent, InputProps};
use crate::widgets::input::Input;
use crate::{icon, part, rem, Look};
use egui::{pos2, vec2, Align, Align2, FontFamily, FontId, Id, Key, Layout, Rect, Role, Sense, UiBuilder, WidgetInfo};

/// The row's height: the web's h-[2.571rem].
const ROW_H: f32 = rem(2.571);
/// The leading icon's size in px.
const ICON_PX: f32 = 15.0;
/// The gap between the icon and the field (the web's gap-1.5), in px.
const GAP: f32 = 6.0;
/// The row's right padding (the web's pr-2), in px.
const PAD_RIGHT: f32 = 8.0;

pub struct InlineComposer {
    p: InlinecomposerProps,
    /// The title field: the input widget, bare and medium, labelled "Subtask title".
    field: Input,
    /// A dismissal ends a session: blur, Escape, Tab and an empty Enter report once per session.
    /// Typing or focusing the field starts a new session.
    dismissed: bool,
    /// The field had keyboard focus at the end of the last frame.
    was_focused: bool,
}

/// The props of the title field: a bare, medium input that stops key propagation.
fn field_props(value: String, auto_focus: bool) -> InputProps {
    InputProps {
        value,
        mode: "bare".to_owned(),
        size: "md".to_owned(),
        placeholder: "Subtask title".to_owned(),
        ariaLabel: "Subtask title".to_owned(),
        autoFocus: auto_focus,
        stopKeyPropagation: true,
        ..Default::default()
    }
}

/// The id egui gives the field's text editor (the input widget draws its editor under this id).
#[allow(deprecated)]
fn field_id() -> Id {
    Id::unique("ui.input.field")
}

impl InlineComposer {
    pub fn new(p: InlinecomposerProps) -> Self {
        let field = Input::new(field_props(String::new(), p.autoFocus));
        Self { p, field, dismissed: false, was_focused: false }
    }

    fn dismiss(&mut self, events: &mut Vec<InlinecomposerEvent>) {
        if !self.dismissed {
            self.dismissed = true;
            events.push(InlinecomposerEvent::Dismissed {});
        }
    }

    /// One frame: draws the composer and returns the events it raised in this frame.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<InlinecomposerEvent> {
        let mut events = vec![];
        let t = look.t;

        // The row: 36px high, its left inset depth * 24 + 6 px, the field filling the rest.
        let width = ui.available_width();
        let width = if width.is_finite() { width } else { rem(20.0) };
        let (row, _) = ui.allocate_exact_size(vec2(width, ROW_H), Sense::hover());
        let left = (row.left() + self.p.depth as f32 * 24.0 + 6.0).min(row.right());
        let inner = Rect::from_min_max(
            pos2(left, row.top()),
            pos2((row.right() - PAD_RIGHT).max(left), row.bottom()),
        );
        let mut line = ui.new_child(UiBuilder::new().max_rect(inner).layout(Layout::left_to_right(Align::Center)));
        line.spacing_mut().item_spacing = vec2(0.0, 0.0);

        // The leading icon: decorative, with a node under the icon part's id.
        let (icon_rect, icon_resp) = line.allocate_exact_size(vec2(ICON_PX, ICON_PX), Sense::hover());
        let glyph = icon("subdirectory_arrow_right", ICON_PX);
        line.painter().text(
            icon_rect.center(),
            Align2::CENTER_CENTER,
            glyph.text(),
            FontId::new(ICON_PX, FontFamily::Name("Icons".into())),
            t.content_3,
        );
        icon_resp.widget_info(|| WidgetInfo::new(Role::GenericContainer));
        part(&line, &icon_resp, parts::ICON);
        line.add_space(GAP);

        // The field: the input widget draws the text editor. Its node is the composer's field part.
        let fid = field_id();
        let input_events = self.field.ui(&mut line, look);
        line.ctx().accesskit_node_builder(fid, |node| node.set_author_id(parts::FIELD.to_owned()));

        let mut submitted: Option<String> = None;
        let mut cancelled = false;
        for e in input_events {
            match e {
                InputEvent::Submit { value } => submitted = Some(value),
                InputEvent::Cancel { .. } => cancelled = true,
                InputEvent::ValueChange { .. } => self.dismissed = false,
                _ => {}
            }
        }

        // Focus: gaining it starts a session; leaving it dismisses. Tab leaves the field, so it dismisses too.
        let focused = ui.memory(|m| m.has_focus(fid));
        if focused && !self.was_focused {
            self.dismissed = false;
        }
        if cancelled {
            self.dismiss(&mut events);
        }
        if let Some(title) = submitted {
            if title.is_empty() {
                self.dismiss(&mut events);
            } else {
                events.push(InlinecomposerEvent::Submitted { title });
                // The field clears and keeps focus, so the next subtask can follow at once.
                self.field = Input::new(field_props(String::new(), false));
            }
        }
        if focused && ui.input(|i| i.key_pressed(Key::Tab)) {
            self.dismiss(&mut events);
        }
        if self.was_focused && !focused {
            self.dismiss(&mut events);
        }
        self.was_focused = focused;

        events
    }
}

crate::mounted!(InlineComposer, InlinecomposerProps);
