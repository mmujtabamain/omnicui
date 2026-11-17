//! contract: omnicui/contracts/tabpanel.contract.yaml (id: ui.tabpanel). The scrolling content pane of the active Settings section: it scrolls when its content overflows and reports when it reaches the bottom.
use crate::omnicui::tabpanel::{parts, TabpanelEvent, TabpanelProps};
use crate::{part, rem, weight, Look};
use egui::{pos2, vec2, Align2, CornerRadius, FontId, Rect, Role, Sense, Stroke, StrokeKind, UiBuilder, WidgetInfo};

/// The web's px-8 and py-7 padding, in px.
const PAD_X: f32 = 32.0;
const PAD_Y: f32 = 28.0;
/// One settings row: h-[2.5rem].
const ROW_H: f32 = rem(2.5);
/// One arrow-key step, in px.
const ARROW_STEP: f32 = 40.0;

pub struct Tabpanel {
    p: TabpanelProps,
    /// The atBottom last reported, so that `scrolled` is emitted only when it changes.
    at_bottom: bool,
    /// The scroll offset and the largest it can be, as the last frame left them.
    offset: f32,
    max: f32,
    /// Whether the content overflows: the pane takes keyboard focus only then.
    overflowing: bool,
}

impl Tabpanel {
    pub fn new(p: TabpanelProps) -> Self {
        Self { p, at_bottom: false, offset: 0.0, max: 0.0, overflowing: false }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<TabpanelEvent> {
        self.show(ui, look, |_| {})
    }

    /// One frame: draws the pane around `add_contents` and returns the events it raised.
    pub fn show(&mut self, ui: &mut egui::Ui, look: Look, add_contents: impl FnOnce(&mut egui::Ui)) -> Vec<TabpanelEvent> {
        let t = look.t;
        let mut events = vec![];
        let scroll = self.p.scroll;
        // Only an overflowing scroll region takes focus (the web's tabIndex 0 then), and only then scrolls with keys.
        let sense = if scroll && self.overflowing { Sense::focusable_noninteractive() } else { Sense::hover() };
        let rect = ui.available_rect_before_wrap();
        let _ = ui.allocate_rect(rect, Sense::empty());
        let mut child = ui.new_child(UiBuilder::new().max_rect(rect).layout(egui::Layout::top_down(egui::Align::Min)));
        // The panel is the scroll area's own node: the scroll area's id is the panel's id, so a scroll
        // aimed at the panel reaches the area that moves.
        let scroll_id = child.make_persistent_id("scroll");
        let response = ui.interact(rect, scroll_id, sense);
        let focused = response.has_focus();
        let (offset, max) = (self.offset, self.max);
        let target = if scroll && focused {
            let page = rect.height() * 0.9;
            ui.input(|i| {
                if i.key_pressed(egui::Key::ArrowDown) {
                    Some(offset + ARROW_STEP)
                } else if i.key_pressed(egui::Key::ArrowUp) {
                    Some(offset - ARROW_STEP)
                } else if i.key_pressed(egui::Key::PageDown) {
                    Some(offset + page)
                } else if i.key_pressed(egui::Key::PageUp) {
                    Some(offset - page)
                } else if i.key_pressed(egui::Key::Home) {
                    Some(0.0)
                } else if i.key_pressed(egui::Key::End) {
                    Some(max)
                } else {
                    None
                }
            })
            .map(|y| y.clamp(0.0, max))
        } else {
            None
        };

        ui.painter().rect_filled(rect, CornerRadius::same(0), t.surface_2);
        if scroll {
            let mut area = egui::ScrollArea::vertical()
                .id_salt("scroll")
                .auto_shrink([false, false]);
            if let Some(y) = target {
                area = area.vertical_scroll_offset(y);
            }
            let out = area.show(&mut child, |ui| contents(ui, add_contents));
            let now = out.state.clamped_offset().y;
            self.max = (out.content_size.y - out.inner_rect.height()).max(0.0);
            self.overflowing = self.max > 1.0;
            // Like the web's onScroll: only a scroll can change atBottom, so a pane that never moved reports nothing.
            if now != self.offset {
                let bottom = now >= self.max - 1.0;
                if bottom != self.at_bottom {
                    self.at_bottom = bottom;
                    events.push(TabpanelEvent::Scrolled { atBottom: bottom });
                }
            }
            self.offset = now;
            // The scroll range goes on the node as AccessKit's scroll properties: a scroll aimed at the
            // panel works out how far it can move from them.
            let (y, max) = (now as f64, self.max as f64);
            ui.ctx().accesskit_node_builder(response.id, |node| {
                node.set_scroll_y_min(0.0);
                node.set_scroll_y_max(max);
                node.set_scroll_y(y);
                node.add_action(egui::accesskit::Action::ScrollDown);
                node.add_action(egui::accesskit::Action::ScrollUp);
            });
        } else {
            // No scroll region: the content is clipped to the pane and nothing scrolls.
            child.set_clip_rect(rect);
            contents(&mut child, add_contents);
        }

        if focused {
            ui.painter().rect_stroke(rect, CornerRadius::same(0), Stroke::new(2.0, t.focus_1), StrokeKind::Inside);
        }
        response.widget_info(|| WidgetInfo::new(Role::GenericContainer));
        part(ui, &response, parts::PANEL);
        events
    }
}

/// The pane's padding around its contents. It sits inside the scrolled area, so it counts toward the scroll height.
fn contents(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
    ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
    // Empty sense: the rows and padding are not hit by the pointer, so the scroll area under them keeps the wheel.
    let _ = ui.allocate_exact_size(vec2(0.0, PAD_Y), Sense::empty());
    add(ui);
    let _ = ui.allocate_exact_size(vec2(0.0, PAD_Y), Sense::empty());
}

/// A full-width block inside the pane's side padding, `height` tall, at the cursor.
fn block(ui: &mut egui::Ui, height: f32) -> Rect {
    let avail = ui.available_rect_before_wrap();
    let rect = Rect::from_min_size(pos2(avail.left() + PAD_X, avail.top()), vec2(avail.width() - 2.0 * PAD_X, height));
    let _ = ui.allocate_rect(rect, Sense::empty());
    rect
}

/// One Settings section: a title, then a card of label and value rows (the web's Group).
type Section = (&'static str, &'static [(&'static str, &'static str)]);

static SECTIONS: [Section; 4] = [
    ("General", &[("Launch at login", "On"), ("Show in menu bar", "On"), ("Language", "English")]),
    ("Startup", &[("Open last workspace", "On"), ("Restore windows", "Off"), ("Startup sound", "Off")]),
    ("Appearance", &[("Theme", "System"), ("Density", "Comfortable"), ("Reduce motion", "Off")]),
    ("Notifications", &[("Task reminders", "On")]),
];

fn draw_sections(ui: &mut egui::Ui, look: Look, sections: &[Section]) {
    let t = look.t;
    for &(title, rows) in sections {
        // The title: 11 px text on a 1.5 line, then mb-1.5.
        let head = block(ui, 16.5);
        // Each title and row is a labelled node, so the content has bounds inside the pane.
        let title_resp = ui.interact(head, ui.scope_id().with(("title", title)), Sense::hover());
        title_resp.widget_info(|| WidgetInfo::labeled(Role::Label, true, title));
        ui.painter().text(head.left_center(), Align2::LEFT_CENTER, title, FontId::new(rem(0.786), weight(500)), t.content_3);
        ui.add_space(6.0);
        // The card: a 1 px border around rows of 2.5 rem, a 1 px rule between rows.
        let card = block(ui, 2.0 + ROW_H * rows.len() as f32);
        let painter = ui.painter();
        painter.rect(card, CornerRadius::same(10), t.surface_3, Stroke::new(1.0, t.edge_1), StrokeKind::Inside);
        for (i, &(label, value)) in rows.iter().enumerate() {
            let top = card.top() + 1.0 + i as f32 * ROW_H;
            let row_rect = Rect::from_min_size(pos2(card.left(), top), vec2(card.width(), ROW_H));
            let row_resp = ui.interact(row_rect, ui.scope_id().with(("row", title, label)), Sense::hover());
            row_resp.widget_info(|| WidgetInfo::labeled(Role::Label, true, format!("{label}: {value}")));
            if i > 0 {
                painter.line_segment([pos2(card.left(), top), pos2(card.right(), top)], Stroke::new(1.0, t.edge_1));
            }
            let mid = top + ROW_H / 2.0;
            painter.text(pos2(card.left() + 12.0, mid), Align2::LEFT_CENTER, label, FontId::new(13.0, weight(400)), t.content_1);
            painter.text(pos2(card.right() - 12.0, mid), Align2::RIGHT_CENTER, value, FontId::new(13.0, weight(400)), t.content_2);
        }
        // mb-5 after each section.
        ui.add_space(20.0);
    }
}

/// The desktop mount: the panel with the Settings sections the web harness renders inside it.
pub struct TabpanelMount {
    panel: Tabpanel,
    sections: usize,
}

impl crate::Mounted for TabpanelMount {
    fn frame(&mut self, ui: &mut egui::Ui, look: Look, emit: &mut dyn FnMut(&'static str, serde_json::Value)) {
        let sections = &SECTIONS[..self.sections];
        for e in self.panel.show(ui, look, |ui| draw_sections(ui, look, sections)) {
            let (name, fields) = e.to_json();
            emit(name, fields);
        }
    }
}

/// Builds the widget from a test's props.
pub fn mount(props: serde_json::Value) -> Result<Box<dyn crate::Mounted>, String> {
    let p: TabpanelProps = serde_json::from_value(props).map_err(|e| format!("props: {e}"))?;
    // The web harness renders only the first two sections when scroll is off; they fit without overflowing.
    let sections = if p.scroll { SECTIONS.len() } else { 2 };
    Ok(Box::new(TabpanelMount { panel: Tabpanel::new(p), sections }))
}
