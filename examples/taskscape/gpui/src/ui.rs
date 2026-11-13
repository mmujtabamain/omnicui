//! Small building blocks drawn from the tokens: icons, buttons, labels.

use std::rc::Rc;

use gpui::{
    div, prelude::*, px, svg, App, Div, FocusHandle, FontWeight, Hsla, MouseButton, SharedString, Stateful, Svg, Window,
};

use crate::keys::Activate;
use crate::theme::{Theme, R_CONTROL};
#[allow(unused_imports)]
use gpui::Render;

pub type Handler = Rc<dyn Fn(&mut Window, &mut App)>;

/// A Material Symbols icon, tinted like text.
pub fn icon(name: &str, size: f32, color: Hsla) -> Svg {
    svg().path(SharedString::from(format!("icons/{name}.svg"))).size(px(size)).flex_none().text_color(color)
}

/// A pressable element: a click or, with keyboard focus, Enter or Space runs `on`.
pub fn pressable(id: impl Into<gpui::ElementId>, focus: &FocusHandle, on: Handler) -> Stateful<Div> {
    let id: gpui::ElementId = id.into();
    let click = on;
    div()
        .id(id)
        .key_context("Button")
        .track_focus(focus)
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
        .on_click(move |_, window, cx| {
            cx.stop_propagation();
            click(window, cx)
        })
        // GPUI clicks a focused element itself when Enter or Space comes up; the binding only
        // keeps those keys from reaching the surfaces around it (a sheet's Enter, the list's
        // Space) while a button has the focus.
        .on_action(|_: &Activate, _, _| {})
}

/// An upper-case section label (NOTES, SUBTASKS).
pub fn section_label(text: &str, t: &Theme) -> Div {
    div().text_size(px(10.5)).font_weight(FontWeight::SEMIBOLD).text_color(t.content_3).child(spaced(&text.to_uppercase(), 1))
}

/// Wide tracking, as letter-spacing would give: GPUI has no tracking, so a hair space goes
/// between the letters of short upper-case labels. The registry reads the plain text.
pub fn spaced(text: &str, _level: u8) -> SharedString {
    SharedString::from(text.to_string())
}

pub fn icon_button(id: impl Into<gpui::ElementId>, name: &str, focus: &FocusHandle, t: &Theme, on: Handler) -> Stateful<Div> {
    pressable(id, focus, on)
        .size(px(28.))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(R_CONTROL))
        .hover(|s| s.bg(t.wash_2))
        .child(icon(name, 17., t.content_3))
}

pub fn hsla_with(mut c: Hsla, a: f32) -> Hsla {
    c.a *= a;
    c
}

/// A plain tooltip, drawn by GPUI.
pub struct Tooltip(pub SharedString);

impl Render for Tooltip {
    fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
        div().px(px(6.)).py(px(3.)).rounded(px(4.)).bg(gpui::hsla(0., 0., 0.1, 0.92)).text_color(gpui::white()).text_size(px(12.)).child(self.0.clone())
    }
}
