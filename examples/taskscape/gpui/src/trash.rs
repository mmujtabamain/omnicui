//! The Trash (SPEC-PRODUCT.md, "taskscape.trash"), in the rail: deleted tasks, newest first.

use gpui::{div, prelude::*, px, AnyElement, Context, FontWeight, Window};
use taskscape_common::util::now_millis;

use crate::model::relative;
use crate::parts::part;
use crate::theme::{R_CONTROL, R_FIELD};
use crate::ui::{icon, pressable};
use crate::workspace::{SheetKind, Workspace};

impl Workspace {
    pub(crate) fn render_trash(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let t = self.theme();
        let now = now_millis();
        // A deleted task's deleted subtasks go with it and aren't listed.
        let trashed: Vec<taskscape_common::Task> =
            self.data.trashed.iter().filter(|x| x.parent_id.as_deref().map_or(true, |p| !self.data.trashed.iter().any(|y| y.id == p))).cloned().collect();
        let (fh_close, fh_all, fh_empty) = (self.fh("dialogClose", cx), self.fh("restoreAll", cx), self.fh("emptyTrash", cx));
        let top_ids: Vec<String> = trashed.iter().map(|x| x.id.clone()).collect();
        let all_ids: Vec<String> = self.data.trashed.iter().map(|x| x.id.clone()).collect();
        let text_btn = |name: &'static str, label: &'static str, ic: &'static str, danger: bool, fh: &gpui::FocusHandle, h: crate::ui::Handler, disabled: bool, window: &Window| {
            part(name, "button").text(label).disabled(disabled).focus(fh, window).wrap(
                pressable(name, fh, h)
                    .h(px(26.))
                    .px(px(8.))
                    .flex()
                    .items_center()
                    .gap(px(4.))
                    .rounded(px(R_CONTROL))
                    .text_size(px(12.5))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(if danger { t.danger_500 } else { t.content_2 })
                    .when(disabled, |d| d.opacity(0.45))
                    .hover(|s| s.bg(t.wash_2))
                    .child(icon(ic, 14., if danger { t.danger_500 } else { t.content_3 }))
                    .child(label),
            )
        };
        let header = div()
            .h(px(46.))
            .flex_none()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(8.))
            .px(px(12.))
            .border_b_1()
            .border_color(t.edge_1)
            .child(icon("delete", 17., t.content_3))
            .child(div().flex_1().text_size(px(15.)).font_weight(FontWeight::SEMIBOLD).child("Trash"))
            .child(
                part("dialogClose", "button").name("Close").focus(&fh_close, window).wrap(
                    pressable("dialogClose", &fh_close, self.on(cx, |ws, _, cx| ws.close_rail_panel(cx)))
                        .size(px(24.))
                        .rounded(px(R_FIELD))
                        .flex()
                        .items_center()
                        .justify_center()
                        .hover(|s| s.bg(t.wash_2))
                        .child(icon("close", 16., t.content_3)),
                ),
            );
        let empty = trashed.is_empty();
        let actions = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(4.))
            .px(px(8.))
            .py(px(6.))
            .child(text_btn("restoreAll", "Restore all", "restore", false, &fh_all, self.on(cx, move |ws, _, cx| ws.restore(&top_ids, &all_ids, cx)), empty, window))
            .child(div().flex_1())
            .child(text_btn("emptyTrash", "Empty", "delete_sweep", true, &fh_empty, self.on(cx, |ws, w, cx| ws.open_sheet(SheetKind::EmptyTrash, w, cx)), empty, window));
        let mut list = div().id("trashList").flex_1().min_h_0().overflow_y_scroll().flex().flex_col();
        if empty {
            list = list.child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap(px(6.))
                    .pt(px(200.))
                    .child(icon("delete", 28., t.content_3))
                    .child(part("trashEmptyTitle", "heading").text("Trash is empty").wrap(div().text_size(px(15.)).font_weight(FontWeight::MEDIUM).text_color(t.content_2).child("Trash is empty"))),
            );
        }
        for (i, task) in trashed.iter().enumerate() {
            let k = i + 1;
            let lname = self.list(&task.list_id).map(|l| l.name.clone()).unwrap_or_else(|| "Deleted list".into());
            let when = format!("{lname} · deleted {}", relative(task.deleted_at.unwrap_or(now), now));
            let (fh_r, fh_p) = (self.fh(&format!("trashed{k}Restore"), cx), self.fh(&format!("trashed{k}Purge"), cx));
            let stamped: Vec<String> = {
                let mut v = vec![task.id.clone()];
                let mut frontier = vec![task.id.clone()];
                while let Some(p) = frontier.pop() {
                    for c in self.data.trashed.iter().filter(|x| x.parent_id.as_deref() == Some(&p)) {
                        v.push(c.id.clone());
                        frontier.push(c.id.clone());
                    }
                }
                v
            };
            let (id1, id2) = (task.id.clone(), task.id.clone());
            let small = |name: String, ic: &'static str, label: &'static str, danger: bool, fh: &gpui::FocusHandle, h: crate::ui::Handler, window: &Window| {
                part(name.clone(), "button").name(label).focus(fh, window).wrap(
                    pressable(gpui::SharedString::from(name), fh, h)
                        .size(px(24.))
                        .rounded(px(R_FIELD))
                        .flex()
                        .items_center()
                        .justify_center()
                        .opacity(0.)
                        .group_hover("trashed", |s| s.opacity(1.))
                        .hover(|s| s.bg(t.wash_2))
                        .child(icon(ic, 15., if danger { t.danger_500 } else { t.content_3 })),
                )
            };
            list = list.child(
                div()
                    .id(("trashed", k))
                    .group("trashed")
                    .min_h(px(46.))
                    .px(px(12.))
                    .py(px(5.))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(8.))
                    .border_b_1()
                    .border_color(t.edge_1)
                    .hover(|s| s.bg(t.wash_1))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .child(part(format!("trashed{k}Title"), "generic").text(task.title.clone()).wrap(div().text_size(px(14.)).font_weight(FontWeight::MEDIUM).truncate().child(task.title.clone())))
                            .child(part(format!("trashed{k}When"), "status").text(when.clone()).wrap(div().text_size(px(11.5)).text_color(t.content_3).truncate().child(when))),
                    )
                    .child(small(format!("trashed{k}Restore"), "restore", "Restore", false, &fh_r, self.on(cx, move |ws, _, cx| ws.restore(&[id1.clone()], &stamped, cx)), window))
                    .child(small(format!("trashed{k}Purge"), "delete_forever", "Delete permanently", true, &fh_p, self.on(cx, move |ws, _, cx| ws.purge(&[id2.clone()], cx)), window)),
            );
        }
        div().size_full().flex().flex_col().bg(t.surface_1).child(header).child(actions).child(list).into_any_element()
    }
}
