//! The preview rail (SPEC-PRODUCT.md, "taskscape.inspector"): the focused task, a draft, the
//! selection, or nothing. It rises in (110 ms, 3 px, a fade) when what it shows changes.

use gpui::{div, prelude::*, px, AnyElement, Context, FontWeight, HighlightStyle, MouseButton, StyledText, Window};
use serde_json::json;
use taskscape_common::{util::now_millis, LinkType, Task};

use crate::model::{self, relative};
use crate::motion::RISE;
use crate::parts::part;
use crate::report;
use crate::theme::{Theme, R_CONTROL, R_FIELD};
use crate::ui::{icon, pressable, section_label};
use crate::workspace::{SheetKind, Workspace};

impl Workspace {
    pub(crate) fn render_inspector(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let t = self.theme();
        let multi = self.multi();
        let (key, body): (String, AnyElement) = if multi.len() >= 2 {
            ("multi".into(), self.render_multi(&multi, &t, window, cx))
        } else if let Some(task) = self.focused_task.as_deref().and_then(|id| self.task(id)).cloned() {
            (format!("task:{}", task.id), self.render_task(&task, &t, window, cx))
        } else if let Some(list) = self.draft.clone() {
            (format!("draft:{list}"), self.render_draft(&list, &t, window, cx))
        } else {
            self.anims.reset("inspector");
            return div()
                .size_full()
                .bg(t.surface_1)
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap(px(6.))
                .child(icon("left_click", 28., t.content_3))
                .child(part("previewEmptyTitle", "heading").text("No task selected").wrap(div().text_size(px(15.)).font_weight(FontWeight::MEDIUM).text_color(t.content_2).child("No task selected")))
                .child(part("previewEmptySubtitle", "generic").text("Select a task to inspect it").wrap(div().text_size(px(12.5)).text_color(t.content_3).child("Select a task to inspect it")))
                .into_any_element();
        };
        let p = self.anims.progress("inspector", key, RISE, window);
        let inner = div().relative().top(px(3. * (1. - p))).opacity(p).size_full().child(body);
        div()
            .id("rail")
            .size_full()
            .bg(t.surface_1)
            .overflow_y_scroll()
            .child(part("inspector", "generic").opacity(p).wrap(inner))
            .into_any_element()
    }

    fn render_task(&mut self, task: &Task, t: &Theme, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let id = task.id.clone();
        let now = now_millis();
        let renaming = self.renaming.as_deref() == Some(&id);
        let fh_done = self.fh("inspectorDone", cx);
        let done_id = id.clone();
        let done = task.done;
        let check = part("inspectorDone", "checkbox").checked(done).name(if done { "Mark not done" } else { "Mark done" }).focus(&fh_done, window).wrap(
            pressable("inspectorDone", &fh_done, self.on(cx, move |ws, _, cx| ws.set_done(&done_id, !done, cx)))
                .size(px(18.))
                .mt(px(3.))
                .flex_none()
                .rounded(px(R_FIELD))
                .flex()
                .items_center()
                .justify_center()
                .when(done, |d| d.bg(t.accent_500).child(icon("check", 14., t.on_accent)))
                .when(!done, |d| d.border_1().border_color(t.edge_3).bg(t.surface_2)),
        );
        let title: AnyElement = if renaming {
            let f = self.title_field.clone();
            f.update(cx, |f, _| {
                f.caret = t.accent_500;
                f.placeholder_color = t.content_3;
                f.selection_color = t.selection_1;
            });
            let fh = f.read(cx).focus.clone();
            part("inspectorTitle", "textbox")
                .text(f.read(cx).text())
                .name("Title")
                .focus(&fh, window)
                .wrap(div().flex_1().min_w_0().text_size(px(18.)).line_height(px(24.)).font_weight(FontWeight::SEMIBOLD).child(f))
                .into_any_element()
        } else {
            let fh = self.fh("inspectorTitleText", cx);
            part("inspectorTitle", "textbox")
                .text(task.title.clone())
                .name("Title")
                .focus(&fh, window)
                .wrap(
                    div()
                        .id("inspectorTitle")
                        .track_focus(&fh)
                        .flex_1()
                        .min_w_0()
                        .text_size(px(18.))
                        .line_height(px(24.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(if done { t.content_3 } else { t.content_1 })
                        .on_click(cx.listener(|ws, e: &gpui::ClickEvent, w, cx| {
                            if e.click_count() >= 2 {
                                ws.start_rename(w, cx)
                            }
                        }))
                        .child(task.title.clone()),
                )
                .into_any_element()
        };
        let tool = |ws: &mut Workspace, name: &'static str, ic: &'static str, label: &'static str, h: crate::ui::Handler, window: &Window, cx: &mut Context<Workspace>| {
            let fh = ws.fh(name, cx);
            part(name, "button").name(label).focus(&fh, window).wrap(
                pressable(name, &fh, h).size(px(22.)).rounded(px(R_FIELD)).flex().items_center().justify_center().hover(|s| s.bg(t.wash_2)).child(icon(ic, 15., t.content_3)),
            )
        };
        let tid = id.clone();
        let tools = div()
            .absolute()
            .top(px(-2.))
            .right(px(-4.))
            .flex()
            .flex_col()
            .gap(px(1.))
            .child(tool(self, "inspectorClose", "last_page", "Close", self.on(cx, |ws, _, cx| ws.set_preview(false, cx)), window, cx))
            .child(tool(self, "inspectorRename", "edit", "Rename", self.on(cx, |ws, w, cx| ws.start_rename(w, cx)), window, cx))
            .child(tool(self, "inspectorCopy", "content_copy", "Copy", self.on(cx, { let tid = tid.clone(); move |ws, _, cx| ws.copy_tasks(&[tid.clone()], cx) }), window, cx))
            .child(tool(self, "inspectorDelete", "delete", "Delete", self.on(cx, { let tid = tid.clone(); move |ws, w, cx| ws.ask_delete(vec![tid.clone()], w, cx) }), window, cx));

        let where_s = self.where_of(&task.list_id);
        let meta_line = |ic: &'static str, child: AnyElement| div().flex().items_center().gap(px(6.)).h(px(20.)).child(icon(ic, 13., t.content_3)).child(child);
        let created = format!("Created {}", relative(task.created_at, now));
        let updated = format!("Updated {}", relative(task.updated_at, now));
        let meta = div()
            .pl(px(26.))
            .pt(px(4.))
            .flex()
            .flex_col()
            .text_size(px(11.5))
            .text_color(t.content_3)
            .child(meta_line("folder_open", part("inspectorWhere", "status").text(where_s.clone()).wrap(div().text_color(t.content_2).child(where_s)).into_any_element()))
            .child(meta_line("schedule", part("inspectorCreated", "status").text(created.clone()).wrap(div().child(created)).into_any_element()))
            .child(meta_line("update", part("inspectorUpdated", "status").text(updated.clone()).wrap(div().child(updated)).into_any_element()));

        let header = div()
            .relative()
            .flex()
            .flex_row()
            .items_start()
            .gap(px(8.))
            .child(div().absolute().left(px(-12.)).top(px(5.)).w(px(3.)).h(px(18.)).rounded_r(px(2.)).bg(t.accent_500))
            .pr(px(22.))
            .child(check)
            .child(title)
            .child(tools);

        div()
            .flex()
            .flex_col()
            .p(px(12.))
            .gap(px(14.))
            .child(div().flex().flex_col().child(header).child(meta))
            .child(self.render_notes(task, t, window, cx))
            .child(self.render_subtasks(task, t, window, cx))
            .child(self.render_attachments(task, t, window, cx))
            .into_any_element()
    }

    fn header_row(label: &str, t: &Theme) -> gpui::Div {
        div().flex().flex_row().items_center().gap(px(8.)).h(px(22.)).child(section_label(label, t)).child(div().flex_1().h(px(1.)).bg(t.edge_1))
    }

    fn render_notes(&mut self, task: &Task, t: &Theme, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let id = task.id.clone();
        let n = task.note_items.len();
        let fh_add = self.fh("addNote", cx);
        let add_h = self.on(cx, { let id = id.clone(); move |ws, w, cx| ws.open_note_editor(&id, None, w, cx) });
        let editing_new = self.note_edit.as_ref().is_some_and(|e| e.task == id && e.note.is_none());
        let mut col = div().flex().flex_col().gap(px(8.));
        let mut head = Self::header_row("Notes", t);
        if n > 0 {
            head = head.child(part("notesCount", "status").text(n.to_string()).wrap(div().text_size(px(11.)).text_color(t.content_3).child(n.to_string()))).child(
                part("addNote", "button").text("Note").name("Add note").focus(&fh_add, window).wrap(
                    pressable("addNote", &fh_add, add_h.clone())
                        .h(px(22.))
                        .px(px(6.))
                        .flex()
                        .items_center()
                        .gap(px(3.))
                        .rounded(px(R_CONTROL))
                        .text_size(px(12.5))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(t.content_3)
                        .hover(|s| s.bg(t.wash_2))
                        .child(icon("add", 13., t.content_3))
                        .child("Note"),
                ),
            );
        }
        col = col.child(head);
        for (i, note) in task.note_items.iter().enumerate() {
            let k = i + 1;
            let editing = self.note_edit.as_ref().filter(|e| e.note.as_deref() == Some(&note.id)).map(|e| e.field.clone());
            let text_el: AnyElement = if let Some(f) = editing {
                f.update(cx, |f, _| {
                    f.caret = t.accent_500;
                    f.placeholder_color = t.content_3;
                    f.selection_color = t.selection_1;
                });
                let fh = f.read(cx).focus.clone();
                part(format!("note{k}Text"), "textbox").text(f.read(cx).text()).name("Note").focus(&fh, window).wrap(div().w_full().child(f)).into_any_element()
            } else {
                let spans = model::note_spans(&note.content);
                let text: String = spans.iter().map(|s| s.text.as_str()).collect();
                let mut highlights = vec![];
                let mut at = 0;
                for s in &spans {
                    let mut h = HighlightStyle::default();
                    if s.bold {
                        h.font_weight = Some(FontWeight::BOLD);
                    }
                    if s.italic {
                        h.font_style = Some(gpui::FontStyle::Italic);
                    }
                    if s.code {
                        h.background_color = Some(t.wash_2);
                    }
                    if s.link {
                        h.color = Some(t.accent_500);
                    }
                    if h != HighlightStyle::default() {
                        highlights.push((at..at + s.text.len(), h));
                    }
                    at += s.text.len();
                }
                let nid = note.id.clone();
                let tid = id.clone();
                part(format!("note{k}Text"), "textbox")
                    .text(text.clone())
                    .name("Note")
                    .wrap(
                        div()
                            .id(("noteText", k))
                            .w_full()
                            .cursor_text()
                            .on_mouse_down(MouseButton::Left, cx.listener(move |ws, _, w, cx| ws.open_note_editor(&tid, Some(&nid), w, cx)))
                            .child(StyledText::new(text).with_highlights(highlights)),
                    )
                    .into_any_element()
            };
            let fh_del = self.fh(&format!("note{k}Delete"), cx);
            let nid = note.id.clone();
            let del = part(format!("note{k}Delete"), "button").name("Delete note").focus(&fh_del, window).wrap(
                pressable(("noteDelete", k), &fh_del, self.on(cx, move |ws, w, cx| ws.open_sheet(SheetKind::DeleteNote(nid.clone()), w, cx)))
                    .absolute()
                    .top(px(-8.))
                    .right(px(-8.))
                    .size(px(22.))
                    .rounded_full()
                    .bg(t.surface_3)
                    .border_1()
                    .border_color(t.edge_2)
                    .flex()
                    .items_center()
                    .justify_center()
                    .opacity(0.)
                    .group_hover("note", |s| s.opacity(1.))
                    .child(icon("delete", 13., t.danger_500)),
            );
            col = col.child(
                part(format!("note{k}"), "generic").wrap(
                    div()
                        .id(("note", k))
                        .group("note")
                        .relative()
                        .w_full()
                        .p(px(10.))
                        .rounded(px(R_CONTROL))
                        .bg(t.surface_2)
                        .border_1()
                        .border_color(t.edge_2)
                        .text_size(px(13.5))
                        .line_height(px(19.))
                        .text_color(t.content_1)
                        .child(text_el)
                        .child(del),
                ),
            );
        }
        if editing_new {
            let f = self.note_edit.as_ref().map(|e| e.field.clone()).unwrap();
            f.update(cx, |f, _| {
                f.caret = t.accent_500;
                f.placeholder_color = t.content_3;
                f.selection_color = t.selection_1;
            });
            let fh = f.read(cx).focus.clone();
            col = col.child(
                part("newNote", "textbox").text(f.read(cx).text()).name("New note").focus(&fh, window).wrap(
                    div().w_full().p(px(10.)).rounded(px(R_CONTROL)).bg(t.surface_2).border_1().border_color(t.accent_500).text_size(px(13.5)).line_height(px(19.)).child(f),
                ),
            );
        } else if n == 0 {
            col = col.child(
                part("addNote", "button").text("Add note").focus(&fh_add, window).wrap(
                    pressable("addNote", &fh_add, add_h)
                        .h(px(84.))
                        .w_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .gap(px(4.))
                        .rounded(px(R_CONTROL))
                        .border_1()
                        .border_dashed()
                        .border_color(t.edge_3)
                        .text_size(px(14.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(t.content_3)
                        .hover(|s| s.bg(t.wash_1))
                        .child(icon("add", 15., t.content_3))
                        .child("Add note"),
                ),
            );
        }
        col.into_any_element()
    }

    fn render_subtasks(&mut self, task: &Task, t: &Theme, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let subs: Vec<(Task, usize)> = {
            let tree = self.tree(&task.list_id);
            let base = tree.depth(&task.id) + 1;
            tree.descendants(&task.id).into_iter().map(|d| (d.clone(), tree.depth(&d.id) - base)).collect()
        };
        if subs.is_empty() {
            return div().into_any_element();
        }
        let direct: Vec<&(Task, usize)> = subs.iter().filter(|(_, d)| *d == 0).collect();
        let count = format!("{} done / {}", direct.iter().filter(|(s, _)| s.done).count(), direct.len());
        let mut col = div()
            .flex()
            .flex_col()
            .gap(px(2.))
            .child(Self::header_row("Subtasks", t).child(part("subtasksCount", "status").text(count.clone()).wrap(div().text_size(px(11.5)).text_color(t.content_3).child(count))));
        for (i, (s, depth)) in subs.iter().enumerate() {
            let k = i + 1;
            let (fh_c, fh_n) = (self.fh(&format!("subtask{k}Check"), cx), self.fh(&format!("subtask{k}Name"), cx));
            let (sid, sid2, done) = (s.id.clone(), s.id.clone(), s.done);
            let check = part(format!("subtask{k}Check"), "checkbox").checked(done).name(s.title.clone()).focus(&fh_c, window).wrap(
                pressable(("subtaskCheck", k), &fh_c, self.on(cx, move |ws, _, cx| ws.set_done(&sid, !done, cx)))
                    .size(px(16.))
                    .flex_none()
                    .rounded(px(R_FIELD))
                    .flex()
                    .items_center()
                    .justify_center()
                    .when(done, |d| d.bg(t.accent_500).child(icon("check", 12., t.on_accent)))
                    .when(!done, |d| d.border_1().border_color(t.edge_3).bg(t.surface_2)),
            );
            let mut name = div().text_size(px(14.)).text_color(if done { t.content_3 } else { t.content_1 }).truncate().child(s.title.clone());
            if done {
                name = name.line_through();
            }
            let name = part(format!("subtask{k}Name"), "button").text(s.title.clone()).focus(&fh_n, window).wrap(
                pressable(("subtaskName", k), &fh_n, self.on(cx, move |ws, w, cx| ws.focus_task(&sid2, w, cx))).flex_1().min_w_0().child(name),
            );
            col = col.child(div().h(px(28.)).flex().items_center().gap(px(8.)).pl(px(2. + 18. * *depth as f32)).child(check).child(name));
        }
        col.into_any_element()
    }

    fn render_attachments(&mut self, task: &Task, t: &Theme, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let id = task.id.clone();
        let small = |ws: &mut Workspace, name: &'static str, ic: &'static str, label: &'static str, h: crate::ui::Handler, window: &Window, cx: &mut Context<Workspace>| {
            let fh = ws.fh(name, cx);
            part(name, "button").text(label).focus(&fh, window).wrap(
                pressable(name, &fh, h)
                    .h(px(22.))
                    .px(px(5.))
                    .flex()
                    .items_center()
                    .gap(px(3.))
                    .rounded(px(R_CONTROL))
                    .text_size(px(12.5))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(t.content_3)
                    .hover(|s| s.bg(t.wash_2))
                    .child(icon(ic, 13., t.content_3))
                    .child(label),
            )
        };
        let link_h = self.on(cx, { let id = id.clone(); move |ws, w, cx| ws.open_sheet(SheetKind::LinkAttachment(id.clone()), w, cx) });
        let shot_h = self.on(cx, |_, _, cx| report(cx, "screenshotRequested", json!({})));
        // "File" picks a file with the system's dialog and copies it into the data folder.
        let file_h = self.on(cx, {
            let id = id.clone();
            move |_, _, cx| {
                let rx = cx.prompt_for_paths(gpui::PathPromptOptions { files: true, directories: false, multiple: true, prompt: None });
                let id = id.clone();
                cx.spawn(async move |this, cx| {
                    let Ok(Ok(Some(paths))) = rx.await else { return };
                    this.update(cx, |ws, cx| {
                        for p in paths {
                            let src = p.display().to_string();
                            if let Ok(a) = ws.db.run(taskscape_common::attachments::attach_copy(&ws.db.store, &id, &src, None)) {
                                report(cx, "attachmentAdded", json!({ "taskId": a.task_id, "name": a.name }));
                            }
                        }
                        ws.reload();
                        cx.notify();
                    })
                    .ok();
                })
                .detach();
            }
        });
        let head = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(4.))
            .h(px(22.))
            .child(section_label("Attachments", t))
            .child(div().flex_1())
            .child(small(self, "attachShot", "screenshot_monitor", "Shot", shot_h, window, cx))
            .child(small(self, "attachLink", "add_link", "Link", link_h.clone(), window, cx))
            .child(small(self, "attachFile", "note_add", "File", file_h.clone(), window, cx));
        let mut col = div().flex().flex_col().gap(px(8.)).child(head);
        if task.attachments.is_empty() {
            let half = |label: &'static str, ic: &'static str, h: crate::ui::Handler, key: &'static str, ws: &mut Workspace, cx: &mut Context<Workspace>| {
                let fh = ws.fh(key, cx);
                pressable(key, &fh, h)
                    .flex_1()
                    .h_full()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap(px(4.))
                    .text_size(px(12.5))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(t.content_3)
                    .hover(|s| s.bg(t.wash_1))
                    .child(icon(ic, 16., t.content_3))
                    .child(label)
            };
            col = col.child(
                div()
                    .h(px(70.))
                    .flex()
                    .flex_row()
                    .rounded(px(R_CONTROL))
                    .border_1()
                    .border_dashed()
                    .border_color(t.edge_3)
                    .child(half("Copy a file", "note_add", file_h, "attachEmptyFile", self, cx))
                    .child(div().w(px(1.)).my(px(10.)).bg(t.edge_2))
                    .child(half("Add a link", "add_link", link_h, "attachEmptyLink", self, cx)),
            );
        } else {
            let mut grid = div().flex().flex_row().flex_wrap().gap(px(5.));
            for (i, a) in task.attachments.iter().enumerate() {
                let k = i + 1;
                let fh = self.fh(&format!("attachment{k}Tile"), cx);
                let is_image = a.link_type == LinkType::Copy && [".png", ".jpg", ".jpeg", ".gif", ".webp"].iter().any(|e| a.name.to_lowercase().ends_with(e));
                let location = a.location.clone();
                let is_link = a.link_type == LinkType::Reference;
                let h = self.on(cx, move |_, _, cx| {
                    if is_link {
                        report(cx, "attachmentOpened", json!({ "location": location }));
                        cx.open_url(&location);
                    } else {
                        report(cx, "attachmentPreviewed", json!({ "location": location }));
                    }
                });
                let tile = part(format!("attachment{k}Tile"), "button").name(a.name.clone()).focus(&fh, window).wrap(
                    pressable(("attachmentTile", k), &fh, h)
                        .size(px(94.))
                        .rounded(px(R_CONTROL))
                        .bg(t.surface_2)
                        .border_1()
                        .border_color(t.edge_2)
                        .flex()
                        .items_center()
                        .justify_center()
                        .hover(|s| s.border_color(t.edge_3))
                        .child(icon(if is_image { "image" } else if is_link { "link" } else { "description" }, 20., t.content_3)),
                );
                grid = grid.child(
                    div().w(px(94.)).flex().flex_col().items_center().gap(px(4.)).child(tile).child(
                        part(format!("attachment{k}Name"), "status").text(a.name.clone()).wrap(div().w_full().text_size(px(11.5)).text_color(t.content_3).text_center().truncate().child(a.name.clone())),
                    ),
                );
            }
            col = col.child(grid);
        }
        col.into_any_element()
    }

    fn render_draft(&mut self, list: &str, t: &Theme, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let f = self.draft_title.clone();
        f.update(cx, |f, _| {
            f.caret = t.accent_500;
            f.placeholder_color = t.content_3;
            f.selection_color = t.selection_1;
        });
        let fh = f.read(cx).focus.clone();
        let where_s = self.where_of(list);
        let (fh_note, fh_shot, fh_discard) = (self.fh("draftNote", cx), self.fh("draftScreenshot", cx), self.fh("draftDiscard", cx));
        let chip = |name: &'static str, fh: &gpui::FocusHandle, ic: &'static str, label: &'static str, h: crate::ui::Handler, window: &Window| {
            part(name, "button").text(label).focus(fh, window).wrap(
                pressable(name, fh, h)
                    .h(px(28.))
                    .px(px(10.))
                    .flex()
                    .items_center()
                    .gap(px(5.))
                    .rounded(px(R_CONTROL))
                    .bg(t.surface_2)
                    .border_1()
                    .border_color(t.edge_2)
                    .text_size(px(13.))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(t.content_1)
                    .hover(|s| s.bg(t.wash_1))
                    .child(icon(ic, 15., t.content_3))
                    .child(label),
            )
        };
        div()
            .flex()
            .flex_col()
            .p(px(12.))
            .gap(px(12.))
            .child(
                div().flex().flex_row().items_start().gap(px(8.)).child(div().size(px(18.)).mt(px(3.)).rounded(px(R_FIELD)).border_1().border_dashed().border_color(t.edge_3)).child(
                    part("draftTitle", "textbox")
                        .text(f.read(cx).text())
                        .name("Untitled")
                        .focus(&fh, window)
                        .wrap(div().flex_1().text_size(px(18.)).line_height(px(24.)).font_weight(FontWeight::SEMIBOLD).child(f)),
                ),
            )
            .child(
                div().pl(px(26.)).flex().items_center().gap(px(6.)).text_size(px(11.5)).text_color(t.content_2).child(icon("folder_open", 13., t.content_3)).child(part("draftWhere", "status").text(where_s.clone()).wrap(div().child(where_s))),
            )
            .child(section_label("Start with", t))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap(px(6.))
                    .child(chip("draftNote", &fh_note, "notes", "Note", self.on(cx, |ws, w, cx| ws.draft_with_note(w, cx)), window))
                    .child(chip("draftScreenshot", &fh_shot, "screenshot_monitor", "Screenshot", self.on(cx, |_, _, cx| report(cx, "screenshotRequested", json!({}))), window)),
            )
            .child(div().text_size(px(12.5)).line_height(px(17.)).text_color(t.content_3).child("Type a title, add a note, or take a screenshot — the task is created the moment it has something in it."))
            .child(
                part("draftDiscard", "button").text("Discard draft").focus(&fh_discard, window).wrap(
                    pressable("draftDiscard", &fh_discard, self.on(cx, |ws, w, cx| {
                        ws.draft = None;
                        w.focus(&ws.root_focus);
                        cx.notify()
                    }))
                    .text_size(px(12.5))
                    .text_color(t.content_3)
                    .hover(|s| s.text_color(t.content_1))
                    .child("Discard draft"),
                ),
            )
            .into_any_element()
    }

    fn render_multi(&mut self, ids: &[String], t: &Theme, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let tasks: Vec<Task> = ids.iter().filter_map(|i| self.task(i).cloned()).collect();
        let done = tasks.iter().filter(|t| t.done).count();
        let count = format!("{} tasks selected", tasks.len());
        let done_s = format!("{done} done / {}", tasks.len());
        let action = |ws: &mut Workspace, name: &'static str, ic: &'static str, label: &'static str, danger: bool, h: crate::ui::Handler, window: &Window, cx: &mut Context<Workspace>| {
            let fh = ws.fh(name, cx);
            part(name, "button").text(label).focus(&fh, window).wrap(
                pressable(name, &fh, h)
                    .h(px(28.))
                    .px(px(9.))
                    .flex()
                    .items_center()
                    .gap(px(4.))
                    .rounded(px(R_CONTROL))
                    .bg(t.surface_2)
                    .border_1()
                    .border_color(t.edge_2)
                    .text_size(px(12.5))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(if danger { t.danger_500 } else { t.content_1 })
                    .hover(|s| s.bg(t.wash_1))
                    .child(icon(ic, 14., if danger { t.danger_500 } else { t.content_3 }))
                    .child(label),
            )
        };
        let v = ids.to_vec();
        let mut actions = div().flex().flex_row().flex_wrap().gap(px(5.));
        actions = actions.child(action(self, "multiDone", "check", "Done", false, self.on(cx, { let v = v.clone(); move |ws, _, cx| ws.set_many_done(&v, true, cx) }), window, cx));
        actions = actions.child(action(self, "multiUndone", "radio_button_unchecked", "Undone", false, self.on(cx, { let v = v.clone(); move |ws, _, cx| ws.set_many_done(&v, false, cx) }), window, cx));
        actions = actions.child(action(self, "multiMove", "arrow_forward", "Move", false, self.on(cx, { let v = v.clone(); move |ws, _, cx| { if let Some(f) = v.first() { ws.open_menu(f, gpui::point(px(900.), px(140.)), cx); if let Some(m) = &mut ws.menu { m.submenu = true; } } } }), window, cx));
        actions = actions.child(action(self, "multiCopy", "content_copy", "Copy", false, self.on(cx, { let v = v.clone(); move |ws, _, cx| ws.copy_tasks(&v, cx) }), window, cx));
        actions = actions.child(action(self, "multiDelete", "delete", "Delete", true, self.on(cx, { let v = v.clone(); move |ws, w, cx| ws.ask_delete(v.clone(), w, cx) }), window, cx));
        let fh_clear = self.fh("multiClear", cx);
        let mut list = div().flex().flex_col().gap(px(2.));
        for (i, task) in tasks.iter().enumerate() {
            let k = i + 1;
            let fh = self.fh(&format!("picked{k}Name"), cx);
            let lname = self.list(&task.list_id).map(|l| l.name.clone()).unwrap_or_default();
            let tid = task.id.clone();
            list = list.child(
                div()
                    .h(px(30.))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(icon(if task.done { "task_alt" } else { "radio_button_unchecked" }, 15., if task.done { t.done_lamp_1 } else { t.content_3 }))
                    .child(
                        part(format!("picked{k}Name"), "button")
                            .text(task.title.clone())
                            .focus(&fh, window)
                            .wrap(pressable(("pickedName", k), &fh, self.on(cx, move |ws, w, cx| ws.focus_task(&tid, w, cx))).flex_1().min_w_0().truncate().text_size(px(13.5)).child(task.title.clone())),
                    )
                    .child(part(format!("picked{k}List"), "status").text(lname.clone()).wrap(div().text_size(px(11.5)).text_color(t.content_3).child(lname))),
            );
        }
        div()
            .flex()
            .flex_col()
            .p(px(12.))
            .gap(px(12.))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .child(part("multiCount", "heading").text(count.clone()).wrap(div().flex_1().text_size(px(18.)).line_height(px(24.)).font_weight(FontWeight::SEMIBOLD).child(count)))
                    .child(
                        part("multiClear", "button").name("Clear selection").text("Clear").focus(&fh_clear, window).wrap(
                            pressable("multiClear", &fh_clear, self.on(cx, |ws, _, cx| ws.clear_selection(cx)))
                                .h(px(24.))
                                .px(px(7.))
                                .rounded(px(R_CONTROL))
                                .flex()
                                .items_center()
                                .text_size(px(12.5))
                                .text_color(t.content_3)
                                .hover(|s| s.bg(t.wash_2))
                                .child("Clear"),
                        ),
                    ),
            )
            .child(part("multiDoneCount", "status").text(done_s.clone()).wrap(div().text_size(px(11.5)).text_color(t.content_3).child(done_s)))
            .child(actions)
            .child(section_label("Selected", t))
            .child(list)
            .into_any_element()
    }
}
