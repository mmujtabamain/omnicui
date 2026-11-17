//! contract: omnicui/contracts/listtab.contract.yaml (id: ui.listtab). The full list tab in the desktop title bar: a tab with hover, drop-target and drag feedback, inline rename, and a delete button on its right.
use crate::omnicui::listtab::{parts, ListtabEvent, ListtabProps};
use crate::{icon, part, rem, weight, Look};
use egui::{pos2, vec2, Align2, Color32, CornerRadius, CursorIcon, FontFamily, FontId, Key, Modifiers, Rect, Role, Sense, Stroke, StrokeKind, WidgetInfo};

/// Tab height: 1.75rem (24.5px). The delete button is a square of the same size.
const TAB_H: f32 = rem(1.75);
/// Text: 0.929rem (13px), font-medium.
const TEXT_PX: f32 = 13.0;
/// Horizontal padding: px-2 (8px). Border: 1px on each side.
const PAD: f32 = 8.0;
const BORDER: f32 = 1.0;
/// The tab's max width, max-w-[11.429rem] (160px), border and padding included.
const MAX_W: f32 = rem(11.429);
/// The split-view marker: 0.5rem (7px), then a gap of 0.357rem (5px) to the name.
const MARKER: f32 = rem(0.5);
const GAP: f32 = rem(0.357);
/// The row's gap-px between the tab and the delete button.
const ROW_GAP: f32 = 1.0;
/// The row's min-w-[7.143rem] (100px) while renaming.
const EDIT_MIN_W: f32 = rem(7.143);
/// The web's --radius-field (5px), and the control radius (7px).
const FIELD_RADIUS: u8 = 5;
const CONTROL_RADIUS: u8 = 7;
/// Hover and reveal fades: the web's base duration (0.18 s).
const FADE: f32 = 0.18;
/// opacity-40 while the tab is dragged.
const DRAG_ALPHA: f32 = 0.4;

/// Shortens `name` with an ellipsis until it fits in `max_w` pixels.
fn fit(painter: &egui::Painter, name: &str, font: &FontId, color: Color32, max_w: f32) -> String {
    let width = |s: &str| painter.layout_no_wrap(s.to_owned(), font.clone(), color).size().x;
    if width(name) <= max_w {
        return name.to_owned();
    }
    let chars: Vec<char> = name.chars().collect();
    let (mut lo, mut hi) = (0usize, chars.len());
    while lo < hi {
        let mid = (lo + hi + 1) / 2;
        let cand: String = chars[..mid].iter().collect::<String>() + "\u{2026}";
        if width(&cand) <= max_w {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    chars[..lo].iter().collect::<String>() + "\u{2026}"
}

pub struct ListTab {
    p: ListtabProps,
    /// The rename input's text while editing.
    draft: String,
    /// Editing was on last frame, so an edit starts (and the input takes focus) once.
    was_editing: bool,
    /// The rename input had focus last frame, so a blur and a key for it can be seen.
    had_focus: bool,
    /// Enter, Escape or a commit has settled this edit, so a later blur does not emit again.
    settled: bool,
    /// Select the input's text once its state exists.
    select_pending: bool,
}

impl ListTab {
    pub fn new(p: ListtabProps) -> Self {
        let draft = p.name.clone();
        Self { p, draft, was_editing: false, had_focus: false, settled: false, select_pending: false }
    }

    /// One frame: draws the tab, its delete button and the rename input, and returns the events raised this frame.
    /// `selected` and `editing` are host-controlled: a press never changes them.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<ListtabEvent> {
        let mut events = vec![];
        let t = look.t;
        let name = self.p.name.clone();
        let selected = self.p.selected;
        let editing = self.p.editing;
        let drop = self.p.dropTarget;
        let alpha = if self.p.dragging { DRAG_ALPHA } else { 1.0 };
        let k = |c: Color32| c.gamma_multiply(alpha);
        let base = ui.scope_id().with("listtab");
        let input_id = base.with("rename");
        let font = FontId::new(TEXT_PX, weight(500));
        let ink = if selected { t.content_1 } else { t.content_3 };
        let radius = CornerRadius::same(FIELD_RADIUS);

        // An edit starts once: the draft is the name, and the input takes focus with its text selected.
        if editing && !self.was_editing {
            self.draft = name.clone();
            self.settled = false;
            self.select_pending = true;
            ui.memory_mut(|m| m.request_focus(input_id));
        }
        self.was_editing = editing;

        // Enter and Escape are read before the input draws, so the text field does not act on them. The input
        // is taken to have focus if it had it last frame: egui may already have moved focus on Escape this frame.
        let was_focused = editing && self.had_focus;
        let enter = was_focused && ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Enter));
        let escape = was_focused && ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape));

        // Widths: the name, truncated so the tab stays within max-w.
        let marker_w = if self.p.paneMarker { MARKER + GAP } else { 0.0 };
        let max_text = MAX_W - 2.0 * BORDER - 2.0 * PAD - marker_w;
        let label = fit(ui.painter(), &name, &font, ink, max_text);
        let text_w = ui.painter().layout_no_wrap(label.clone(), font.clone(), ink).size().x;
        let tab_w = 2.0 * BORDER + 2.0 * PAD + marker_w + text_w;
        let row_w = if editing { tab_w.max(EDIT_MIN_W) } else { tab_w + ROW_GAP + TAB_H };

        // The row (the wrapper): a plain container around the tab, the delete button and the rename input.
        let (wrap, wrap_resp) = ui.allocate_exact_size(vec2(row_w, TAB_H), Sense::hover());
        wrap_resp.widget_info(|| WidgetInfo::new(Role::GenericContainer));
        part(ui, &wrap_resp, parts::WRAPPER);
        let row_hover = ui.rect_contains_pointer(wrap);

        // The tab: role button (ARIA tab in the web), named by the list, selection on the node.
        let tab = Rect::from_min_size(wrap.min, vec2(tab_w, TAB_H));
        let tab_resp = ui.interact(tab, base.with("tab"), Sense::click()).on_hover_cursor(CursorIcon::PointingHand);
        if tab_resp.clicked() && !selected {
            events.push(ListtabEvent::Select {});
        }
        tab_resp.widget_info(|| WidgetInfo::selected(Role::Button, true, selected, name.clone()));
        part(ui, &tab_resp, parts::TAB);
        ui.ctx().accesskit_node_builder(tab_resp.id, |node| node.set_selected(selected));
        if tab_resp.has_focus() {
            // Arrows are the strip's to move between tabs; egui must not move focus off a lone tab.
            ui.memory_mut(|m| m.set_focus_lock_filter(tab_resp.id, egui::EventFilter { horizontal_arrows: true, ..Default::default() }));
            if ui.input(|i| i.key_pressed(Key::F2)) {
                events.push(ListtabEvent::RenameStart {});
            }
            if ui.input(|i| i.key_pressed(Key::F10) && i.modifiers.shift) {
                events.push(ListtabEvent::ContextMenu {});
            }
        }

        // The split-view marker: decorative, a node of its own, inside the tab before the name.
        let marker_c = pos2(tab.left() + BORDER + PAD + MARKER / 2.0, tab.center().y);
        if self.p.paneMarker {
            let mr = ui.interact(Rect::from_center_size(marker_c, vec2(MARKER, MARKER)), tab_resp.id.with("paneMarker"), Sense::hover());
            mr.widget_info(|| WidgetInfo::new(Role::GenericContainer));
            part(ui, &mr, parts::PANEMARKER);
        }

        // The delete button: a sibling of the tab, never a child. Not present while renaming.
        let del_rect = Rect::from_min_size(pos2(tab.right() + ROW_GAP, wrap.top()), vec2(TAB_H, TAB_H));
        let del = if editing {
            None
        } else {
            let r = ui.interact(del_rect, base.with("delete"), Sense::click()).on_hover_cursor(CursorIcon::PointingHand);
            if r.clicked() {
                events.push(ListtabEvent::Delete {});
            }
            r.widget_info(|| WidgetInfo::labeled(Role::Button, true, format!("Delete {name}")));
            part(ui, &r, parts::DELETEBUTTON);
            Some(r)
        };

        // Drawing the tab: the drop target, the selected fill, or the idle wash on hover.
        let hover_t = ui.ctx().animate_bool_with_time(base.with("hover"), tab_resp.hovered() && !selected && !drop, FADE);
        let painter = ui.painter().clone();
        if drop {
            painter.rect_filled(tab, radius, k(t.selection_1));
        } else if selected {
            painter.rect_filled(tab, radius, k(t.surface_2));
            painter.rect_stroke(tab, radius, Stroke::new(BORDER, k(t.edge_1)), StrokeKind::Inside);
        } else if hover_t > 0.0 {
            painter.rect_filled(tab, radius, k(t.wash_1).gamma_multiply(hover_t));
        }
        if self.p.paneMarker {
            painter.circle_filled(marker_c, MARKER / 2.0, k(t.accent_500));
        }
        painter.text(pos2(tab.left() + BORDER + PAD + marker_w, tab.center().y), Align2::LEFT_CENTER, label, font.clone(), k(ink));
        if tab_resp.has_focus() {
            painter.rect_stroke(tab.expand(2.0), CornerRadius::same(FIELD_RADIUS + 2), Stroke::new(2.0, t.focus_1), StrokeKind::Inside);
        }

        // Drawing the delete button: revealed while its row is hovered, as the web's reveal="row".
        if let Some(r) = &del {
            let lit = r.hovered() || r.has_focus();
            let shown = row_hover || lit;
            let reveal = ui.ctx().animate_bool_with_time(base.with("reveal"), shown, FADE);
            if lit {
                painter.rect_filled(del_rect, CornerRadius::same(CONTROL_RADIUS), t.danger_100.gamma_multiply(alpha * reveal));
            }
            let color = if lit { t.danger_500 } else { t.content_2 };
            let glyph = icon("close", 14.0);
            painter.text(
                del_rect.center(),
                Align2::CENTER_CENTER,
                glyph.text(),
                FontId::new(14.0, FontFamily::Name("Icons".into())),
                color.gamma_multiply(alpha * reveal),
            );
            if r.has_focus() {
                painter.rect_stroke(del_rect.expand(2.0), CornerRadius::same(CONTROL_RADIUS + 2), Stroke::new(2.0, t.focus_1), StrokeKind::Inside);
            }
        }

        // The rename input: over the row where the tab is, after the tab in focus order.
        if editing {
            let focused = ui.memory(|m| m.has_focus(input_id));
            let input_rect = Rect::from_min_size(wrap.min, vec2(row_w, TAB_H));
            let edge = if focused { t.focus_1 } else { t.edge_2 };
            painter.rect_filled(input_rect, radius, t.surface_3);
            painter.rect_stroke(input_rect, radius, Stroke::new(BORDER, edge), StrokeKind::Inside);
            if focused {
                painter.rect_stroke(input_rect.expand(2.0), CornerRadius::same(FIELD_RADIUS + 2), Stroke::new(2.0, t.focus_1), StrokeKind::Inside);
            }

            // Escape restores the name before the field draws, so it shows the name this frame.
            if escape {
                self.settled = true;
                self.draft = name.clone();
                events.push(ListtabEvent::RenameCancel {});
            }

            let row_h = painter.layout_no_wrap("Ag".to_owned(), font.clone(), t.content_1).size().y;
            let inner = input_rect.shrink2(vec2(BORDER + PAD, 0.0));
            let text_rect = Rect::from_center_size(inner.center(), vec2(inner.width(), row_h));
            let edit = egui::TextEdit::singleline(&mut self.draft)
                .id(input_id)
                .font(font.clone())
                .text_color(t.content_1)
                .frame(egui::Frame::NONE)
                .desired_width(inner.width());
            let input = ui.put(text_rect, edit);
            part(ui, &input, parts::RENAMEINPUT);
            let input_focused = ui.memory(|m| m.has_focus(input_id));
            if input_focused {
                // Escape keeps focus in the field, as the web input does.
                ui.memory_mut(|m| m.set_focus_lock_filter(input_id, egui::EventFilter { escape: true, ..Default::default() }));
            }

            // The edit starts with the whole name selected, as the web input does on focus.
            if self.select_pending && input_focused {
                if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), input_id) {
                    let end = self.draft.chars().count();
                    state.cursor.set_char_range(Some(egui::text_selection::CCursorRange::two(
                        egui::text::CCursor::new(0),
                        egui::text::CCursor::new(end),
                    )));
                    state.store(ui.ctx(), input_id);
                    self.select_pending = false;
                }
            }

            if enter {
                // Enter commits the typed name, including what was typed this frame.
                self.settled = true;
                events.push(ListtabEvent::RenameCommit { name: self.draft.clone() });
            } else if self.had_focus && !input_focused && !self.settled && self.draft != name {
                // Blur with a changed name commits it; an unchanged name emits nothing.
                self.settled = true;
                events.push(ListtabEvent::RenameCommit { name: self.draft.clone() });
            }
            self.had_focus = input_focused;
        } else {
            self.had_focus = false;
        }

        events
    }
}

crate::mounted!(ListTab, ListtabProps);
