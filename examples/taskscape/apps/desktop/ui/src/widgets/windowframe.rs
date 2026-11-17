//! contract: omnicui/contracts/windowframe.contract.yaml (id: ui.windowframe). The root of one window: titled is the app window with a title bar above its body, frameless the desktop capture card with a title field and a notes field.
use crate::omnicui::windowframe::{parts, WindowframeEvent, WindowframeProps};
use crate::{part, rem, text, weight, Look};
use egui::{pos2, vec2, CornerRadius, FontId, Key, Modifiers, Rect, Role, Sense, Stroke, StrokeKind, TextEdit, UiBuilder, WidgetInfo};

/// A part with no widget of its own: a node over `rect` that takes no input, tagged with its contract id.
fn tag(ui: &egui::Ui, rect: Rect, id: &str) {
    let response = ui.interact(rect, ui.scope_id().with(id), Sense::hover());
    response.widget_info(|| WidgetInfo::new(Role::GenericContainer));
    part(ui, &response, id);
}

/// The 2 px focus ring, drawn inside the field as the web's `ring-inset` does.
fn ring(ui: &egui::Ui, rect: Rect, color: egui::Color32) {
    ui.painter().rect_stroke(rect, CornerRadius::same(0), Stroke::new(2.0, color), StrokeKind::Inside);
}

pub struct WindowFrame {
    p: WindowframeProps,
    title: String,
    notes: String,
}

impl WindowFrame {
    pub fn new(p: WindowframeProps) -> Self {
        Self { p, title: String::new(), notes: String::new() }
    }

    /// One frame with no content of the app's own around its parts.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<WindowframeEvent> {
        self.ui_with(ui, look, |_: &mut egui::Ui| {}, |_: &mut egui::Ui| {})
    }

    /// One frame. A titled frame draws `title_content` in its title bar and `body_content` in its body;
    /// a frameless card draws its own two fields and ignores both.
    pub fn ui_with(
        &mut self,
        ui: &mut egui::Ui,
        look: Look,
        title_content: impl FnOnce(&mut egui::Ui),
        body_content: impl FnOnce(&mut egui::Ui),
    ) -> Vec<WindowframeEvent> {
        let mut events = vec![];
        // The frame fills its window, not the panel's inset, so it and its clip take the whole content area.
        let full = ui.ctx().content_rect();
        ui.set_clip_rect(full);
        if self.p.chrome == "frameless" {
            self.capture_bar(ui, look, full, &mut events);
        } else {
            let t = look.t;
            ui.painter().rect_filled(full, CornerRadius::same(0), t.surface_1);
            let title = Rect::from_min_size(full.min, vec2(full.width(), rem(2.5)));
            let body = Rect::from_min_max(pos2(full.left(), title.bottom()), full.max);
            tag(ui, full, parts::FRAME);
            tag(ui, full, parts::OVERLAYS);
            tag(ui, title, parts::TITLEBAR);
            tag(ui, body, parts::BODY);
            // The title bar's bottom border sits inside its height.
            ui.painter().hline(title.x_range(), title.bottom() - 0.5, Stroke::new(1.0, t.edge_1));
            let title_inner = Rect::from_min_max(title.min, pos2(title.right(), title.bottom() - 1.0));
            ui.scope_builder(UiBuilder::new().max_rect(title_inner), |ui| title_content(ui));
            ui.scope_builder(UiBuilder::new().max_rect(body), |ui| body_content(ui));
        }
        events
    }

    /// The frameless capture card: a rounded, bordered light panel with a title field over a notes field.
    /// Escape dismisses from either field, Enter in the title and Mod+Enter in either submit, and Tab
    /// moves between the two fields (wrapping).
    fn capture_bar(&mut self, ui: &mut egui::Ui, look: Look, full: Rect, events: &mut Vec<WindowframeEvent>) {
        let t = look.t;
        let title_id = ui.scope_id().with(parts::CAPTURETITLE);
        let notes_id = ui.scope_id().with(parts::CAPTURENOTES);
        let (title_focus, _notes_focus) = ui.memory(|m| (m.has_focus(title_id), m.has_focus(notes_id)));

        // The keys are taken before the fields see them. Escape and Mod+Enter act on the whole bar (the
        // window is the bar); plain Enter is a newline in the notes, so only the title takes it.
        let cmd_enter = ui.input_mut(|i| {
            let mut found = false;
            i.events.retain(|e| {
                let hit = matches!(
                    e,
                    egui::Event::Key { key: Key::Enter, pressed: true, modifiers, .. }
                        if modifiers.command || modifiers.ctrl || modifiers.mac_cmd
                );
                found |= hit;
                !hit
            });
            found
        });
        let (esc, enter, tab) = ui.input_mut(|i| {
            let esc = i.consume_key(Modifiers::NONE, Key::Escape);
            let enter = title_focus && i.consume_key(Modifiers::NONE, Key::Enter);
            let tab = i.consume_key(Modifiers::NONE, Key::Tab) | i.consume_key(Modifiers::SHIFT, Key::Tab);
            (esc, enter, tab)
        });
        if esc {
            events.push(WindowframeEvent::Dismissed { reason: "escape".to_owned() });
        }
        if cmd_enter {
            events.push(WindowframeEvent::Submitted { via: "shortcut".to_owned() });
        } else if enter {
            events.push(WindowframeEvent::Submitted { via: "enter".to_owned() });
        }
        if tab {
            let next = if title_focus { notes_id } else { title_id };
            ui.memory_mut(|m| m.request_focus(next));
        }

        let title_rect = Rect::from_min_size(full.min, vec2(full.width(), rem(2.25)));
        let line_y = title_rect.bottom();
        let notes_rect = Rect::from_min_max(pos2(full.left(), line_y + 1.0), full.max);

        ui.painter().rect(full, CornerRadius::same(10), t.surface_2, Stroke::new(1.0, t.edge_2), StrokeKind::Inside);
        tag(ui, full, parts::FRAME);

        let title = ui.put(
            title_rect,
            TextEdit::singleline(&mut self.title)
                .id(title_id)
                .frame(egui::Frame::NONE.inner_margin(egui::Margin { left: 12, right: 12, top: 6, bottom: 6 }))
                .font(FontId::new(rem(1.143), weight(500)))
                .text_color(t.content_1)
                .hint_text(text("What needs doing?", rem(1.143), 500).color(t.content_3))
                .lock_focus(true)
                .desired_width(full.width())
                .min_size(title_rect.size()),
        );
        part(ui, &title, parts::CAPTURETITLE);
        if title.has_focus() {
            ring(ui, title.rect, t.focus_1);
        }

        // The notes' top border, then the notes field filling the rest of the card.
        ui.painter().hline(full.x_range(), line_y + 0.5, Stroke::new(1.0, t.edge_1));
        let notes = ui.put(
            notes_rect,
            TextEdit::multiline(&mut self.notes)
                .id(notes_id)
                .frame(egui::Frame::NONE.inner_margin(egui::Margin { left: 12, right: 12, top: 8, bottom: 8 }))
                .font(FontId::new(rem(0.929), weight(400)))
                .text_color(t.content_2)
                .hint_text(text("Notes", rem(0.929), 400).color(t.content_3))
                .lock_focus(true)
                .desired_width(notes_rect.width())
                .min_size(notes_rect.size()),
        );
        part(ui, &notes, parts::CAPTURENOTES);
        if notes.has_focus() {
            ring(ui, notes.rect, t.focus_1);
        }
    }
}

/// The titled title bar's fixture: the app's name, as the web harness shows it.
fn title_fixture(ui: &mut egui::Ui, look: Look) {
    ui.horizontal_centered(|ui| {
        ui.add_space(12.0);
        ui.label(text("Taskscape", rem(0.929), 500).color(look.t.content_1));
    });
}

/// The titled body's fixture: a sidebar of projects beside a pane, as the web harness shows them.
fn body_fixture(ui: &mut egui::Ui, look: Look) {
    let t = look.t;
    let body = ui.max_rect();
    let side = Rect::from_min_size(body.min, vec2(rem(11.0), body.height()));
    let pane = Rect::from_min_max(pos2(side.right(), body.top()), body.max);
    ui.painter().rect_filled(side, CornerRadius::same(0), t.surface_0);
    ui.painter().vline(side.right() - 0.5, side.y_range(), Stroke::new(1.0, t.edge_1));
    ui.scope_builder(UiBuilder::new().max_rect(side.shrink(8.0)), |ui| {
        ui.label(text("Projects", rem(0.929), 400).color(t.content_2));
    });
    ui.scope_builder(UiBuilder::new().max_rect(pane.shrink(12.0)), |ui| {
        ui.label(text("Pane content", rem(0.929), 400).color(t.content_1));
    });
}

/// The window as the desktop mount draws it: the web harness's fixtures around the frame.
pub struct WindowFrameScreen {
    widget: WindowFrame,
}

impl crate::Mounted for WindowFrameScreen {
    fn frame(&mut self, ui: &mut egui::Ui, look: Look, emit: &mut dyn FnMut(&'static str, serde_json::Value)) {
        let events = self.widget.ui_with(
            ui,
            look,
            |ui: &mut egui::Ui| title_fixture(ui, look),
            |ui: &mut egui::Ui| body_fixture(ui, look),
        );
        for e in events {
            let (name, fields) = e.to_json();
            emit(name, fields);
        }
    }
}

/// Builds the window from a test's props (the same shape the `mounted!` macro gives a widget).
pub fn mount(props: serde_json::Value) -> Result<Box<dyn crate::Mounted>, String> {
    let p: WindowframeProps = serde_json::from_value(props).map_err(|e| format!("props: {e}"))?;
    Ok(Box::new(WindowFrameScreen { widget: WindowFrame::new(p) }))
}
