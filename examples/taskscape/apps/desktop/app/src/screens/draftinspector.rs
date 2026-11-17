//! contract: omnicui/contracts/draftinspector.contract.yaml (id: app.draftinspector). The new-task draft in the preview panel: the title field, the breadcrumb (project and list), and the Start with block (Note, Image and a hint).
use crate::state::Aside;
use crate::App;
use egui::text::{LayoutJob, TextFormat};
use egui::{
    pos2, vec2, Align, Align2, Color32, CornerRadius, FontFamily, FontId, Galley, Id, Key, Layout, Margin, Modifiers, Pos2, Rect, Response, Role, RichText, ScrollArea, Sense, Shape, Stroke,
    StrokeKind, TextEdit, UiBuilder, WidgetInfo,
};
use std::sync::Arc;
use taskscape_ui::omnicui::draftinspector::{parts, DraftinspectorEvent, DraftinspectorProps};
use taskscape_ui::{icon, part, weight, Look, Mounted, Tokens};

/// The title field's text inset (the web's px-space-4).
const PAD: f32 = 5.25;
/// The title's line height, and the box it grows to (two lines) before it scrolls: the Input widget's sizes.
const LINE: f32 = 21.0;
/// The discard button's box (the web's icon button, size default).
const DISCARD: f32 = 21.0;
/// The dashed Note and Screenshot buttons' height (the web's dashedShort).
const CHIP_H: f32 = 35.0;
/// The gap between the title and the discard button (gap-3), and between a glyph and its label (gap-1.5).
const GAP: f32 = 12.0;
const SMALL_GAP: f32 = 6.0;
/// The hint under the buttons: the web's copy (the lead clause is not decided by the contract).
const HINT: &str = "Type a title, add a note, or add an image — the task is created the moment it has something in it.";

/// One glyph, centred on a point.
fn draw_glyph(ui: &egui::Ui, name: &str, centre: Pos2, px: f32, colour: Color32) {
    let glyph = icon(name, px);
    ui.painter().text(centre, Align2::CENTER_CENTER, glyph.text(), FontId::new(px, FontFamily::Name("Icons".into())), colour);
}

/// The focus ring: 2px of focus_1 outside a rect.
fn ring(ui: &egui::Ui, rect: Rect, radius: u8, t: Tokens) {
    ui.painter().rect_stroke(rect.expand(2.0), CornerRadius::same(radius), Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
}

/// A line of text on one row, ending in an ellipsis when it is wider than max_w.
fn clipped(ui: &egui::Ui, s: String, size: f32, w: u16, colour: Color32, max_w: f32) -> Arc<Galley> {
    let mut job = LayoutJob::single_section(s, TextFormat { font_id: FontId::new(size, weight(w)), color: colour, ..Default::default() });
    job.wrap.max_width = max_w;
    job.wrap.max_rows = 1;
    job.wrap.break_anywhere = true;
    job.wrap.overflow_character = Some('…');
    ui.painter().layout_job(job)
}

/// A dashed button with an icon and a label, centred in its box (the web's dashed variant, tone muted).
fn chip(ui: &egui::Ui, t: Tokens, id: Id, rect: Rect, glyph: &str, label: &str) -> Response {
    let response = ui.interact(rect, id, Sense::click());
    let hot = ui.rect_contains_pointer(rect);
    if hot {
        ui.painter().rect_filled(rect, CornerRadius::same(7), t.wash_1);
    }
    let edge = if hot { t.edge_3 } else { t.edge_2 };
    let corners = [rect.left_top(), rect.right_top(), rect.right_bottom(), rect.left_bottom(), rect.left_top()];
    ui.painter().extend(Shape::dashed_line(&corners, Stroke::new(1.0, edge), 4.0, 3.0));
    let fg = if hot { t.content_2 } else { t.content_3 };
    let text = ui.painter().layout_no_wrap(label.to_owned(), FontId::new(13.0, weight(500)), fg);
    let (text_w, text_h) = (text.size().x, text.size().y);
    let total = 16.0 + SMALL_GAP + text_w;
    let x = rect.center().x - total / 2.0;
    draw_glyph(ui, glyph, pos2(x + 8.0, rect.center().y), 16.0, fg);
    ui.painter().galley(pos2(x + 16.0 + SMALL_GAP, rect.center().y - text_h / 2.0), text, fg);
    if response.has_focus() {
        ring(ui, rect, 7, t);
    }
    response.widget_info(|| WidgetInfo::labeled(Role::Button, true, label));
    response
}

/// The new-task draft. Its title is focused on mount. Enter or Shift+Enter commits the title as a task, as
/// does Tab or a click away; Escape and Discard draft drop it; Note creates the task and opens it with a note
/// ready. Any of those closes the draft: from then on it draws nothing and reports nothing.
pub struct DraftInspector {
    /// The title as typed (raw; the action gets it trimmed).
    title: String,
    /// The draft is on screen.
    open: bool,
    /// Focus goes to the title on the first frame.
    focus_pending: bool,
    /// The title had focus at the end of the last frame: losing it is a click-away.
    was_focused: bool,
}

impl Default for DraftInspector {
    fn default() -> Self {
        Self::new()
    }
}

impl DraftInspector {
    pub fn new() -> Self {
        Self { title: String::new(), open: true, focus_pending: true, was_focused: false }
    }

    /// One frame, on the app: what the main window calls, and what the test mounts. Returns the draft action
    /// the frame ran, if any.
    pub fn show(&mut self, app: &mut App, ui: &mut egui::Ui, look: Look) -> Vec<DraftinspectorEvent> {
        if !self.open {
            return vec![];
        }
        let t = look.t;
        let title_id = Id::new("app.draftinspector.title");
        let discard_id = Id::new("app.draftinspector.discardDraft");
        let note_id = Id::new("app.draftinspector.noteButton");
        let image_id = Id::new("app.draftinspector.imageButton");

        // Keys are read before the field sees them, so Enter never adds a line. The title counts as focused
        // from the last frame too: egui takes the focus away on Escape and moves it on Tab before this frame.
        if self.focus_pending {
            ui.memory_mut(|m| m.request_focus(title_id));
            self.focus_pending = false;
        }
        if self.was_focused || ui.memory(|m| m.has_focus(title_id)) {
            let (enter, shift_enter, escape, tab) = ui.input_mut(|i| {
                (
                    i.consume_key(Modifiers::NONE, Key::Enter),
                    i.consume_key(Modifiers::SHIFT, Key::Enter),
                    i.consume_key(Modifiers::NONE, Key::Escape),
                    i.consume_key(Modifiers::NONE, Key::Tab) || i.consume_key(Modifiers::SHIFT, Key::Tab),
                )
            });
            if enter || shift_enter || tab {
                // Tab leaves the title for the next control, which is the web's click-away: it commits here.
                let value = self.title.trim().to_owned();
                return vec![self.run(app, "commit", value)];
            }
            if escape {
                return vec![self.run(app, "cancel", String::new())];
            }
        }

        let origin = ui.cursor().min;
        let w = ui.available_width();
        let (x0, y0) = (origin.x, origin.y);
        let left = x0 + 16.0;
        let inner_w = w - 32.0;
        let right = x0 + w;
        let top = y0 + 16.0;

        // The accent bar at the panel's left edge, beside the title's first line.
        ui.painter().rect_filled(Rect::from_min_size(pos2(x0, y0 + 16.0), vec2(2.0, 16.0)), CornerRadius::same(0), t.accent_500);

        // Title: a draft field on surface_0, one line that grows to two and then scrolls.
        let title_font = FontId::new(18.0, weight(600));
        let col_w = inner_w - GAP - DISCARD;
        let field_w = (col_w - 2.0 * PAD).max(0.0);
        // One row's height in this font; the field is one row, grows to two rows, then scrolls.
        let row_h = ui.painter().layout_no_wrap("Ay".to_owned(), title_font.clone(), t.content_1).size().y.max(LINE);
        let text_h = ui.painter().layout(self.title.clone(), title_font.clone(), t.content_1, field_w).size().y;
        let grown = text_h > row_h + 0.5;
        let box_h = if grown { 2.0 * row_h } else { row_h };
        let title_rect = Rect::from_min_size(pos2(left, top), vec2(col_w, box_h));
        ui.painter().rect_filled(title_rect, CornerRadius::same(4), t.surface_0);
        let inner = Rect::from_min_size(title_rect.min + vec2(PAD, 0.0), vec2(field_w, box_h));
        let mut child = ui.new_child(UiBuilder::new().max_rect(inner).layout(Layout::top_down(Align::Min)));
        child.set_clip_rect(title_rect);
        child.visuals_mut().selection.bg_fill = t.selection_1;
        let field = TextEdit::multiline(&mut self.title)
            .id(title_id)
            .frame(egui::Frame::NONE)
            .margin(Margin::ZERO)
            .font(title_font)
            .text_color(t.content_1)
            .hint_text(RichText::new("Untitled").color(t.content_3))
            .desired_rows(1)
            .desired_width(field_w);
        let out = if grown {
            ScrollArea::vertical()
                .id_salt(title_id.with("scroll"))
                .max_height(box_h)
                .auto_shrink([false, true])
                .show(&mut child, |ui| field.show(ui))
                .inner
        } else {
            field.show(&mut child)
        };
        let field_resp = out.response;
        let value = self.title.clone();
        ui.ctx().accesskit_node_builder(title_id, |node| node.set_value(value));
        field_resp.widget_info(|| WidgetInfo::labeled(Role::TextInput, true, "Title"));
        part(ui, &field_resp, parts::TITLE);
        if field_resp.has_focus() {
            ring(ui, title_rect, 4, t);
        }

        // Discard draft: a close glyph at the title's top right.
        let discard_rect = Rect::from_min_size(pos2(right - 16.0 - DISCARD, top), vec2(DISCARD, DISCARD));
        let discard = ui.interact(discard_rect, discard_id, Sense::click());
        let hot = ui.rect_contains_pointer(discard_rect);
        if hot {
            ui.painter().rect_filled(discard_rect, CornerRadius::same(4), t.wash_1);
        }
        draw_glyph(ui, "close", discard_rect.center(), 16.0, if hot { t.content_2 } else { t.content_3 });
        if discard.has_focus() {
            ring(ui, discard_rect, 4, t);
        }
        discard.widget_info(|| WidgetInfo::labeled(Role::Button, true, "Discard draft"));
        part(ui, &discard, parts::DISCARDDRAFT);
        let discard = discard.on_hover_text("Discard draft");
        if discard.clicked() {
            return vec![self.run(app, "cancel", String::new())];
        }

        // Breadcrumb: the folder, the project, then "/ list" when the draft has a list.
        let bc_top = top + box_h + 8.0;
        let mid = bc_top + 8.0;
        let project = app.ui.project.as_deref().and_then(|p| app.data.project(p)).map(|p| p.name.clone()).unwrap_or_else(|| "—".to_owned());
        let list = app.ui.draft_list.as_deref().and_then(|l| app.data.list(l)).map(|l| l.name.clone());
        let list_text = list.as_ref().map(|l| ui.painter().layout_no_wrap(format!("/ {l}"), FontId::new(11.0, weight(400)), t.content_3));
        let list_w = list_text.as_ref().map_or(0.0, |g| g.size().x + SMALL_GAP);
        let name_text = clipped(ui, project.clone(), 11.0, 400, t.content_2, (inner_w - 13.0 - SMALL_GAP - list_w).max(0.0));
        let name_x = left + 13.0 + SMALL_GAP;
        let (name_w, name_h) = (name_text.size().x, name_text.size().y);
        draw_glyph(ui, "folder_open", pos2(left + 6.5, mid), 13.0, t.content_3);
        ui.painter().galley(pos2(name_x, mid - name_h / 2.0), name_text, t.content_2);
        if let Some(g) = list_text {
            let h = g.size().y;
            ui.painter().galley(pos2(name_x + name_w + SMALL_GAP, mid - h / 2.0), g, t.content_3);
        }
        let label = match &list {
            Some(l) => format!("{project} / {l}"),
            None => project.clone(),
        };
        let bc = ui.interact(Rect::from_min_size(pos2(left, bc_top), vec2(inner_w, 16.0)), Id::new("app.draftinspector.breadcrumb"), Sense::hover());
        bc.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, &label));
        part(ui, &bc, parts::BREADCRUMB);

        // Start with: the section label with a rule running to the right edge.
        let sw_top = bc_top + 16.0 + 16.0;
        let sw_mid = sw_top + 8.0;
        let sw_label = ui.painter().layout_no_wrap("START WITH".to_owned(), FontId::new(11.0, weight(500)), t.content_3);
        let (sw_label_w, sw_label_h) = (sw_label.size().x, sw_label.size().y);
        ui.painter().galley(pos2(left, sw_mid - sw_label_h / 2.0), sw_label, t.content_3);
        ui.painter().line_segment([pos2(left + sw_label_w + 8.0, sw_mid), pos2(left + inner_w, sw_mid)], Stroke::new(1.0, t.edge_1));
        let sw = ui.interact(Rect::from_min_size(pos2(left, sw_top), vec2(inner_w, 16.0)), Id::new("app.draftinspector.startWith"), Sense::hover());
        sw.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, "Start with"));
        part(ui, &sw, parts::STARTWITH);

        // The two dashed buttons side by side: Note, then Image (the contract's imageButton). The desktop has
        // no file chooser yet, so a press of Image does nothing.
        let btn_top = sw_top + 16.0 + 8.0;
        let chip_w = (inner_w - SMALL_GAP) / 2.0;
        let note_rect = Rect::from_min_size(pos2(left, btn_top), vec2(chip_w, CHIP_H));
        let image_rect = Rect::from_min_size(pos2(left + chip_w + SMALL_GAP, btn_top), vec2(chip_w, CHIP_H));
        let note = chip(ui, t, note_id, note_rect, "add", "Note");
        part(ui, &note, parts::NOTEBUTTON);
        let image = chip(ui, t, image_id, image_rect, "image", "Image");
        part(ui, &image, parts::IMAGEBUTTON);
        if note.clicked() {
            let value = self.title.trim().to_owned();
            return vec![self.run(app, "note", value)];
        }

        // The hint under the buttons.
        let hint_top = btn_top + CHIP_H + 12.0;
        let mut job = LayoutJob::default();
        job.append(
            HINT,
            0.0,
            TextFormat { font_id: FontId::new(12.0, weight(400)), color: t.content_3, line_height: Some(16.0), ..Default::default() },
        );
        job.wrap.max_width = inner_w;
        let hint = ui.painter().layout_job(job);
        let hint_rect = Rect::from_min_size(pos2(left, hint_top), hint.size());
        let hint_node = ui.interact(hint_rect, Id::new("app.draftinspector.hint"), Sense::hover());
        hint_node.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, HINT));
        part(ui, &hint_node, parts::HINT);
        ui.painter().galley(hint_rect.min, hint, t.content_3);
        let end = hint_rect.bottom() + 16.0;

        // Focus leaving the title is a click-away, which commits, unless it went to one of the draft's own
        // controls. A press or click on one of them keeps the title's focus, as the web's mousedown does.
        let on_control = [note_rect, image_rect, discard_rect].iter().any(|r| ui.rect_contains_pointer(*r));
        let mut focused_now = ui.memory(|m| m.has_focus(title_id));
        if self.was_focused && !focused_now {
            if on_control {
                ui.memory_mut(|m| m.request_focus(title_id));
                focused_now = true;
            } else if ![note_id, image_id, discard_id].iter().any(|id| ui.memory(|m| m.has_focus(*id))) {
                let value = self.title.trim().to_owned();
                return vec![self.run(app, "commit", value)];
            }
        }
        self.was_focused = focused_now;

        let _ = ui.allocate_exact_size(vec2(w, end - y0), Sense::hover());
        vec![]
    }

    /// Runs one draft action once and closes the draft. Commit creates a task from a non-empty title (in the
    /// draft's list, when it has one); Note creates one (Untitled when empty) and opens it with a note ready;
    /// Cancel writes nothing. Returns the event to report.
    fn run(&mut self, app: &mut App, action: &str, value: String) -> DraftinspectorEvent {
        self.open = false;
        let list = app.ui.draft_list.take();
        if let Some(list) = list {
            match action {
                "commit" if !value.is_empty() => {
                    if let Err(e) = app.data.create_task(&list, &value, None, None) {
                        app.refused(e);
                    }
                }
                "note" => {
                    let title = if value.is_empty() { "Untitled" } else { value.as_str() };
                    match app.data.create_task(&list, title, None, None) {
                        Ok(task) => {
                            app.ui.aside = Aside::Preview;
                            app.ui.note_for = Some(task.id.clone());
                            app.ui.preview = Some(task.id);
                        }
                        Err(e) => app.refused(e),
                    }
                }
                _ => {}
            }
        }
        let title = if action == "cancel" { String::new() } else { value };
        DraftinspectorEvent::DraftAction { action: action.to_owned(), title }
    }
}

/// The test's mount: the app on the props' data set, with the draft started in the props' list.
struct Mount {
    app: App,
    screen: DraftInspector,
}

impl Mounted for Mount {
    fn frame(&mut self, ui: &mut egui::Ui, look: Look, emit: &mut dyn FnMut(&'static str, serde_json::Value)) {
        for e in self.screen.show(&mut self.app, ui, look) {
            let (name, fields) = e.to_json();
            emit(name, fields);
        }
    }
}

pub fn mount(mut app: App, props: serde_json::Value) -> Result<Box<dyn Mounted>, String> {
    let p: DraftinspectorProps = serde_json::from_value(props).map_err(|e| e.to_string())?;
    // The new draft targets the Home project's list by name; "" starts it with no list.
    app.open_named(Some("Home"), None);
    let list = if p.list.is_empty() { None } else { app.data.list_named(app.ui.project.as_deref(), &p.list).map(|l| l.id.clone()) };
    app.ui.draft_list = list;
    Ok(Box::new(Mount { app, screen: DraftInspector::new() }))
}
