//! The task pane (SPEC-PRODUCT.md, "taskscape.pane"; DESIGN.md, "Task pane"): the search well,
//! the task tree in GPUI's virtualized list, and the footer that is the stats or the bulk bar.

use gpui::{
    anchored, deferred, div, list, point, prelude::*, px, AnyElement, BoxShadow, ClickEvent, Context, FontWeight,
    ListAlignment, ListState, MouseButton, Pixels, Window,
};
use taskscape_common::util::now_millis;

use crate::keys::*;
use crate::parts::part;
use crate::theme::{Theme, R_CONTROL, R_FIELD, R_PANEL};
use crate::ui::{icon, pressable};
use crate::workspace::{SheetKind, Workspace};

/// One item of the list: a task's row, or the subtask field under a task.
#[derive(Clone)]
pub enum Item {
    Row(usize),
    Composer(usize),
}

pub struct ListCache {
    pub state: ListState,
    pub key: Vec<(String, bool, usize)>,
    pub items: Vec<Item>,
    pub rows: Vec<crate::model::Row>,
}

impl Workspace {
    fn list_cache(&mut self, pane: usize, cx: &mut Context<Self>) -> ListState {
        let rows = self.rows(pane, cx);
        let composer = self.panes[pane].composer.clone();
        let mut items = vec![];
        let mut pending: Option<usize> = None;
        for (i, r) in rows.iter().enumerate() {
            if let Some(d) = pending {
                if r.depth <= d {
                    items.push(Item::Composer(d + 1));
                    pending = None;
                }
            }
            items.push(Item::Row(i));
            if composer.as_deref() == Some(r.id.as_str()) {
                pending = Some(r.depth);
            }
        }
        if let Some(d) = pending {
            items.push(Item::Composer(d + 1));
        }
        let key: Vec<(String, bool, usize)> = items
            .iter()
            .map(|it| match it {
                Item::Row(i) => {
                    let t = self.task(&rows[*i].id);
                    (rows[*i].id.clone(), t.and_then(|t| t.notes.as_ref()).is_some_and(|n| !n.trim().is_empty()), rows[*i].depth)
                }
                Item::Composer(d) => ("\u{0}composer".into(), false, *d),
            })
            .collect();
        let cache = self.lists.entry(pane).or_insert_with(|| ListCache { state: ListState::new(0, ListAlignment::Top, px(400.)), key: vec![], items: vec![], rows: vec![] });
        if cache.key != key {
            cache.state.reset(items.len());
            cache.key = key;
        }
        cache.items = items;
        cache.rows = rows;
        cache.state.clone()
    }

    pub(crate) fn render_pane(&mut self, pane: usize, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let t = self.theme();
        let Some(list_id) = self.panes[pane].list.clone() else {
            return div().size_full().bg(t.surface_2).into_any_element();
        };
        let list_name = self.list(&list_id).map(|l| l.name.clone()).unwrap_or_default();
        let state = self.list_cache(pane, cx);
        let query = self.query(pane, cx);
        let searching = !query.trim().is_empty();
        let rows_empty = self.lists.get(&pane).map(|c| c.rows.is_empty()).unwrap_or(true);
        let multi = if pane == self.focused_pane { self.multi() } else { vec![] };

        // ---- the search well
        let search_field = self.panes[pane].search.clone();
        let search_focus = search_field.read(cx).focus.clone();
        let fh_opts = self.fh(&format!("searchOptions{pane}"), cx);
        let fh_new = self.fh(&format!("newTask{pane}"), cx);
        let fh_clear = self.fh(&format!("clearSearch{pane}"), cx);
        let match_count = self.lists.get(&pane).map(|c| c.rows.iter().filter(|r| r.matched).count()).unwrap_or(0);
        search_field.update(cx, |f, _| {
            f.caret = t.accent_500;
            f.placeholder_color = t.content_3;
            f.selection_color = t.selection_1;
        });
        let mut well = div()
            .mx(px(14.))
            .mt(px(10.5))
            .h(px(35.))
            .flex_none()
            .flex()
            .flex_row()
            .items_center()
            .rounded(px(R_CONTROL))
            .bg(t.surface_0)
            .pl(px(10.))
            .pr(px(4.))
            .gap(px(8.))
            .child(icon("search", 16., t.content_3))
            .child(
                part("search", "textbox")
                    .text(search_field.read(cx).text())
                    .name(search_field.read(cx).placeholder().to_string())
                    .focus(&search_focus, window)
                    .wrap(div().flex_1().min_w_0().text_size(px(14.)).child(search_field.clone())),
            );
        if searching {
            well = well
                .child(part("matchCount", "status").text(match_count.to_string()).wrap(div().text_size(px(11.)).font_weight(FontWeight::SEMIBOLD).text_color(t.content_3).child(match_count.to_string())))
                .child(
                    part("clearSearch", "button").name("Clear search").focus(&fh_clear, window).wrap(
                        pressable(("clearSearch", pane), &fh_clear, self.on(cx, move |ws, w, cx| {
                            ws.panes[pane].search.update(cx, |f, cx| f.set_text("", cx));
                            let f = ws.panes[pane].search.read(cx).focus.clone();
                            w.focus(&f);
                        }))
                        .size(px(22.))
                        .rounded(px(R_FIELD))
                        .flex()
                        .items_center()
                        .justify_center()
                        .hover(|s| s.bg(t.wash_2))
                        .child(icon("close", 15., t.content_3)),
                    ),
                );
        }
        well = well
            .child(
                part("searchOptions", "button").name("Search options").focus(&fh_opts, window).wrap(
                    pressable(("searchOptions", pane), &fh_opts, self.on(cx, move |ws, _, cx| {
                        let p = &mut ws.panes[pane];
                        p.fields = match p.fields {
                            crate::model::SearchFields::Both => crate::model::SearchFields::Title,
                            crate::model::SearchFields::Title => crate::model::SearchFields::Notes,
                            crate::model::SearchFields::Notes => crate::model::SearchFields::Both,
                        };
                        cx.notify()
                    }))
                    .size(px(26.))
                    .rounded(px(R_FIELD))
                    .flex()
                    .items_center()
                    .justify_center()
                    .hover(|s| s.bg(t.wash_2))
                    .child(icon("tune", 16., t.content_3)),
                ),
            )
            .child(div().w(px(1.)).h(px(20.)).bg(t.edge_2))
            .child(
                part("newTask", "button").name("New task").focus(&fh_new, window).wrap(
                    pressable(("newTask", pane), &fh_new, self.on(cx, move |ws, w, cx| {
                        ws.focused_pane = pane;
                        if let Some(l) = ws.panes[pane].list.clone() {
                            ws.open_draft(&l, w, cx)
                        }
                    }))
                    .size(px(26.))
                    .rounded(px(R_FIELD))
                    .flex()
                    .items_center()
                    .justify_center()
                    .hover(|s| s.bg(t.wash_2))
                    .child(icon("add", 18., t.content_2)),
                ),
            );
        if self.panes.len() > 1 {
            let fh = self.fh(&format!("closePane{pane}"), cx);
            well = well.child(
                pressable(("closePane", pane), &fh, self.on(cx, move |ws, w, cx| {
                    if pane == 1 {
                        ws.set_split(None, w, cx);
                    } else {
                        ws.panes.remove(0);
                        ws.focused_pane = 0;
                        ws.set_split(None, w, cx);
                    }
                }))
                .size(px(26.))
                .flex()
                .items_center()
                .justify_center()
                .child(icon("close", 16., t.content_3)),
            );
        }

        // ---- the rows, or the empty state
        let body: AnyElement = if rows_empty {
            let (title, sub) = if searching {
                (format!("No matches in {list_name}"), "Try a different search or scope".to_string())
            } else {
                let capture = self.glyphs("toggle_capture_bar");
                let new = self.glyphs("new_task");
                let sub = if capture.is_empty() {
                    format!("Press + or {new} to add a task")
                } else {
                    format!("Press + or {new} to add a task, or {capture} anywhere to capture one")
                };
                ("Nothing here yet".to_string(), sub)
            };
            div()
                .flex_1()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap(px(6.))
                .child(icon(if searching { "search" } else { "task_alt" }, 30., t.content_3))
                .child(part("paneEmptyTitle", "heading").text(title.clone()).wrap(div().text_size(px(15.)).font_weight(FontWeight::MEDIUM).text_color(t.content_2).child(title)))
                .child(part("paneEmptySubtitle", "generic").text(sub.clone()).wrap(div().text_size(px(12.5)).line_height(px(17.)).text_color(t.content_3).child(sub)))
                .into_any_element()
        } else {
            let render = cx.processor(move |ws: &mut Workspace, ix: usize, window: &mut Window, cx: &mut Context<Workspace>| ws.render_item(pane, ix, window, cx));
            list(state, move |ix, window, cx| render(ix, window, cx)).flex_1().w_full().pt(px(4.)).into_any_element()
        };

        let list_focus = self.panes[pane].focus.clone();
        let tasklist = div()
            .id(("tasklist", pane))
            .key_context("TaskList")
            .track_focus(&list_focus)
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .on_action(cx.listener(|ws, _: &FocusPrev, w, cx| ws.step_focus(-1, false, w, cx)))
            .on_action(cx.listener(|ws, _: &FocusNext, w, cx| ws.step_focus(1, false, w, cx)))
            .on_action(cx.listener(|ws, _: &ExtendPrev, w, cx| ws.step_focus(-1, true, w, cx)))
            .on_action(cx.listener(|ws, _: &ExtendNext, w, cx| ws.step_focus(1, true, w, cx)))
            .on_action(cx.listener(|ws, _: &CollapseOrParent, w, cx| ws.collapse_or_parent(false, w, cx)))
            .on_action(cx.listener(|ws, _: &ExpandOrChild, w, cx| ws.collapse_or_parent(true, w, cx)))
            .on_action(cx.listener(|ws, _: &Indent, _, cx| ws.indent(false, cx)))
            .on_action(cx.listener(|ws, _: &Outdent, _, cx| ws.indent(true, cx)))
            .on_action(cx.listener(|ws, _: &ToggleSelect, _, cx| {
                if let Some(f) = ws.focused_task.clone() {
                    let sel = &mut ws.panes[ws.focused_pane].selection;
                    if let Some(i) = sel.iter().position(|s| *s == f) {
                        sel.remove(i);
                    } else {
                        sel.push(f);
                    }
                    cx.notify();
                }
            }))
            .on_action(cx.listener(|ws, _: &ToggleDone, _, cx| {
                if let Some(f) = ws.focused_task.clone() {
                    let done = ws.task(&f).map(|t| t.done).unwrap_or(false);
                    ws.set_done(&f, !done, cx);
                }
            }))
            .on_action(cx.listener(|ws, _: &Rename, w, cx| {
                if ws.focused_task.is_some() {
                    ws.preview = true;
                    ws.start_rename(w, cx)
                }
            }))
            .on_action(cx.listener(|ws, _: &DeleteTask, w, cx| {
                let multi = ws.multi();
                let ids = if !multi.is_empty() { multi } else { ws.focused_task.clone().into_iter().collect() };
                ws.ask_delete(ids, w, cx);
            }))
            .on_action(cx.listener(|ws, _: &MoveDown, _, cx| ws.move_by(true, cx)))
            .on_action(cx.listener(|ws, _: &MoveUp, _, cx| ws.move_by(false, cx)))
            .on_action(cx.listener(move |ws, _: &SelectAll, _, cx| {
                let ids: Vec<String> = ws.rows(pane, cx).into_iter().map(|r| r.id).collect();
                ws.panes[pane].selection = ids;
                cx.notify();
            }))
            .on_key_down(cx.listener(move |ws, e: &gpui::KeyDownEvent, w, cx| {
                // A letter with no field focused starts a search.
                if !ws.panes[pane].focus.is_focused(w) {
                    return;
                }
                let k = &e.keystroke;
                if k.modifiers.platform || k.modifiers.control || k.modifiers.alt {
                    return;
                }
                if let Some(ch) = k.key_char.as_ref().filter(|c| c.chars().count() == 1 && c.chars().all(|c| c.is_alphanumeric())) {
                    let ch = ch.clone();
                    let field = ws.panes[pane].search.clone();
                    field.update(cx, |f, cx| f.set_text(ch, cx));
                    let fh = field.read(cx).focus.clone();
                    w.focus(&fh);
                    cx.stop_propagation();
                }
            }))
            .child(body);

        // ---- the footer
        let footer = self.render_footer(pane, &list_id, &multi, &t, window, cx);

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(t.surface_2)
            .child(well)
            .child(tasklist)
            .child(footer)
            .into_any_element()
    }

    fn render_item(&mut self, pane: usize, ix: usize, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let Some(item) = self.lists.get(&pane).and_then(|c| c.items.get(ix).cloned()) else { return div().into_any_element() };
        match item {
            Item::Row(r) => self.render_row(pane, r, window, cx),
            Item::Composer(depth) => {
                let t = self.theme();
                let f = self.subtask_field.clone();
                f.update(cx, |f, _| {
                    f.caret = t.accent_500;
                    f.placeholder_color = t.content_3;
                    f.selection_color = t.selection_1;
                });
                let fh = f.read(cx).focus.clone();
                div()
                    .h(px(31.5))
                    .flex()
                    .items_center()
                    .pl(px(20. + 24. * depth as f32))
                    .pr(px(14.))
                    .gap(px(8.))
                    .child(icon("subdirectory_arrow_right", 16., t.content_3))
                    .child(part("subtaskTitle", "textbox").text(f.read(cx).text()).name("Subtask title").focus(&fh, window).wrap(div().flex_1().text_size(px(14.)).child(f)))
                    .into_any_element()
            }
        }
    }

    fn render_row(&mut self, pane: usize, r: usize, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let t = self.theme();
        let Some(row) = self.lists.get(&pane).and_then(|c| c.rows.get(r).cloned()) else { return div().into_any_element() };
        let Some(task) = self.task(&row.id).cloned() else { return div().into_any_element() };
        let n = r + 1;
        let id = task.id.clone();
        let depth = row.depth as f32;
        let notes = task.notes.clone().filter(|s| !s.trim().is_empty());
        let selected = self.panes[pane].selection.contains(&id) || (self.focused_task.as_deref() == Some(&id) && pane == self.focused_pane);
        // The subtask badge: done / all of its direct subtasks that the list shows.
        let kids: Vec<bool> = {
            let tree = self.tree(&task.list_id);
            tree.kids(&id).iter().filter(|k| self.show_completed || !k.done).map(|k| k.done).collect()
        };
        let (fh_check, fh_more, fh_toggle, fh_row) = (self.fh(&format!("row{n}Check.{pane}"), cx), self.fh(&format!("row{n}More.{pane}"), cx), self.fh(&format!("row{n}Toggle.{pane}"), cx), self.fh(&format!("row{n}.{pane}"), cx));
        let _ = fh_row;
        let group = format!("row{pane}-{n}");

        // The round check: completes the task (the design review's direction).
        let check_id = id.clone();
        let check = part(format!("row{n}Check"), "button")
            .name(if task.done { "Mark not done" } else { "Mark done" })
            .focus(&fh_check, window)
            .wrap(
                pressable(("rowCheck", pane * 100000 + n), &fh_check, self.on(cx, move |ws, _, cx| {
                    let done = ws.task(&check_id).map(|t| t.done).unwrap_or(false);
                    ws.set_done(&check_id, !done, cx);
                }))
                .size(px(16.))
                .mt(px(1.5))
                .flex_none()
                .rounded_full()
                .flex()
                .items_center()
                .justify_center()
                .when(task.done, |d| d.bg(t.done_lamp_1).child(icon("check", 12., t.on_accent)))
                .when(!task.done, |d| d.border_1().border_color(t.edge_3).hover(|s| s.border_color(t.accent_500))),
            );

        let toggle_id = id.clone();
        let toggle = row.has_kids.then(|| {
            part(format!("row{n}Toggle"), "button").name(if row.expanded { "Collapse" } else { "Expand" }).focus(&fh_toggle, window).wrap(
                pressable(("rowToggle", pane * 100000 + n), &fh_toggle, self.on(cx, move |ws, _, cx| {
                    let collapsed = ws.panes[pane].collapsed.contains(&toggle_id);
                    ws.set_expanded(pane, &toggle_id, collapsed, cx);
                }))
                .absolute()
                .left(px(24. * depth + 1.))
                .top(px(7.))
                .size(px(18.))
                .rounded(px(R_FIELD))
                .flex()
                .items_center()
                .justify_center()
                .when(row.expanded, |d| d.opacity(0.).group_hover(group.clone(), |s| s.opacity(1.)))
                .hover(|s| s.bg(t.wash_2))
                .child(icon(if row.expanded { "expand_more" } else { "chevron_right" }, 14., t.content_3)),
            )
        });

        let title_color = if task.done { t.content_3 } else { t.content_1 };
        let mut title_el = div().text_size(px(14.)).line_height(px(19.)).font_weight(FontWeight::MEDIUM).text_color(title_color).truncate().child(task.title.clone());
        if task.done {
            title_el = title_el.line_through();
        }
        let mut text_col = div().flex_1().min_w_0().flex().flex_col().child(part(format!("row{n}Title"), "generic").text(task.title.clone()).wrap(title_el));
        if let Some(nt) = &notes {
            let mut c = t.content_2;
            if task.done {
                c = t.content_3;
                c.a *= 0.55;
            }
            text_col = text_col.child(part(format!("row{n}Notes"), "generic").text(nt.clone()).wrap(div().text_size(px(12.5)).line_height(px(17.)).text_color(c).truncate().child(nt.clone())));
        }

        let chip = |child: AnyElement| {
            div()
                .h(px(17.))
                .px(px(4.))
                .flex()
                .items_center()
                .gap(px(2.))
                .rounded(px(R_FIELD))
                .border_1()
                .border_color(t.edge_2)
                .text_size(px(11.))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(t.content_3)
                .child(child)
        };
        let mut badges = div().flex().flex_row().items_center().gap(px(6.)).flex_none();
        if !kids.is_empty() {
            let s = format!("{}/{}", kids.iter().filter(|d| **d).count(), kids.len());
            badges = badges.child(part(format!("row{n}Subtasks"), "status").text(s.clone()).wrap(chip(div().child(s).into_any_element())));
        }
        if !task.attachments.is_empty() {
            let s = task.attachments.len().to_string();
            badges = badges.child(part(format!("row{n}Attachments"), "status").text(s.clone()).wrap(chip(div().flex().items_center().gap(px(2.)).child(icon("attach_file", 11., t.content_3)).child(s).into_any_element())));
        }
        let more_id = id.clone();
        let more = part(format!("row{n}More"), "button").name("More").focus(&fh_more, window).wrap(
            pressable(("rowMore", pane * 100000 + n), &fh_more, Rc::new(|_, _| {}))
                .on_click(cx.listener(move |ws, e: &ClickEvent, _, cx| {
                    cx.stop_propagation();
                    ws.focused_pane = pane;
                    ws.open_menu(&more_id, e.position(), cx);
                }))
                .w(px(22.))
                .h(px(19.))
                .rounded(px(R_FIELD))
                .flex()
                .items_center()
                .justify_center()
                .hover(|s| s.bg(t.wash_2))
                .child(icon("more_horiz", 16., t.content_3)),
        );

        let press_id = id.clone();
        let row_el = div()
            .id(("row", pane * 100000 + n))
            .group(group.clone())
            .relative()
            .w_full()
            .min_h(px(31.5))
            .flex()
            .flex_row()
            .items_start()
            .gap(px(12.))
            .pl(px(20. + 24. * depth))
            .pr(px(12.))
            .py(px(if notes.is_some() { 5. } else { 6.25 }))
            .when(selected, |d| d.bg(t.selection_1))
            .when(!selected, |d| d.hover(|s| s.bg(t.wash_1)))
            .on_mouse_down(MouseButton::Left, cx.listener(move |ws, e: &gpui::MouseDownEvent, w, cx| {
                ws.press_row(pane, &press_id, e.modifiers.shift, e.modifiers.platform || e.modifiers.control, w, cx);
            }))
            .on_mouse_down(MouseButton::Right, cx.listener({
                let id = id.clone();
                move |ws, e: &gpui::MouseDownEvent, _, cx| ws.open_menu(&id, e.position, cx)
            }))
            .children(toggle)
            .child(check)
            .child(text_col)
            .child(badges)
            .child(more)
            .child(div().absolute().bottom_0().right_0().left(px(24. * depth + 44.)).h(px(1.)).bg(t.edge_1));
        part(format!("row{n}"), "button").text(task.title.clone()).name(task.title.clone()).selected(selected).wrap(row_el).into_any_element()
    }

    fn render_footer(&mut self, pane: usize, list_id: &str, multi: &[String], t: &Theme, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let base = div().h(px(32.)).flex_none().flex().flex_row().items_center().border_t_1().border_color(t.edge_2).px(px(8.)).gap(px(8.));
        if multi.len() >= 2 {
            let n = multi.len();
            let mut bar = base.child(
                part("selectedCount", "status")
                    .text(format!("{n} SELECTED"))
                    .wrap(div().pl(px(6.)).text_size(px(11.)).font_weight(FontWeight::SEMIBOLD).text_color(t.accent_500).child(format!("{n} SELECTED"))),
            );
            bar = bar.child(div().flex_1());
            let items: [(&str, &str, &str); 6] =
                [("bulkDone", "Done", "check"), ("bulkUndone", "Undone", "radio_button_unchecked"), ("bulkMove", "Move to…", "arrow_forward"), ("bulkCopy", "Copy", "content_copy"), ("bulkDelete", "Delete", "delete"), ("bulkClear", "Clear", "close")];
            for (name, label, ic) in items {
                let fh = self.fh(name, cx);
                let ids = multi.to_vec();
                let h = self.on(cx, move |ws, w, cx| match name {
                    "bulkDone" => ws.set_many_done(&ids, true, cx),
                    "bulkUndone" => ws.set_many_done(&ids, false, cx),
                    "bulkCopy" => ws.copy_tasks(&ids, cx),
                    "bulkDelete" => ws.ask_delete(ids.clone(), w, cx),
                    "bulkClear" => ws.panes[pane].selection.clear(),
                    "bulkMove" => {
                        if let Some(first) = ids.first() {
                            ws.open_menu(first, point(px(400.), px(600.)), cx);
                            if let Some(m) = &mut ws.menu {
                                m.submenu = true;
                            }
                        }
                    }
                    _ => {}
                });
                let danger = name == "bulkDelete";
                bar = bar.child(
                    part(name, "button").text(label).focus(&fh, window).wrap(
                        pressable(name, &fh, h)
                            .h(px(24.))
                            .px(px(7.))
                            .flex()
                            .items_center()
                            .gap(px(4.))
                            .rounded(px(R_CONTROL))
                            .text_size(px(12.5))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(if danger { t.danger_500 } else { t.content_2 })
                            .hover(|s| s.bg(t.wash_2))
                            .child(icon(ic, 14., if danger { t.danger_500 } else { t.content_3 }))
                            .child(label),
                    ),
                );
            }
            return bar.into_any_element();
        }
        let all = self.list_tasks(list_id);
        let total = all.len();
        let done = all.iter().filter(|t| t.done).count();
        let pct = if total == 0 { 0 } else { ((done as f64 / total as f64) * 100.0).round() as usize };
        let filtered = !self.view_of(list_id).is_default();
        let fh = self.fh(&format!("viewControl{pane}"), cx);
        let _ = now_millis();
        base.child(
            part("viewControl", "button").name(if filtered { "Filtered" } else { "Sort and filter" }).text(if filtered { "Filtered" } else { "" }).focus(&fh, window).wrap(
                pressable(("viewControl", pane), &fh, self.on(cx, move |ws, _, cx| {
                    ws.focused_pane = pane;
                    ws.toggle_filter(cx)
                }))
                .h(px(24.))
                .px(px(4.))
                .flex()
                .items_center()
                .gap(px(4.))
                .rounded(px(R_CONTROL))
                .hover(|s| s.bg(t.wash_2))
                .child(icon("tune", 15., if filtered { t.accent_500 } else { t.content_3 }))
                .when(filtered, |d| d.child(div().text_size(px(11.)).font_weight(FontWeight::SEMIBOLD).text_color(t.accent_500).child("Filtered"))),
            ),
        )
        .child(div().w(px(1.)).h(px(16.)).bg(t.edge_2))
        .child(part("doneCount", "status").text(format!("{done}/{total} DONE")).wrap(div().text_size(px(11.)).font_weight(FontWeight::SEMIBOLD).text_color(t.content_3).child(format!("{done}/{total} DONE"))))
        .child(div().flex_1())
        .child(div().w(px(84.)).h(px(4.)).rounded_full().bg(t.wash_2).child(div().h_full().rounded_full().bg(t.accent_500).w(px(84. * pct as f32 / 100.))))
        .child(part("donePercent", "status").text(format!("{pct}%")).wrap(div().pr(px(4.)).text_size(px(11.)).font_weight(FontWeight::SEMIBOLD).text_color(t.content_2).child(format!("{pct}%"))))
        .into_any_element()
    }

    /// The row's menu, at the pointer.
    pub(crate) fn render_menu(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let t = self.theme();
        let Some(menu) = &self.menu else { return div().into_any_element() };
        let (task_id, at, submenu) = (menu.task.clone(), menu.at, menu.submenu);
        let Some(task) = self.task(&task_id).cloned() else { return div().into_any_element() };
        let item = |ws: &mut Workspace, name: &'static str, label: String, ic: &'static str, danger: bool, h: crate::ui::Handler, window: &Window, cx: &mut Context<Workspace>| {
            let fh = ws.fh(name, cx);
            part(name, "button").text(label.clone()).focus(&fh, window).wrap(
                pressable(name, &fh, h)
                    .h(px(28.))
                    .px(px(8.))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .rounded(px(R_CONTROL))
                    .text_size(px(13.))
                    .text_color(if danger { t.danger_500 } else { t.content_1 })
                    .hover(|s| s.bg(t.wash_2))
                    .child(icon(ic, 16., if danger { t.danger_500 } else { t.content_3 }))
                    .child(label),
            )
        };
        let id = task_id.clone();
        let mut items: Vec<AnyElement> = vec![];
        let h = self.on(cx, { let id = id.clone(); move |ws, _, cx| { ws.menu = None; ws.copy_tasks(&[id.clone()], cx) } });
        items.push(item(self, "menuCopy", "Copy".into(), "content_copy", false, h, window, cx).into_any_element());
        let h = self.on(cx, { let id = id.clone(); move |ws, w, cx| ws.open_composer(&id, w, cx) });
        items.push(item(self, "menuSubtask", "Add subtask".into(), "subdirectory_arrow_right", false, h, window, cx).into_any_element());
        let h = self.on(cx, { let id = id.clone(); move |ws, w, cx| { ws.menu = None; ws.focus_task(&id, w, cx); ws.preview = true; ws.start_rename(w, cx) } });
        items.push(item(self, "menuRename", "Rename".into(), "edit", false, h, window, cx).into_any_element());
        let h = self.on(cx, |ws, _, cx| { if let Some(m) = &mut ws.menu { m.submenu = !m.submenu; } cx.notify() });
        let mv = item(self, "menuMove", "Move to list".into(), "arrow_forward", false, h, window, cx)
            .into_any_element();
        let hover_open = cx.listener(|ws, hovered: &bool, _, cx| {
            if *hovered {
                if let Some(m) = &mut ws.menu {
                    m.submenu = true;
                    cx.notify();
                }
            }
        });
        items.push(div().id("menuMoveHover").on_hover(hover_open).child(mv).into_any_element());
        if task.parent_id.is_some() {
            let h = self.on(cx, { let id = id.clone(); move |ws, _, cx| { ws.menu = None; let l = ws.task(&id).map(|t| t.list_id.clone()); ws.move_task(&id, None, l.as_deref(), None, cx) } });
            items.push(item(self, "menuPromote", "Promote to top level".into(), "account_tree", false, h, window, cx).into_any_element());
        }
        items.push(div().h(px(1.)).my(px(4.)).bg(t.edge_1).into_any_element());
        let h = self.on(cx, { let id = id.clone(); move |ws, w, cx| { ws.menu = None; ws.ask_delete(vec![id.clone()], w, cx) } });
        items.push(item(self, "menuDelete", "Delete task…".into(), "delete", true, h, window, cx).into_any_element());

        let card = |children: Vec<AnyElement>| {
            div()
                .id("menu")
                .key_context("Menu")
                .occlude()
                .w(px(200.))
                .p(px(4.))
                .rounded(px(R_PANEL))
                .bg(t.surface_3)
                .border_1()
                .border_color(t.edge_2)
                .shadow(vec![
                    BoxShadow { color: t.shadow_alpha(0.20), offset: point(px(0.), px(10.)), blur_radius: px(28.), spread_radius: px(0.) },
                    BoxShadow { color: t.shadow_alpha(0.12), offset: point(px(0.), px(1.)), blur_radius: px(3.), spread_radius: px(0.) },
                ])
                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                .children(children)
        };
        let mut row = div().flex().flex_row().items_start().gap(px(4.)).child(card(items));
        if submenu {
            let lists: Vec<(String, String)> = self.project_lists().iter().map(|l| (l.id.clone(), l.name.clone())).collect();
            let mut subs: Vec<AnyElement> = vec![];
            for (i, (lid, name)) in lists.into_iter().enumerate() {
                let tid = id.clone();
                let name_s = format!("submenuItem{}", i + 1);
                let fh = self.fh(&name_s, cx);
                let current = lid == task.list_id;
                let h = self.on(cx, move |ws, _, cx| {
                    ws.menu = None;
                    let multi = ws.multi();
                    let ids = if multi.contains(&tid) { multi } else { vec![tid.clone()] };
                    for i in ids {
                        ws.move_task(&i, None, Some(&lid), None, cx);
                    }
                });
                subs.push(
                    part(name_s.clone(), "button").text(name.clone()).disabled(current).focus(&fh, window).wrap(
                        pressable(gpui::SharedString::from(name_s), &fh, h)
                            .h(px(28.))
                            .px(px(8.))
                            .flex()
                            .items_center()
                            .rounded(px(R_CONTROL))
                            .text_size(px(13.))
                            .text_color(if current { t.content_3 } else { t.content_1 })
                            .hover(|s| s.bg(t.wash_2))
                            .child(name),
                    )
                    .into_any_element(),
                );
            }
            row = row.child(div().mt(px(84.)).child(card(subs)));
        }
        let _ = window;
        deferred(anchored().position(at).snap_to_window().child(row)).with_priority(2).into_any_element()
    }
}

use std::rc::Rc;
#[allow(dead_code)]
fn _unused(_: Pixels, _: SheetKind) {}
