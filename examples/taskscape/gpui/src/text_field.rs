//! A text field drawn with GPUI's own text system and fed by the platform's text input
//! (`EntityInputHandler`): the same construction as GPUI's input example, with wrapping for
//! multi-line fields (notes). Enter, Escape and Tab are reported to the owner.

use std::ops::Range;

use gpui::{
    actions, div, fill, point, prelude::*, px, relative, size, App, Bounds, ClipboardItem, Context, CursorStyle,
    Element, ElementId, ElementInputHandler, Entity, EntityInputHandler, EventEmitter, FocusHandle, Focusable,
    GlobalElementId, Hsla, InspectorElementId, KeyBinding, LayoutId, MouseButton, MouseDownEvent, MouseMoveEvent,
    MouseUpEvent, Pixels, Point, SharedString, Style, TextAlign, TextRun, UTF16Selection, Window, WrappedLine,
};
use smallvec::SmallVec;

actions!(
    text_field,
    [Backspace, Delete, Left, Right, Up, Down, SelectLeft, SelectRight, SelectAll, Home, End, Paste, Copy, Cut, Submit, Newline, Cancel, TabOut, BackTab, InsertSpace]
);

pub fn bindings() -> Vec<KeyBinding> {
    let c = Some("TextField");
    vec![
        KeyBinding::new("backspace", Backspace, c),
        KeyBinding::new("delete", Delete, c),
        KeyBinding::new("left", Left, c),
        KeyBinding::new("right", Right, c),
        KeyBinding::new("shift-left", SelectLeft, c),
        KeyBinding::new("shift-right", SelectRight, c),
        KeyBinding::new("cmd-a", SelectAll, c),
        KeyBinding::new("ctrl-a", SelectAll, c),
        KeyBinding::new("cmd-v", Paste, c),
        KeyBinding::new("cmd-c", Copy, c),
        KeyBinding::new("cmd-x", Cut, c),
        KeyBinding::new("home", Home, c),
        KeyBinding::new("end", End, c),
        KeyBinding::new("cmd-left", Home, c),
        KeyBinding::new("cmd-right", End, c),
        KeyBinding::new("enter", Submit, c),
        KeyBinding::new("shift-enter", Newline, c),
        KeyBinding::new("escape", Cancel, c),
        KeyBinding::new("tab", TabOut, c),
        KeyBinding::new("shift-tab", BackTab, c),
        // A field inside the task list keeps Space for itself (the list selects with it).
        KeyBinding::new("space", InsertSpace, c),
    ]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldEvent {
    Changed,
    Submit,
    Cancel,
    Tab,
    BackTab,
    Blur,
}

pub struct TextField {
    pub focus: FocusHandle,
    content: String,
    placeholder: SharedString,
    selected: Range<usize>,
    reversed: bool,
    marked: Option<Range<usize>>,
    pub multiline: bool,
    pub caret: Hsla,
    pub placeholder_color: Hsla,
    pub selection_color: Hsla,
    last_lines: SmallVec<[WrappedLine; 1]>,
    last_bounds: Option<Bounds<Pixels>>,
    last_line_height: Pixels,
    selecting: bool,
}

impl EventEmitter<FieldEvent> for TextField {}

impl Focusable for TextField {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl TextField {
    pub fn new(placeholder: impl Into<SharedString>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus = cx.focus_handle();
        cx.on_blur(&focus, window, |_, _, cx| cx.emit(FieldEvent::Blur)).detach();
        TextField {
            focus,
            content: String::new(),
            placeholder: placeholder.into(),
            selected: 0..0,
            reversed: false,
            marked: None,
            multiline: false,
            caret: gpui::blue(),
            placeholder_color: gpui::hsla(0., 0., 0.5, 1.),
            selection_color: gpui::hsla(0.58, 1., 0.5, 0.25),
            last_lines: SmallVec::new(),
            last_bounds: None,
            last_line_height: px(16.),
            selecting: false,
        }
    }

    pub fn text(&self) -> &str {
        &self.content
    }

    pub fn placeholder(&self) -> &SharedString {
        &self.placeholder
    }

    pub fn set_placeholder(&mut self, p: impl Into<SharedString>) {
        self.placeholder = p.into();
    }

    /// Replaces the text, with the caret at the end; no Changed event (the owner set it).
    pub fn set_text(&mut self, text: impl Into<String>, cx: &mut Context<Self>) {
        self.content = text.into();
        self.selected = self.content.len()..self.content.len();
        self.reversed = false;
        self.marked = None;
        cx.notify();
    }

    pub fn select_all_text(&mut self, cx: &mut Context<Self>) {
        self.selected = 0..self.content.len();
        self.reversed = false;
        cx.notify();
    }

    fn changed(&mut self, cx: &mut Context<Self>) {
        cx.emit(FieldEvent::Changed);
        cx.notify();
    }

    fn cursor(&self) -> usize {
        if self.reversed {
            self.selected.start
        } else {
            self.selected.end
        }
    }

    fn move_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        self.selected = offset..offset;
        self.reversed = false;
        cx.notify();
    }

    fn select_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        if self.reversed {
            self.selected.start = offset
        } else {
            self.selected.end = offset
        }
        if self.selected.end < self.selected.start {
            self.reversed = !self.reversed;
            self.selected = self.selected.end..self.selected.start;
        }
        cx.notify();
    }

    fn prev(&self, offset: usize) -> usize {
        self.content[..offset].char_indices().next_back().map(|(i, _)| i).unwrap_or(0)
    }

    fn next(&self, offset: usize) -> usize {
        self.content[offset..].chars().next().map(|c| offset + c.len_utf8()).unwrap_or(self.content.len())
    }

    fn left(&mut self, _: &Left, _: &mut Window, cx: &mut Context<Self>) {
        if self.selected.is_empty() {
            self.move_to(self.prev(self.cursor()), cx)
        } else {
            self.move_to(self.selected.start, cx)
        }
    }
    fn right(&mut self, _: &Right, _: &mut Window, cx: &mut Context<Self>) {
        if self.selected.is_empty() {
            self.move_to(self.next(self.selected.end), cx)
        } else {
            self.move_to(self.selected.end, cx)
        }
    }
    fn select_left(&mut self, _: &SelectLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.prev(self.cursor()), cx)
    }
    fn select_right(&mut self, _: &SelectRight, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.next(self.cursor()), cx)
    }
    fn select_all(&mut self, _: &SelectAll, _: &mut Window, cx: &mut Context<Self>) {
        self.select_all_text(cx)
    }
    fn home(&mut self, _: &Home, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(0, cx)
    }
    fn end(&mut self, _: &End, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(self.content.len(), cx)
    }
    fn backspace(&mut self, _: &Backspace, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected.is_empty() {
            self.select_to(self.prev(self.cursor()), cx)
        }
        self.replace_text_in_range(None, "", window, cx)
    }
    fn delete(&mut self, _: &Delete, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected.is_empty() {
            self.select_to(self.next(self.cursor()), cx)
        }
        self.replace_text_in_range(None, "", window, cx)
    }
    fn paste(&mut self, _: &Paste, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(text) = cx.read_from_clipboard().and_then(|i| i.text()) {
            let text = if self.multiline { text } else { text.replace('\n', " ") };
            self.replace_text_in_range(None, &text, window, cx);
        }
    }
    fn copy(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>) {
        if !self.selected.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(self.content[self.selected.clone()].to_string()));
        }
    }
    fn cut(&mut self, _: &Cut, window: &mut Window, cx: &mut Context<Self>) {
        if !self.selected.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(self.content[self.selected.clone()].to_string()));
            self.replace_text_in_range(None, "", window, cx)
        }
    }
    fn submit(&mut self, _: &Submit, window: &mut Window, cx: &mut Context<Self>) {
        if self.multiline {
            self.replace_text_in_range(None, "\n", window, cx)
        } else {
            cx.emit(FieldEvent::Submit)
        }
    }
    fn newline(&mut self, _: &Newline, window: &mut Window, cx: &mut Context<Self>) {
        if self.multiline {
            self.replace_text_in_range(None, "\n", window, cx)
        } else {
            cx.emit(FieldEvent::Submit)
        }
    }
    fn insert_space(&mut self, _: &InsertSpace, window: &mut Window, cx: &mut Context<Self>) {
        self.replace_text_in_range(None, " ", window, cx)
    }
    fn cancel(&mut self, _: &Cancel, _: &mut Window, cx: &mut Context<Self>) {
        cx.emit(FieldEvent::Cancel)
    }
    fn tab(&mut self, _: &TabOut, _: &mut Window, cx: &mut Context<Self>) {
        cx.emit(FieldEvent::Tab)
    }
    fn back_tab(&mut self, _: &BackTab, _: &mut Window, cx: &mut Context<Self>) {
        cx.emit(FieldEvent::BackTab)
    }

    fn index_for(&self, position: Point<Pixels>) -> usize {
        let Some(bounds) = self.last_bounds else { return 0 };
        let lh = self.last_line_height;
        let mut y = bounds.top();
        let mut start = 0;
        for line in &self.last_lines {
            let h = line.size(lh).height;
            if position.y < y + h || std::ptr::eq(line, self.last_lines.last().unwrap()) {
                let local = point(position.x - bounds.left(), (position.y - y).max(px(0.)));
                let ix = match line.closest_index_for_position(local, lh) {
                    Ok(i) | Err(i) => i,
                };
                return (start + ix).min(self.content.len());
            }
            y += h;
            start += line.len() + 1;
        }
        self.content.len()
    }

    fn on_mouse_down(&mut self, e: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.selecting = true;
        window.focus(&self.focus);
        let ix = self.index_for(e.position);
        if e.modifiers.shift {
            self.select_to(ix, cx)
        } else {
            self.move_to(ix, cx)
        }
    }
    fn on_mouse_up(&mut self, _: &MouseUpEvent, _: &mut Window, _: &mut Context<Self>) {
        self.selecting = false;
    }
    fn on_mouse_move(&mut self, e: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        if self.selecting {
            self.select_to(self.index_for(e.position), cx)
        }
    }

    fn utf16_to_utf8(&self, offset: usize) -> usize {
        let (mut u8o, mut u16c) = (0, 0);
        for ch in self.content.chars() {
            if u16c >= offset {
                break;
            }
            u16c += ch.len_utf16();
            u8o += ch.len_utf8();
        }
        u8o
    }
    fn utf8_to_utf16(&self, offset: usize) -> usize {
        let (mut u16o, mut u8c) = (0, 0);
        for ch in self.content.chars() {
            if u8c >= offset {
                break;
            }
            u8c += ch.len_utf8();
            u16o += ch.len_utf16();
        }
        u16o
    }
    fn range_to_utf16(&self, r: &Range<usize>) -> Range<usize> {
        self.utf8_to_utf16(r.start)..self.utf8_to_utf16(r.end)
    }
    fn range_from_utf16(&self, r: &Range<usize>) -> Range<usize> {
        self.utf16_to_utf8(r.start)..self.utf16_to_utf8(r.end)
    }
}

impl EntityInputHandler for TextField {
    fn text_for_range(&mut self, r: Range<usize>, actual: &mut Option<Range<usize>>, _: &mut Window, _: &mut Context<Self>) -> Option<String> {
        let range = self.range_from_utf16(&r);
        actual.replace(self.range_to_utf16(&range));
        Some(self.content[range].to_string())
    }
    fn selected_text_range(&mut self, _: bool, _: &mut Window, _: &mut Context<Self>) -> Option<UTF16Selection> {
        Some(UTF16Selection { range: self.range_to_utf16(&self.selected), reversed: self.reversed })
    }
    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        self.marked.as_ref().map(|r| self.range_to_utf16(r))
    }
    fn unmark_text(&mut self, _: &mut Window, _: &mut Context<Self>) {
        self.marked = None;
    }
    fn replace_text_in_range(&mut self, r: Option<Range<usize>>, new_text: &str, _: &mut Window, cx: &mut Context<Self>) {
        let range = r.as_ref().map(|r| self.range_from_utf16(r)).or(self.marked.clone()).unwrap_or(self.selected.clone());
        let new_text = if self.multiline { new_text.to_string() } else { new_text.replace('\n', " ") };
        self.content = format!("{}{}{}", &self.content[..range.start], new_text, &self.content[range.end..]);
        self.selected = range.start + new_text.len()..range.start + new_text.len();
        self.reversed = false;
        self.marked = None;
        self.changed(cx);
    }
    fn replace_and_mark_text_in_range(&mut self, r: Option<Range<usize>>, new_text: &str, new_sel: Option<Range<usize>>, _: &mut Window, cx: &mut Context<Self>) {
        let range = r.as_ref().map(|r| self.range_from_utf16(r)).or(self.marked.clone()).unwrap_or(self.selected.clone());
        self.content = format!("{}{}{}", &self.content[..range.start], new_text, &self.content[range.end..]);
        self.marked = (!new_text.is_empty()).then(|| range.start..range.start + new_text.len());
        self.selected = new_sel
            .as_ref()
            .map(|r| self.range_from_utf16(r))
            .map(|n| n.start + range.start..n.end + range.start)
            .unwrap_or_else(|| range.start + new_text.len()..range.start + new_text.len());
        self.changed(cx);
    }
    fn bounds_for_range(&mut self, r: Range<usize>, bounds: Bounds<Pixels>, _: &mut Window, _: &mut Context<Self>) -> Option<Bounds<Pixels>> {
        let range = self.range_from_utf16(&r);
        let line = self.last_lines.first()?;
        let p = line.position_for_index(range.start.min(line.len()), self.last_line_height)?;
        Some(Bounds::new(bounds.origin + p, size(px(1.), self.last_line_height)))
    }
    fn character_index_for_point(&mut self, p: Point<Pixels>, _: &mut Window, _: &mut Context<Self>) -> Option<usize> {
        Some(self.utf8_to_utf16(self.index_for(p)))
    }
}

impl Render for TextField {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .key_context("TextField")
            .track_focus(&self.focus)
            .cursor(CursorStyle::IBeam)
            .w_full()
            .on_action(cx.listener(Self::backspace))
            .on_action(cx.listener(Self::delete))
            .on_action(cx.listener(Self::left))
            .on_action(cx.listener(Self::right))
            .on_action(cx.listener(Self::select_left))
            .on_action(cx.listener(Self::select_right))
            .on_action(cx.listener(Self::select_all))
            .on_action(cx.listener(Self::home))
            .on_action(cx.listener(Self::end))
            .on_action(cx.listener(Self::paste))
            .on_action(cx.listener(Self::copy))
            .on_action(cx.listener(Self::cut))
            .on_action(cx.listener(Self::submit))
            .on_action(cx.listener(Self::newline))
            .on_action(cx.listener(Self::cancel))
            .on_action(cx.listener(Self::insert_space))
            .on_action(cx.listener(Self::tab))
            .on_action(cx.listener(Self::back_tab))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .child(TextElement { field: cx.entity() })
    }
}

struct TextElement {
    field: Entity<TextField>,
}

struct Prepainted {
    lines: SmallVec<[WrappedLine; 1]>,
    line_height: Pixels,
    caret: Option<Bounds<Pixels>>,
    selection: Vec<Bounds<Pixels>>,
}

impl IntoElement for TextElement {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

fn runs_for(text_len: usize, window: &Window, color: Hsla) -> Vec<TextRun> {
    let style = window.text_style();
    vec![TextRun { len: text_len, font: style.font(), color, background_color: None, underline: None, strikethrough: None }]
}

impl Element for TextElement {
    type RequestLayoutState = ();
    type PrepaintState = Prepainted;

    fn id(&self) -> Option<ElementId> {
        None
    }
    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(&mut self, _: Option<&GlobalElementId>, _: Option<&InspectorElementId>, window: &mut Window, cx: &mut App) -> (LayoutId, ()) {
        let f = self.field.read(cx);
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        let line_height = window.line_height();
        if !f.multiline {
            style.size.height = line_height.into();
            return (window.request_layout(style, [], cx), ());
        }
        let text: SharedString = if f.content.is_empty() { f.placeholder.clone() } else { f.content.clone().into() };
        let font_size = window.text_style().font_size.to_pixels(window.rem_size());
        let runs = runs_for(text.len(), window, gpui::black());
        let id = window.request_measured_layout(style, move |known, available, window, _| {
            let width = known.width.or(match available.width {
                gpui::AvailableSpace::Definite(w) => Some(w),
                _ => None,
            });
            let lines = window.text_system().shape_text(text.clone(), font_size, &runs, width, None).unwrap_or_default();
            let h: Pixels = lines.iter().map(|l| l.size(line_height).height).fold(px(0.), |a, b| a + b);
            size(width.unwrap_or(px(100.)), h.max(line_height))
        });
        (id, ())
    }

    fn prepaint(&mut self, _: Option<&GlobalElementId>, _: Option<&InspectorElementId>, bounds: Bounds<Pixels>, _: &mut (), window: &mut Window, cx: &mut App) -> Prepainted {
        let f = self.field.read(cx);
        let style = window.text_style();
        let line_height = window.line_height();
        let font_size = style.font_size.to_pixels(window.rem_size());
        let (text, color): (SharedString, Hsla) =
            if f.content.is_empty() { (f.placeholder.clone(), f.placeholder_color) } else { (f.content.clone().into(), style.color) };
        let runs = runs_for(text.len(), window, color);
        let wrap = f.multiline.then_some(bounds.size.width);
        let lines = window.text_system().shape_text(text, font_size, &runs, wrap, None).unwrap_or_default();

        // Where an index is: (line, its top, its offset in that line).
        let locate = |ix: usize| -> Point<Pixels> {
            let (mut start, mut y) = (0usize, px(0.));
            for (i, l) in lines.iter().enumerate() {
                let end = start + l.len();
                if ix <= end || i + 1 == lines.len() {
                    let p = l.position_for_index((ix.saturating_sub(start)).min(l.len()), line_height).unwrap_or(point(px(0.), px(0.)));
                    return point(p.x, y + p.y);
                }
                y += l.size(line_height).height;
                start = end + 1;
            }
            point(px(0.), px(0.))
        };
        let (caret, selection) = if f.content.is_empty() {
            (Some(Bounds::new(bounds.origin, size(px(1.5), line_height))), vec![])
        } else if f.selected.is_empty() {
            let p = locate(f.cursor());
            (Some(Bounds::new(bounds.origin + p, size(px(1.5), line_height))), vec![])
        } else {
            let (a, b) = (locate(f.selected.start), locate(f.selected.end));
            let sel = if a.y == b.y {
                vec![Bounds::from_corners(bounds.origin + a, bounds.origin + point(b.x, b.y + line_height))]
            } else {
                vec![
                    Bounds::from_corners(bounds.origin + a, point(bounds.right(), bounds.top() + a.y + line_height)),
                    Bounds::from_corners(point(bounds.left(), bounds.top() + a.y + line_height), point(bounds.right(), bounds.top() + b.y)),
                    Bounds::from_corners(point(bounds.left(), bounds.top() + b.y), bounds.origin + point(b.x, b.y + line_height)),
                ]
            };
            (None, sel)
        };
        Prepainted { lines, line_height, caret, selection }
    }

    fn paint(&mut self, _: Option<&GlobalElementId>, _: Option<&InspectorElementId>, bounds: Bounds<Pixels>, _: &mut (), pre: &mut Prepainted, window: &mut Window, cx: &mut App) {
        let (focus, caret_color, sel_color) = {
            let f = self.field.read(cx);
            (f.focus.clone(), f.caret, f.selection_color)
        };
        window.handle_input(&focus, ElementInputHandler::new(bounds, self.field.clone()), cx);
        let focused = focus.is_focused(window);
        if focused {
            for s in &pre.selection {
                window.paint_quad(fill(*s, sel_color));
            }
        }
        let mut y = bounds.top();
        for line in &pre.lines {
            let _ = line.paint(point(bounds.left(), y), pre.line_height, TextAlign::Left, Some(bounds), window, cx);
            y += line.size(pre.line_height).height;
        }
        if focused {
            if let Some(c) = pre.caret {
                window.paint_quad(fill(c, caret_color));
            }
        }
        let lines = std::mem::take(&mut pre.lines);
        let lh = pre.line_height;
        self.field.update(cx, |f, _| {
            f.last_lines = lines;
            f.last_bounds = Some(bounds);
            f.last_line_height = lh;
        });
    }
}
