//! contract: omnicui/contracts/listrow.contract.yaml (id: ui.listrow). One row of a plain list or a project switcher: a title, an optional meta line, an accent check when selected, and trailing actions revealed on hover or focus.
use crate::omnicui::listrow::{parts, ListrowEvent, ListrowProps};
use crate::{icon, part, rem, weight, Look};
use egui::{pos2, vec2, Align2, Color32, CornerRadius, FontFamily, FontId, Key, Rect, Role, Sense, Stroke, StrokeKind, WidgetInfo};

/// The row is 2.25rem tall.
const HEIGHT: f32 = rem(2.25);
/// A list row (trash) pads px-space-5 with gap-space-5; a menu row (project) pads px-space-6 with gap-space-4.
const PAD_LIST: f32 = 7.0;
const GAP_LIST: f32 = 7.0;
const PAD_MENU: f32 = 8.75;
const GAP_MENU: f32 = 5.25;
/// The check slot (w-4) and the check and action icons, 1rem = 14px.
const SLOT: f32 = 14.0;
const ICON: f32 = 14.0;
/// pr-[4rem]: the room the text keeps clear of the actions.
const ACTIONS_PAD: f32 = 56.0;
/// The action buttons: a square, the gap between them (gap-space-3) and the inset from the right edge (right-space-5).
const ACTION: f32 = 22.0;
const ACTION_GAP: f32 = 3.5;
const ACTION_INSET: f32 = 7.0;
/// The title is 13px on a 16px line; the meta line is 11px on a 14px line.
const TITLE_LINE: f32 = 16.0;
const META_LINE: f32 = 14.0;
/// The fast duration the web uses for hover and reveal.
const FAST: f32 = 0.18;

pub struct Listrow {
    p: ListrowProps,
    /// Radix's pointer highlight (menu host): a row the pointer moves over takes focus without a ring, until it blurs.
    pointer_highlight: bool,
    was_focused: bool,
    /// Escape was pressed on the focused menu row this frame: the menu closes and nothing is chosen.
    dismissed: bool,
}

impl Listrow {
    pub fn new(p: ListrowProps) -> Self {
        Self { p, pointer_highlight: false, was_focused: false, dismissed: false }
    }

    /// Whether Escape closed the menu this frame.
    pub fn dismissed(&self) -> bool {
        self.dismissed
    }

    /// One frame of the row. Returns the events it raised.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<ListrowEvent> {
        let mut events = vec![];
        let t = look.t;
        let menu = self.p.host == "menu";
        let selected = self.p.selected;
        let check = self.p.indicator == "check";
        let actions = self.p.actions == "hover";
        let title = self.p.title.clone();
        let meta = self.p.meta.clone();
        let (pad, gap) = if menu { (PAD_MENU, GAP_MENU) } else { (PAD_LIST, GAP_LIST) };

        // The body is allocated first: it is the focus stop, and the actions follow it in the tab order.
        let width = ui.available_width();
        let (rect, response) = ui.allocate_exact_size(vec2(width, HEIGHT), Sense::click());
        let id = response.id;
        let focused = response.has_focus();
        let hovered = ui.rect_contains_pointer(rect);

        // A menu row takes focus under the pointer, as Radix does; its ring stays off until it blurs.
        if menu && hovered && ui.input(|i| i.pointer.delta() != egui::Vec2::ZERO) {
            response.request_focus();
            self.pointer_highlight = true;
        }
        if self.was_focused && !focused {
            self.pointer_highlight = false;
        }
        self.was_focused = focused;
        self.dismissed = menu && focused && ui.input(|i| i.key_pressed(Key::Escape));

        // The actions: restore or edit, then delete. They exist at rest (their names are read); only their paint fades.
        let first_id = id.with(if menu { "edit" } else { "restore" });
        let delete_id = id.with("delete");
        let within = actions && ui.ctx().memory(|m| m.has_focus(first_id) || m.has_focus(delete_id));
        let revealed = actions && (hovered || focused || within);
        let reveal = if actions { ui.ctx().animate_bool_with_time(id.with("reveal"), revealed, FAST) } else { 0.0 };

        let painter = ui.painter().clone();
        let radius = CornerRadius::same(5);
        if selected {
            painter.rect_filled(rect, radius, t.selection_1);
        } else if hovered || (menu && focused) {
            painter.rect_filled(rect, radius, t.wash_1);
        }
        if focused && !(menu && self.pointer_highlight) {
            painter.rect_stroke(rect.expand(2.0), CornerRadius::same(7), Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
        }

        // The body: a button, which for a menu row is a menuitemradio (its selected state is the row's).
        if menu {
            response.widget_info(|| WidgetInfo::selected(Role::CheckBox, true, selected, &title));
            ui.ctx().accesskit_node_builder(id, |node| {
                node.set_role(Role::Button);
                node.set_selected(selected);
            });
        } else {
            response.widget_info(|| WidgetInfo::labeled(Role::Button, true, &title));
        }
        part(ui, &response, parts::BODY);

        // The list host's listitem wraps the body and the actions.
        if !menu {
            let lr = ui.interact(rect, id.with("listRow"), Sense::hover());
            lr.widget_info(|| WidgetInfo::new(Role::GenericContainer));
            part(ui, &lr, parts::LISTROW);
        }

        // Layout, left to right: the check slot (menu rows with indicator check), then the title and meta block.
        let cy = rect.center().y;
        let mut x = rect.left() + pad;
        if check {
            let slot = Rect::from_center_size(pos2(x + SLOT / 2.0, cy), vec2(SLOT, SLOT));
            if selected {
                let r = ui.interact(slot, id.with("check"), Sense::hover());
                r.widget_info(|| WidgetInfo::new(Role::GenericContainer));
                part(ui, &r, parts::CHECK);
                glyph(&painter, "check", ICON, slot.center(), t.accent_500);
            }
            x += SLOT + gap;
        }
        let right = rect.right() - (if actions { ACTIONS_PAD } else { pad });
        let text_w = (right - x).max(0.0);
        let has_meta = !meta.is_empty();
        let block = if has_meta { TITLE_LINE + META_LINE } else { TITLE_LINE };
        let top = cy - block / 2.0;

        let title_rect = Rect::from_min_size(pos2(x, top), vec2(text_w, TITLE_LINE));
        line(&painter, &title, 13.0, t.content_1, title_rect);
        let tr = ui.interact(title_rect, id.with("title"), Sense::hover());
        tr.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, &title));
        part(ui, &tr, parts::TITLE);

        if has_meta {
            let meta_rect = Rect::from_min_size(pos2(x, top + TITLE_LINE), vec2(text_w, META_LINE));
            line(&painter, &meta, 11.0, t.content_3, meta_rect);
            let mr = ui.interact(meta_rect, id.with("meta"), Sense::hover());
            mr.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, &meta));
            part(ui, &mr, parts::META);
        }

        // The actions sit at the right edge, centred on the row, drawn over the body.
        if actions {
            let del = Rect::from_min_size(pos2(rect.right() - ACTION_INSET - ACTION, cy - ACTION / 2.0), vec2(ACTION, ACTION));
            let first_rect = del.translate(vec2(-(ACTION + ACTION_GAP), 0.0));
            let (first_name, first_part, first_icon, first_action) = if menu {
                (format!("Edit {title}"), parts::EDIT, "edit", "edit")
            } else {
                (format!("Restore {title}"), parts::RESTORE, "restore_from_trash", "restore")
            };
            let buttons = [
                (first_rect, first_id, first_name, first_part, first_icon, first_action, false),
                (del, delete_id, format!("Delete {title}"), parts::DELETEACTION, "delete", "delete", true),
            ];
            for (b, aid, name, part_id, glyph_name, action_name, danger) in buttons {
                let br = ui.interact(b, aid, Sense::click());
                let tone = if danger { t.danger_500 } else { t.content_2 };
                if ui.rect_contains_pointer(b) {
                    let wash = if danger { t.danger_100 } else { t.wash_2 };
                    painter.rect_filled(b, CornerRadius::same(5), wash.gamma_multiply(reveal));
                }
                if br.has_focus() {
                    painter.rect_stroke(b.expand(2.0), CornerRadius::same(7), Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
                }
                glyph(&painter, glyph_name, ICON, b.center(), tone.gamma_multiply(reveal));
                br.widget_info(|| WidgetInfo::labeled(Role::Button, true, &name));
                part(ui, &br, part_id);
                if br.clicked() {
                    events.push(ListrowEvent::ActionRequested { action: action_name.to_owned() });
                }
            }
        }

        // Enter and Space click the focused body (egui does that for a click-sense widget); a press does too.
        if response.clicked() {
            events.push(ListrowEvent::Activated { title: title.clone() });
        }
        events
    }
}

/// Draws a Material Symbols glyph centred on `at`. An unknown name draws nothing.
fn glyph(painter: &egui::Painter, name: &str, size: f32, at: egui::Pos2, color: Color32) {
    let g = icon(name, size);
    painter.text(at, Align2::CENTER_CENTER, g.text(), FontId::new(size, FontFamily::Name("Icons".into())), color);
}

/// Draws one line of text, truncated with an ellipsis to the rect's width, centred vertically in the rect.
fn line(painter: &egui::Painter, s: &str, size: f32, color: Color32, rect: Rect) {
    let mut job = egui::text::LayoutJob::simple_singleline(s.to_owned(), FontId::new(size, weight(400)), color);
    job.wrap.max_width = rect.width();
    job.wrap.max_rows = 1;
    job.wrap.break_anywhere = true;
    job.wrap.overflow_character = Some('…');
    let galley = painter.layout_job(job);
    let at = pos2(rect.left(), rect.center().y - galley.size().y / 2.0);
    painter.galley(at, galley, color);
}

/// The fixture the web harness renders around the row: a padded list row, or the row in the open project switcher
/// panel (17.5rem wide, p-space-4). The menu closes on Escape and when a project is chosen, as the DropdownMenu does.
pub struct ListrowHarness {
    row: Listrow,
    menu: bool,
    open: bool,
}

impl ListrowHarness {
    fn new(p: ListrowProps) -> Self {
        let menu = p.host == "menu";
        Self { row: Listrow::new(p), menu, open: true }
    }

    /// One frame: the row in its panel or padding. Returns the row's events.
    fn draw(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<ListrowEvent> {
        if !self.open {
            return vec![];
        }
        let menu = self.menu;
        let row = &mut self.row;
        let mut events = Vec::new();
        if menu {
            egui::Frame::new()
                .fill(look.t.surface_3)
                .corner_radius(CornerRadius::same(10))
                .inner_margin(egui::Margin::same(5))
                .show(ui, |ui| {
                    ui.set_width(rem(17.5) - 10.0);
                    events = row.ui(ui, look);
                });
        } else {
            egui::Frame::new().inner_margin(egui::Margin::same(7)).show(ui, |ui| {
                events = row.ui(ui, look);
            });
        }
        if row.dismissed() || (menu && events.iter().any(|e| matches!(e, ListrowEvent::Activated { .. }))) {
            self.open = false;
        }
        events
    }
}

impl crate::Mounted for ListrowHarness {
    fn frame(&mut self, ui: &mut egui::Ui, look: Look, emit: &mut dyn FnMut(&'static str, serde_json::Value)) {
        for e in self.draw(ui, look) {
            let (name, fields) = e.to_json();
            emit(name, fields);
        }
    }
}

/// Builds the row, in its harness, from a test's props.
pub fn mount(props: serde_json::Value) -> Result<Box<dyn crate::Mounted>, String> {
    let p: ListrowProps = serde_json::from_value(props).map_err(|e| format!("props: {e}"))?;
    Ok(Box::new(ListrowHarness::new(p)))
}
