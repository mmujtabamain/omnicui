//! contract: omnicui/contracts/settingrow.contract.yaml (id: ui.settingrow). One settings line: a title and an optional description, a control (a toggle, a segmented radio group, a path or button, or a hotkey keycap), an optional footnote or rebind error, and an optional read-only value.
use crate::omnicui::settingrow::{parts, SettingrowEvent, SettingrowProps};
use crate::{icon, part, rem, weight, Look};
use egui::{
    pos2, vec2, Align, Align2, Color32, CornerRadius, FontFamily, FontId, Id, Key, Layout, Painter, Pos2, Rect, Response, Role,
    Sense, Stroke, StrokeKind, Ui, Vec2, WidgetInfo,
};

/// The chord the keycap shows. The web harness supplies this demo chord, since the contract cannot pass one.
const DEMO_KEYS: &str = "⌘N";
/// The keycap's label while it records a new chord.
const RECORDING_TEXT: &str = "Press keys";
/// The key after the command glyph on the keycap.
const HOTKEY_KEY: &str = "N";
/// The command-key glyph's size, in px.
const KEYCAP_ICON: f32 = 14.0;

pub struct Settingrow {
    p: SettingrowProps,
    /// The toggle's state; a press flips it.
    checked: bool,
    /// The selected segment's label, or for pathButton the path shown.
    choice: String,
    /// The hotkey keycap is recording a new chord.
    recording: bool,
}

impl Settingrow {
    pub fn new(p: SettingrowProps) -> Self {
        Self { checked: p.checked, choice: p.choice.clone(), recording: p.recording, p }
    }

    /// One frame: draws the row and returns the events it raised in this frame.
    pub fn ui(&mut self, ui: &mut Ui, look: Look) -> Vec<SettingrowEvent> {
        let mut events = vec![];
        let p = self.p.clone();
        let base = ui.scope_id().with("settingrow");
        let t = look.t;
        let mut frame = egui::Frame::new();
        if p.searchOnly {
            frame = frame
                .fill(t.surface_2)
                .stroke(Stroke::new(1.0, t.edge_1))
                .corner_radius(CornerRadius::same(11))
                .inner_margin(egui::Margin::symmetric(10, 6));
        }
        let out = frame.show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            self.body(ui, look, base, &p, &mut events);
            if !p.footnote.is_empty() {
                ui.add_space(rem(0.214));
                footnote(ui, look, base, &p);
            }
        });
        let row = out.response;
        if p.control != "none" {
            let title = p.title.clone();
            row.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, title.as_str()));
        } else {
            row.widget_info(|| WidgetInfo::new(Role::GenericContainer));
        }
        part(ui, &row, parts::ROW);
        events
    }

    /// The row's body: the text, then the control and the value (inline: on the right; stacked: below).
    fn body(&mut self, ui: &mut Ui, look: Look, base: Id, p: &SettingrowProps, events: &mut Vec<SettingrowEvent>) {
        let has_control = p.control != "none";
        let has_value = !p.value.is_empty();
        if p.layout == "stacked" {
            text_line(ui, look, p);
            if has_control {
                ui.add_space(rem(0.5));
                self.control(ui, look, base, p, true, events);
            }
            if has_value {
                ui.add_space(rem(0.5));
                value_node(ui, look, p);
            }
            return;
        }
        let min_h = if p.control == "hotkey" { rem(2.571) } else { rem(2.0) };
        let right_w = right_width(ui, p);
        ui.horizontal(|ui| {
            ui.set_min_height(min_h);
            let gap = rem(0.857);
            let left_w = if right_w > 0.0 { (ui.available_width() - right_w - gap).max(0.0) } else { ui.available_width() };
            let used = ui
                .scope(|ui| {
                    ui.set_max_width(left_w);
                    text_line(ui, look, p);
                })
                .response
                .rect
                .width();
            if right_w > 0.0 {
                // Push the control block to the right edge, the way the web's justify-between does.
                ui.add_space((left_w - used).max(0.0) + gap);
                ui.allocate_ui_with_layout(vec2(right_w, min_h), Layout::left_to_right(Align::Center), |ui| {
                    if has_control {
                        self.control(ui, look, base, p, false, events);
                    }
                    if has_control && has_value {
                        ui.add_space(rem(0.5));
                    }
                    if has_value {
                        value_node(ui, look, p);
                    }
                });
            }
        });
    }

    fn control(&mut self, ui: &mut Ui, look: Look, base: Id, p: &SettingrowProps, full: bool, events: &mut Vec<SettingrowEvent>) {
        match p.control.as_str() {
            "toggle" => self.toggle(ui, look, base, p, events),
            "segmented" => self.segmented(ui, look, base, p, full, events),
            "pathButton" => path_button(ui, look, base, p, full),
            "button" => button_control(ui, look, base, p, full),
            "hotkey" => self.hotkey(ui, look, base, p),
            _ => {}
        }
    }

    /// The switch, drawn as the desktop switch is, but tagged with this row's toggle part.
    fn toggle(&mut self, ui: &mut Ui, look: Look, base: Id, p: &SettingrowProps, events: &mut Vec<SettingrowEvent>) {
        let t = look.t;
        let (rect, _) = ui.allocate_exact_size(vec2(rem(2.5), rem(1.357)), Sense::hover());
        let mut resp = ui.interact(rect, base.with("toggle"), Sense::click());
        // Space and Enter both flip a focused switch; egui may also report the key as a click, so they are OR-ed.
        let (space, enter) = ui.input(|i| (i.key_pressed(Key::Space), i.key_pressed(Key::Enter)));
        let key = resp.has_focus() && (space || enter);
        if resp.clicked() || key {
            self.checked = !self.checked;
            resp.mark_changed();
            events.push(SettingrowEvent::Toggled { checked: self.checked });
        }
        let checked = self.checked;
        resp.widget_info(|| WidgetInfo::selected(Role::Switch, true, checked, p.title.as_str()));
        part(ui, &resp, parts::TOGGLE);

        let on = ui.ctx().animate_bool_with_time(base.with("toggle_on"), checked, 0.18);
        let hover = resp.hovered();
        let track = match (hover, checked) {
            (true, true) => t.accent_600,
            (true, false) => t.content_3,
            _ => t.edge_3.lerp_to_gamma(t.accent_500, on),
        };
        let radius = CornerRadius::same((rect.height() / 2.0) as u8);
        let painter = ui.painter();
        painter.rect_filled(rect, radius, track);
        if resp.has_focus() {
            painter.rect_stroke(rect.expand(1.0), radius, Stroke::new(2.0, t.focus_1), StrokeKind::Middle);
        }
        let d = rem(1.0);
        let x = rect.left() + rem(0.1) + 1.0 + rem(1.157) * on;
        let thumb = Rect::from_min_size(pos2(x, rect.center().y - d / 2.0), vec2(d, d));
        painter.circle_filled(thumb.center(), d / 2.0, t.on_accent);
    }

    /// A radio group of up to three segments: one tab stop, arrow keys move focus and check the new segment.
    fn segmented(&mut self, ui: &mut Ui, look: Look, base: Id, p: &SettingrowProps, full: bool, events: &mut Vec<SettingrowEvent>) {
        let t = look.t;
        let segs = segments(p);
        let n = segs.len();
        let ids = [parts::SEGMENTONE, parts::SEGMENTTWO, parts::SEGMENTTHREE];
        let inner = rem(0.143);
        let gap = rem(0.143);
        let h = rem(2.0);
        let total_gap = gap * n.saturating_sub(1) as f32;
        let mut widths: Vec<f32> = Vec::with_capacity(n);
        if full {
            let each = ((ui.available_width() - 2.0 * inner - total_gap) / n.max(1) as f32).max(0.0);
            widths = vec![each; n];
        } else {
            for s in &segs {
                widths.push(segment_width(ui, s));
            }
        }
        let tray_w = if full { ui.available_width() } else { widths.iter().sum::<f32>() + total_gap + 2.0 * inner };
        let (tray, tray_resp) = ui.allocate_exact_size(vec2(tray_w, h), Sense::hover());
        tray_resp.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, p.title.as_str()));
        part(ui, &tray_resp, parts::CONTROL);
        ui.painter().rect_filled(tray, CornerRadius::same(4), t.wash_2);

        let mut moved = false;
        let mut x = tray.left() + inner;
        for (i, label) in segs.iter().enumerate() {
            let w = widths[i];
            let r = Rect::from_min_size(pos2(x, tray.top() + inner), vec2(w, h - 2.0 * inner));
            x += w + gap;
            let resp = ui.interact(r, base.with(("segment", i)), Sense::click());
            if resp.has_focus() {
                keep_focus(ui, resp.id, true, false);
            }
            // Arrow keys on the focused segment move to the next one and check it. Only one move per frame.
            if resp.has_focus() && !moved {
                let step: i32 = ui.input(|k| {
                    if k.key_pressed(Key::ArrowRight) || k.key_pressed(Key::ArrowDown) {
                        1
                    } else if k.key_pressed(Key::ArrowLeft) || k.key_pressed(Key::ArrowUp) {
                        -1
                    } else {
                        0
                    }
                });
                if step != 0 {
                    let next = (i as i32 + step).rem_euclid(n as i32) as usize;
                    let value = segs[next].clone();
                    self.choice = value.clone();
                    events.push(SettingrowEvent::Changed { value });
                    ui.ctx().memory_mut(|m| m.request_focus(base.with(("segment", next))));
                    ui.ctx().request_repaint();
                    moved = true;
                }
            }
            if resp.clicked() {
                self.choice = label.clone();
                events.push(SettingrowEvent::Changed { value: label.clone() });
                resp.request_focus();
            }
            let selected = *label == self.choice;
            resp.widget_info(|| WidgetInfo::selected(Role::Button, true, selected, label.as_str()));
            set_selected(ui, resp.id, selected);
            part(ui, &resp, ids[i]);
            // The raised chip eases in and out over the fast duration, as the web's transition does.
            let on = ui.ctx().animate_bool_with_time(base.with(("segment_on", i)), selected, 0.18);

            let painter = ui.painter();
            let radius = CornerRadius::same(4);
            if on > 0.0 {
                painter.rect(r, radius, t.surface_3.gamma_multiply(on), Stroke::new(1.0, t.edge_1.gamma_multiply(on)), StrokeKind::Inside);
            } else if resp.hovered() {
                painter.rect_filled(r, radius, t.wash_1);
            }
            let color = if selected { t.content_1 } else { t.content_2 };
            painter.text(r.center(), Align2::CENTER_CENTER, label, fnt(12.0, 500), color);
            if resp.has_focus() {
                painter.rect_stroke(r.expand(2.0), radius, Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
            }
        }
    }

    /// The keycap: names the row by its title, shows the chord (or "Press keys" while recording).
    fn hotkey(&mut self, ui: &mut Ui, look: Look, base: Id, p: &SettingrowProps) {
        let t = look.t;
        let w = hotkey_width(ui);
        let (rect, _) = ui.allocate_exact_size(vec2(w, rem(2.0)), Sense::hover());
        let resp = ui.interact(rect, base.with("button"), Sense::click());
        if resp.has_focus() {
            keep_focus(ui, resp.id, false, true);
        }
        if resp.clicked() {
            self.recording = true;
        }
        if resp.has_focus() && self.recording && ui.input(|i| i.key_pressed(Key::Escape)) {
            self.recording = false;
            resp.request_focus();
        }
        let recording = self.recording;
        let value = if recording { RECORDING_TEXT } else { DEMO_KEYS };
        resp.widget_info(|| WidgetInfo::labeled(Role::Button, true, p.title.as_str()));
        set_value(ui, resp.id, value);
        part(ui, &resp, parts::BUTTON);
        let rec = ui.ctx().animate_bool_with_time(base.with("recording_on"), recording, 0.18);
        let border = t.edge_2.lerp_to_gamma(t.accent_500, rec);
        let color = if recording { t.accent_600 } else { t.content_1 };
        paint_button(ui, look, rect, &resp, t.surface_3, Some(border));
        let font = fnt(12.0, 500);
        let cy = rect.center().y;
        if recording {
            ui.painter().text(rect.center(), Align2::CENTER_CENTER, RECORDING_TEXT, font, color);
        } else {
            // The chord: the command-key glyph (Material Symbols; Inter has no ⌘) then the key.
            let key_w = KEYCAP_ICON + rem(0.143) + text_w(ui, HOTKEY_KEY, &font);
            let x0 = rect.center().x - key_w / 2.0;
            draw_icon(ui.painter(), "keyboard_command_key", KEYCAP_ICON, pos2(x0, cy), Align2::LEFT_CENTER, color);
            ui.painter().text(pos2(x0 + KEYCAP_ICON + rem(0.143), cy), Align2::LEFT_CENTER, HOTKEY_KEY, font, color);
        }
    }
}

/// The title line: the optional icon, then the title and the optional description.
fn text_line(ui: &mut Ui, look: Look, p: &SettingrowProps) {
    let t = look.t;
    ui.horizontal(|ui| {
        if p.icon {
            let (rect, resp) = ui.allocate_exact_size(vec2(15.0, 15.0), Sense::hover());
            draw_icon(ui.painter(), "filter_list", 15.0, rect.center(), Align2::CENTER_CENTER, t.content_2);
            resp.widget_info(|| WidgetInfo::new(Role::GenericContainer));
            part(ui, &resp, parts::ICON);
            ui.add_space(rem(0.5));
        }
        ui.vertical(|ui| {
            let title_color = if p.tone == "secondary" { t.content_2 } else { t.content_1 };
            text_node(ui, &p.title, fnt(13.0, 500), title_color, rem(1.393), parts::TITLE);
            if !p.description.is_empty() {
                ui.add_space(rem(0.143));
                text_node(ui, &p.description, fnt(12.0, 400), t.content_2, rem(1.2), parts::DESCRIPTION);
            }
        });
    });
}

/// The footnote: a note, or the rebind error, which is also a polite status inside the footnote.
fn footnote(ui: &mut Ui, look: Look, base: Id, p: &SettingrowProps) {
    let t = look.t;
    let error = p.footnoteTone == "error";
    let color = if error { t.danger_500 } else { t.content_2 };
    let note = text_node(ui, &p.footnote, fnt(11.0, 400), color, rem(1.1), parts::FOOTNOTE);
    if error {
        let status = ui.interact(note.rect, base.with("rebindError"), Sense::hover());
        status.widget_info(|| WidgetInfo::labeled(Role::Status, true, p.footnote.as_str()));
        part(ui, &status, parts::REBINDERROR);
    }
}

fn value_node(ui: &mut Ui, look: Look, p: &SettingrowProps) {
    text_node(ui, &p.value, fnt(13.0, 400), look.t.content_2, rem(1.393), parts::VALUE);
}

/// The path button: a folder, then the path shown (its value), named by the row's title.
fn path_button(ui: &mut Ui, look: Look, base: Id, p: &SettingrowProps, full: bool) {
    let t = look.t;
    let w = if full { ui.available_width() } else { path_width(ui, &p.choice) };
    let (rect, _) = ui.allocate_exact_size(vec2(w, rem(2.0)), Sense::hover());
    let resp = ui.interact(rect, base.with("button"), Sense::click());
    resp.widget_info(|| WidgetInfo::labeled(Role::Button, true, p.title.as_str()));
    set_value(ui, resp.id, &p.choice);
    part(ui, &resp, parts::BUTTON);
    paint_button(ui, look, rect, &resp, t.surface_3, Some(t.edge_2));
    let painter = ui.painter();
    let cy = rect.center().y;
    let x = rect.left() + rem(0.714);
    draw_icon(painter, "folder", 15.0, pos2(x, cy), Align2::LEFT_CENTER, t.content_2);
    painter.text(pos2(x + 15.0 + rem(0.429), cy), Align2::LEFT_CENTER, &p.choice, fnt(12.0, 500), t.content_1);
}

/// The button (Restore defaults): a neutral button whose visible label is the row's title.
fn button_control(ui: &mut Ui, look: Look, base: Id, p: &SettingrowProps, full: bool) {
    let t = look.t;
    let w = if full { ui.available_width() } else { button_width(ui, &p.title) };
    let (rect, _) = ui.allocate_exact_size(vec2(w, rem(2.0)), Sense::hover());
    let resp = ui.interact(rect, base.with("button"), Sense::click());
    resp.widget_info(|| WidgetInfo::labeled(Role::Button, true, p.title.as_str()));
    part(ui, &resp, parts::BUTTON);
    paint_button(ui, look, rect, &resp, t.wash_2, None);
    ui.painter().text(rect.center(), Align2::CENTER_CENTER, &p.title, fnt(12.0, 500), t.content_1);
}

/// A text node: a run of text, tagged as a part. Its line box is `line` tall, as the web's leading.
fn text_node(ui: &mut Ui, s: &str, font: FontId, color: Color32, line: f32, id: &str) -> Response {
    let wrap = ui.available_width().max(0.0);
    let galley = ui.painter().layout(s.to_owned(), font, color, wrap);
    let (w, h) = (galley.size().x, galley.size().y);
    let (rect, response) = ui.allocate_exact_size(vec2(w, line.max(h)), Sense::hover());
    let top = rect.top() + (rect.height() - h) / 2.0;
    ui.painter().galley(pos2(rect.left(), top), galley, color);
    response.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, s));
    part(ui, &response, id);
    response
}

/// The control's frame: a filled rounded rectangle, an optional border, a hover wash and the focus ring.
fn paint_button(ui: &Ui, look: Look, rect: Rect, resp: &Response, fill: Color32, border: Option<Color32>) {
    let t = look.t;
    let radius = CornerRadius::same(7);
    let painter = ui.painter();
    if let Some(c) = border {
        painter.rect(rect, radius, fill, Stroke::new(1.0, c), StrokeKind::Inside);
    } else {
        painter.rect_filled(rect, radius, fill);
    }
    if resp.hovered() {
        painter.rect_filled(rect, radius, t.wash_1);
    }
    if resp.has_focus() {
        painter.rect_stroke(rect.expand(2.0), radius, Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
    }
}

fn draw_icon(painter: &Painter, name: &str, size: f32, pos: Pos2, anchor: Align2, color: Color32) {
    let glyph = icon(name, size);
    painter.text(pos, anchor, glyph.text(), FontId::new(size, FontFamily::Name("Icons".into())), color);
}

/// The value an accessible node reports as its text, when it differs from its label.
fn set_value(ui: &Ui, id: Id, value: &str) {
    let value = value.to_owned();
    ui.ctx().accesskit_node_builder(id, |node| node.set_value(value.clone()));
}

/// Keeps keyboard focus on a focused widget through the keys egui would otherwise use to move or drop it:
/// the arrow keys (`arrows`) and Escape (`escape`).
fn keep_focus(ui: &Ui, id: Id, arrows: bool, escape: bool) {
    ui.ctx().memory_mut(|m| {
        m.set_focus_lock_filter(
            id,
            egui::EventFilter { horizontal_arrows: arrows, vertical_arrows: arrows, escape, ..Default::default() },
        )
    });
}

/// The selected state of a node the widget info cannot carry for its role (a segment is a button).
fn set_selected(ui: &Ui, id: Id, selected: bool) {
    ui.ctx().accesskit_node_builder(id, |node| node.set_selected(selected));
}

fn fnt(size: f32, w: u16) -> FontId {
    FontId::new(size, weight(w))
}

fn text_w(ui: &Ui, s: &str, font: &FontId) -> f32 {
    ui.painter().layout(s.to_owned(), font.clone(), Color32::WHITE, f32::INFINITY).size().x
}

/// The segment labels, pipe-separated, at most three.
fn segments(p: &SettingrowProps) -> Vec<String> {
    p.choices.split('|').take(3).map(str::to_owned).collect()
}

fn segment_width(ui: &Ui, label: &str) -> f32 {
    text_w(ui, label, &fnt(12.0, 500)) + 2.0 * rem(0.714)
}

fn tray_inline_width(ui: &Ui, segs: &[String]) -> f32 {
    let gap = rem(0.143);
    let inner = rem(0.143);
    let total: f32 = segs.iter().map(|s| segment_width(ui, s)).sum();
    total + gap * (segs.len().saturating_sub(1) as f32) + 2.0 * inner
}

fn button_width(ui: &Ui, label: &str) -> f32 {
    text_w(ui, label, &fnt(12.0, 500)) + 2.0 * rem(0.714)
}

fn path_width(ui: &Ui, path: &str) -> f32 {
    2.0 * rem(0.714) + 15.0 + rem(0.429) + text_w(ui, path, &fnt(12.0, 500))
}

fn hotkey_width(ui: &Ui) -> f32 {
    let font = fnt(12.0, 500);
    let key = KEYCAP_ICON + rem(0.143) + text_w(ui, HOTKEY_KEY, &font);
    let longest = key.max(text_w(ui, RECORDING_TEXT, &font));
    (longest + 2.0 * rem(0.714)).max(rem(2.5))
}

/// The width the inline right-hand block (control, then value) needs.
fn right_width(ui: &Ui, p: &SettingrowProps) -> f32 {
    let control = match p.control.as_str() {
        "toggle" => rem(2.5),
        "segmented" => tray_inline_width(ui, &segments(p)),
        "pathButton" => path_width(ui, &p.choice),
        "button" => button_width(ui, &p.title),
        "hotkey" => hotkey_width(ui),
        _ => 0.0,
    };
    let value = if p.value.is_empty() { 0.0 } else { text_w(ui, &p.value, &fnt(13.0, 400)) };
    let gap = if control > 0.0 && value > 0.0 { rem(0.5) } else { 0.0 };
    control + gap + value
}

crate::mounted!(Settingrow, SettingrowProps);
