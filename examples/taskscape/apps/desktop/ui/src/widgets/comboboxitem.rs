//! contract: omnicui/contracts/comboboxitem.contract.yaml (id: ui.comboboxitem). One option row in a listbox whose focus stays in a text field: a palette command or jump row, or a mention's attachment row, with an optional trailing shortcut.
use crate::omnicui::comboboxitem::{parts, ComboboxitemEvent, ComboboxitemProps};
use crate::omnicui::hotkeyhint::HotkeyhintProps;
use crate::widgets::hotkeyhint::Hotkeyhint;
use crate::{icon, part, rem, text, weight, Look, Mounted};
use egui::text::LayoutJob;
use egui::{pos2, vec2, Align, Align2, CornerRadius, FontFamily, FontId, Id, Key, Layout, Rect, Role, Sense, TextEdit, UiBuilder, WidgetInfo};

/// The palette row is 488 px wide (34.857rem), and the listbox panel is as wide.
const WIDTH: f32 = rem(34.857);
/// Side padding (0.643rem) and the gap between the row's parts (0.5rem).
const PAD: f32 = rem(0.643);
const GAP: f32 = rem(0.5);
/// The label's size and line height (0.929rem on 1.393rem).
const FONT: f32 = rem(0.929);
const LINE: f32 = rem(1.393);
/// The host input's height, and the row's: 30 px, or 32 px with a hotkey.
const INPUT_H: f32 = rem(2.143);
const ROW_H: f32 = rem(2.143);
const HOTKEY_ROW_H: f32 = rem(2.286);

/// The hotkey hint's props for a shortcut such as "Mod+Shift+Period": Mod is Command on macOS and
/// Control elsewhere, as the web's canonicalAccel does.
fn hotkey_props(shortcut: &str) -> HotkeyhintProps {
    let mac = cfg!(target_os = "macos");
    let accel: Vec<&str> = shortcut
        .split('+')
        .map(str::trim)
        .map(|name| match name {
            "Mod" if mac => "Meta",
            "Mod" => "Control",
            other => other,
        })
        .collect();
    let platform = if mac { "macos" } else { "other" };
    let json = serde_json::json!({
        "accel": accel.join("+"),
        "platform": platform,
        "size": "sm",
        "tone": "muted",
    });
    serde_json::from_value(json).expect("hotkeyhint props")
}

/// What the row says for its shortcut, for the part's name and text: Ctrl or Super, or the Apple
/// symbols on macOS.
fn shortcut_label(shortcut: &str) -> String {
    let mac = cfg!(target_os = "macos");
    shortcut
        .split('+')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|name| match (mac, name) {
            (true, "Mod" | "Meta") => "⌘",
            (true, "Alt") => "⌥",
            (true, "Control") => "⌃",
            (true, "Shift") => "⇧",
            (false, "Mod" | "Control") => "Ctrl",
            (false, "Meta") => "Super",
            (_, other) => other,
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// One option row. It holds the active state that the keys and the pointer change, and draws the row.
pub struct ComboboxItem {
    p: ComboboxitemProps,
    active: bool,
    /// Whether the pointer was over the row last frame: entering the row counts as a move.
    hovered: bool,
    /// The trailing shortcut's hotkey hint (palette rows with a shortcut only).
    hint: Option<Hotkeyhint>,
    shortcut_label: String,
}

impl ComboboxItem {
    pub fn new(p: ComboboxitemProps) -> Self {
        let active = p.active;
        let (hint, shortcut_label) = if p.context != "mention" && !p.shortcut.is_empty() {
            (Some(Hotkeyhint::new(hotkey_props(&p.shortcut))), shortcut_label(&p.shortcut))
        } else {
            (None, String::new())
        };
        Self { p, active, hovered: false, hint, shortcut_label }
    }

    fn mention(&self) -> bool {
        self.p.context == "mention"
    }

    /// Makes the row active, and reports it, unless it already is (the harness's highlight).
    fn highlight(&mut self, events: &mut Vec<ComboboxitemEvent>) {
        if !self.active {
            self.active = true;
            events.push(ComboboxitemEvent::Highlighted { id: self.p.id.clone() });
        }
    }

    /// The keys the host input forwards to this row: ArrowDown and ArrowUp move the highlight,
    /// Enter picks the active row, and Tab picks it in a mention. Escape does nothing here.
    pub fn keys(&mut self, ui: &egui::Ui, input: Id) -> Vec<ComboboxitemEvent> {
        let (down, up, enter, tab) = ui.input(|i| {
            (
                i.key_pressed(Key::ArrowDown),
                i.key_pressed(Key::ArrowUp),
                i.key_pressed(Key::Enter),
                i.key_pressed(Key::Tab),
            )
        });
        let mut events = Vec::new();
        if down {
            self.highlight(&mut events);
        }
        if up {
            if self.active {
                self.active = false;
            } else {
                self.highlight(&mut events);
            }
        }
        if self.active && (enter || (tab && self.mention())) {
            events.push(ComboboxitemEvent::Picked { id: self.p.id.clone() });
            // Committing keeps the focus in the input (the mention's editor).
            ui.memory_mut(|m| m.request_focus(input));
        }
        events
    }

    /// One frame of the row: the pointer makes it active or runs it, and the row is drawn.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look, input: Id) -> Vec<ComboboxitemEvent> {
        let t = look.t;
        let mention = self.mention();
        let row_h = if self.hint.is_some() { HOTKEY_ROW_H } else { ROW_H };
        let (rect, response) = ui.allocate_exact_size(vec2(WIDTH, row_h), Sense::click());
        let mut events = Vec::new();

        // Pointer: entering the row, or moving over it while it is not active, makes it active (the
        // web's pointermove). A mention runs on the press (mousedown); a palette row on the click.
        let over = ui.input(|i| i.pointer.hover_pos().is_some_and(|p| rect.contains(p)));
        let moved = ui.input(|i| i.pointer.delta() != vec2(0.0, 0.0));
        if over && !self.active && (moved || !self.hovered) {
            self.highlight(&mut events);
        }
        self.hovered = over;
        let pressed = mention && over && ui.input(|i| i.pointer.primary_pressed());
        if pressed || (!mention && response.clicked()) {
            events.push(ComboboxitemEvent::Picked { id: self.p.id.clone() });
        }
        if pressed || response.clicked() {
            // The press keeps the focus in the input (the web's preventDefault on mousedown).
            ui.memory_mut(|m| m.request_focus(input));
        }

        // Paint: the active wash, then the icon, the label and the trailing shortcut.
        let color = if mention && !self.active { t.content_2 } else { t.content_1 };
        if self.active {
            ui.painter().rect_filled(rect, CornerRadius::same(4), t.selection_1);
        }
        let cy = rect.center().y;
        let mut x = rect.left() + PAD;

        if !self.p.icon.is_empty() {
            let size = if mention { 15.0 } else { 16.0 };
            let glyph = icon(&self.p.icon, size);
            let at = Rect::from_min_size(pos2(x, cy - size / 2.0), vec2(size, size));
            let node = ui.interact(at, response.id.with("icon"), Sense::hover());
            node.widget_info(|| WidgetInfo::new(Role::GenericContainer));
            part(ui, &node, parts::ICON);
            ui.painter().text(
                at.center(),
                Align2::CENTER_CENTER,
                glyph.text(),
                FontId::new(size, FontFamily::Name("Icons".into())),
                color,
            );
            x = at.right() + GAP;
        }

        let mut right = rect.right() - PAD;
        if let Some(hint) = self.hint.as_mut() {
            // The hint sits at the right edge, laid out right to left in its own slot so the label
            // gets the width that is left.
            let slot_rect = Rect::from_min_max(pos2(x, rect.top()), pos2(right, rect.bottom()));
            let mut slot = ui.new_child(
                UiBuilder::new()
                    .max_rect(slot_rect)
                    .layout(Layout::right_to_left(Align::Center)),
            );
            let used = slot.scope(|s| hint.ui(s, look)).response.rect;
            let node = ui.interact(used, response.id.with("shortcut"), Sense::hover());
            node.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, &self.shortcut_label));
            part(ui, &node, parts::SHORTCUT);
            right = used.left() - GAP;
        }

        // The label truncates with an ellipsis inside what is left of the row.
        let label_w = (right - x).max(0.0);
        let mut job = LayoutJob::simple(self.p.label.clone(), FontId::new(FONT, weight(400)), color, label_w);
        job.wrap.max_rows = 1;
        job.wrap.break_anywhere = true;
        job.wrap.overflow_character = Some('…');
        let galley = ui.painter().layout_job(job);
        let label_rect = Rect::from_min_size(pos2(x, cy - LINE / 2.0), vec2(label_w, LINE));
        let node = ui.interact(label_rect, response.id.with("label"), Sense::hover());
        node.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, &self.p.label));
        part(ui, &node, parts::LABEL);
        let galley_y = cy - galley.size().y / 2.0;
        ui.painter().galley(pos2(x, galley_y), galley, color);

        // The row's own node: an option named by its label, selected while it is active.
        response.widget_info(|| WidgetInfo::labeled(Role::Button, true, &self.p.label));
        // WidgetInfo does not carry aria-selected for a button, so the node's selected flag is set directly.
        let selected = self.active;
        ui.ctx().accesskit_node_builder(response.id, |node| node.set_selected(selected));
        part(ui, &response, parts::ROW);
        events
    }
}

/// The fixture the web harness draws around the row: the host's search input (its text is the
/// palette filter, and it takes the focus), then the listbox with the one row.
pub struct ComboboxItemMounted {
    input_text: String,
    row: ComboboxItem,
}

impl ComboboxItemMounted {
    fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<ComboboxitemEvent> {
        let t = look.t;
        let mut events = Vec::new();
        egui::Frame::NONE
            .fill(t.surface_2)
            .corner_radius(CornerRadius::same(10))
            .show(ui, |ui| {
                ui.set_width(WIDTH);
                ui.spacing_mut().item_spacing = vec2(0.0, 0.0);

                let input = ui.add_sized(
                    [WIDTH, INPUT_H],
                    TextEdit::singleline(&mut self.input_text)
                        .hint_text(text("Type a command or a name", FONT, 400).color(t.content_3))
                        .font(FontId::new(FONT, weight(400)))
                        .text_color(t.content_1)
                        .frame(egui::Frame::NONE.inner_margin(egui::Margin::symmetric(9, 7)))
                        .lock_focus(true),
                );
                part(ui, &input, parts::INPUT);
                if ui.memory(|m| m.focused().is_none()) {
                    input.request_focus();
                }

                events.extend(self.row.keys(ui, input.id));
                events.extend(self.row.ui(ui, look, input.id));
            });
        events
    }
}

impl Mounted for ComboboxItemMounted {
    fn frame(&mut self, ui: &mut egui::Ui, look: Look, emit: &mut dyn FnMut(&'static str, serde_json::Value)) {
        for e in self.ui(ui, look) {
            let (name, fields) = e.to_json();
            emit(name, fields);
        }
    }
}

/// Builds the fixture from a test's props (the registry calls this).
pub fn mount(props: serde_json::Value) -> Result<Box<dyn Mounted>, String> {
    let p: ComboboxitemProps = serde_json::from_value(props).map_err(|e| format!("props: {e}"))?;
    Ok(Box::new(ComboboxItemMounted { input_text: String::new(), row: ComboboxItem::new(p) }))
}
