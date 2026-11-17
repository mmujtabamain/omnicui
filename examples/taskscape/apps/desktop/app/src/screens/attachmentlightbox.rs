//! contract: omnicui/contracts/attachmentlightbox.contract.yaml (id: app.attachmentlightbox). One attachment of a task in its own window, with no scrim: the top bar (kind, name, counter, open, rename, remove, close), the stage with its chevrons, and the rename and remove sheets.
use crate::data::{Id, LinkType};
use crate::App;
use serde_json::json;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use taskscape_storage::Attachment;
use taskscape_ui::omnicui::attachmentlightbox::{parts, AttachmentlightboxEvent, AttachmentlightboxProps};
use taskscape_ui::omnicui::attachmentpreview::{parts as preview_parts, AttachmentpreviewEvent, AttachmentpreviewProps};
use taskscape_ui::omnicui::button::{parts as button_parts, ButtonEvent, ButtonProps};
use taskscape_ui::omnicui::icon::{parts as icon_parts, IconProps};
use taskscape_ui::omnicui::input::{parts as input_parts, InputEvent, InputProps};
use taskscape_ui::omnicui::overlay::{OverlayEvent, OverlayProps};
use taskscape_ui::widgets::attachmentpreview::AttachmentPreview;
use taskscape_ui::widgets::button::Button;
use taskscape_ui::widgets::icon::Icon;
use taskscape_ui::widgets::input::Input;
use taskscape_ui::widgets::overlay::Overlay;
use taskscape_ui::{as_part, icon as glyph_text, part, rem, weight, Look, Mounted};
use egui::text::{LayoutJob, TextFormat};
use egui::{pos2, vec2, Align, Align2, Color32, CornerRadius, FontFamily, FontId, Frame, Galley, Key, Layout, Margin, Modifiers, Pos2, Rect, Role, Sense, Shadow, Stroke, StrokeKind, UiBuilder, Vec2, WidgetInfo};

/// The rename and remove sheets are 26rem wide; each is as tall as its rows (the rename sheet has a field row).
const SHEET_W: f32 = rem(26.0);
const RENAME_H: f32 = 128.0;
const REMOVE_H: f32 = 95.5;
/// The No inline preview card is 24rem wide, and as tall as its padding and rows.
const CARD_W: f32 = rem(24.0);
const CARD_H: f32 = 230.0;
/// The audio panel, at its content.
const AUDIO_H: f32 = 150.0;

/// The lightbox: one attachment of a task, with its top bar, its stage and its sheets. The screen keeps
/// which task and attachment it shows and whether it is open; the main window opens it with `open_on`.
pub struct AttachmentLightbox {
    /// The task whose attachments are stepped through, and the attachment shown (both by id).
    task: Option<Id>,
    current: Option<Id>,
    /// Shown. A close clears it; the window is then the main window's to close.
    open: bool,
    /// The sheet over the lightbox, its layer, and the rename prompt's field.
    sheet: Option<Sheet>,
    overlay: Option<Overlay>,
    rename_field: Option<Input>,
    /// The rename prompt's base name as it is typed (what Rename accepts).
    rename_text: String,
    /// The preview of the shown attachment, kept while it stays the same attachment under the same name.
    shown: Option<Shown>,
    open_external: Button,
    rename_button: Button,
    close_button: Button,
    previous: Button,
    next: Button,
    card_open: Button,
    rename_cancel: Button,
    rename_accept: Button,
    remove_cancel: Button,
    remove_accept: Button,
}

/// The two sheets the lightbox can lay over itself.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Sheet {
    Rename,
    Remove,
}

/// What the stage shows for the attachment: the preview widget, or the No inline preview card with its status.
enum Body {
    Preview(AttachmentPreview),
    Card(&'static str),
}

struct Shown {
    id: String,
    name: String,
    body: Body,
}

/// A thing the user did this frame, applied once the frame has been drawn.
enum Cmd {
    Close(&'static str),
    Step(i32),
    OpenExternal,
    OpenRename,
    OpenRemove,
    SheetCancel,
    RenameSubmit(String),
    ConfirmRemove,
}

impl Default for AttachmentLightbox {
    fn default() -> Self {
        Self::new()
    }
}

impl AttachmentLightbox {
    pub fn new() -> Self {
        Self {
            task: None,
            current: None,
            open: true,
            sheet: None,
            overlay: None,
            rename_field: None,
            rename_text: String::new(),
            shown: None,
            open_external: mk_button(json!({"surface": "media", "variant": "ghost", "shape": "icon", "size": "xl", "icon": "open_in_new", "ariaLabel": "Open externally", "tabbable": true})),
            rename_button: mk_button(json!({"surface": "media", "variant": "ghost", "shape": "icon", "size": "xl", "icon": "drive_file_rename_outline", "ariaLabel": "Rename", "tabbable": true})),
            close_button: mk_button(json!({"surface": "media", "variant": "ghost", "shape": "icon", "size": "xl", "icon": "close", "ariaLabel": "Close", "tabbable": true})),
            previous: mk_button(json!({"surface": "media", "variant": "filled", "shape": "round", "size": "step", "icon": "chevron_left", "ariaLabel": "Previous", "tabbable": true})),
            next: mk_button(json!({"surface": "media", "variant": "filled", "shape": "round", "size": "step", "icon": "chevron_right", "ariaLabel": "Next", "tabbable": true})),
            card_open: mk_button(json!({"variant": "primary", "width": "content", "shape": "text", "size": "md", "icon": "open_in_new", "label": "Open externally", "tabbable": true})),
            rename_cancel: mk_button(json!({"variant": "neutral", "width": "content", "shape": "text", "size": "md", "label": "Cancel", "tabbable": true})),
            rename_accept: mk_button(json!({"variant": "primary", "width": "content", "shape": "text", "size": "md", "label": "Rename", "tabbable": true})),
            remove_cancel: mk_button(json!({"variant": "neutral", "width": "content", "shape": "text", "size": "md", "label": "Cancel", "tabbable": true})),
            remove_accept: mk_button(json!({"variant": "danger", "width": "content", "shape": "text", "size": "md", "label": "Remove", "tabbable": true})),
        }
    }

    /// Opens the lightbox on an attachment of a task (both by id), as the main window does and a test's props say.
    pub fn open_on(&mut self, task: Id, attachment: Id) {
        self.task = Some(task);
        self.current = Some(attachment);
        self.open = true;
        self.shown = None;
        self.close_sheet();
    }

    /// One frame: draws the lightbox over the window and returns the events it raised in this frame.
    pub fn show(&mut self, app: &mut App, ui: &mut egui::Ui, look: Look) -> Vec<AttachmentlightboxEvent> {
        let mut events = vec![];
        if !self.open {
            return events;
        }
        let Some((atts, index)) = self.listing(app) else {
            return events;
        };
        let cur = atts[index].clone();
        let count = atts.len();
        let kind = kind_of(&cur);
        let full = ui.available_rect_before_wrap();
        let mut cmds: Vec<Cmd> = vec![];

        // Escape closes the lightbox, and the arrow keys step it (wrapping), while no sheet is open.
        if self.sheet.is_none() {
            if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
                cmds.push(Cmd::Close("escape"));
            }
            if count > 1 {
                if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowRight)) {
                    cmds.push(Cmd::Step(1));
                }
                if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowLeft)) {
                    cmds.push(Cmd::Step(-1));
                }
            }
        }

        // The window's own backdrop, dark as the web's scrim is. The desktop draws no scrim (decision 28).
        ui.painter().rect_filled(full, CornerRadius::ZERO, Color32::from_black_alpha(214));
        let top = Rect::from_min_size(full.min, vec2(full.width(), rem(3.25)));
        let stage = Rect::from_min_max(
            pos2(full.left(), top.bottom()),
            pos2(full.right(), (full.bottom() - rem(2.0)).max(top.bottom())),
        );

        self.top_bar(ui, look, top, &cur, kind, index, count, &mut cmds);

        // The stage: a press on empty stage closes the lightbox, as the web's does. The media takes its own presses.
        let stage_resp = ui.interact(stage, ui.scope_id().with("app.attachmentlightbox.stage"), Sense::click());
        if stage_resp.clicked() && self.sheet.is_none() {
            cmds.push(Cmd::Close("stage"));
        }
        let area = Rect::from_center_size(stage.center(), media_size(kind, stage));
        let _ = ui.interact(area, ui.scope_id().with("app.attachmentlightbox.media"), Sense::click());
        let root = data_root();
        self.media(ui, look, area, &cur, kind, &root, &mut cmds);

        // The chevrons: drawn only with more than one attachment, at the stage's edges (left-4 and right-4).
        if count > 1 {
            let y = stage.center().y - 19.5;
            let prev = Rect::from_min_size(pos2(stage.left() + rem(1.0), y), vec2(39.0, 39.0));
            let next = Rect::from_min_size(pos2(stage.right() - rem(1.0) - 39.0, y), vec2(39.0, 39.0));
            if btn_in(ui, look, prev, parts::PREVIOUS, &mut self.previous, false, false) {
                cmds.push(Cmd::Step(-1));
            }
            if btn_in(ui, look, next, parts::NEXT, &mut self.next, false, false) {
                cmds.push(Cmd::Step(1));
            }
        }

        let mut repaint = !cmds.is_empty();
        for c in cmds {
            if !self.open {
                break;
            }
            self.apply(app, c, &mut events);
        }
        if !self.open {
            return events;
        }

        // The sheet, over the lightbox, laid on this frame if the top bar opened it.
        let mut sheet_cmds: Vec<Cmd> = vec![];
        if let Some(sheet) = self.sheet {
            if let Some(mut overlay) = self.overlay.take() {
                let shown = overlay.show(ui, look, |ui, focus_first| {
                    self.sheet_card(ui, look, sheet, &cur, focus_first, &mut sheet_cmds);
                });
                let dismissed = shown
                    .iter()
                    .any(|e| matches!(e, OverlayEvent::Dismissed { .. } | OverlayEvent::OpenChanged { .. }));
                self.overlay = Some(overlay);
                if dismissed {
                    sheet_cmds.push(Cmd::SheetCancel);
                }
            }
        }
        repaint |= !sheet_cmds.is_empty();
        for c in sheet_cmds {
            if !self.open {
                break;
            }
            self.apply(app, c, &mut events);
        }
        if repaint {
            ui.ctx().request_repaint();
        }
        events
    }

    /// The task's attachments, and the index of the one shown, while it is still there.
    fn listing(&self, app: &App) -> Option<(Vec<Attachment>, usize)> {
        let task = self.task.as_ref()?;
        let atts = app.data.attachments_of(task);
        let index = atts.iter().position(|a| Some(&a.id) == self.current.as_ref())?;
        Some((atts, index))
    }

    /// Applies one thing the user did, through the app's own data methods.
    fn apply(&mut self, app: &mut App, cmd: Cmd, events: &mut Vec<AttachmentlightboxEvent>) {
        match cmd {
            Cmd::OpenRename => {
                let Some((atts, index)) = self.listing(app) else {
                    return;
                };
                let (base, _) = split_name(&atts[index]);
                self.rename_text = base.clone();
                self.rename_field = Some(rename_input(&base));
                self.sheet = Some(Sheet::Rename);
                self.overlay = Some(sheet_overlay());
            }
            Cmd::OpenRemove => {
                self.sheet = Some(Sheet::Remove);
                self.overlay = Some(sheet_overlay());
            }
            Cmd::SheetCancel => self.close_sheet(),
            other => {
                let Some((atts, index)) = self.listing(app) else {
                    return;
                };
                let cur = atts[index].clone();
                match other {
                    Cmd::Close(via) => {
                        self.open = false;
                        events.push(AttachmentlightboxEvent::Closed { via: via.to_owned(), name: cur.name.clone() });
                    }
                    Cmd::Step(delta) => {
                        if atts.len() < 2 {
                            return;
                        }
                        let n = atts.len() as i64;
                        let next = (index as i64 + delta as i64).rem_euclid(n) as usize;
                        self.current = Some(atts[next].id.clone());
                        events.push(AttachmentlightboxEvent::Stepped { name: atts[next].name.clone() });
                    }
                    Cmd::OpenExternal => {
                        open_in_os(&source(&data_root(), &cur));
                        events.push(AttachmentlightboxEvent::OpenedExternally { name: cur.name.clone() });
                    }
                    Cmd::RenameSubmit(value) => {
                        self.close_sheet();
                        let typed = value.trim().to_owned();
                        let (base, tail) = split_name(&cur);
                        if typed.is_empty() || typed == base {
                            return;
                        }
                        let full = format!("{typed}{tail}");
                        if full == cur.name {
                            return;
                        }
                        match app.data.rename_attachment(&cur.id, &full) {
                            Ok(_) => events.push(AttachmentlightboxEvent::Renamed { name: full }),
                            Err(e) => app.refused(e),
                        }
                    }
                    Cmd::ConfirmRemove => {
                        self.close_sheet();
                        // The next attachment takes its place, or the last one before it; with none left, the lightbox closes.
                        let rest: Vec<&Attachment> = atts.iter().filter(|a| a.id != cur.id).collect();
                        let next = if rest.is_empty() { None } else { Some(rest[index.min(rest.len() - 1)].id.clone()) };
                        match app.data.delete_attachment(&cur.id) {
                            Ok(()) => {
                                events.push(AttachmentlightboxEvent::Removed { name: cur.name.clone() });
                                match next {
                                    Some(id) => self.current = Some(id),
                                    None => self.open = false,
                                }
                            }
                            Err(e) => app.refused(e),
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn close_sheet(&mut self) {
        self.sheet = None;
        self.overlay = None;
        self.rename_field = None;
    }

    /// The top bar: the kind glyph, the file name, the counter, and the four actions at the right.
    #[allow(clippy::too_many_arguments)]
    fn top_bar(&mut self, ui: &mut egui::Ui, look: Look, top: Rect, cur: &Attachment, kind: &'static str, index: usize, count: usize, cmds: &mut Vec<Cmd>) {
        let cy = top.center().y;
        let pad = rem(0.857);
        let gap = rem(0.429);
        let step = 28.0 + rem(0.357);
        let group_l = top.right() - pad - (4.0 * 28.0 + 3.0 * rem(0.357));
        let slot = |i: f32| Rect::from_min_size(pos2(group_l + i * step, cy - 14.0), vec2(28.0, 28.0));

        // The kind glyph: 16px, onMediaMuted.
        let kind_r = Rect::from_min_size(pos2(top.left() + pad, cy - 8.0), vec2(16.0, 16.0));
        ui.scope_builder(UiBuilder::new().max_rect(kind_r), |ui| {
            as_part(ui, &[(icon_parts::GLYPH, parts::KINDICON)], |ui| {
                mk_icon(json!({"name": kind_glyph(kind), "size": "md", "tone": "onMediaMuted", "tint": "none"})).ui(ui, look)
            });
        });

        // The counter, 12px and muted, sits left of the actions with its margin; the name fills what is left.
        let mut name_r = group_l - gap;
        if count > 1 {
            let label = format!("{} / {}", index + 1, count);
            let muted = Color32::from_white_alpha(153);
            let g = galley(ui, &label, 12.0, 400, muted, f32::INFINITY, true);
            let at = pos2(group_l - gap - rem(0.25) - g.size().x, cy - g.size().y / 2.0);
            name_r = at.x - gap;
            text_at(ui, at, g, &label, Role::GenericContainer, parts::COUNTER, muted);
        }
        let name_l = kind_r.right() + gap;
        let g = galley(ui, &cur.name, 13.0, 400, Color32::WHITE, (name_r - name_l).max(0.0), true);
        let at = pos2(name_l, cy - g.size().y / 2.0);
        text_at(ui, at, g, &cur.name, Role::GenericContainer, parts::FILENAME, Color32::WHITE);

        if btn_in(ui, look, slot(0.0), parts::OPENEXTERNALLY, &mut self.open_external, false, false) {
            cmds.push(Cmd::OpenExternal);
        }
        if btn_in(ui, look, slot(1.0), parts::RENAME, &mut self.rename_button, false, false) {
            cmds.push(Cmd::OpenRename);
        }
        remove_button(ui, look, slot(2.0), cmds);
        if btn_in(ui, look, slot(3.0), parts::CLOSE, &mut self.close_button, false, false) {
            cmds.push(Cmd::Close("button"));
        }
    }

    /// The stage's media: the preview widget for a file it can draw, else the No inline preview card.
    fn media(&mut self, ui: &mut egui::Ui, look: Look, area: Rect, cur: &Attachment, kind: &'static str, root: &Path, cmds: &mut Vec<Cmd>) {
        self.ensure_shown(cur, kind, root);
        let Some(shown) = self.shown.as_mut() else {
            return;
        };
        let card_status = match &mut shown.body {
            Body::Preview(preview) => {
                let evs = ui
                    .scope_builder(UiBuilder::new().max_rect(area), |ui| {
                        as_part(ui, &[(preview_parts::IMAGE, parts::IMAGE)], |ui| preview.ui(ui, look))
                    })
                    .inner;
                // A file the preview could not draw shows the card instead, as the web's broken image does.
                evs.iter().any(|e| matches!(e, AttachmentpreviewEvent::Error { .. })).then_some("No inline preview")
            }
            Body::Card(status) => Some(*status),
        };
        if let Some(status) = card_status {
            shown.body = Body::Card(status);
            card(ui, look, area, kind, &cur.name, status, &mut self.card_open, cmds);
        }
    }

    /// Makes the preview for the attachment shown, unless it is already the one.
    fn ensure_shown(&mut self, cur: &Attachment, kind: &'static str, root: &Path) {
        if let Some(s) = &self.shown {
            if s.id == cur.id && s.name == cur.name {
                return;
            }
        }
        let body = match kind {
            "image" | "video" | "audio" | "pdf" => Body::Preview(mk_preview(json!({"kind": kind, "src": source(root, cur), "name": cur.name}))),
            "text" => match std::fs::read_to_string(source(root, cur)) {
                Ok(text) => Body::Preview(mk_preview(json!({"kind": "text", "text": text, "name": cur.name}))),
                Err(_) => Body::Card("Couldn’t read this file"),
            },
            _ => Body::Card("No inline preview"),
        };
        self.shown = Some(Shown { id: cur.id.clone(), name: cur.name.clone(), body });
    }

    /// The sheet's card: 26rem, centred, with the padding the web's sheet has (15, 10, 10, 15).
    fn sheet_card(&mut self, ui: &mut egui::Ui, look: Look, sheet: Sheet, cur: &Attachment, focus_first: bool, cmds: &mut Vec<Cmd>) {
        let t = look.t;
        let screen = ui.ctx().content_rect();
        let h = match sheet {
            Sheet::Rename => RENAME_H,
            Sheet::Remove => REMOVE_H,
        };
        let card_r = Rect::from_center_size(screen.center(), vec2(SHEET_W, h));
        let frame = Frame::new()
            .fill(t.surface_2)
            .stroke(Stroke::new(1.0, t.edge_1))
            .corner_radius(CornerRadius::same(10))
            .shadow(Shadow { offset: [0, 4], blur: 16, spread: 0, color: Color32::from_black_alpha(64) })
            .inner_margin(Margin { left: 15, right: 10, top: 15, bottom: 10 });
        ui.scope_builder(UiBuilder::new().max_rect(card_r), |ui| {
            frame.show(ui, |ui| {
                ui.set_width(SHEET_W - 25.0);
                ui.spacing_mut().item_spacing = vec2(0.0, rem(0.571));
                self.sheet_body(ui, look, sheet, cur, focus_first, cmds);
            });
        });
    }

    fn sheet_body(&mut self, ui: &mut egui::Ui, look: Look, sheet: Sheet, cur: &Attachment, focus_first: bool, cmds: &mut Vec<Cmd>) {
        let t = look.t;
        let (_, tail) = split_name(cur);
        let width = ui.available_width();
        let (glyph_name, tint, heading, detail) = match sheet {
            Sheet::Rename => (
                "drive_file_rename_outline",
                "accent",
                "Rename attachment",
                if is_link(cur) {
                    "Renames the label only — the linked file is untouched.".to_owned()
                } else {
                    "The stored file is renamed to match.".to_owned()
                },
            ),
            Sheet::Remove => (
                "delete_forever",
                "danger",
                "Remove attachment?",
                format!("“{}” — attachments skip the Trash.", cur.name),
            ),
        };

        // The header: the badge, the headline and the detail line.
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = rem(0.571);
            mk_icon(json!({"name": glyph_name, "tint": tint})).ui(ui, look);
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = rem(0.286);
                let w = ui.available_width();
                text_flow(ui, heading, rem(1.071), 600, t.content_1, w, false, parts::SHEETHEADING, Role::Heading);
                text_flow(ui, &detail, rem(0.929), 400, t.content_2, w, false, parts::SHEETDETAIL, Role::GenericContainer);
            });
        });

        match sheet {
            Sheet::Rename => {
                // The field: the base name, with the extension beside it (none for a link).
                let (row, _) = ui.allocate_exact_size(vec2(width, rem(1.75)), Sense::hover());
                ui.painter().rect(row, CornerRadius::same(5), t.surface_3, Stroke::new(1.0, t.edge_2), StrokeKind::Inside);
                let inner = row.shrink2(vec2(rem(0.571), 0.0));
                let tail_g = (!tail.is_empty()).then(|| galley(ui, &tail, 13.0, 400, t.content_3, f32::INFINITY, true));
                let tail_w = tail_g.as_ref().map_or(0.0, |g| g.size().x + rem(0.143));
                let field = Rect::from_min_size(
                    pos2(inner.left(), inner.center().y - 10.5),
                    vec2((inner.width() - tail_w).max(0.0), 21.0),
                );
                if let Some(input) = self.rename_field.as_mut() {
                    let evs = ui
                        .scope_builder(UiBuilder::new().max_rect(field), |ui| {
                            as_part(ui, &[(input_parts::FIELD, parts::RENAMEFIELD)], |ui| input.ui(ui, look))
                        })
                        .inner;
                    for e in evs {
                        match e {
                            InputEvent::ValueChange { value } => self.rename_text = value,
                            InputEvent::Submit { value } => cmds.push(Cmd::RenameSubmit(value)),
                            _ => {}
                        }
                    }
                }
                if let Some(g) = tail_g {
                    let at = pos2(inner.right() - g.size().x, inner.center().y - g.size().y / 2.0);
                    text_at(ui, at, g, &tail, Role::GenericContainer, parts::SHEETTAIL, t.content_3);
                }

                // The footer: Cancel, then Rename (the primary action), right-aligned.
                let (row, _) = ui.allocate_exact_size(vec2(width, rem(1.75)), Sense::hover());
                let (accept, cancel) = footer_rects(ui, row, "Rename", "Cancel");
                if btn_in(ui, look, cancel, parts::SHEETCANCEL, &mut self.rename_cancel, false, false) {
                    cmds.push(Cmd::SheetCancel);
                }
                if btn_in(ui, look, accept, parts::SHEETACCEPT, &mut self.rename_accept, false, false) {
                    cmds.push(Cmd::RenameSubmit(self.rename_text.clone()));
                }
            }
            Sheet::Remove => {
                // The footer: Cancel (the default focus) and Remove (danger). Enter does not confirm (decision 10).
                let (row, _) = ui.allocate_exact_size(vec2(width, rem(1.75)), Sense::hover());
                let (accept, cancel) = footer_rects(ui, row, "Remove", "Cancel");
                if btn_in(ui, look, cancel, parts::SHEETCANCEL, &mut self.remove_cancel, focus_first, false) {
                    cmds.push(Cmd::SheetCancel);
                }
                if btn_in(ui, look, accept, parts::SHEETACCEPT, &mut self.remove_accept, false, true) {
                    cmds.push(Cmd::ConfirmRemove);
                }
            }
        }
    }
}

/// The mounted screen: the app on the props' data set, with the lightbox over the props' attachment.
struct Mount {
    app: App,
    screen: AttachmentLightbox,
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
    let p: AttachmentlightboxProps = serde_json::from_value(props).map_err(|e| e.to_string())?;
    let task = app
        .data
        .task_named(&p.task)
        .map(|t| t.id.clone())
        .ok_or_else(|| format!("task \"{}\" is not in the data set", p.task))?;
    let attachment = app
        .data
        .attachments_of(&task)
        .into_iter()
        .find(|a| a.name == p.attachment)
        .map(|a| a.id)
        .ok_or_else(|| format!("attachment \"{}\" is not on the task", p.attachment))?;
    let mut screen = AttachmentLightbox::new();
    screen.open_on(task, attachment);
    Ok(Box::new(Mount { app, screen }))
}

// ---- helpers --------------------------------------------------------------------------------------

fn mk_button(v: serde_json::Value) -> Button {
    Button::new(serde_json::from_value::<ButtonProps>(v).expect("button props"))
}

fn mk_icon(v: serde_json::Value) -> Icon {
    Icon::new(serde_json::from_value::<IconProps>(v).expect("icon props"))
}

fn mk_preview(v: serde_json::Value) -> AttachmentPreview {
    AttachmentPreview::new(serde_json::from_value::<AttachmentpreviewProps>(v).expect("preview props"))
}

fn rename_input(base: &str) -> Input {
    Input::new(
        serde_json::from_value::<InputProps>(json!({
            "mode": "bare",
            "size": "md",
            "value": base,
            "ariaLabel": "Attachment name",
            "autoFocus": true,
            "selectOnOpen": false,
            "selectOnFocus": false
        }))
        .expect("input props"),
    )
}

/// The layer under a sheet: it covers the lightbox, holds focus inside, and takes Escape (a cancel).
fn sheet_overlay() -> Overlay {
    Overlay::new(
        serde_json::from_value::<OverlayProps>(json!({
            "open": true,
            "placement": "fill",
            "dismissOn": "none",
            "captureEscape": true
        }))
        .expect("overlay props"),
    )
}

/// Whether a button was pressed this frame.
fn pressed(events: &[ButtonEvent]) -> bool {
    events.iter().any(|e| matches!(e, ButtonEvent::Activated { .. }))
}

/// A library button drawn at `rect` under the screen's part name. `focus` takes keyboard focus this frame
/// (the remove prompt's default focus, Cancel); `no_enter` keeps Enter from pressing it (decision 10).
fn btn_in(ui: &mut egui::Ui, look: Look, rect: Rect, part_id: &'static str, b: &mut Button, focus: bool, no_enter: bool) -> bool {
    let events = ui
        .scope_builder(UiBuilder::new().max_rect(rect), |ui| btn_here(ui, look, part_id, b, focus, no_enter))
        .inner;
    pressed(&events)
}

fn btn_here(ui: &mut egui::Ui, look: Look, part_id: &'static str, b: &mut Button, focus: bool, no_enter: bool) -> Vec<ButtonEvent> {
    // The id the button's own allocation takes, known before it draws.
    let id = ui.next_auto_id();
    if focus {
        ui.ctx().memory_mut(|m| m.request_focus(id));
    }
    if no_enter && ui.memory(|m| m.has_focus(id)) {
        let _ = ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Enter));
    }
    as_part(ui, &[(button_parts::BUTTON, part_id)], |ui| b.ui(ui, look))
}

/// The remove action in the top bar: a 28px icon button whose glyph is danger-coloured at rest and white on a
/// danger fill when hovered (the review decision). Drawn here because the library button's media glyph is white.
fn remove_button(ui: &mut egui::Ui, look: Look, rect: Rect, cmds: &mut Vec<Cmd>) {
    let t = look.t;
    let resp = ui.interact(rect, ui.scope_id().with("app.attachmentlightbox.remove"), Sense::click());
    resp.widget_info(|| WidgetInfo::labeled(Role::Button, true, "Remove attachment"));
    part(ui, &resp, parts::REMOVE);
    let hot = resp.hovered();
    let painter = ui.painter();
    if hot {
        painter.rect_filled(rect, CornerRadius::same(7), t.danger_500);
    }
    if resp.has_focus() {
        painter.rect_stroke(rect.expand(2.0), CornerRadius::same(9), Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
    }
    let glyph_color = if hot { Color32::WHITE } else { t.danger_500 };
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        glyph_text("delete", 17.0).text(),
        FontId::new(17.0, FontFamily::Name("Icons".into())),
        glyph_color,
    );
    if resp.clicked() {
        cmds.push(Cmd::OpenRemove);
    }
}

/// The No inline preview card: a surface-2 panel with the kind's glyph, the name, the status and Open externally.
fn card(ui: &mut egui::Ui, look: Look, area: Rect, kind: &str, name: &str, status: &str, open: &mut Button, cmds: &mut Vec<Cmd>) {
    let t = look.t;
    let r = Rect::from_center_size(area.center(), vec2(area.width().min(CARD_W), CARD_H.min(area.height())));
    ui.painter().rect_filled(r, CornerRadius::same(11), t.surface_2);
    // px-10 (40px) and py-12 (48px).
    let inner = r.shrink2(vec2(40.0, 48.0));
    let glyph_name = kind_glyph(kind);
    ui.scope_builder(UiBuilder::new().max_rect(inner).layout(Layout::top_down(Align::Center)), |ui| {
        ui.spacing_mut().item_spacing = vec2(0.0, rem(0.714));
        mk_icon(json!({"name": glyph_name, "size": "display", "tone": "muted", "tint": "none"})).ui(ui, look);
        let w = inner.width().min(rem(20.0));
        text_flow(ui, name, rem(1.0), 400, t.content_1, w, true, parts::FALLBACKNAME, Role::GenericContainer);
        text_flow(ui, status, rem(0.857), 400, t.content_3, w, false, parts::FALLBACKSTATUS, Role::GenericContainer);
        ui.add_space(rem(0.25));
        if pressed(&btn_here(ui, look, parts::FALLBACKOPEN, open, false, false)) {
            cmds.push(Cmd::OpenExternal);
        }
    });
}

/// The stage size of the media for a kind: the preview widget fits its own shape into it.
fn media_size(kind: &str, stage: Rect) -> Vec2 {
    let fit = |max_w: f32, ratio: f32| {
        let mut w = stage.width().min(max_w);
        let mut h = w * ratio;
        if h > stage.height() {
            h = stage.height();
            w = h / ratio;
        }
        vec2(w, h)
    };
    match kind {
        "image" => fit(rem(34.286), 10.0 / 16.0),
        "video" => fit(rem(34.286), 9.0 / 16.0),
        "link" | "file" => vec2(stage.width().min(CARD_W), CARD_H.min(stage.height())),
        "audio" => vec2(stage.width().min(rem(28.0)), AUDIO_H.min(stage.height())),
        _ => vec2(stage.width().min(rem(48.0)), stage.height().min(rem(34.0))),
    }
}

/// The two footer buttons' rects: Accept at the right edge, Cancel left of it, both as wide as their labels.
fn footer_rects(ui: &egui::Ui, row: Rect, accept: &str, cancel: &str) -> (Rect, Rect) {
    let aw = content_w(ui, accept);
    let cw = content_w(ui, cancel);
    let a = Rect::from_min_size(pos2(row.right() - aw, row.top()), vec2(aw, row.height()));
    let c = Rect::from_min_size(pos2(a.left() - rem(0.429) - cw, row.top()), vec2(cw, row.height()));
    (a, c)
}

/// A content-width button's width: its label at 13px medium, with 8px padding each side.
fn content_w(ui: &egui::Ui, label: &str) -> f32 {
    galley(ui, label, 13.0, 500, Color32::WHITE, f32::INFINITY, true).size().x + 16.0
}

fn kind_glyph(kind: &str) -> &'static str {
    match kind {
        "image" => "image",
        "video" => "movie",
        "audio" => "audio_file",
        "pdf" => "picture_as_pdf",
        "text" => "description",
        "link" => "link",
        _ => "draft",
    }
}

/// What a row is drawn as (the web's kinds.ts): by extension, or a link.
fn kind_of(a: &Attachment) -> &'static str {
    if is_link(a) {
        return "link";
    }
    match extension_of(&a.name).as_str() {
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "bmp" | "avif" => "image",
        "mp4" | "webm" | "mov" | "m4v" => "video",
        "mp3" | "wav" | "ogg" | "m4a" | "flac" | "aac" => "audio",
        "pdf" => "pdf",
        "txt" | "md" | "csv" | "json" | "log" => "text",
        _ => "file",
    }
}

fn extension_of(name: &str) -> String {
    match name.rfind('.') {
        Some(dot) if dot > 0 => name[dot + 1..].to_lowercase(),
        _ => String::new(),
    }
}

/// The base name shown in the rename field, and the extension beside it (empty for a link).
fn split_name(a: &Attachment) -> (String, String) {
    if is_link(a) {
        return (a.name.clone(), String::new());
    }
    match a.name.rfind('.') {
        Some(dot) if dot > 0 => (a.name[..dot].to_owned(), a.name[dot..].to_owned()),
        _ => (a.name.clone(), String::new()),
    }
}

/// Whether the attachment is a remote link (the web's kind "link").
fn is_link(a: &Attachment) -> bool {
    matches!(a.link_type, LinkType::Reference)
}

/// Where the attachment's file, or a link's address, is.
fn source(root: &Path, a: &Attachment) -> String {
    if is_link(a) {
        return a.location.clone();
    }
    root.join(&a.location).to_string_lossy().into_owned()
}

/// The folder an attachment's location is relative to: the store's data folder, which the store opens in
/// TASKSCAPE_DATA_DIR, or ~/.taskscape when that is not set (data.rs).
fn data_root() -> PathBuf {
    if let Some(dir) = std::env::var_os("TASKSCAPE_DATA_DIR") {
        return PathBuf::from(dir);
    }
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")).map(PathBuf::from).unwrap_or_default();
    home.join(".taskscape")
}

/// A text run, laid out in Inter at a px size and weight. One line truncates with an ellipsis when wider than max_w.
fn galley(ui: &egui::Ui, text: &str, px: f32, w: u16, color: Color32, max_w: f32, one_line: bool) -> Arc<Galley> {
    let format = TextFormat { font_id: FontId::new(px, weight(w)), color, ..Default::default() };
    let mut job = LayoutJob::single_section(text.to_owned(), format);
    job.wrap.max_width = if max_w.is_finite() { max_w.max(0.0) } else { f32::INFINITY };
    if one_line {
        job.wrap.max_rows = 1;
        job.wrap.break_anywhere = true;
        job.wrap.overflow_character = Some('…');
    }
    ui.painter().layout_job(job)
}

/// A text run at a position, tagged as a contract part (its label is the text).
fn text_at(ui: &mut egui::Ui, at: Pos2, g: Arc<Galley>, label: &str, role: Role, part_id: &'static str, color: Color32) {
    let rect = Rect::from_min_size(at, g.size());
    let resp = ui.interact(rect, ui.scope_id().with(part_id), Sense::hover());
    resp.widget_info(|| WidgetInfo::labeled(role, true, label));
    part(ui, &resp, part_id);
    ui.painter().galley(at, g, color);
}

/// A text run in the flow of the layout, tagged as a contract part (its label is the text).
#[allow(clippy::too_many_arguments)]
fn text_flow(ui: &mut egui::Ui, text: &str, px: f32, w: u16, color: Color32, max_w: f32, one_line: bool, part_id: &'static str, role: Role) {
    let g = galley(ui, text, px, w, color, max_w, one_line);
    let (rect, resp) = ui.allocate_exact_size(g.size(), Sense::hover());
    resp.widget_info(|| WidgetInfo::labeled(role, true, text));
    part(ui, &resp, part_id);
    ui.painter().galley(rect.min, g, color);
}

/// Hands a file, or a link's address, to the OS's default app. It does not wait for it.
fn open_in_os(target: &str) {
    #[cfg(target_os = "windows")]
    let spawned = std::process::Command::new("cmd").args(["/C", "start", "", target]).spawn();
    #[cfg(target_os = "macos")]
    let spawned = std::process::Command::new("open").arg(target).spawn();
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let spawned = std::process::Command::new("xdg-open").arg(target).spawn();
    if let Ok(mut child) = spawned {
        std::thread::spawn(move || {
            let _ = child.wait();
        });
    }
}
