//! What is drawn over the window: a sheet (SPEC-PRODUCT.md, "taskscape.sheet"), the command
//! palette ("taskscape.palette") and the toast ("taskscape.toast"). All are drawn in the window
//! (WHY.md: nothing opens in a window that has to load first), and scale in where DESIGN.md says.

use gpui::{deferred, div, point, prelude::*, px, AnyElement, BoxShadow, Context, FontWeight, MouseButton, Window};
use serde_json::json;

use crate::keys::{Confirm, Dismiss, PaletteDown, PaletteUp};
use crate::motion::{RISE, SCRIM_IN, SHEET_IN};
use crate::parts::part;
use crate::report;
use crate::theme::{Theme, R_CONTROL, R_PANEL};
use crate::ui::{icon, pressable};
use crate::workspace::Workspace;

/// The menu elevation, faded with the box. GPUI's opacity doesn't reach a box's shadow, and GPUI
/// draws the shadow under the box as well as around it (CSS only around it), so while the card
/// is still translucent the shadow would darken it: it comes in only as the card becomes opaque.
pub(crate) fn menu_shadow(t: &Theme, p: f32) -> Vec<BoxShadow> {
    let p = p.powi(4);
    vec![
        BoxShadow { color: t.shadow_alpha(0.20 * p), offset: point(px(0.), px(10.)), blur_radius: px(28.), spread_radius: px(0.) },
        BoxShadow { color: t.shadow_alpha(0.12 * p), offset: point(px(0.), px(1.)), blur_radius: px(3.), spread_radius: px(0.) },
    ]
}

/// GPUI has no transform for a box, so "scale 0.96 → 1" is drawn as the box growing from 96 %
/// of its width around its centre, with the fade.
fn scale_in(width: f32, p: f32) -> f32 {
    width * (0.96 + 0.04 * p)
}

impl Workspace {
    pub(crate) fn render_sheet(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let t = self.theme();
        let Some(sheet) = &self.sheet else { return div().into_any_element() };
        let serial = sheet.serial;
        let (headline, detail, accept, instead, danger, glyph) = (sheet.headline.clone(), sheet.detail.clone(), sheet.accept.clone(), sheet.instead.clone(), sheet.danger, sheet.icon);
        let field = sheet.field.clone();
        let p = self.anims.progress("sheet", serial.to_string(), SHEET_IN, window);
        let tone = if danger { t.danger_500 } else { t.accent_500 };
        let tone_soft = if danger { t.danger_100 } else { t.selection_1 };
        let blank = field.as_ref().is_some_and(|f| f.read(cx).text().trim().is_empty());

        let mut body = div()
            .flex()
            .flex_col()
            .gap(px(8.))
            .p(px(14.))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(10.))
                    .child(div().size(px(28.)).flex_none().rounded(px(R_CONTROL)).bg(tone_soft).flex().items_center().justify_center().child(icon(glyph, 16., tone)))
                    .child(part("sheetHeadline", "heading").text(headline.clone()).wrap(div().text_size(px(15.)).font_weight(FontWeight::SEMIBOLD).truncate().child(headline.clone()))),
            );
        if let Some(d) = &detail {
            body = body.child(part("sheetDetail", "generic").text(d.clone()).wrap(div().text_size(px(12.5)).line_height(px(17.)).text_color(t.content_2).child(d.clone())));
        }
        if let Some(f) = &field {
            f.update(cx, |f, _| {
                f.caret = t.accent_500;
                f.placeholder_color = t.content_3;
                f.selection_color = t.selection_1;
            });
            let fh = f.read(cx).focus.clone();
            let mut row = div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(6.))
                .child(
                    part("sheetField", "textbox").text(f.read(cx).text()).name(headline.clone()).focus(&fh, window).wrap(
                        div().flex_1().h(px(32.)).px(px(10.)).flex().items_center().rounded(px(R_CONTROL)).bg(t.surface_0).border_1().border_color(if fh.is_focused(window) { t.focus_1 } else { t.edge_2 }).text_size(px(14.)).child(f.clone()),
                    ),
                );
            if headline == "New list" || headline == "New project" {
                let fh_d = self.fh("sheetDice", cx);
                let f2 = f.clone();
                row = row.child(
                    part("sheetDice", "button").name("Roll another name").focus(&fh_d, window).wrap(
                        pressable("sheetDice", &fh_d, self.on(cx, move |_, w, cx| {
                            let name = taskscape_common::names::suggest_list_name();
                            f2.update(cx, |f, cx| {
                                f.set_text(name, cx);
                                f.select_all_text(cx);
                            });
                            let fh = f2.read(cx).focus.clone();
                            w.focus(&fh);
                        }))
                        .size(px(32.))
                        .rounded(px(R_CONTROL))
                        .flex()
                        .items_center()
                        .justify_center()
                        .hover(|s| s.bg(t.wash_2))
                        .child(icon("casino", 17., t.content_3)),
                    ),
                );
            }
            body = body.child(row);
        }

        let button = |ws: &mut Workspace, name: &'static str, label: String, primary: bool, disabled: bool, action: &'static str, window: &Window, cx: &mut Context<Workspace>| {
            let fh = ws.fh(name, cx);
            let h = ws.on(cx, move |ws, w, cx| ws.answer_sheet(action, w, cx));
            let mut b = pressable(name, &fh, h)
                .h(px(28.))
                .min_w(px(84.))
                .px(px(12.))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(R_CONTROL))
                .text_size(px(12.5))
                .font_weight(FontWeight::SEMIBOLD);
            // One anatomy for every key action (the app's KeyAction): a fill with the same edge
            // over it; only a danger accept is filled with its tone.
            b = b.border_1().border_color(t.edge_2);
            b = if primary && danger {
                b.bg(t.danger_500).text_color(t.on_accent).hover(|s| s.border_color(t.edge_3).bg(t.danger_600))
            } else {
                b.bg(t.surface_1).text_color(t.content_1).hover(|s| s.border_color(t.edge_3).bg(t.surface_3))
            };
            if fh.is_focused(window) {
                b = b.border_2().border_color(t.focus_1);
            }
            if disabled {
                b = b.opacity(0.45);
            }
            part(name, "button").text(label.clone()).disabled(disabled).focus(&fh, window).wrap(b.child(label))
        };
        let mut footer = div().flex().flex_row().items_center().gap(px(8.)).px(px(10.)).py(px(9.)).border_t_1().border_color(t.edge_1);
        if let Some(i) = instead {
            footer = footer.child(button(self, "sheetInstead", i, false, false, "instead", window, cx));
        }
        footer = footer
            .child(div().flex_1())
            .child(button(self, "sheetCancel", "Cancel".into(), false, false, "dismiss", window, cx))
            .child(button(self, "sheetAccept", accept, true, blank, "accept", window, cx));

        let w = scale_in(364., p);
        let card = div()
            .id("sheetCard")
            .key_context("Sheet")
            .occlude()
            .w(px(w))
            .rounded(px(R_PANEL))
            .bg(t.surface_2)
            .border_1()
            .border_color(t.edge_2)
            .shadow(menu_shadow(&t, p))
            .opacity(p)
            .on_action(cx.listener(|ws, _: &Confirm, w, cx| ws.answer_sheet("accept", w, cx)))
            .on_action(cx.listener(|ws, _: &Dismiss, w, cx| ws.answer_sheet("dismiss", w, cx)))
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .child(body)
            .child(footer);
        // The window dims behind it: black at 30 % (the app's `soft` backdrop).
        let sp = self.anims.progress("scrim", serial.to_string(), SCRIM_IN, window);
        let scrim = gpui::hsla(0., 0., 0., 0.30 * sp);
        deferred(
            div()
                .id("sheetScrim")
                .absolute()
                .inset_0()
                .size_full()
                .bg(scrim)
                .occlude()
                .flex()
                .items_center()
                .justify_center()
                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                .child(part("sheet", "generic").opacity(p).wrap(card)),
        )
        .with_priority(3)
        .into_any_element()
    }

    pub(crate) fn render_palette(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let t = self.theme();
        let Some(pal) = &self.palette else { return div().into_any_element() };
        let (input, highlight, serial) = (pal.input.clone(), pal.highlight, pal.serial);
        let p = self.anims.progress("palette", serial.to_string(), SHEET_IN, window);
        let sp = self.anims.progress("scrim", serial.to_string(), SCRIM_IN, window);
        input.update(cx, |f, _| {
            f.caret = t.accent_500;
            f.placeholder_color = t.content_3;
            f.selection_color = t.selection_1;
        });
        let fh = input.read(cx).focus.clone();
        let cmds = self.palette_commands(cx);
        let highlight = highlight.min(cmds.len().saturating_sub(1));
        let mut list = div().id("paletteList").flex().flex_col().p(px(4.)).overflow_y_scroll().max_h(px(760. * 0.6 - 48.));
        if cmds.is_empty() {
            list = list.child(part("paletteEmpty", "status").text("No matching commands").wrap(div().p(px(14.)).text_size(px(13.)).text_color(t.content_3).child("No matching commands")));
        }
        let mut group: Option<&'static str> = None;
        let mut gi = 0;
        for (i, c) in cmds.iter().enumerate() {
            if group != Some(c.group) {
                group = Some(c.group);
                gi += 1;
                let g = c.group.to_uppercase();
                list = list.child(part(format!("commandGroup{gi}"), "heading").text(g.clone()).wrap(div().px(px(8.)).pt(px(6.)).pb(px(3.)).text_size(px(10.5)).font_weight(FontWeight::SEMIBOLD).text_color(t.content_3).child(g)));
            }
            let k = i + 1;
            let fh_c = self.fh(&format!("command{k}"), cx);
            let id = c.id.clone();
            let text = if c.accel.is_empty() { c.label.clone() } else { format!("{} {}", c.label, c.accel) };
            list = list.child(
                part(format!("command{k}"), "button").text(text).name(c.label.clone()).selected(i == highlight).focus(&fh_c, window).wrap(
                    pressable(("command", k), &fh_c, self.on(cx, move |ws, w, cx| ws.run_command(&id, w, cx)))
                        .h(px(31.))
                        .px(px(8.))
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .rounded(px(R_CONTROL))
                        .when(i == highlight, |d| d.bg(t.selection_1))
                        .when(i != highlight, |d| d.hover(|s| s.bg(t.wash_1)))
                        .child(icon(c.icon, 15., t.content_3))
                        .child(div().flex_1().text_size(px(13.)).font_weight(FontWeight::MEDIUM).child(c.label.clone()))
                        .child(div().text_size(px(11.5)).text_color(t.content_3).child(c.accel.clone())),
                ),
            );
        }
        let n = cmds.len();
        let card = div()
            .id("palette")
            .key_context("Palette")
            .occlude()
            .w(px(scale_in(490., p)))
            .mt(px(46.))
            .rounded(px(R_PANEL))
            .bg(t.surface_2)
            .border_1()
            .border_color(t.edge_2)
            .shadow(menu_shadow(&t, p))
            .opacity(p)
            .on_action(cx.listener(move |ws, _: &PaletteUp, _, cx| {
                if let Some(pl) = &mut ws.palette {
                    pl.highlight = pl.highlight.saturating_sub(1);
                }
                cx.notify()
            }))
            .on_action(cx.listener(move |ws, _: &PaletteDown, _, cx| {
                if let Some(pl) = &mut ws.palette {
                    pl.highlight = (pl.highlight + 1).min(n.saturating_sub(1));
                }
                cx.notify()
            }))
            .on_action(cx.listener(|ws, _: &Dismiss, w, cx| ws.close_palette(w, cx)))
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .child(
                div()
                    .h(px(38.))
                    .px(px(12.))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .border_b_1()
                    .border_color(t.edge_1)
                    .child(icon("bolt", 16., t.content_3))
                    .child(part("paletteInput", "textbox").text(input.read(cx).text()).name("Run a command or jump to…").focus(&fh, window).wrap(div().flex_1().text_size(px(14.)).child(input.clone()))),
            )
            .child(list);
        deferred(
            div()
                .id("paletteScrim")
                .absolute()
                .inset_0()
                .size_full()
                .bg(gpui::hsla(0., 0., 0., 0.30 * sp))
                .occlude()
                .flex()
                .flex_col()
                .items_center()
                .on_mouse_down(MouseButton::Left, cx.listener(|ws, _, w, cx| ws.close_palette(w, cx)))
                .child(part("palette", "generic").opacity(p).wrap(card)),
        )
        .with_priority(3)
        .into_any_element()
    }

    pub(crate) fn render_toast(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let t = self.theme();
        let Some(toast) = &self.toast else { return div().into_any_element() };
        let (message, serial) = (toast.message.clone(), toast.serial);
        let p = self.anims.progress("toast", serial.to_string(), RISE, window);
        let (fh_a, fh_d) = (self.fh("toastAction", cx), self.fh("toastDismiss", cx));
        let card = div()
            .relative()
            .top(px(3. * (1. - p)))
            .opacity(p)
            .h(px(38.))
            .pl(px(14.))
            .pr(px(6.))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(10.))
            .rounded(px(R_CONTROL))
            .bg(t.surface_3)
            .border_1()
            .border_color(t.edge_2)
            .shadow(vec![BoxShadow { color: t.shadow_alpha(0.20 * p.powi(4)), offset: point(px(0.), px(2.)), blur_radius: px(8.), spread_radius: px(0.) }])
            .child(part("toastMessage", "status").text(message.clone()).wrap(div().text_size(px(13.)).font_weight(FontWeight::MEDIUM).child(message)))
            .child(
                part("toastAction", "button").text("Undo").focus(&fh_a, window).wrap(
                    pressable("toastAction", &fh_a, self.on(cx, |ws, _, cx| ws.toast_undo(cx)))
                        .h(px(26.))
                        .px(px(9.))
                        .flex()
                        .items_center()
                        .rounded(px(R_CONTROL))
                        .text_size(px(13.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .gap(px(4.))
                        .text_color(t.accent_500)
                        .hover(|s| s.bg(t.wash_2))
                        .when(fh_a.is_focused(window), |d| d.border_2().border_color(t.focus_1))
                        .child(icon("undo", 14., t.accent_500))
                        .child("Undo"),
                ),
            )
            .child(
                part("toastDismiss", "button").name("Dismiss").focus(&fh_d, window).wrap(
                    pressable("toastDismiss", &fh_d, self.on(cx, |ws, _, cx| {
                        ws.toast = None;
                        report(cx, "toastDismissed", json!({}));
                    }))
                    .size(px(26.))
                    .rounded(px(R_CONTROL))
                    .flex()
                    .items_center()
                    .justify_center()
                    .hover(|s| s.bg(t.wash_2))
                    .when(fh_d.is_focused(window), |d| d.border_2().border_color(t.focus_1))
                    .child(icon("close", 15., t.content_3)),
                ),
            );
        deferred(div().absolute().left_0().right_0().bottom(px(17.5)).flex().justify_center().child(part("toast", "generic").opacity(p).wrap(card))).with_priority(1).into_any_element()
    }
}
