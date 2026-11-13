//! Sort and filter (SPEC-PRODUCT.md, "taskscape.filter"), in the rail. Display only: it never
//! changes a task.

use std::collections::HashSet;

use gpui::{div, prelude::*, px, AnyElement, Context, FontWeight, Window};
use taskscape_common::util::now_millis;

use crate::model::{self, Has, Show, Sort, View};
use crate::parts::part;
use crate::theme::{Theme, R_CONTROL, R_FIELD};
use crate::ui::{icon, pressable, section_label};
use crate::workspace::Workspace;

impl Workspace {
    /// A segmented control (the app's Segmented): the chosen segment filled with the accent.
    #[allow(clippy::too_many_arguments)]
    fn segments<T: Copy + PartialEq + 'static>(&mut self, list: &str, opts: &[(&'static str, &'static str, T)], cur: T, set: fn(&mut View, T), compact: bool, t: &Theme, window: &Window, cx: &mut Context<Self>) -> AnyElement {
        let mut row = div().flex().flex_row().p(px(2.)).gap(px(2.)).rounded(px(R_CONTROL)).bg(t.surface_0);
        if compact {
            row = row.w(px(138.)).flex_none();
        }
        for (name, label, v) in opts.iter().copied() {
            let fh = self.fh(name, cx);
            let l = list.to_string();
            let on = cur == v;
            row = row.child(
                part(name, "button").text(label).selected(on).focus(&fh, window).wrap(
                    pressable(name, &fh, self.on(cx, move |ws, _, cx| {
                        let mut view = ws.view_of(&l);
                        set(&mut view, v);
                        ws.set_view(&l, view, cx);
                    }))
                    .flex_1()
                    .h(px(if compact { 22. } else { 24. }))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(R_FIELD))
                    .text_size(px(if compact { 11.5 } else { 12.5 }))
                    .font_weight(FontWeight::SEMIBOLD)
                    .when(on, |d| d.bg(t.accent_500).text_color(t.on_accent))
                    .when(!on, |d| d.text_color(t.content_2).hover(|s| s.bg(t.wash_2)))
                    .child(label),
                ),
            );
        }
        row.into_any_element()
    }

    /// A select drawn as the app draws one: a recessed well with the value and a chevron;
    /// a press steps to the next option.
    fn select_well(&mut self, name: &'static str, ic: &'static str, label: String, t: &Theme, h: crate::ui::Handler, window: &Window, cx: &mut Context<Self>) -> AnyElement {
        let fh = self.fh(name, cx);
        part(name, "button").text(label.clone()).focus(&fh, window).wrap(
            pressable(name, &fh, h)
                .flex_1()
                .h(px(26.))
                .px(px(8.))
                .flex()
                .items_center()
                .gap(px(6.))
                .rounded(px(R_CONTROL))
                .bg(t.surface_0)
                .hover(|s| s.bg(t.wash_2))
                .child(icon(ic, 13., t.content_3))
                .child(div().flex_1().text_size(px(13.)).child(label))
                .child(icon("unfold_more", 13., t.content_3)),
        )
        .into_any_element()
    }

    pub(crate) fn render_filter(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let t = self.theme();
        let Some(list) = self.pane_list() else { return div().size_full().bg(t.surface_1).into_any_element() };
        let view = self.view_of(&list);
        let total = self.list_tasks(&list).len();
        let shown = {
            let tree = self.tree(&list);
            model::rows(&tree, &HashSet::new(), true, None, &view, now_millis()).len()
        };
        let summary = format!("Showing {shown} of {total} tasks");
        let list_name = self.list(&list).map(|l| l.name.clone()).unwrap_or_default();
        let (fh_close, fh_reset, fh_done) = (self.fh("filterClose", cx), self.fh("filterReset", cx), self.fh("filterDone", cx));
        let label = |s: &str| div().flex().items_center().gap(px(8.)).pt(px(4.)).child(section_label(s, &t)).child(div().flex_1().h(px(1.)).bg(t.edge_1));
        let refine_row = |ic: &'static str, s: &'static str, seg: AnyElement| {
            div().h(px(31.)).flex().items_center().gap(px(8.)).child(icon(ic, 14., t.content_3)).child(div().flex_1().text_size(px(13.)).text_color(t.content_2).child(s)).child(seg)
        };

        let l = list.clone();
        let sort_sel = self.select_well("sortSelect", "drag_indicator", view.sort.label().to_string(), &t, self.on(cx, move |ws, _, cx| {
            let mut v = ws.view_of(&l);
            let i = Sort::ALL.iter().position(|s| *s == v.sort).unwrap_or(0);
            v.sort = Sort::ALL[(i + 1) % Sort::ALL.len()];
            ws.set_view(&l, v, cx);
        }), window, cx);
        let dir = |ws: &mut Workspace, name: &'static str, ic: &'static str, asc: bool, window: &Window, cx: &mut Context<Workspace>| {
            let fh = ws.fh(name, cx);
            let l = list.clone();
            let on = view.asc == asc && view.sort != Sort::Manual;
            part(name, "button").name(if asc { "Ascending" } else { "Descending" }).selected(on).focus(&fh, window).wrap(
                pressable(name, &fh, ws.on(cx, move |ws, _, cx| {
                    let mut v = ws.view_of(&l);
                    if v.sort != Sort::Manual {
                        v.asc = asc;
                        ws.set_view(&l, v, cx);
                    }
                }))
                .size(px(24.))
                .rounded(px(R_FIELD))
                .flex()
                .items_center()
                .justify_center()
                .when(on, |d| d.bg(t.surface_3))
                .child(icon(ic, 13., if view.sort == Sort::Manual { t.edge_3 } else { t.content_2 })),
            )
        };
        let dirs = div().flex().flex_row().p(px(1.)).gap(px(1.)).rounded(px(R_CONTROL)).bg(t.surface_0).child(dir(self, "sortAsc", "arrow_upward", true, window, cx)).child(dir(self, "sortDesc", "arrow_downward", false, window, cx));

        let show = self.segments(&list, &[("showAll", "All", Show::All), ("showActive", "Active", Show::Active), ("showCompleted", "Done", Show::Completed)], view.show, |v, x| v.show = x, false, &t, window, cx);
        let notes = self.segments(&list, &[("notesAny", "Any", Has::Any), ("notesHas", "With", Has::With), ("notesNone", "Without", Has::Without)], view.notes, |v, x| v.notes = x, true, &t, window, cx);
        let atts = self.segments(&list, &[("attachmentsAny", "Any", Has::Any), ("attachmentsHas", "With", Has::With), ("attachmentsNone", "Without", Has::Without)], view.attachments, |v, x| v.attachments = x, true, &t, window, cx);
        let subs = self.segments(&list, &[("subtasksAny", "Any", Has::Any), ("subtasksHas", "With", Has::With), ("subtasksNone", "Without", Has::Without)], view.subtasks, |v, x| v.subtasks = x, true, &t, window, cx);
        let date_field = self.segments(&list, &[("dateCreated", "Created", false), ("dateUpdated", "Updated", true)], view.date_field_updated, |v, x| v.date_field_updated = x, false, &t, window, cx);
        let windows: [(Option<u32>, &str); 4] = [(None, "Any time"), (Some(1), "Last 24 hours"), (Some(7), "Last 7 days"), (Some(30), "Last 30 days")];
        let cur_w = windows.iter().find(|w| w.0 == view.within_days).map(|w| w.1).unwrap_or("Any time").to_string();
        let l = list.clone();
        let date = self.select_well("dateWindow", "schedule", cur_w, &t, self.on(cx, move |ws, _, cx| {
            let mut v = ws.view_of(&l);
            let i = windows.iter().position(|w| w.0 == v.within_days).unwrap_or(0);
            v.within_days = windows[(i + 1) % windows.len()].0;
            ws.set_view(&l, v, cx);
        }), window, cx);

        let is_default = view.is_default();
        let l2 = list.clone();
        let header = div()
            .h(px(38.))
            .flex_none()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(8.))
            .px(px(10.))
            .border_b_1()
            .border_color(t.edge_1)
            .child(icon("tune", 15., t.content_3))
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .child(div().text_size(px(14.)).line_height(px(17.)).font_weight(FontWeight::SEMIBOLD).child("Filter & Sort"))
                    .child(div().text_size(px(11.)).line_height(px(13.)).text_color(t.content_3).child(list_name)),
            )
            .child(
                part("filterClose", "button").name("Close").focus(&fh_close, window).wrap(
                    pressable("filterClose", &fh_close, self.on(cx, |ws, _, cx| ws.close_rail_panel(cx)))
                        .size(px(24.))
                        .rounded(px(R_FIELD))
                        .flex()
                        .items_center()
                        .justify_center()
                        .hover(|s| s.bg(t.wash_2))
                        .child(icon("close", 15., t.content_3)),
                ),
            );
        let body = div()
            .id("filterBody")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .p(px(12.))
            .flex()
            .flex_col()
            .gap(px(9.))
            .child(label("Sort"))
            .child(div().flex().flex_row().gap(px(4.)).child(sort_sel).child(dirs))
            .child(label("Show"))
            .child(show)
            .child(label("Refine"))
            .child(div().flex().flex_col().child(refine_row("notes", "Notes", notes)).child(refine_row("attach_file", "Attachments", atts)).child(refine_row("account_tree", "Subtasks", subs)))
            .child(label("Date"))
            .child(date_field)
            .child(div().flex().child(date));
        let summary_row = div()
            .h(px(32.))
            .flex_none()
            .flex()
            .items_center()
            .gap(px(6.))
            .px(px(12.))
            .border_t_1()
            .border_color(t.edge_2)
            .child(div().size(px(5.)).rounded_full().bg(if is_default { t.content_3 } else { t.accent_500 }))
            .child(part("filterSummary", "status").text(summary.clone()).wrap(div().text_size(px(12.)).text_color(t.content_2).child(summary)));
        let footer = div()
            .h(px(46.))
            .flex_none()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(8.))
            .px(px(10.))
            .border_t_1()
            .border_color(t.edge_2)
            .child(
                part("filterReset", "button").text("Reset").disabled(is_default).focus(&fh_reset, window).wrap(
                    pressable("filterReset", &fh_reset, self.on(cx, move |ws, _, cx| {
                        if !ws.view_of(&l2).is_default() {
                            ws.set_view(&l2, View::default(), cx)
                        }
                    }))
                    .h(px(28.))
                    .px(px(8.))
                    .flex()
                    .items_center()
                    .gap(px(4.))
                    .rounded(px(R_CONTROL))
                    .text_size(px(13.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(t.content_2)
                    .when(is_default, |d| d.opacity(0.45))
                    .hover(|s| s.bg(t.wash_2))
                    .child(icon("restart_alt", 14., t.content_3))
                    .child("Reset"),
                ),
            )
            .child(div().flex_1())
            .child(
                part("filterDone", "button").text("Done").focus(&fh_done, window).wrap(
                    pressable("filterDone", &fh_done, self.on(cx, |ws, _, cx| ws.close_rail_panel(cx)))
                        .h(px(28.))
                        .px(px(14.))
                        .flex()
                        .items_center()
                        .rounded(px(R_CONTROL))
                        .bg(t.accent_500)
                        .text_size(px(13.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(t.on_accent)
                        .child("Done"),
                ),
            );
        let _ = sort_part;
        div().size_full().flex().flex_col().bg(t.surface_1).child(header).child(body).child(summary_row).child(footer).into_any_element()
    }
}

fn sort_part(s: Sort) -> &'static str {
    match s {
        Sort::Manual => "sortManual",
        Sort::Created => "sortCreated",
        Sort::Updated => "sortUpdated",
        Sort::Alpha => "sortAlpha",
        Sort::DoneLast => "sortDoneLast",
    }
}
