//! contract: omnicui/contracts/select.contract.yaml (id: ui.select). Single choice from a fixed preset list: the field shows the current value, and a press, Enter, Space or ArrowDown opens the list, where the arrows, Home, End and type-ahead move the highlight and Enter, Space or a press picks it.
use crate::omnicui::select::{parts, SelectEvent, SelectProps};
use crate::{icon, part, weight, Look};
use egui::emath::TSTransform;
use egui::{pos2, vec2, Align2, Color32, CornerRadius, Event, FontFamily, FontId, Key, Rect, Role, Sense, Stroke, StrokeKind, WidgetInfo};

/// The sort presets (value, label), in order.
const SORT: [(&str, &str); 5] = [
    ("manual", "Manual"),
    ("dateCreated", "Date created"),
    ("dateUpdated", "Date updated"),
    ("alphabetical", "Alphabetical"),
    ("doneLast", "Done last"),
];

/// The date range presets (value, label), in order.
const DATE_RANGE: [(&str, &str); 5] = [
    ("anyTime", "Any time"),
    ("last24Hours", "Last 24 hours"),
    ("last7Days", "Last 7 days"),
    ("last30Days", "Last 30 days"),
    ("custom", "Custom..."),
];

const OPTION_PARTS: [&str; 5] = [parts::OPTION1, parts::OPTION2, parts::OPTION3, parts::OPTION4, parts::OPTION5];

/// The trigger is 26 px tall (1.857rem) and its text 12.5 px (0.893rem), weight 500.
const TRIGGER_H: f32 = 26.0;
const TEXT_PX: f32 = 12.5;
/// One option row: 4 px padding above and below an 18.75 px line.
const ITEM_H: f32 = 26.75;
/// Typed letters build a search that resets after a second without a key.
const SEARCH_RESET_S: f64 = 1.0;

fn options(set: &str) -> [(&'static str, &'static str); 5] {
    if set == "dateRange" {
        DATE_RANGE
    } else {
        SORT
    }
}

/// The option index a typed prefix selects, from `from` on and wrapping. A repeated letter
/// ("dd") moves on to the next option that starts with it.
fn match_index(labels: &[&str], text: &str, from: usize) -> Option<usize> {
    let n = labels.len();
    let chars: Vec<char> = text.chars().collect();
    let repeated = chars.len() > 1 && chars.iter().all(|c| *c == chars[0]);
    let (start, prefix) = if repeated {
        (from + 1, chars[0].to_string())
    } else {
        (from, text.to_string())
    };
    (0..n)
        .map(|step| (start + step) % n)
        .find(|&i| labels[i].to_lowercase().starts_with(&prefix))
}

fn text_width(ui: &egui::Ui, s: &str, font: &FontId) -> f32 {
    ui.painter().layout_no_wrap(s.to_owned(), font.clone(), Color32::WHITE).size().x
}

/// The text cut to `max` px, with an ellipsis when it does not fit.
fn fit(ui: &egui::Ui, s: &str, font: &FontId, max: f32) -> String {
    if text_width(ui, s, font) <= max {
        return s.to_owned();
    }
    let mut chars: Vec<char> = s.chars().collect();
    while chars.pop().is_some() {
        let cand: String = chars.iter().collect::<String>() + "\u{2026}";
        if text_width(ui, &cand, font) <= max {
            return cand;
        }
    }
    String::new()
}

/// The Material Symbols glyph for a name, as a string for the painter.
fn glyph(name: &str, size: f32) -> String {
    icon(name, size).text().to_owned()
}

fn icons(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("Icons".into()))
}

fn key(ui: &egui::Ui, k: Key) -> bool {
    ui.input(|i| i.key_pressed(k))
}

/// The printable characters typed this frame (Space is not a search key). Text events carry them;
/// where a frame has none, a letter or digit key press stands for the character it types.
fn typed(ui: &egui::Ui) -> Vec<char> {
    ui.input(|i| {
        if i.modifiers.ctrl || i.modifiers.command || i.modifiers.alt {
            return Vec::new();
        }
        let text: Vec<char> = i
            .events
            .iter()
            .filter_map(|e| match e {
                Event::Text(s) => Some(s.chars().collect::<Vec<char>>()),
                _ => None,
            })
            .flatten()
            .filter(|c| *c != ' ' && !c.is_control())
            .collect();
        if !text.is_empty() {
            return text;
        }
        i.events
            .iter()
            .filter_map(|e| match e {
                Event::Key { key, pressed: true, .. } => {
                    let mut chars = key.name().chars();
                    match (chars.next(), chars.next()) {
                        (Some(c), None) if c.is_ascii_alphanumeric() => Some(c.to_ascii_lowercase()),
                        _ => None,
                    }
                }
                _ => None,
            })
            .collect()
    })
}

pub struct Select {
    p: SelectProps,
    open: bool,
    highlight: usize,
    search: String,
    search_at: f64,
    pointer_mode: bool,
    list_rect: Option<Rect>,
}

impl Select {
    pub fn new(p: SelectProps) -> Self {
        Self {
            p,
            open: false,
            highlight: 0,
            search: String::new(),
            search_at: f64::NEG_INFINITY,
            pointer_mode: false,
            list_rect: None,
        }
    }

    fn close(&mut self) {
        self.open = false;
        self.list_rect = None;
    }

    /// A pick: closes the list, and reports the value only when it differs from the current one.
    fn choose(&mut self, value: &str, events: &mut Vec<SelectEvent>) {
        self.close();
        if value != self.p.value {
            events.push(SelectEvent::ValueChanged { value: value.to_owned() });
        }
    }

    /// Adds a typed letter to the search and returns the search so far.
    fn type_ahead(&mut self, now: f64, c: char) -> String {
        if now - self.search_at > SEARCH_RESET_S {
            self.search.clear();
        }
        self.search_at = now;
        self.search.extend(c.to_lowercase());
        self.search.clone()
    }

    /// One frame: draws the field, and the list while it is open, and returns the events raised.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<SelectEvent> {
        let mut events = vec![];
        let t = look.t;
        let enabled = !self.p.disabled;
        let opts = options(&self.p.optionSet);
        let labels: Vec<&str> = opts.iter().map(|o| o.1).collect();
        let last = labels.len() - 1;
        let value = self.p.value.clone();
        let selected = opts.iter().position(|o| o.0 == value).unwrap_or(0);
        let current = opts.iter().find(|o| o.0 == value).map_or("", |o| o.1);
        let has_icon = !self.p.leadingIcon.is_empty();
        let aria = self.p.ariaLabel.clone();
        let font = FontId::new(TEXT_PX, weight(500));
        let now = ui.input(|i| i.time);
        let alpha = if enabled { 1.0 } else { 0.45 };

        let enter_space = ui.input(|i| i.key_pressed(Key::Enter) || i.key_pressed(Key::Space));
        if ui.input(|i| i.pointer.any_pressed()) {
            self.pointer_mode = true;
        }
        if ui.input(|i| i.events.iter().any(|e| matches!(e, Event::Key { pressed: true, .. }))) {
            self.pointer_mode = false;
        }

        // The field: the panel's width when full, else the content's (8 px inset, glyph, text, chevron).
        let inset = if has_icon { 28.0 } else { 8.0 };
        let width = if self.p.fullWidth {
            ui.available_width()
        } else {
            inset + text_width(ui, current, &font) + 28.0
        };
        let sense = if enabled { Sense::click() } else { Sense::hover() };
        let (rect, response) = ui.allocate_exact_size(vec2(width, TRIGGER_H), sense);
        let trigger_id = response.id;

        // Opening: a press, or Enter or Space (egui's click), or ArrowDown or a typed letter on the focused field.
        let mut opened = false;
        let mut typed_now = Vec::new();
        if enabled && response.clicked() {
            if self.open {
                self.close();
            } else {
                opened = true;
            }
        } else if enabled && !self.open && response.has_focus() {
            typed_now = typed(ui);
            opened = key(ui, Key::ArrowDown) || !typed_now.is_empty();
        }
        if opened {
            self.open = true;
            self.highlight = selected;
            for c in typed_now {
                let s = self.type_ahead(now, c);
                if let Some(i) = match_index(&labels, &s, self.highlight) {
                    self.highlight = i;
                }
            }
        }

        // Keys while the list is open. The frame that opened it does not also act on the keys.
        if enabled && self.open && !opened {
            if key(ui, Key::Escape) {
                self.close();
                response.request_focus();
            } else if enter_space {
                let picked = opts[self.highlight].0;
                self.choose(picked, &mut events);
                response.request_focus();
            } else {
                if key(ui, Key::ArrowDown) {
                    self.highlight = (self.highlight + 1).min(last);
                }
                if key(ui, Key::ArrowUp) {
                    self.highlight = self.highlight.saturating_sub(1);
                }
                if key(ui, Key::Home) {
                    self.highlight = 0;
                }
                if key(ui, Key::End) {
                    self.highlight = last;
                }
                for c in typed(ui) {
                    let s = self.type_ahead(now, c);
                    if let Some(i) = match_index(&labels, &s, self.highlight) {
                        self.highlight = i;
                    }
                }
            }
        }

        // A press outside both the field and the list dismisses it, without moving focus.
        if self.open && !opened && ui.input(|i| i.pointer.any_pressed()) {
            if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                if !rect.contains(pos) && !self.list_rect.is_some_and(|r| r.contains(pos)) {
                    self.close();
                }
            }
        }

        // The field's accessible node, its value node and its glyph node.
        response.widget_info(|| WidgetInfo::labeled(Role::Button, enabled, aria.clone()));
        part(ui, &response, parts::TRIGGER);

        let mid = rect.center().y;
        let hovered = enabled && response.hovered();
        let fill = if hovered { t.wash_1 } else { t.surface_0 };
        let ink = t.content_1.gamma_multiply(alpha);
        ui.painter().rect_filled(rect, CornerRadius::same(5), fill.gamma_multiply(alpha));
        if response.has_focus() && !self.pointer_mode {
            ui.painter().rect_stroke(rect, CornerRadius::same(5), Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
        }

        if has_icon {
            let icon_rect = Rect::from_center_size(pos2(rect.left() + 14.0, mid), vec2(16.0, 16.0));
            let node = ui.interact(icon_rect, trigger_id.with("icon"), Sense::hover());
            node.widget_info(|| WidgetInfo::new(Role::GenericContainer));
            part(ui, &node, parts::ICON);
            ui.painter().text(icon_rect.center(), Align2::CENTER_CENTER, glyph(&self.p.leadingIcon, 16.0), icons(16.0), ink);
        }

        let text_x = rect.left() + inset;
        let max_w = (rect.right() - 28.0 - text_x).max(0.0);
        let shown = fit(ui, current, &font, max_w);
        let label_rect = Rect::from_min_max(pos2(text_x, rect.top()), pos2(text_x + max_w, rect.bottom()));
        let node = ui.interact(label_rect, trigger_id.with("current"), Sense::hover());
        node.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, current));
        part(ui, &node, parts::CURRENT);
        ui.painter().text(pos2(text_x, mid), Align2::LEFT_CENTER, shown, font.clone(), ink);
        ui.painter().text(pos2(rect.right() - 14.0, mid), Align2::CENTER_CENTER, glyph("expand_more", 16.0), icons(16.0), ink);

        // The list: fades and scales in from its top-left corner (the web's scale-in, 0.18 s).
        let anim = ui.ctx().animate_bool_with_time(trigger_id.with("open"), self.open, 0.18);
        if self.open {
            // Focus follows the highlight, so the keys move it on.
            let hid = trigger_id.with(("option", self.highlight));
            ui.ctx().memory_mut(|m| m.request_focus(hid));

            let widest = opts.iter().map(|o| text_width(ui, o.1, &font)).fold(0.0f32, f32::max);
            // Padding 4 px and a 1 px border on each side; the list is at least as wide as the field.
            let panel_w = (widest + 46.0).max(rect.width());
            let inner_w = panel_w - 10.0;
            let pos = pos2(rect.left(), rect.bottom() + 4.0);
            let fade = anim;
            let area = egui::Area::new(trigger_id.with("listbox"))
                .order(egui::Order::Foreground)
                .fixed_pos(pos)
                .show(ui.ctx(), |ui| {
                    let frame = egui::Frame::new()
                        .fill(t.surface_3.gamma_multiply(fade))
                        .stroke(Stroke::new(1.0, t.edge_2.gamma_multiply(fade)))
                        .corner_radius(CornerRadius::same(10))
                        .inner_margin(egui::Margin::same(4));
                    let inner = frame.show(ui, |ui| {
                        ui.set_width(inner_w);
                        ui.spacing_mut().item_spacing.y = 0.0;
                        for (i, opt) in opts.iter().enumerate() {
                            let (irect, _) = ui.allocate_exact_size(vec2(inner_w, ITEM_H), Sense::hover());
                            let ir = ui.interact(irect, trigger_id.with(("option", i)), Sense::click());
                            let is_sel = i == selected;
                            ir.widget_info(|| WidgetInfo::selected(Role::Button, true, is_sel, opt.1));
                            // The option's selected flag is set on its node directly; the button role
                            // does not carry it from WidgetInfo.
                            ui.ctx().accesskit_node_builder(ir.id, |node| node.set_selected(is_sel));
                            part(ui, &ir, OPTION_PARTS[i]);
                            let item_ink = t.content_1.gamma_multiply(fade);
                            if self.highlight == i {
                                ui.painter().rect_filled(irect, CornerRadius::same(5), t.wash_2.gamma_multiply(fade));
                            }
                            if is_sel {
                                ui.painter().text(
                                    pos2(irect.left() + 16.0, irect.center().y),
                                    Align2::CENTER_CENTER,
                                    glyph("check", 16.0),
                                    icons(16.0),
                                    item_ink,
                                );
                            }
                            ui.painter().text(
                                pos2(irect.left() + 28.0, irect.center().y),
                                Align2::LEFT_CENTER,
                                opt.1,
                                font.clone(),
                                item_ink,
                            );
                            // A keyboard Enter or Space is handled above, so only a press picks here.
                            if ir.clicked() && !enter_space && !opened {
                                self.choose(opt.0, &mut events);
                                response.request_focus();
                            }
                        }
                    });
                    let lr = inner.response.rect;
                    self.list_rect = Some(lr);
                    let node = ui.interact(lr, trigger_id.with("listbox-node"), Sense::hover());
                    node.widget_info(|| WidgetInfo::new(Role::GenericContainer));
                    part(ui, &node, parts::LISTBOX);
                });
            let scale = 0.95 + 0.05 * anim;
            ui.ctx().set_transform_layer(
                area.response.layer_id,
                TSTransform::new(pos.to_vec2() * (1.0 - scale), scale),
            );
        }
        events
    }
}

crate::mounted!(Select, SelectProps);
