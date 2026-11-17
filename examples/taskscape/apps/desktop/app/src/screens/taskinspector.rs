//! contract: omnicui/contracts/taskinspector.contract.yaml (id: app.taskinspector). The task inspector: one task's title, done toggle, breadcrumb and times, notes, subtasks and attachments, for the task shown in the active pane.
use crate::data::{Id, LinkType};
use crate::App;
use egui::{pos2, vec2, Align, CornerRadius, FontFamily, FontId, Layout, Rect, Role, Sense, Stroke, StrokeKind, UiBuilder, WidgetInfo};
use taskscape_storage::{Attachment, Note, Task};
use taskscape_ui::omnicui::attachmenttile::{parts as tile_parts, AttachmenttileEvent};
use taskscape_ui::omnicui::button::{parts as button_parts, ButtonEvent, ButtonProps};
use taskscape_ui::omnicui::charactercounter::parts as counter_parts;
use taskscape_ui::omnicui::checkbox::{parts as checkbox_parts, CheckboxEvent};
use taskscape_ui::omnicui::input::{parts as input_parts, InputEvent};
use taskscape_ui::omnicui::lightbox::{parts as lightbox_parts, LightboxEvent};
use taskscape_ui::omnicui::richtexteditor::{parts as editor_parts, RichtexteditorEvent};
use taskscape_ui::omnicui::taskinspector::{parts, TaskinspectorEvent, TaskinspectorProps};
use taskscape_ui::omnicui::text::parts as text_parts;
use taskscape_ui::widgets::attachmenttile::AttachmentTile;
use taskscape_ui::widgets::button::Button;
use taskscape_ui::widgets::charactercounter::CharacterCounter;
use taskscape_ui::widgets::checkbox::Checkbox;
use taskscape_ui::widgets::input::Input;
use taskscape_ui::widgets::lightbox::Lightbox;
use taskscape_ui::widgets::richtexteditor::RichTextEditor;
use taskscape_ui::widgets::text::Text;
use taskscape_ui::{as_part, icon, part, rem, weight, Look, Mounted};

/// The note limit (the web's NOTE_LIMIT).
const NOTE_LIMIT: i64 = 2000;
const NOTE_PLACEHOLDER: &str = "Write a note ... (drop an attachment to @mention it)";

/// Builds a contract props struct from its JSON form (the names are the contract's).
macro_rules! props {
    ($($j:tt)+) => {
        serde_json::from_value(serde_json::json!({ $($j)+ })).expect("props match the contract")
    };
}

/// One rich-text editor and what the screen keeps of it.
struct NoteEditor {
    editor: RichTextEditor,
    /// The text as the editor last reported it.
    text: String,
    /// The text last written to the database (empty: not saved).
    last: String,
    /// The saved note, once there is one.
    note: Option<Id>,
    /// The note as last written: HTML, with the formatting the plain `last` does not show.
    html: String,
}

pub struct TaskInspector {
    /// The task the state below belongs to.
    task: Option<Id>,
    /// The title field: inline (reads as text until clicked) or a draft editor from Rename.
    title: Option<Input>,
    /// Editors of the saved notes, by note id.
    notes: Vec<(Id, NoteEditor)>,
    /// The add-note editor, while adding.
    draft: Option<NoteEditor>,
    adding: bool,
    /// The attachment lightbox, while one is open.
    lightbox: Option<Lightbox>,
    /// When Copy task was pressed (Copied shows for 1.2 s).
    copied_at: Option<f64>,
}

impl Default for TaskInspector {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskInspector {
    pub fn new() -> Self {
        Self { task: None, title: None, notes: vec![], draft: None, adding: false, lightbox: None, copied_at: None }
    }

    /// One frame: what the main window calls, and what the test mounts.
    pub fn show(&mut self, app: &mut App, ui: &mut egui::Ui, look: Look) -> Vec<TaskinspectorEvent> {
        let mut out = vec![];
        let Some(list) = app.active_list() else { return out };
        let Some(shown) = app.ui.preview.clone() else { return out };
        let Some(task) = app.data.task(&shown).cloned() else { return out };
        if task.list_id != list {
            return out;
        }
        if self.task.as_deref() != Some(task.id.as_str()) {
            self.reset(&task);
        }
        let attachments = app.data.attachments_of(&task.id);
        let mentions = mentions_json(&attachments);
        self.sync_notes(&app.data.notes_of(&task.id), &mentions);

        let now = now_ms();
        let copied = self.copied_at.is_some_and(|at| ui.input(|i| i.time) - at < 1.2);
        if copied {
            ui.ctx().request_repaint_after(std::time::Duration::from_millis(1250));
        }
        let project = app.ui.project.as_ref().and_then(|p| app.data.project(p)).map(|p| p.name.clone()).unwrap_or_else(|| "-".into());
        let crumb = match app.data.list(&list).map(|l| l.name.clone()) {
            Some(name) => format!("{project} / {name}"),
            None => project,
        };
        let created = format!("Created {}", relative(task.created_at, now));
        let updated = format!("Updated {}", relative(task.updated_at, now));

        // ---- the header: done toggle, title and meta lines, and the four header buttons ----
        let mut done_req: Option<bool> = None;
        let mut title_commit: Option<String> = None;
        let mut title_cancel = false;
        let (mut rename_req, mut close_req, mut copy_req, mut delete_req) = (false, false, false, false);
        let total_w = ui.available_width();
        // The header sits a little below the panel's top edge, as the reference's does.
        ui.add_space(rem(0.857));
        let header = ui.horizontal_top(|ui| {
            ui.add_space(rem(0.929));
            let mut cb = Checkbox::new(props!(
                "checked": task.done,
                "ariaLabel": if task.done { "Mark not done" } else { "Mark done" },
                "intent": "done",
                "size": "sm"
            ));
            // Centred on the title's first line (21 px), as the reference's (#12357).
            let events = ui
                .allocate_ui_with_layout(vec2(rem(1.143), 21.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    as_part(ui, &[(checkbox_parts::CONTROL, parts::DONETOGGLE)], |ui| cb.ui(ui, look))
                })
                .inner;
            for e in events {
                let CheckboxEvent::CheckedChange { checked } = e;
                done_req = Some(checked);
            }
            // The text column takes the room up to the button column, which sits at the panel's right edge.
            let text_w = (ui.available_width() - rem(1.5) - ui.spacing().item_spacing.x - rem(0.857)).max(0.0);
            let _ = total_w;
            ui.vertical(|ui| {
                ui.set_min_width(text_w);
                ui.set_max_width(text_w);
                ui.spacing_mut().item_spacing.y = rem(0.07);
                if let Some(field) = self.title.as_mut() {
                    for e in as_part(ui, &[(input_parts::FIELD, parts::TITLE)], |ui| field.ui(ui, look)) {
                        match e {
                            InputEvent::Commit { value } => title_commit = Some(value),
                            InputEvent::Cancel {} => title_cancel = true,
                            _ => {}
                        }
                    }
                }
                meta_line(ui, look, "folder_open", parts::BREADCRUMB, crumb);
                meta_line(ui, look, "schedule", parts::CREATED, created);
                meta_line(ui, look, "update", parts::UPDATED, updated);
            });
            ui.vertical(|ui| {
                ui.add_space(rem(0.357));
                close_req |= button(ui, look, parts::CLOSEPANEL, props!(
                    "icon": "last_page", "iconSize": 16, "variant": "ghost", "size": "sm", "shape": "icon",
                    "tone": "muted", "ariaLabel": "Close panel", "tabbable": true
                ));
                rename_req |= button(ui, look, parts::RENAME, props!(
                    "icon": "edit", "iconSize": 15, "variant": "ghost", "size": "sm", "shape": "icon",
                    "tone": "muted", "ariaLabel": "Rename", "tabbable": true
                ));
                copy_req |= button(ui, look, parts::COPYTASK, props!(
                    "icon": if copied { "check" } else { "content_copy" }, "iconSize": 15, "variant": "ghost",
                    "size": "sm", "shape": "icon", "tone": "muted",
                    "ariaLabel": if copied { "Copied" } else { "Copy task" }, "tabbable": true
                ));
                delete_req |= button(ui, look, parts::DELETETASK, props!(
                    "icon": "delete", "iconSize": 15, "variant": "ghost", "size": "sm", "shape": "icon",
                    "tone": "muted", "ariaLabel": "Delete task", "tabbable": true
                ));
            });
        });

        // The accent tick at the header's left edge (decorative, but a node for the contract).
        let r = header.response.rect;
        let tick = Rect::from_min_size(pos2(r.left(), r.top() + rem(1.143)), vec2(2.0, 16.0));
        let tick_resp = ui.interact(tick, ui.id().with("accentTick"), Sense::hover());
        tick_resp.widget_info(|| WidgetInfo::new(Role::GenericContainer));
        part(ui, &tick_resp, parts::ACCENTTICK);
        ui.painter().rect_filled(tick, CornerRadius::same(0), look.t.accent_500);

        // ---- the header's actions, applied to the data at once ----
        if let Some(done) = done_req {
            match app.data.update_task(&task.id, None, None, Some(done)) {
                Ok(_) => out.push(TaskinspectorEvent::DoneChanged { title: task.title.clone(), done }),
                Err(e) => app.refused(e),
            }
        }
        if let Some(text) = title_commit {
            let text = text.trim().to_owned();
            let mut saved = task.title.clone();
            if !text.is_empty() && text != task.title {
                match app.data.update_task(&task.id, Some(&text), None, None) {
                    Ok(_) => {
                        out.push(TaskinspectorEvent::TitleChanged { title: text.clone() });
                        saved = text;
                    }
                    Err(e) => app.refused(e),
                }
            }
            self.title = Some(title_input(&saved, false));
            ui.ctx().memory_mut(|m| m.surrender_focus(egui::Id::new("ui.input.field")));
        } else if title_cancel {
            self.title = Some(title_input(&task.title, false));
            ui.ctx().memory_mut(|m| m.surrender_focus(egui::Id::new("ui.input.field")));
        }
        if rename_req {
            self.title = Some(title_input(&task.title, true));
        }
        if copy_req {
            ui.ctx().copy_text(task.title.clone());
            self.copied_at = Some(ui.input(|i| i.time));
        }
        if close_req {
            out.push(TaskinspectorEvent::PreviewOpenChanged { open: false });
        }
        if delete_req {
            out.push(TaskinspectorEvent::DeleteRequested { target: "task".into(), title: task.title.clone() });
        }

        // ---- the body: notes, subtasks, attachments ----
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.add_space(rem(1.143));
            padded(ui, rem(0.929), |ui| self.notes_section(app, ui, look, &task, &mut out));
            ui.add_space(rem(1.143));
            if !app.data.children_of(&task.id).is_empty() {
                padded(ui, rem(0.929), |ui| subtasks_section(app, ui, look, &task, &mut out));
                ui.add_space(rem(1.143));
            }
            padded(ui, rem(0.929), |ui| self.attachments_section(app, ui, look, &task, &attachments, &mut out));
            ui.add_space(rem(0.929));
        });

        if let Some(lb) = self.lightbox.as_mut() {
            let evs = as_part(ui, &[(lightbox_parts::LIGHTBOX, parts::LIGHTBOX)], |ui| lb.ui(ui, look));
            if evs.iter().any(|e| matches!(e, LightboxEvent::Close { .. })) {
                self.lightbox = None;
            }
        }
        out
    }

    /// A new task shown: its title field, notes and open states start again.
    fn reset(&mut self, task: &Task) {
        self.task = Some(task.id.clone());
        self.title = Some(title_input(&task.title, false));
        self.notes.clear();
        self.draft = None;
        self.adding = false;
        self.lightbox = None;
        self.copied_at = None;
    }

    /// Keeps one editor per saved note (the draft's note stands in for its editor while it is typed).
    fn sync_notes(&mut self, saved: &[Note], mentions: &str) {
        self.notes.retain(|(id, _)| saved.iter().any(|n| n.id == *id));
        for n in saved {
            if !self.notes.iter().any(|(id, _)| *id == n.id) {
                let mut ne = new_editor(&n.content, "", false, mentions);
                ne.note = Some(n.id.clone());
                self.notes.push((n.id.clone(), ne));
            }
        }
    }

    fn notes_section(&mut self, app: &mut App, ui: &mut egui::Ui, look: Look, task: &Task, out: &mut Vec<TaskinspectorEvent>) {
        let saved = app.data.notes_of(&task.id);
        let count = saved.len();
        let draft_note = self.draft.as_ref().and_then(|d| d.note.clone());
        let shown: Vec<Note> = saved.iter().filter(|n| Some(&n.id) != draft_note.as_ref()).cloned().collect();
        let mut add_req = false;
        heading(ui, look, "Notes", parts::NOTESHEADING, true, |ui| {
            if count > 0 {
                add_req |= button(ui, look, parts::ADDNOTE, props!(
                    "label": "Note", "icon": "add", "iconSize": 14, "variant": "ghost", "size": "sm",
                    "width": "content", "tone": "muted", "ariaLabel": "Add note", "tabbable": true
                ));
                text_only(ui, look, parts::NOTESCOUNT, count.to_string());
            }
        });
        let mentions = mentions_json(&app.data.attachments_of(&task.id));

        ui.vertical(|ui| {
            for (i, note) in shown.iter().enumerate() {
                let (body, counter, del) = match i {
                    0 => (parts::NOTEBODY, parts::NOTECHARS, parts::NOTEDELETE),
                    1 => (parts::NOTEBODYSECOND, parts::NOTECHARSSECOND, "app.taskinspector.noteDeleteMore"),
                    _ => ("app.taskinspector.noteBodyMore", "app.taskinspector.noteCharsMore", "app.taskinspector.noteDeleteMore"),
                };
                let Some(pos) = self.notes.iter().position(|(id, _)| *id == note.id) else { continue };
                let (evs, _) = editor_ui(ui, look, &mut self.notes[pos].1, body, counter);
                let mut removed = false;
                for e in evs {
                    removed |= self.saved_note_event(app, out, task, &note.id, e);
                }
                if removed {
                    self.notes.retain(|(id, _)| *id != note.id);
                    continue;
                }
                let delete = button(ui, look, del, props!(
                    "icon": "close", "iconSize": 14, "variant": "filled", "size": "sm", "shape": "icon",
                    "tone": "muted", "reveal": "row", "ariaLabel": "Delete note", "tabbable": true
                ));
                if delete {
                    out.push(TaskinspectorEvent::DeleteRequested { target: "note".into(), title: task.title.clone() });
                }
            }
            if self.adding {
                if let Some(mut draft) = self.draft.take() {
                    let (evs, _) = editor_ui(ui, look, &mut draft, parts::NEWNOTEEDITOR, parts::NOTECHARS);
                    self.draft = Some(draft);
                    for e in evs {
                        self.draft_note_event(app, out, task, e);
                    }
                }
            }
            if count == 0 && !self.adding {
                let add = button(ui, look, parts::ADDNOTE, props!(
                    "variant": "dashed", "size": "dashedTall", "width": "full", "icon": "add", "iconSize": 16,
                    "label": "Add note", "tone": "muted", "ariaLabel": "Add note", "tabbable": true
                ));
                if add {
                    add_req = true;
                }
            }
        });

        if add_req {
            self.adding = true;
            self.draft = Some(new_editor("", NOTE_PLACEHOLDER, true, &mentions));
        }
    }

    /// One event of a saved note's editor. Returns true when the note was deleted.
    fn saved_note_event(&mut self, app: &mut App, out: &mut Vec<TaskinspectorEvent>, task: &Task, id: &Id, e: RichtexteditorEvent) -> bool {
        let Some(pos) = self.notes.iter().position(|(n, _)| n == id) else { return false };
        match e {
            RichtexteditorEvent::Changed { value, .. } => {
                let ne = &mut self.notes[pos].1;
                ne.text = value.clone();
                if !value.is_empty() && value != ne.last {
                    let html = ne.editor.html();
                    save_note(app, out, id, &value, &html, &mut ne.last, &mut ne.html);
                }
            }
            RichtexteditorEvent::Blurred { value } => {
                let ne = &mut self.notes[pos].1;
                ne.text = value.clone();
                if value.is_empty() {
                    if ne.last.is_empty() {
                        return false;
                    }
                    match app.data.delete_note(id) {
                        Ok(()) => {
                            out.push(TaskinspectorEvent::NotesChanged { count: app.data.notes_of(&task.id).len() as i64 });
                            return true;
                        }
                        Err(e) => app.refused(e),
                    }
                } else if value != ne.last || ne.editor.html() != ne.html {
                    // A formatting change alone (bold on a word) is saved when the note loses focus.
                    let html = ne.editor.html();
                    save_note(app, out, id, &value, &html, &mut ne.last, &mut ne.html);
                }
            }
            RichtexteditorEvent::LinkRequested {} => out.push(TaskinspectorEvent::LinkRequested { r#where: "note".into() }),
            RichtexteditorEvent::MentionOpened { id: attachment } => self.open_attachment(app, out, &task.id, &attachment),
            _ => {}
        }
        false
    }

    /// One event of the add-note editor: the note is created on the first text and kept up to date.
    fn draft_note_event(&mut self, app: &mut App, out: &mut Vec<TaskinspectorEvent>, task: &Task, e: RichtexteditorEvent) {
        match e {
            RichtexteditorEvent::Changed { value, .. } => self.draft_text(app, out, task, value),
            RichtexteditorEvent::Blurred { value } => {
                self.draft_text(app, out, task, value);
                self.adding = false;
                self.draft = None;
            }
            RichtexteditorEvent::LinkRequested {} => out.push(TaskinspectorEvent::LinkRequested { r#where: "note".into() }),
            RichtexteditorEvent::MentionOpened { id } => self.open_attachment(app, out, &task.id, &id),
            _ => {}
        }
    }

    fn draft_text(&mut self, app: &mut App, out: &mut Vec<TaskinspectorEvent>, task: &Task, value: String) {
        let Some(d) = self.draft.as_mut() else { return };
        d.text = value.clone();
        if value.is_empty() {
            if let Some(id) = d.note.take() {
                d.last.clear();
                match app.data.delete_note(&id) {
                    Ok(()) => out.push(TaskinspectorEvent::NotesChanged { count: app.data.notes_of(&task.id).len() as i64 }),
                    Err(e) => app.refused(e),
                }
            }
        } else if let Some(id) = d.note.clone() {
            let html = d.editor.html();
            if value != d.last || html != d.html {
                save_note(app, out, &id, &value, &html, &mut d.last, &mut d.html);
            }
        } else {
            let html = d.editor.html();
            match app.data.create_note(&task.id, &html) {
                Ok(note) => {
                    d.note = Some(note.id.clone());
                    d.last = value.clone();
                    d.html = html;
                    out.push(TaskinspectorEvent::NotesChanged { count: app.data.notes_of(&task.id).len() as i64 });
                    out.push(TaskinspectorEvent::NoteSaved { text: value });
                }
                Err(e) => app.refused(e),
            }
        }
    }

    /// A tile press or a mention chip: the attachment opens (a link leaves the app; a local file opens the lightbox).
    fn open_attachment(&mut self, app: &mut App, out: &mut Vec<TaskinspectorEvent>, task: &Id, id: &Id) {
        let Some(a) = app.data.attachments_of(task).into_iter().find(|a| a.id == *id) else { return };
        out.push(TaskinspectorEvent::AttachmentAction { action: "open".into(), name: a.name.clone() });
        if !is_remote(&a) {
            self.lightbox = Some(Lightbox::new(props!("open": true, "presentation": "window")));
        }
    }

    fn attachments_section(&mut self, app: &mut App, ui: &mut egui::Ui, look: Look, task: &Task, atts: &[Attachment], out: &mut Vec<TaskinspectorEvent>) {
        let (mut link_req, mut file_req, mut shot_req) = (false, false, false);
        heading(ui, look, "Attachments", parts::ATTACHMENTSHEADING, false, |ui| {
            file_req |= button(ui, look, parts::ATTACHFILE, props!(
                "label": "File", "icon": "note_add", "iconSize": 14, "variant": "ghost", "size": "sm",
                "width": "content", "tone": "muted", "tabbable": true
            ));
            link_req |= button(ui, look, parts::ATTACHLINK, props!(
                "label": "Link", "icon": "add_link", "iconSize": 14, "variant": "ghost", "size": "sm",
                "width": "content", "tone": "muted", "ariaLabel": "Add a link", "tabbable": true
            ));
            // Shot: captures the full screen and attaches it (the reference's, decision 27). Laid out from the
            // right, so it comes first in reading order: Shot, Link, File.
            shot_req |= button(ui, look, parts::ATTACHSHOT, props!(
                "label": "Shot", "icon": "screenshot_monitor", "iconSize": 14, "variant": "ghost", "size": "sm",
                "width": "content", "tone": "muted", "ariaLabel": "Screenshot",
                "description": "Capture the full screen and attach it", "tabbable": true
            ));
        });
        if shot_req && app.native_window {
            match taskscape_storage::screenshot::capture() {
                Ok(Some(path)) => {
                    if let Err(e) = app.data.add_file(&task.id, &path.to_string_lossy()) {
                        app.refused(e);
                    }
                }
                Ok(None) => {}
                Err(e) => app.refused(e),
            }
        }
        if file_req && app.native_window {
            if let Some(path) = crate::dialogs::open_any() {
                if let Err(e) = app.data.add_file(&task.id, &path.to_string_lossy()) {
                    app.refused(e);
                }
            }
        }
        if file_req {
            out.push(TaskinspectorEvent::AttachRequested { kind: "file".into() });
        }
        if link_req {
            out.push(TaskinspectorEvent::LinkRequested { r#where: "attachments".into() });
        }

        if atts.is_empty() {
            // The reference's empty state: a dashed box split in two, each half an icon over its label.
            let t = look.t;
            let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), rem(5.0)), Sense::hover());
            ui.painter().rect_filled(rect, CornerRadius::same(7), t.surface_3);
            dashed_rect(ui.painter(), rect.shrink(0.5), 7.0, Stroke::new(1.0, t.edge_3));
            let mid = rect.center().x;
            ui.painter().vline(mid, (rect.top() + 10.0)..=(rect.bottom() - 10.0), Stroke::new(1.0, t.edge_2));
            let half = |left: bool| if left { Rect::from_min_max(rect.min, pos2(mid, rect.max.y)) } else { Rect::from_min_max(pos2(mid, rect.min.y), rect.max) };
            let copy = empty_choice(ui, look, half(true), "note_add", "Copy a file", parts::EMPTYCOPYFILE);
            let add = empty_choice(ui, look, half(false), "add_link", "Add a link", parts::EMPTYADDLINK);
            if copy {
                out.push(TaskinspectorEvent::AttachRequested { kind: "file".into() });
            }
            if add {
                out.push(TaskinspectorEvent::LinkRequested { r#where: "emptyBox".into() });
            }
        } else {
            let mut opened: Vec<Id> = vec![];
            // The reference's grid: as many columns of at least 72 px as fit, sharing the row's width.
            let gap = 6.0;
            let avail = ui.available_width();
            let cols = ((avail + gap) / (72.0 + gap)).floor().max(1.0);
            let side = ((avail - gap * (cols - 1.0)) / cols).floor();
            ui.spacing_mut().item_spacing = vec2(gap, gap);
            ui.horizontal_wrapped(|ui| {
                for a in atts {
                    let remote = is_remote(a);
                    let image = !remote && is_image(&a.name);
                    let tile_part = if remote {
                        parts::ATTACHMENTTILELINK
                    } else if image {
                        parts::ATTACHMENTTILEIMAGE
                    } else {
                        "app.taskinspector.attachmentTileOther"
                    };
                    let mut tile = AttachmentTile::new(props!(
                        "kind": if image { "image" } else { "link" },
                        "name": a.name.clone(),
                        // A copied file's location is relative to the data folder.
                        "thumbnailSrc": if image { thumb_path(&a.location) } else { String::new() },
                        "draggable": false
                    ));
                    tile.side = side;
                    ui.vertical(|ui| {
                        ui.set_width(side);
                        for e in as_part(ui, &[(tile_parts::TILE, tile_part)], |ui| tile.ui(ui, look)) {
                            if let AttachmenttileEvent::Opened { .. } = e {
                                opened.push(a.id.clone());
                            }
                        }
                        caption(ui, look, side, &a.name);
                    });
                }
            });
            for id in opened {
                self.open_attachment(app, out, &task.id, &id);
            }
        }
    }
}


/// The Subtasks section: shown only when the task has children.
fn subtasks_section(app: &mut App, ui: &mut egui::Ui, look: Look, task: &Task, out: &mut Vec<TaskinspectorEvent>) {
    let kids: Vec<Task> = app.data.children_of(&task.id).into_iter().cloned().collect();
    let done = kids.iter().filter(|k| k.done).count();
    heading(ui, look, "Subtasks", parts::SUBTASKSHEADING, true, |ui| {
        text_only(ui, look, parts::SUBTASKSCOUNT, format!("{done} done / {}", kids.len()));
    });
    subtask_rows(app, ui, look, &task.id, 0, out);
}

/// The rows under a parent: a done checkbox and a title that selects the subtask.
fn subtask_rows(app: &mut App, ui: &mut egui::Ui, look: Look, parent: &Id, depth: usize, out: &mut Vec<TaskinspectorEvent>) {
    let kids: Vec<Task> = app.data.children_of(parent).into_iter().cloned().collect();
    for (i, kid) in kids.iter().enumerate() {
        let first = depth == 0 && i == 0;
        // The web's SubtaskItem: a done checkbox, then a title button that selects the subtask.
        let checkbox_part = if first { parts::SUBTASKDONE } else { "app.taskinspector.subtaskDoneMore" };
        let title_part = if kid.title == "Sourdough loaf" {
            parts::SUBTASKSOURDOUGH
        } else if first {
            parts::SUBTASKTITLE
        } else {
            "app.taskinspector.subtaskTitleMore"
        };
        let mut toggled: Option<bool> = None;
        let mut selected = false;
        // A row of the title button's height, everything centred on it: in a plain horizontal row the
        // checkbox is placed before the taller button and stays at the top (#12348).
        let row_w = ui.available_width();
        ui.allocate_ui_with_layout(vec2(row_w, rem(2.2857)), egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.add_space(rem(0.4286) + depth as f32 * rem(1.2857));
            let label = if kid.done { "Mark not done" } else { "Mark done" };
            let mut cb = Checkbox::new(props!("checked": kid.done, "ariaLabel": label, "intent": "done", "size": "sm"));
            for e in as_part(ui, &[(checkbox_parts::CONTROL, checkbox_part)], |ui| cb.ui(ui, look)) {
                let CheckboxEvent::CheckedChange { checked } = e;
                toggled = Some(checked);
            }
            let tone = if kid.done { "muted" } else { "strong" };
            selected |= button(ui, look, title_part, props!(
                "label": kid.title.clone(), "variant": "ghost", "size": "xl", "shape": "text",
                "width": "content", "tone": tone, "tabbable": true
            ));
        });
        if let Some(done) = toggled {
            match app.data.update_task(&kid.id, None, None, Some(done)) {
                Ok(_) => out.push(TaskinspectorEvent::DoneChanged { title: kid.title.clone(), done }),
                Err(e) => app.refused(e),
            }
        }
        if selected {
            app.ui.preview = Some(kid.id.clone());
            out.push(TaskinspectorEvent::TaskSelected { title: kid.title.clone() });
        }
        subtask_rows(app, ui, look, &kid.id, depth + 1, out);
    }
}

/// One editor for a note: its toolbar and, while focused, its counter (the widget's own counter shows only near the limit).
fn editor_ui(ui: &mut egui::Ui, look: Look, ne: &mut NoteEditor, body: &str, counter: &str) -> (Vec<RichtexteditorEvent>, bool) {
    let map = [
        (editor_parts::EDITOR, body),
        (editor_parts::BOLD, parts::NOTEBOLD),
        (editor_parts::ITALIC, parts::NOTEITALIC),
        (editor_parts::UNDERLINE, parts::NOTEUNDERLINE),
        (editor_parts::STRIKETHROUGH, parts::NOTESTRIKE),
        (editor_parts::BULLETLIST, parts::NOTEBULLET),
        (editor_parts::ORDEREDLIST, parts::NOTENUMBERED),
        (editor_parts::LINK, parts::NOTELINK),
        (editor_parts::MENTIONCHIP, parts::MENTIONCHIP),
        (editor_parts::MENTIONLIST, parts::MENTIONPOPUP),
        (editor_parts::MENTIONOPTION, parts::MENTIONITEM),
        // The count sits in the floating bar, as the reference's.
        (taskscape_ui::widgets::richtexteditor::BAR_COUNT, counter),
    ];
    // Each editor gets a scope of its own: the widget's ids come from the scope, so two editors must not share one.
    let (evs, focused) = ui
        .push_id(body, |ui| {
            as_part(ui, &map, |ui| {
                let eid = ui.scope_id().with("richtexteditor").with("editor");
                let evs = ne.editor.ui(ui, look);
                (evs, ui.memory(|m| m.has_focus(eid)))
            })
        })
        .inner;
    (evs, focused)
}

/// Saves a note (autosave while typing, and on blur): its HTML, as the reference stores notes.
fn save_note(app: &mut App, out: &mut Vec<TaskinspectorEvent>, id: &Id, value: &str, html: &str, last: &mut String, last_html: &mut String) {
    match app.data.update_note(id, html) {
        Ok(_) => {
            *last = value.to_owned();
            *last_html = html.to_owned();
            out.push(TaskinspectorEvent::NoteSaved { text: value.to_owned() });
        }
        Err(e) => app.refused(e),
    }
}

fn new_editor(value: &str, placeholder: &str, auto_focus: bool, mentions: &str) -> NoteEditor {
    let editor = RichTextEditor::new(props!(
        "value": value,
        "placeholder": placeholder,
        "ariaLabel": "Note",
        "maxLength": NOTE_LIMIT,
        "toolbar": "floating",
        "size": "md",
        "autoFocus": auto_focus,
        "mentions": mentions
    ));
    // `value` may be the reference's HTML: the editor shows its text, and the note is compared by both.
    let text = taskscape_ui::rich::parse(value).text;
    let html = if value.is_empty() { String::new() } else { editor.html() };
    NoteEditor { editor, text: text.clone(), last: text, note: None, html }
}

fn title_input(value: &str, rename: bool) -> Input {
    Input::new(props!(
        "value": value,
        "mode": if rename { "draft" } else { "inline" },
        "size": "display",
        "multiline": true,
        "placeholder": "Untitled",
        "ariaLabel": "Title",
        "selectOnOpen": true,
        "selectOnFocus": true,
        "autoFocus": rename
    ))
}

/// The section heading: an eyebrow label, a rule, and the trailing controls at the right.
fn heading(ui: &mut egui::Ui, look: Look, label: &str, part_id: &str, rule: bool, trailing: impl FnOnce(&mut egui::Ui)) {
    let t = look.t;
    ui.horizontal(|ui| {
        let g = taskscape_ui::eyebrow(ui.painter(), label, t.content_2);
        let (r, _) = ui.allocate_exact_size(g.size(), Sense::hover());
        ui.painter().galley(r.min, g, t.content_2);
        let r = ui.interact(r, egui::Id::new(("inspector.heading", part_id)), Sense::hover());
        r.widget_info(|| WidgetInfo::labeled(Role::Heading, true, label));
        part(ui, &r, part_id);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            trailing(ui);
            if rule {
                let w = ui.available_width() - 6.0;
                if w > 0.0 {
                    let (r, _) = ui.allocate_exact_size(vec2(w, 1.0), Sense::hover());
                    ui.painter().rect_filled(r, CornerRadius::same(0), t.edge_2);
                }
            }
        });
    });
}

/// A meta line: a glyph and the text beside it.
fn meta_line(ui: &mut egui::Ui, look: Look, glyph: &str, part_id: &str, content: String) {
    ui.horizontal(|ui| {
        let g = icon(glyph, 13.0).text().to_string();
        ui.label(egui::RichText::new(g).font(FontId::new(13.0, FontFamily::Name("Icons".into()))).color(look.t.content_3));
        text_only(ui, look, part_id, content);
    });
}

/// The web's Text widget, under a part id.
fn text_only(ui: &mut egui::Ui, look: Look, part_id: &str, content: String) {
    let mut t = Text::new(props!("content": content));
    let _ = as_part(ui, &[(text_parts::TEXT, part_id)], |ui| t.ui(ui, look));
}

/// One icon-or-labelled button; true when it was pressed.
fn button(ui: &mut egui::Ui, look: Look, part_id: &str, p: ButtonProps) -> bool {
    let mut b = Button::new(p);
    let evs = as_part(ui, &[(button_parts::BUTTON, part_id)], |ui| b.ui(ui, look));
    evs.iter().any(|e| matches!(e, ButtonEvent::Activated { .. }))
}

/// Lays out content inside the inspector's side padding.
fn padded(ui: &mut egui::Ui, pad: f32, content: impl FnOnce(&mut egui::Ui)) {
    ui.horizontal(|ui| {
        ui.add_space(pad);
        ui.vertical(|ui| {
            ui.set_max_width((ui.available_width() - pad).max(0.0));
            content(ui);
        });
    });
}

fn mentions_json(atts: &[Attachment]) -> String {
    let list: Vec<serde_json::Value> = atts.iter().map(|a| serde_json::json!({ "id": a.id, "name": a.name })).collect();
    serde_json::to_string(&list).unwrap_or_default()
}

fn is_remote(a: &Attachment) -> bool {
    matches!(a.link_type, LinkType::Reference)
}

fn is_image(name: &str) -> bool {
    let n = name.to_lowercase();
    [".png", ".jpg", ".jpeg", ".gif", ".webp", ".svg", ".bmp", ".avif"].iter().any(|e| n.ends_with(e))
}

fn now_ms() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

/// "5m ago", "2d ago" and so on, as the web's relative time.
fn relative(ms: i64, now: i64) -> String {
    crate::time::relative(ms, now)
}

/// The test's mount: the app on the props' data set, with the props' task shown in its list.
struct Mount {
    app: App,
    screen: TaskInspector,
}

impl Mounted for Mount {
    fn frame(&mut self, ui: &mut egui::Ui, look: Look, emit: &mut dyn FnMut(&'static str, serde_json::Value)) {
        for e in self.screen.show(&mut self.app, ui, look) {
            let (n, f) = e.to_json();
            emit(n, f);
        }
    }
}

pub fn mount(mut app: App, props: serde_json::Value) -> Result<Box<dyn Mounted>, String> {
    let p: TaskinspectorProps = serde_json::from_value(props).map_err(|e| e.to_string())?;
    let found = app.data.task_named(&p.task).cloned().ok_or_else(|| format!("No task titled \"{}\" in the data set", p.task))?;
    if let Some(project) = app.data.list(&found.list_id).map(|l| l.project_id.clone()) {
        app.ui.set_project(Some(project));
    }
    app.ui.open_list(Some(found.list_id.clone()), Some(0));
    app.ui.preview = Some(found.id.clone());
    Ok(Box::new(Mount { app, screen: TaskInspector::new() }))
}

/// Where a thumbnail loads from: a copied file's location is relative to the data folder.
fn thumb_path(location: &str) -> String {
    if location.contains("://") || std::path::Path::new(location).is_absolute() {
        location.to_owned()
    } else {
        taskscape_storage::paths::root_dir().join(location).display().to_string()
    }
}

/// A tile's name: one line, centred, cut with an ellipsis (the reference's truncated 11 px label).
fn caption(ui: &mut egui::Ui, look: Look, width: f32, name: &str) {
    let mut job = egui::text::LayoutJob::single_section(name.to_owned(), egui::TextFormat { font_id: FontId::new(11.0, weight(400)), color: look.t.content_2, ..Default::default() });
    job.wrap.max_width = width;
    job.wrap.max_rows = 1;
    job.wrap.break_anywhere = true;
    job.wrap.overflow_character = Some('…');
    let g = ui.painter().layout_job(job);
    let (rect, resp) = ui.allocate_exact_size(vec2(width, g.size().y + 2.0), Sense::hover());
    resp.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, name));
    part(ui, &resp, "app.taskinspector.attachmentCaption");
    ui.painter().galley(pos2(rect.center().x - g.size().x / 2.0, rect.top() + 2.0), g, look.t.content_2);
}

/// One half of the empty attachments box: a press target with its icon over its label.
fn empty_choice(ui: &mut egui::Ui, look: Look, rect: Rect, glyph: &str, label: &str, part_id: &str) -> bool {
    let t = look.t;
    let resp = ui.interact(rect.shrink(4.0), egui::Id::new(("inspector.empty", part_id)), Sense::click());
    resp.widget_info(|| WidgetInfo::labeled(Role::Button, true, label));
    part(ui, &resp, part_id);
    let ink = if resp.hovered() { t.content_1 } else { t.content_3 };
    if resp.hovered() {
        ui.painter().rect_filled(rect.shrink(4.0), CornerRadius::same(5), t.wash_1);
    }
    if resp.has_focus() {
        ui.painter().rect_stroke(rect.shrink(4.0), CornerRadius::same(5), Stroke::new(2.0, t.focus_1), StrokeKind::Inside);
    }
    let c = rect.center();
    ui.painter().text(pos2(c.x, c.y - 10.0), egui::Align2::CENTER_CENTER, icon(glyph, 18.0).text(), FontId::new(18.0, FontFamily::Name("Icons".into())), ink);
    ui.painter().text(pos2(c.x, c.y + 12.0), egui::Align2::CENTER_CENTER, label, FontId::new(rem(0.929), weight(600)), ink);
    resp.clicked()
}

/// A rounded rectangle's outline in dashes (egui has no dashed stroke for rects).
fn dashed_rect(p: &egui::Painter, r: Rect, radius: f32, stroke: Stroke) {
    let (dash, gap) = (4.0, 3.0);
    let segs = [
        [pos2(r.left() + radius, r.top()), pos2(r.right() - radius, r.top())],
        [pos2(r.right(), r.top() + radius), pos2(r.right(), r.bottom() - radius)],
        [pos2(r.right() - radius, r.bottom()), pos2(r.left() + radius, r.bottom())],
        [pos2(r.left(), r.bottom() - radius), pos2(r.left(), r.top() + radius)],
    ];
    for seg in segs {
        p.extend(egui::Shape::dashed_line(&seg, stroke, dash, gap));
    }
    // The corners: quarter arcs, drawn solid (too short to dash).
    for (c, a0) in [(pos2(r.right() - radius, r.top() + radius), -90.0f32), (pos2(r.right() - radius, r.bottom() - radius), 0.0), (pos2(r.left() + radius, r.bottom() - radius), 90.0), (pos2(r.left() + radius, r.top() + radius), 180.0)] {
        let pts: Vec<egui::Pos2> = (0..=6).map(|i| {
            let a = (a0 + 15.0 * i as f32).to_radians();
            pos2(c.x + radius * a.cos(), c.y + radius * a.sin())
        }).collect();
        p.add(egui::Shape::line(pts, stroke));
    }
}
