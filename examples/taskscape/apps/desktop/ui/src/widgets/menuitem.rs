//! contract: omnicui/contracts/menuitem.contract.yaml (id: ui.menuitem). One row of a menu: an action, a radio choice in a labelled group, a row that opens a submenu, or the separated footer action.
use crate::omnicui::menuitem::{parts, MenuitemEvent, MenuitemProps};
use crate::{icon, part, rem, weight, Look};
use egui::{pos2, vec2, Align2, Color32, CornerRadius, FontFamily, FontId, Id, Key, Order, Pos2, Rect, Role, Sense, Stroke, StrokeKind, WidgetInfo};

/// The check slot (w-4, 14px), and the gap between a row's children (gap-space-5, 7px).
const SLOT: f32 = 14.0;
const GAP: f32 = 7.0;
/// px-space-6: the row's horizontal padding.
const PAD: f32 = 8.75;
/// p-space-5: the padding of the menu panel the web harness draws the row in.
const PANEL_PAD: f32 = 7.0;
/// pr-[4rem]: the room a project row keeps clear for its actions.
const ACTIONS_PAD: f32 = 56.0;
/// Leading, check and trailing icons.
const ICON: f32 = 15.0;
/// The hover-revealed action buttons: a square, the gap between them, and their inset (right-space-5).
const ACTION: f32 = 22.0;
const ACTION_GAP: f32 = 3.5;
const ACTION_INSET: f32 = 7.0;

const LETTERS: [Key; 26] = [
    Key::A, Key::B, Key::C, Key::D, Key::E, Key::F, Key::G, Key::H, Key::I, Key::J, Key::K, Key::L, Key::M,
    Key::N, Key::O, Key::P, Key::Q, Key::R, Key::S, Key::T, Key::U, Key::V, Key::W, Key::X, Key::Y, Key::Z,
];

/// A row as its list sees it: its id, whether it can be reached, and its label.
#[derive(Clone)]
struct Entry {
    id: Id,
    enabled: bool,
    label: String,
}

/// The rows of one list, in draw order. Keys act on the last frame's list, which is complete.
#[derive(Clone, Default)]
struct Rows {
    stamp: u64,
    cur: Vec<Entry>,
    prev: Vec<Entry>,
}

pub struct Menuitem {
    p: MenuitemProps,
    /// A radio row's checked state. Once a row in its group is chosen, the group's value decides it.
    checked: bool,
    /// A submenu row's open state.
    open: bool,
    /// Whether the first frame has run (a radio row claims its group on its first frame).
    synced: bool,
}

impl Menuitem {
    pub fn new(p: MenuitemProps) -> Self {
        let checked = p.checked;
        Self { p, checked, open: false, synced: false }
    }

    /// The row's height: 30px, or 32px for a large or action row.
    fn height(&self) -> f32 {
        if self.p.variant == "action" || self.p.size == "large" {
            rem(2.286)
        } else {
            rem(2.143)
        }
    }

    /// Opens or closes the submenu, reporting only a change (a hover followed by a press is one opening).
    fn set_open(&mut self, next: bool, events: &mut Vec<MenuitemEvent>) {
        if self.open == next || (next && self.p.disabled) {
            return;
        }
        self.open = next;
        events.push(MenuitemEvent::SubmenuToggled { open: next });
    }

    /// One frame of the row. `list` names the rows the row is listed with: arrow keys move between them in
    /// draw order. With `paint` false the row is a harness box: it is hit and focused but draws nothing.
    fn draw(&mut self, ui: &mut egui::Ui, look: Look, part_id: &str, list: Id, paint: bool) -> Vec<MenuitemEvent> {
        let mut events = vec![];
        let t = look.t;
        let disabled = self.p.disabled;
        let radio = self.p.selection == "radio";
        let submenu = self.p.trailing == "submenu";
        let danger = self.p.tone == "danger";
        let action = self.p.variant == "action";
        let large = action || self.p.size == "large";
        let height = self.height();
        let label_size = if large { 13.5 } else { 13.0 };
        let line = if large { 20.25 } else { 19.5 };
        let group = list.with("menuitem.radio");
        let rows_key = list.with("menuitem.rows");

        let sense = if disabled { Sense::hover() } else { Sense::click() };
        let width = ui.available_width();
        let (rect, response) = ui.allocate_exact_size(vec2(width, height), sense);
        let id = response.id;
        let hovered = !disabled && ui.rect_contains_pointer(rect);
        let focused = response.has_focus();

        // Register this row in its list, in draw order, and read the list as of last frame.
        let stamp = ui.ctx().cumulative_pass_nr();
        let entry = Entry { id, enabled: !disabled, label: self.p.label.clone() };
        let prev: Vec<Entry> = ui.ctx().data_mut(|d| {
            let rows = d.get_temp_mut_or_default::<Rows>(rows_key);
            if rows.stamp != stamp {
                rows.prev = std::mem::take(&mut rows.cur);
                rows.stamp = stamp;
            }
            rows.cur.push(entry);
            rows.prev.clone()
        });

        // Radio group: a checked row claims the group on its first frame; later frames follow the group's value.
        if radio {
            if !self.synced {
                self.synced = true;
                if self.checked {
                    let value = self.p.value.clone();
                    ui.data_mut(|d| d.insert_temp(group, value));
                }
            } else if let Some(chosen) = ui.data(|d| d.get_temp::<String>(group)) {
                self.checked = chosen == self.p.value;
            }
        }

        // Keyboard, while the row has focus: arrows, Home, End and typeahead move the highlight to another
        // enabled row (disabled rows are skipped); the submenu keys open and close a submenu row.
        if focused && !disabled {
            let (up, down, home, end, right, left, escape, typed) = ui.input(|i| {
                let typed = LETTERS.iter().position(|k| i.key_pressed(*k)).map(|n| (b'a' + n as u8) as char);
                (
                    i.key_pressed(Key::ArrowUp),
                    i.key_pressed(Key::ArrowDown),
                    i.key_pressed(Key::Home),
                    i.key_pressed(Key::End),
                    i.key_pressed(Key::ArrowRight),
                    i.key_pressed(Key::ArrowLeft),
                    i.key_pressed(Key::Escape),
                    typed,
                )
            });
            let me = prev.iter().position(|e| e.id == id);
            let target: Option<Id> = if down {
                me.and_then(|m| prev[m + 1..].iter().find(|e| e.enabled).map(|e| e.id))
            } else if up {
                me.and_then(|m| prev[..m].iter().rev().find(|e| e.enabled).map(|e| e.id))
            } else if home {
                prev.iter().find(|e| e.enabled).map(|e| e.id)
            } else if end {
                prev.iter().rev().find(|e| e.enabled).map(|e| e.id)
            } else if let (Some(c), Some(m)) = (typed, me) {
                let n = prev.len();
                (1..n)
                    .map(|k| &prev[(m + k) % n])
                    .find(|e| e.enabled && e.label.chars().next().is_some_and(|f| f.to_ascii_lowercase() == c))
                    .map(|e| e.id)
            } else {
                None
            };
            if let Some(next) = target {
                ui.ctx().memory_mut(|m| m.request_focus(next));
            }
            if submenu && right {
                self.set_open(true, &mut events);
            }
            if submenu && (left || escape) {
                self.set_open(false, &mut events);
            }
        }

        // Pointer: a submenu row opens on hover; a row the pointer moves over takes focus, as Radix does.
        if !disabled && response.hovered() {
            if submenu {
                self.set_open(true, &mut events);
            }
            if ui.input(|i| i.pointer.delta() != egui::Vec2::ZERO) {
                response.request_focus();
            }
        }

        // Press, Enter or Space (egui clicks a focused widget on Enter and Space). A submenu row only opens.
        if response.clicked() && !disabled {
            if submenu {
                self.set_open(true, &mut events);
            } else {
                if radio {
                    self.checked = true;
                    let value = self.p.value.clone();
                    ui.data_mut(|d| d.insert_temp(group, value));
                }
                events.push(MenuitemEvent::Activated {
                    value: if radio { self.p.value.clone() } else { String::new() },
                    label: self.p.label.clone(),
                });
            }
        }

        // Accessibility. A radio row is a button part whose checked state is its node's selected state; the
        // WidgetInfo for a check box carries that state, and the role is then set back to button.
        let checked = radio && self.checked;
        let label = self.p.label.clone();
        if radio {
            response.widget_info(|| WidgetInfo::selected(Role::CheckBox, !disabled, checked, &label));
            ui.ctx().accesskit_node_builder(id, |node| {
                node.set_role(Role::Button);
                node.set_selected(checked);
            });
        } else {
            response.widget_info(|| WidgetInfo::labeled(Role::Button, !disabled, &label));
        }
        part(ui, &response, part_id);

        // Colours: the label and icon tones, faded when disabled (opacity-40).
        let alpha = if disabled { 0.4 } else { 1.0 };
        let fade = |c: Color32| c.gamma_multiply(alpha);
        let base = if danger { t.danger_500 } else { t.content_1 };
        let muted = if danger { t.danger_500 } else { t.content_2 };

        // An unseen row paints through a clip rect that is empty, so nothing of it is drawn.
        let painter = if paint { ui.painter().clone() } else { ui.painter().with_clip_rect(Rect::NOTHING) };
        let radius = CornerRadius::same(if action { 0 } else { 5 });
        if (hovered || focused) && !disabled {
            let wash = if danger { t.danger_100 } else { t.wash_2 };
            painter.rect_filled(rect, radius, wash);
        }
        if focused && !hovered && !disabled {
            painter.rect_stroke(rect.expand(2.0), radius, Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
        }
        if action {
            painter.hline(rect.x_range(), rect.top(), Stroke::new(1.0, t.edge_1));
        }

        // Layout, left to right: check slot, leading icon, label, and the trailing chevron on the right.
        let cy = rect.center().y;
        let mut x = rect.left() + PAD;
        if radio {
            if checked {
                let at = pos2(x + SLOT / 2.0, cy);
                let r = ui.interact(Rect::from_center_size(at, vec2(ICON, ICON)), id.with("check"), Sense::hover());
                r.widget_info(|| WidgetInfo::new(Role::GenericContainer));
                part(ui, &r, parts::CHECK);
                glyph(&painter, "check", ICON, at, fade(t.accent_500));
            }
            x += SLOT + GAP;
        }
        if !self.p.icon.is_empty() {
            let at = pos2(x + ICON / 2.0, cy);
            let r = ui.interact(Rect::from_center_size(at, vec2(ICON, ICON)), id.with("icon"), Sense::hover());
            r.widget_info(|| WidgetInfo::new(Role::GenericContainer));
            part(ui, &r, parts::ICON);
            glyph(&painter, &self.p.icon, ICON, at, fade(muted));
            x += ICON + GAP;
        }
        let right_pad = if self.p.rowActions { ACTIONS_PAD } else { PAD };
        let chevron_w = if submenu { ICON + GAP } else { 0.0 };
        let right = (rect.right() - right_pad - chevron_w).max(x);
        let label_rect = Rect::from_min_max(pos2(x, cy - line / 2.0), pos2(right, cy + line / 2.0));
        let color = fade(base);
        let mut job = egui::text::LayoutJob::simple_singleline(self.p.label.clone(), FontId::new(label_size, weight(400)), color);
        if self.p.truncate {
            job.wrap.max_width = label_rect.width();
            job.wrap.max_rows = 1;
            job.wrap.break_anywhere = true;
            job.wrap.overflow_character = Some('…');
        }
        let galley = painter.layout_job(job);
        painter.galley(label_rect.min, galley, color);
        let lr = ui.interact(label_rect, id.with("label"), Sense::hover());
        lr.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, !disabled, &label));
        part(ui, &lr, parts::LABEL);

        if submenu {
            let at = pos2(rect.right() - PAD - ICON / 2.0, cy);
            let r = ui.interact(Rect::from_center_size(at, vec2(ICON, ICON)), id.with("chevron"), Sense::hover());
            r.widget_info(|| WidgetInfo::new(Role::GenericContainer));
            part(ui, &r, parts::CHEVRON);
            glyph(&painter, "chevron_right", ICON, at, fade(muted));
        }

        // Project rows: rename and delete are revealed while the pointer is on the row, as siblings of it.
        if self.p.rowActions && hovered && paint {
            let del = Rect::from_center_size(pos2(rect.right() - ACTION_INSET - ACTION / 2.0, cy), vec2(ACTION, ACTION));
            let ren = Rect::from_center_size(pos2(del.left() - ACTION_GAP - ACTION / 2.0, cy), vec2(ACTION, ACTION));
            for (b, name, title, suffix, part_id, is_rename) in [
                (ren, "edit", "Rename", "rename", parts::RENAMEACTION, true),
                (del, "delete", "Delete", "delete", parts::DELETEACTION, false),
            ] {
                let br = ui.interact(b, id.with(suffix), Sense::click());
                if br.hovered() {
                    painter.rect_filled(b, CornerRadius::same(5), t.wash_2);
                }
                glyph(&painter, name, ICON, b.center(), fade(muted));
                br.widget_info(|| WidgetInfo::labeled(Role::Button, true, title));
                part(ui, &br, part_id);
                if br.clicked() {
                    events.push(if is_rename {
                        MenuitemEvent::RenamePressed { label: label.clone() }
                    } else {
                        MenuitemEvent::DeletePressed { label: label.clone() }
                    });
                }
            }
        }

        // Submenu: a panel beside the row, shown while it is open, with the three harness items.
        if submenu && self.open && paint {
            let at = pos2(rect.right() + 2.0, rect.top() - 4.0);
            let _ = egui::Area::new(id.with("submenu"))
                .order(Order::Foreground)
                .fixed_pos(at)
                .show(ui.ctx(), |ui| {
                    egui::Frame::new()
                        .fill(t.surface_3)
                        .stroke(Stroke::new(1.0, t.edge_2))
                        .corner_radius(CornerRadius::same(10))
                        .inner_margin(egui::Margin::same(7))
                        .show(ui, |ui| {
                            let (sub, resp) = ui.allocate_exact_size(vec2(rem(8.571), 3.0 * rem(2.143)), Sense::hover());
                            resp.widget_info(|| WidgetInfo::new(Role::GenericContainer));
                            part(ui, &resp, parts::SUBMENU);
                            for (i, name) in ["Inbox", "Today", "Someday"].into_iter().enumerate() {
                                let y = sub.top() + rem(2.143) * (i as f32 + 0.5);
                                ui.painter().text(pos2(sub.left() + PAD, y), Align2::LEFT_CENTER, name, FontId::new(13.0, weight(400)), t.content_1);
                            }
                        });
                });
        }

        events
    }
}

/// Draws a Material Symbols glyph centred on `at`. An unknown name draws nothing.
fn glyph(painter: &egui::Painter, name: &str, size: f32, at: egui::Pos2, color: Color32) {
    let g = icon(name, size);
    painter.text(at, Align2::CENTER_CENTER, g.text(), FontId::new(size, FontFamily::Name("Icons".into())), color);
}

/// A row the web harness renders beside the row and never draws: it is reached by keys and presses, and it is
/// placed below the panel, one slot per row (opacity 0 in the web).
fn unseen(ctx: &egui::Context, look: Look, m: &mut Menuitem, part_id: &str, list: Id, at: Pos2, width: f32) {
    let _ = egui::Area::new(list.with(part_id))
        .fixed_pos(at)
        .constrain(false)
        .show(ctx, |ui| {
            ui.set_width(width);
            let _ = m.draw(ui, look, part_id, list, false);
        });
}

/// The menu the web harness mounts the row in: a panel holding the row, with the rows it renders beside it. The
/// previous row (first in the list), the disabled row and the sibling are unseen; for radio rows the row sits in a
/// group named "Date range", and the siblings are radio rows of the same group.
pub struct MenuitemHarness {
    row: Menuitem,
    previous: Menuitem,
    disabled_sibling: Menuitem,
    sibling: Menuitem,
    radio: bool,
}

impl MenuitemHarness {
    fn new(p: MenuitemProps) -> Self {
        let radio = p.selection == "radio";
        let (previous, disabled_sibling, sibling) = if radio {
            (
                MenuitemProps { selection: "radio".into(), value: "90d".into(), label: "Last 90 days".into(), ..Default::default() },
                MenuitemProps { selection: "radio".into(), value: "14d".into(), label: "Last 14 days".into(), disabled: true, ..Default::default() },
                MenuitemProps { selection: "radio".into(), value: "30d".into(), label: "Last 30 days".into(), ..Default::default() },
            )
        } else {
            (
                MenuitemProps { label: "Paste".into(), ..Default::default() },
                MenuitemProps { label: "Archived lists".into(), disabled: true, ..Default::default() },
                MenuitemProps { label: "Move to list".into(), ..Default::default() },
            )
        };
        Self {
            row: Menuitem::new(p),
            previous: Menuitem::new(previous),
            disabled_sibling: Menuitem::new(disabled_sibling),
            sibling: Menuitem::new(sibling),
            radio,
        }
    }

    /// One frame: the panel with the row, and the unseen rows around it. Returns the row's events.
    fn draw(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<MenuitemEvent> {
        let list = ui.scope_id().with("ui.menuitem.harness");
        let height = self.row.height();
        let panel_w = rem(17.5);
        let width = panel_w - 2.0 * PANEL_PAD;
        let panel = Rect::from_min_size(ui.cursor().min, vec2(panel_w, height + 2.0 * PANEL_PAD));
        let below = |slot: f32| pos2(panel.left(), panel.bottom() + rem(2.0) * slot);

        // The previous row is placed above the panel, the way it precedes the row in the list: egui's own arrow
        // navigation finds the row above, so ArrowUp and this list agree.
        let above = pos2(panel.left(), panel.top() - rem(2.0));
        unseen(ui.ctx(), look, &mut self.previous, parts::PREVIOUS, list, above, width);

        let radio = self.radio;
        let row = &mut self.row;
        let mut events = Vec::new();
        egui::Frame::new()
            .fill(look.t.surface_3)
            .corner_radius(CornerRadius::same(10))
            .inner_margin(egui::Margin::same(7))
            .show(ui, |ui| {
                ui.set_width(width);
                if radio {
                    // The labelled group: a container with the row's rect, drawn before the row so it does not
                    // take the row's hover.
                    let r = ui.interact(Rect::from_min_size(ui.cursor().min, vec2(width, height)), list.with("menuitem.group"), Sense::hover());
                    r.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, "Date range"));
                    part(ui, &r, parts::GROUP);
                }
                events = row.draw(ui, look, parts::ROW, list, true);
            });

        unseen(ui.ctx(), look, &mut self.disabled_sibling, parts::DISABLEDSIBLING, list, below(1.0), width);
        unseen(ui.ctx(), look, &mut self.sibling, parts::SIBLING, list, below(2.0), width);
        events
    }
}

impl crate::Mounted for MenuitemHarness {
    fn frame(&mut self, ui: &mut egui::Ui, look: Look, emit: &mut dyn FnMut(&'static str, serde_json::Value)) {
        for e in self.draw(ui, look) {
            let (name, fields) = e.to_json();
            emit(name, fields);
        }
    }
}

/// Builds the row, in its harness, from a test's props.
pub fn mount(props: serde_json::Value) -> Result<Box<dyn crate::Mounted>, String> {
    let p: MenuitemProps = serde_json::from_value(props).map_err(|e| format!("props: {e}"))?;
    Ok(Box::new(MenuitemHarness::new(p)))
}
