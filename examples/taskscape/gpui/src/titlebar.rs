//! The title bar (SPEC-PRODUCT.md, "taskscape.titlebar"; DESIGN.md, "Title bar"), drawn by the
//! app in the window's title area the way Zed draws its own: the project pill, the list tabs and
//! the view buttons, and the platform's window controls (macOS keeps its traffic lights; on
//! Linux and Windows the app draws the controls at the trailing edge, as Zed does there).

use gpui::{anchored, deferred, div, point, prelude::*, px, AnyElement, Context, FontWeight, MouseButton, Window};
use serde_json::json;

use crate::motion::SHEET_IN;
use crate::parts::part;
use crate::report;
use crate::theme::{R_CONTROL, R_FIELD, R_PANEL};
use crate::ui::{icon, icon_button, pressable};
use crate::workspace::{SheetKind, Workspace};

impl Workspace {
    pub(crate) fn render_titlebar(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let t = self.theme();
        let project_name = self.active_project.as_deref().and_then(|p| self.project(p)).map(|p| p.name.clone()).unwrap_or_default();
        let lists: Vec<(String, String)> = self.project_lists().iter().map(|l| (l.id.clone(), l.name.clone())).collect();
        let left = self.panes.first().and_then(|p| p.list.clone());
        let right = self.panes.get(1).and_then(|p| p.list.clone());
        let focused = self.pane_list();
        let split = self.panes.len() > 1;

        let mut bar = div()
            .id("titlebar")
            .h(px(46.))
            .w_full()
            .flex_none()
            .flex()
            .flex_row()
            .items_center()
            .bg(t.surface_1)
            .border_b_1()
            .border_color(t.edge_2)
            .on_mouse_down(MouseButton::Left, |e, window, _| {
                if e.click_count == 2 {
                    window.titlebar_double_click();
                } else {
                    window.start_window_move();
                }
            });

        if cfg!(target_os = "macos") {
            // The native traffic lights sit here.
            bar = bar.child(part("windowControls", "generic").wrap(div().w(px(84.)).h_full()));
        } else {
            bar = bar.child(div().w(px(12.)));
        }
        bar = bar.child(div().px(px(4.)).child(icon("drag_indicator", 16., t.content_3)));

        // The project pill.
        let fh_pill = self.fh("project", cx);
        bar = bar.child(
            part("project", "button").text(project_name.clone()).name(project_name.clone()).focus(&fh_pill, window).wrap(
                pressable("project", &fh_pill, self.on(cx, |ws, _, cx| {
                    ws.project_menu = !ws.project_menu;
                    ws.anims.reset("projectMenu");
                    cx.notify()
                }))
                .tooltip(|_, cx| cx.new(|_| crate::ui::Tooltip("Switch project".into())).into())
                .h(px(28.))
                .px(px(12.))
                .ml(px(4.))
                .mr(px(14.))
                .flex()
                .items_center()
                .gap(px(6.))
                .rounded(px(R_CONTROL))
                .bg(t.surface_0)
                .hover(|s| s.bg(t.wash_2))
                .text_size(px(14.))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(t.content_1)
                .child(project_name)
                .child(icon("unfold_more", 13., t.content_3)),
            ),
        );

        // The list tabs.
        let n_lists = lists.len();
        for (i, (lid, name)) in lists.into_iter().enumerate() {
            let k = i + 1;
            let shown = Some(&lid) == left.as_ref() || Some(&lid) == right.as_ref();
            let is_focused = Some(&lid) == focused.as_ref();
            let (fh_tab, fh_close) = (self.fh(&format!("tab{k}"), cx), self.fh(&format!("tab{k}Close"), cx));
            let l2 = lid.clone();
            let close = part(format!("tab{k}Close"), "button").name("Delete list").focus(&fh_close, window).wrap(
                pressable(("tabClose", k), &fh_close, self.on(cx, move |ws, w, cx| ws.open_sheet(SheetKind::DeleteList(l2.clone()), w, cx)))
                    .size(px(18.))
                    .rounded(px(R_FIELD))
                    .flex()
                    .items_center()
                    .justify_center()
                    .hover(|s| s.bg(t.wash_2))
                    .child(icon("close", 13., t.content_3)),
            );
            let marker = (split && shown).then(|| {
                let side = if Some(&lid) == left.as_ref() { "L" } else { "R" };
                div().text_size(px(9.)).font_weight(FontWeight::BOLD).text_color(t.accent_500).child(side)
            });
            let l3 = lid.clone();
            let tab = pressable(("tab", k), &fh_tab, self.on(cx, move |ws, w, cx| ws.show_list(&l3, w, cx)))
                .on_click(cx.listener({
                    let lid = lid.clone();
                    move |ws, e: &gpui::ClickEvent, w, cx| {
                        if e.click_count() >= 2 {
                            ws.open_sheet(SheetKind::RenameList(lid.clone()), w, cx)
                        }
                    }
                }))
                .h_full()
                .px(px(12.))
                .flex()
                .items_center()
                .gap(px(8.))
                .when(is_focused, |d| d.bg(t.surface_2).border_l_1().border_r_1().border_color(t.edge_2).mb(px(-1.)))
                .when(!is_focused, |d| d.hover(|s| s.bg(t.wash_1)))
                .children(marker)
                .child(
                    part(format!("tab{k}Name"), "generic")
                        .text(name.clone())
                        .wrap(div().text_size(px(13.)).font_weight(FontWeight::MEDIUM).text_color(if is_focused { t.content_1 } else { t.content_3 }).child(name.clone())),
                )
                .child(close);
            bar = bar.child(part(format!("tab{k}"), "button").text(name.clone()).name(name).selected(is_focused).focus(&fh_tab, window).wrap(tab));
            if i + 1 < n_lists && !is_focused {
                bar = bar.child(div().w(px(1.)).h(px(14.)).bg(t.edge_2));
            }
        }
        let fh_new = self.fh("newList", cx);
        bar = bar.child(div().pl(px(8.)).child(part("newList", "button").name("New list").focus(&fh_new, window).wrap(icon_button("newList", "add", &fh_new, &t, self.on(cx, |ws, w, cx| ws.open_sheet(SheetKind::NewList, w, cx))))));
        bar = bar.child(div().flex_1().h_full());

        // The view buttons.
        let mut views = div().flex().flex_row().items_center().gap(px(5.)).pr(px(12.));
        if self.reduced_motion {
            views = views.child(div().px(px(4.)).child(icon("motion_photos_paused", 16., t.content_3)));
        }
        if split {
            let fh = self.fh("swapPanes", cx);
            views = views.child(part("swapPanes", "button").name("Swap panes").focus(&fh, window).wrap(icon_button("swapPanes", "swap_horiz", &fh, &t, self.on(cx, |ws, w, cx| ws.swap_panes(w, cx)))));
        }
        let fh = self.fh("splitView", cx);
        views = views.child(
            part("splitView", "button")
                .name(if split { "Close split view" } else { "Split view" })
                .selected(split)
                .focus(&fh, window)
                .wrap(icon_button("splitView", "vertical_split", &fh, &t, self.on(cx, |ws, w, cx| ws.toggle_split(w, cx))).when(split, |d| d.bg(t.wash_2))),
        );
        let fh = self.fh("togglePreview", cx);
        let pv = format!("{} preview panel  {}", if self.preview { "Hide" } else { "Show" }, self.glyphs("toggle_preview"));
        views = views.child(part("togglePreview", "button").name(pv).selected(self.preview).focus(&fh, window).wrap(icon_button(
            "togglePreview",
            if self.preview { "right_panel_close" } else { "right_panel_open" },
            &fh,
            &t,
            self.on(cx, |ws, _, cx| ws.set_preview(!ws.preview, cx)),
        )));
        let fh = self.fh("openSettings", cx);
        let st = format!("Settings  {}", self.glyphs("open_settings"));
        views = views.child(part("openSettings", "button").name(st).focus(&fh, window).wrap(icon_button("openSettings", "settings", &fh, &t, self.on(cx, |ws, w, cx| ws.open_settings(w, cx)))));
        bar = bar.child(views);

        if !cfg!(target_os = "macos") {
            bar = bar.child(self.window_controls(window, cx));
        }
        bar.into_any_element()
    }

    /// Minimise, zoom and close, drawn by the app as Zed draws them on Linux.
    fn window_controls(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let t = self.theme();
        let ctl = |ws: &mut Workspace, name: &'static str, ic: &'static str, label: &'static str, action: &'static str, window: &Window, cx: &mut Context<Workspace>| {
            let fh = ws.fh(name, cx);
            part(name, "button").name(label).focus(&fh, window).wrap(
                pressable(name, &fh, ws.on(cx, move |_, w, cx| {
                    report(cx, "windowAction", json!({ "action": action }));
                    match action {
                        "close" => w.remove_window(),
                        "minimize" => w.minimize_window(),
                        _ => w.zoom_window(),
                    }
                }))
                .size(px(24.))
                .rounded_full()
                .flex()
                .items_center()
                .justify_center()
                .bg(t.wash_2)
                .hover(|s| s.bg(t.edge_2))
                .child(icon(ic, 14., t.content_2)),
            )
        };
        let row = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(8.))
            .pr(px(12.))
            .pl(px(4.))
            .child(ctl(self, "windowMinimize", "minimize", "Minimize", "minimize", window, cx))
            .child(ctl(self, "windowZoom", "crop_square", "Zoom", "zoom", window, cx))
            .child(ctl(self, "windowClose", "close", "Close", "close", window, cx));
        part("windowControls", "generic").wrap(row).into_any_element()
    }

    pub(crate) fn render_project_menu(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let t = self.theme();
        let p = self.anims.progress("projectMenu", "open", SHEET_IN, window);
        let projects: Vec<(String, String)> = self.data.projects.iter().map(|p| (p.id.clone(), p.name.clone())).collect();
        let mut card = div()
            .id("projectMenu")
            .key_context("Menu")
            .occlude()
            .w(px(220.))
            .p(px(4.))
            .rounded(px(R_PANEL))
            .bg(t.surface_3)
            .border_1()
            .border_color(t.edge_2)
            .opacity(p)
            .shadow(crate::sheet::menu_shadow(&t, p))
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation());
        for (i, (pid, name)) in projects.into_iter().enumerate() {
            let k = i + 1;
            let fh = self.fh(&format!("projectItem{k}"), cx);
            let active = self.active_project.as_deref() == Some(&pid);
            let p2 = pid.clone();
            card = card.child(
                part(format!("projectItem{k}"), "button").text(name.clone()).selected(active).focus(&fh, window).wrap(
                    pressable(("projectItem", k), &fh, self.on(cx, move |ws, w, cx| ws.choose_project(&p2, w, cx)))
                        .h(px(30.))
                        .px(px(8.))
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .rounded(px(R_CONTROL))
                        .hover(|s| s.bg(t.wash_2))
                        .child(div().w(px(16.)).child(if active { icon("check", 15., t.accent_500).into_any_element() } else { div().into_any_element() }))
                        .child(part(format!("projectItem{k}Name"), "generic").text(name.clone()).wrap(div().flex_1().text_size(px(13.5)).text_color(t.content_1).child(name))),
                ),
            );
        }
        let fh = self.fh("newProject", cx);
        card = card.child(div().h(px(1.)).my(px(4.)).bg(t.edge_1)).child(
            part("newProject", "button").text("New project…").focus(&fh, window).wrap(
                pressable("newProject", &fh, self.on(cx, |ws, w, cx| ws.open_sheet(SheetKind::NewProject, w, cx)))
                    .h(px(30.))
                    .px(px(8.))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .rounded(px(R_CONTROL))
                    .text_size(px(13.5))
                    .text_color(t.content_1)
                    .hover(|s| s.bg(t.wash_2))
                    .child(icon("add", 15., t.content_3))
                    .child("New project…"),
            ),
        );
        let x = if cfg!(target_os = "macos") { 112. } else { 40. };
        deferred(anchored().position(point(px(x), px(42.))).child(part("projectMenu", "generic").opacity(p).wrap(card))).with_priority(2).into_any_element()
    }
}
