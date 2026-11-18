//! contract: omnicui/contracts/menu.contract.yaml (id: ui.menu). The one role=menu floating panel: a control, cursor or anchored placement, with the harness's fixed rows, a Move to list submenu and an Actions trigger.
use crate::omnicui::menu::{parts, MenuEvent, MenuProps};
use crate::widgets::separator::Separator;
use crate::{icon, part, rem, weight, Look};
use egui::{pos2, vec2, Align2, Color32, CornerRadius, FontFamily, FontId, Id, Key, Order, Pos2, Rect, Response, Role, Sense, Stroke, StrokeKind, Vec2, WidgetInfo};

/// The check slot (14px), the gap after a slot or an icon (7px), the row's horizontal padding (8.75px) and an icon (15px).
const SLOT: f32 = 14.0;
const GAP: f32 = 7.0;
const PAD: f32 = 8.75;
const ICON: f32 = 15.0;
/// The gap a cursor, anchored or submenu panel keeps from the window's edge.
const INSET: f32 = 4.0;

/// The navigable rows in order, with their labels. The separator and the disabled row are not in this list.
const MAIN: [(&str, &str); 5] = [("first", "Open"), ("second", "Rename"), ("submenu", "Move to list"), ("choice", "Compact"), ("last", "Trash")];
/// Every focusable part's name, for finding which one has focus.
const NAMES: [&str; 7] = ["trigger", "first", "second", "submenu", "choice", "last", "submenuFirst"];
/// The main panel's rows as (label, radio, submenu), for the width a content panel takes.
const ROWS: [(&str, bool, bool); 6] = [
    ("Open", false, false),
    ("Rename", false, false),
    ("Delete", false, false),
    ("Move to list", false, true),
    ("Compact", true, false),
    ("Trash", false, false),
];
/// The submenu's rows as (label, radio, submenu).
const SUB_ROWS: [(&str, bool, bool); 1] = [("Inbox", false, false)];

const LETTERS: [Key; 26] = [
    Key::A, Key::B, Key::C, Key::D, Key::E, Key::F, Key::G, Key::H, Key::I, Key::J, Key::K, Key::L, Key::M,
    Key::N, Key::O, Key::P, Key::Q, Key::R, Key::S, Key::T, Key::U, Key::V, Key::W, Key::X, Key::Y, Key::Z,
];

/// The id of a named part of this menu. Rows, the trigger and the panels each have one.
fn rid(name: &'static str) -> Id {
    Id::unique(("ui.menu", name))
}

/// The keys this frame, read once.
#[derive(Default)]
struct Keys {
    up: bool,
    down: bool,
    home: bool,
    end: bool,
    left: bool,
    right: bool,
    escape: bool,
    tab: bool,
    shift_f10: bool,
    typed: Option<char>,
}

fn read_keys(ui: &egui::Ui) -> Keys {
    ui.input(|i| {
        let typed = if i.modifiers.command || i.modifiers.alt {
            None
        } else {
            LETTERS.iter().position(|k| i.key_pressed(*k)).map(|n| (b'a' + n as u8) as char)
        };
        Keys {
            up: i.key_pressed(Key::ArrowUp),
            down: i.key_pressed(Key::ArrowDown),
            home: i.key_pressed(Key::Home),
            end: i.key_pressed(Key::End),
            left: i.key_pressed(Key::ArrowLeft),
            right: i.key_pressed(Key::ArrowRight),
            escape: i.key_pressed(Key::Escape),
            tab: i.key_pressed(Key::Tab),
            shift_f10: i.key_pressed(Key::F10) && i.modifiers.shift,
            typed,
        }
    })
}

/// The name of the focused part, if it is one of this menu's.
fn focused_name(focus: Option<Id>) -> Option<&'static str> {
    let focus = focus?;
    NAMES.iter().copied().find(|n| rid(*n) == focus)
}

fn main_index(name: &str) -> Option<usize> {
    MAIN.iter().position(|(n, _)| *n == name)
}

/// One row: its part name, its label, and what it is.
#[derive(Clone, Copy)]
struct Spec {
    name: &'static str,
    part: &'static str,
    label: &'static str,
    enabled: bool,
    submenu: bool,
    radio: bool,
}

/// The width of a label at the row's 13px text.
fn label_w(ctx: &egui::Context, s: &str) -> f32 {
    ctx.fonts_mut(|f| f.layout_no_wrap(s.to_owned(), FontId::new(13.0, weight(400)), Color32::WHITE).size().x)
}

/// The width a panel of these rows takes when it is sized by its rows.
fn natural(ctx: &egui::Context, rows: &[(&str, bool, bool)]) -> f32 {
    rows.iter().fold(0.0_f32, |w, &(label, radio, submenu)| {
        let mut r = 2.0 * PAD + label_w(ctx, label);
        if radio {
            r += SLOT + GAP;
        }
        if submenu {
            r += GAP + ICON;
        }
        w.max(r)
    })
}

fn glyph(painter: &egui::Painter, name: &str, size: f32, at: Pos2, color: Color32) {
    let g = icon(name, size);
    painter.text(at, Align2::CENTER_CENTER, g.text(), FontId::new(size, FontFamily::Name("Icons".into())), color);
}

/// An outline button: the trigger and the outside button are both drawn this way.
fn outline_button(ui: &mut egui::Ui, look: Look, id: Id, part_id: &str, label: &str) -> Response {
    let t = look.t;
    let font = FontId::new(13.0, weight(500));
    let text_w = ui.ctx().fonts_mut(|f| f.layout_no_wrap(label.to_owned(), font.clone(), t.content_1).size().x);
    let (rect, _) = ui.allocate_exact_size(vec2(text_w + 2.0 * rem(0.857), rem(2.286)), Sense::hover());
    let response = ui.interact(rect, id, Sense::click());
    let painter = ui.painter().clone();
    let radius = CornerRadius::same(7);
    painter.rect(rect, radius, t.surface_3, Stroke::new(1.0, t.edge_2), StrokeKind::Inside);
    if response.hovered() {
        painter.rect_filled(rect, radius, t.wash_1);
    }
    if response.has_focus() {
        painter.rect_stroke(rect.expand(2.0), radius, Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
    }
    painter.text(rect.center(), Align2::CENTER_CENTER, label, font, t.content_1);
    response.widget_info(|| WidgetInfo::labeled(Role::Button, true, label));
    part(ui, &response, part_id);
    response
}

/// The menu: its trigger, and the panel it opens with the harness's rows.
pub struct Menu {
    p: MenuProps,
    /// The panel is open.
    open: bool,
    /// The Move to list submenu is open.
    sub_open: bool,
    /// The pointer position a right-click opened the menu at, for cursor placement.
    click_point: Option<Pos2>,
    /// Whether the radio row has been picked.
    chosen: bool,
    /// The part to focus at the end of this frame.
    pending: Option<Id>,
    /// The panel's size, from the last frame, for lifting a cursor panel that would run past the bottom.
    size: Vec2,
    /// Where the panel and the submenu were last frame, so a press outside them can be seen.
    panel_rect: Option<Rect>,
    sub_rect: Option<Rect>,
    /// The rule between the two groups of rows.
    rule: Separator,
}

impl Menu {
    pub fn new(p: MenuProps) -> Self {
        let open = p.open;
        let rule = Separator::new(
            serde_json::from_value(serde_json::json!({ "inset": "menu", "orientation": "horizontal" })).expect("separator props"),
        );
        Self {
            p,
            open,
            sub_open: false,
            click_point: None,
            chosen: false,
            pending: None,
            size: Vec2::ZERO,
            panel_rect: None,
            sub_rect: None,
            rule,
        }
    }

    fn control(&self) -> bool {
        self.p.placement == "control"
    }

    /// The panel's width: a fixed width for task and tab, the widest row for content, and at least 210px for project.
    fn width_px(&self, ctx: &egui::Context) -> f32 {
        match self.p.width.as_str() {
            "task" => rem(10.857),
            "tab" => rem(11.143),
            "project" => natural(ctx, &ROWS).max(rem(15.0)),
            _ => natural(ctx, &ROWS),
        }
    }

    /// Opens or closes the panel, reporting only a change.
    fn set_open(&mut self, next: bool, reason: &str, events: &mut Vec<MenuEvent>) {
        if self.open == next {
            return;
        }
        self.open = next;
        if !next {
            self.sub_open = false;
            self.panel_rect = None;
            self.sub_rect = None;
        }
        events.push(MenuEvent::OpenChange { open: next, reason: reason.to_owned() });
    }

    fn open_menu(&mut self, reason: &str, focus_first: bool, events: &mut Vec<MenuEvent>) {
        if self.open {
            return;
        }
        self.set_open(true, reason, events);
        if focus_first {
            self.pending = Some(rid("first"));
        }
    }

    fn open_sub(&mut self, focus: bool) {
        self.sub_open = true;
        if focus {
            self.pending = Some(rid("submenuFirst"));
        }
    }

    fn close_sub(&mut self, focus_row: bool) {
        self.sub_open = false;
        if focus_row {
            self.pending = Some(rid("submenu"));
        }
    }

    /// A row was picked: the pick is reported, and the menu closes if closeOnSelect says so.
    fn activate(&mut self, id: &str, events: &mut Vec<MenuEvent>) {
        events.push(MenuEvent::Pick { id: id.to_owned() });
        if self.p.closeOnSelect {
            self.set_open(false, "select", events);
            self.pending = Some(rid("trigger"));
        }
    }

    /// Arrow keys, Home, End and typeahead from the main row at index `m`.
    fn navigate(&mut self, m: usize, k: &Keys) {
        let n = MAIN.len();
        let target: Option<&'static str> = if k.down {
            MAIN.get(m + 1).map(|r| r.0)
        } else if k.up {
            m.checked_sub(1).map(|i| MAIN[i].0)
        } else if k.home {
            Some(MAIN[0].0)
        } else if k.end {
            Some(MAIN[n - 1].0)
        } else if let Some(c) = k.typed {
            (1..n)
                .map(|d| MAIN[(m + d) % n])
                .find(|(_, label)| label.chars().next().is_some_and(|f| f.to_ascii_lowercase() == c))
                .map(|(name, _)| name)
        } else {
            None
        };
        if let Some(name) = target {
            self.pending = Some(rid(name));
        }
    }

    /// One frame of the menu. Returns the events it raised.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<MenuEvent> {
        let mut events = Vec::new();
        let ctx = ui.ctx().clone();
        let keys = read_keys(ui);

        // The trigger: the element that opens the menu.
        let tr = outline_button(ui, look, rid("trigger"), parts::TRIGGER, "Actions");
        let keyboard = tr.has_focus() && ui.input(|i| i.key_pressed(Key::Enter) || i.key_pressed(Key::Space));
        if self.control() {
            if tr.clicked() {
                if self.open {
                    self.set_open(false, if keyboard { "escape" } else { "outside" }, &mut events);
                } else {
                    self.open_menu(if keyboard { "keyboard" } else { "pointer" }, keyboard, &mut events);
                }
            } else if tr.has_focus() && keys.down && !self.open {
                self.open_menu("keyboard", true, &mut events);
            }
        } else {
            if tr.has_focus() && keys.shift_f10 && !self.open {
                self.click_point = None;
                self.open_menu("keyboard", true, &mut events);
            }
            if tr.secondary_clicked() && !self.open {
                self.click_point = ui.input(|i| i.pointer.interact_pos());
                self.open_menu("pointer", false, &mut events);
            }
        }

        // A press outside the panel and the submenu closes the menu. The trigger is not outside.
        if self.open {
            let press = ui.input(|i| if i.pointer.primary_pressed() { i.pointer.interact_pos() } else { None });
            if let Some(p) = press {
                let inside = self.panel_rect.is_some_and(|r| r.contains(p))
                    || self.sub_rect.is_some_and(|r| r.contains(p))
                    || tr.rect.contains(p);
                if !inside {
                    self.set_open(false, "outside", &mut events);
                }
            }
        }

        // Keys while the menu is open.
        let focus = ctx.memory(|m| m.focused());
        let cur = focused_name(focus);
        if self.open {
            if keys.tab {
                self.set_open(false, "tab", &mut events);
            } else if keys.escape {
                if self.sub_open && cur == Some("submenuFirst") {
                    self.close_sub(true);
                } else {
                    self.set_open(false, "escape", &mut events);
                    self.pending = Some(rid("trigger"));
                }
            } else if self.sub_open && cur == Some("submenuFirst") && keys.left {
                self.close_sub(true);
            } else if cur == Some("submenu") && keys.right {
                self.open_sub(true);
            } else if let Some(m) = cur.and_then(main_index) {
                self.navigate(m, &keys);
            }
        }

        if self.open {
            self.panel(&ctx, look, tr.rect, &mut events);
        }
        if !self.open {
            self.panel_rect = None;
            self.sub_rect = None;
        }
        if let Some(id) = self.pending.take() {
            ctx.memory_mut(|m| m.request_focus(id));
        }
        events
    }

    /// The main panel, placed below the trigger (control) or at the point (cursor, anchored).
    /// Returns the rect of the submenu row, which the submenu panel sits beside.
    fn panel(&mut self, ctx: &egui::Context, look: Look, trig: Rect, events: &mut Vec<MenuEvent>) -> Option<Rect> {
        let screen = ctx.input(|i| i.content_rect());
        let w = self.width_px(ctx);
        let (px, py) = if self.control() {
            (trig.left(), trig.bottom() + self.p.offset as f32)
        } else {
            let p = self.click_point.unwrap_or(pos2(self.p.pointX as f32, self.p.pointY as f32));
            (p.x, p.y.min(screen.bottom() - INSET - self.size.y).max(INSET))
        };
        let x = px.min(screen.right() - INSET - w).max(INSET);
        let area = egui::Area::new(rid("area")).order(Order::Foreground).fixed_pos(pos2(x, py)).show(ctx, |ui| {
            let frame = egui::Frame::popup(ui.style())
                .fill(look.t.surface_3)
                .stroke(Stroke::new(1.0, look.t.edge_1))
                .corner_radius(CornerRadius::same(10))
                .inner_margin(egui::Margin::symmetric(0, 3));
            let f = frame.show(ui, |ui| {
                ui.set_width(w);
                ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                self.rows(ui, look, w, events)
            });
            let rect = f.response.rect;
            let r = ui.interact(rect, rid("panel"), Sense::hover());
            r.widget_info(|| WidgetInfo::new(Role::GenericContainer));
            part(ui, &r, parts::PANEL);
            (rect, f.inner)
        });
        let (rect, sub_row) = area.inner;
        if rect.size() != self.size {
            self.size = rect.size();
            ctx.request_repaint();
        }
        self.panel_rect = Some(rect);
        self.sub_rect = None;
        if self.sub_open {
            if let Some(row) = sub_row {
                self.sub_panel(ctx, look, row, events);
            }
        }
        sub_row
    }

    /// The submenu panel, beside its row. It flips to the row's left when it would overflow the window.
    fn sub_panel(&mut self, ctx: &egui::Context, look: Look, row: Rect, events: &mut Vec<MenuEvent>) {
        let screen = ctx.input(|i| i.content_rect());
        let sw = natural(ctx, &SUB_ROWS);
        let right = row.right() + 2.0;
        let x = if right + sw > screen.right() - INSET { row.left() - 2.0 - sw } else { right };
        let y = row.top() - rem(0.25);
        let area = egui::Area::new(rid("submenuArea")).order(Order::Foreground).fixed_pos(pos2(x, y)).show(ctx, |ui| {
            let frame = egui::Frame::popup(ui.style())
                .fill(look.t.surface_3)
                .stroke(Stroke::new(1.0, look.t.edge_1))
                .corner_radius(CornerRadius::same(10))
                .inner_margin(egui::Margin::symmetric(0, 3));
            let f = frame.show(ui, |ui| {
                ui.set_width(sw);
                ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                let spec = Spec { name: "submenuFirst", part: parts::SUBMENUFIRST, label: "Inbox", enabled: true, submenu: false, radio: false };
                self.item(ui, look, sw, spec, false, events);
            });
            let rect = f.response.rect;
            let r = ui.interact(rect, rid("submenuPanel"), Sense::hover());
            r.widget_info(|| WidgetInfo::new(Role::GenericContainer));
            part(ui, &r, parts::SUBMENUPANEL);
            rect
        });
        self.sub_rect = Some(area.inner);
    }

    /// The rows of the main panel, in order, with the rule between the two groups. Returns the submenu row's rect.
    fn rows(&mut self, ui: &mut egui::Ui, look: Look, w: f32, events: &mut Vec<MenuEvent>) -> Option<Rect> {
        self.item(ui, look, w, Spec { name: "first", part: parts::FIRST, label: "Open", enabled: true, submenu: false, radio: false }, false, events);
        self.item(ui, look, w, Spec { name: "second", part: parts::SECOND, label: "Rename", enabled: true, submenu: false, radio: false }, false, events);
        self.rule(ui, look, w);
        self.item(ui, look, w, Spec { name: "disabledRow", part: parts::DISABLEDROW, label: "Delete", enabled: false, submenu: false, radio: false }, false, events);
        let sub = self.item(ui, look, w, Spec { name: "submenu", part: parts::SUBMENU, label: "Move to list", enabled: true, submenu: true, radio: false }, false, events);
        let chosen = self.chosen;
        self.item(ui, look, w, Spec { name: "choice", part: parts::CHOICE, label: "Compact", enabled: true, submenu: false, radio: true }, chosen, events);
        self.item(ui, look, w, Spec { name: "last", part: parts::LAST, label: "Trash", enabled: true, submenu: false, radio: false }, false, events);
        Some(sub.rect)
    }

    /// The rule, drawn by the separator widget, and tagged with the menu's separator part.
    fn rule(&mut self, ui: &mut egui::Ui, look: Look, w: f32) {
        let mx = rem(1.429);
        let my = rem(0.857);
        let cursor = ui.cursor();
        let _ = self.rule.ui(ui, look);
        let r = ui.interact(
            Rect::from_min_size(pos2(cursor.left() + mx, cursor.top() + my), vec2(w - 2.0 * mx, 1.0)),
            rid("separator"),
            Sense::hover(),
        );
        r.widget_info(|| WidgetInfo::new(Role::GenericContainer));
        part(ui, &r, parts::SEPARATOR);
    }

    /// One row, with its click and hover handled.
    fn item(&mut self, ui: &mut egui::Ui, look: Look, w: f32, spec: Spec, checked: bool, events: &mut Vec<MenuEvent>) -> Response {
        let r = self.row(ui, look, w, spec, checked);
        if spec.enabled {
            if spec.submenu {
                if r.clicked() {
                    self.open_sub(true);
                } else if r.hovered() {
                    self.open_sub(false);
                }
            } else if r.clicked() {
                if spec.radio {
                    self.chosen = true;
                }
                self.activate(spec.name, events);
            }
        }
        r
    }

    /// Draws a row and tags it with its part. A row that the pointer moves over takes focus.
    fn row(&mut self, ui: &mut egui::Ui, look: Look, w: f32, spec: Spec, checked: bool) -> Response {
        let t = look.t;
        let (rect, _) = ui.allocate_exact_size(vec2(w, rem(2.143)), Sense::hover());
        let sense = if spec.enabled { Sense::click() } else { Sense::hover() };
        let response = ui.interact(rect, rid(spec.name), sense);
        let hovered = spec.enabled && response.hovered();
        let focused = spec.enabled && response.has_focus();
        if hovered && ui.input(|i| i.pointer.delta() != Vec2::ZERO) {
            self.pending = Some(rid(spec.name));
        }
        // The focused row keeps the arrows and Escape for the menu: egui would otherwise move focus spatially on an
        // arrow, or drop it on Escape, before the menu reads the key.
        if focused {
            let filter = egui::EventFilter { tab: false, horizontal_arrows: true, vertical_arrows: true, escape: true };
            ui.memory_mut(|m| m.set_focus_lock_filter(response.id, filter));
        }

        let alpha = if spec.enabled { 1.0 } else { 0.4 };
        let fade = |c: Color32| c.gamma_multiply(alpha);
        let painter = ui.painter().clone();
        let radius = CornerRadius::same(5);
        if hovered || focused {
            painter.rect_filled(rect, radius, t.wash_2);
        }
        if focused && !hovered {
            painter.rect_stroke(rect.expand(2.0), radius, Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
        }

        let cy = rect.center().y;
        let mut x = rect.left() + PAD;
        if spec.radio {
            if checked {
                glyph(&painter, "check", ICON, pos2(x + SLOT / 2.0, cy), fade(t.accent_500));
            }
            x += SLOT + GAP;
        }
        painter.text(pos2(x, cy), Align2::LEFT_CENTER, spec.label, FontId::new(13.0, weight(400)), fade(t.content_1));
        if spec.submenu {
            glyph(&painter, "chevron_right", ICON, pos2(rect.right() - PAD - ICON / 2.0, cy), fade(t.content_2));
        }

        if spec.radio {
            response.widget_info(|| WidgetInfo::selected(Role::CheckBox, spec.enabled, checked, spec.label));
            ui.ctx().accesskit_node_builder(response.id, |node| {
                node.set_role(Role::Button);
                node.set_selected(checked);
            });
        } else {
            response.widget_info(|| WidgetInfo::labeled(Role::Button, spec.enabled, spec.label));
        }
        part(ui, &response, spec.part);
        response
    }
}

/// The menu as the harness mounts it: the trigger and the outside button beside it, at the harness's padding.
pub struct MenuMount {
    menu: Menu,
}

impl crate::Mounted for MenuMount {
    fn frame(&mut self, ui: &mut egui::Ui, look: Look, emit: &mut dyn FnMut(&'static str, serde_json::Value)) {
        let mut events = Vec::new();
        ui.add_space(rem(1.25));
        ui.horizontal(|ui| {
            ui.add_space(rem(1.25));
            ui.spacing_mut().item_spacing.x = rem(0.875);
            events = self.menu.ui(ui, look);
            outline_button(ui, look, rid("outside"), parts::OUTSIDE, "Outside");
        });
        for e in events {
            let (name, fields) = e.to_json();
            emit(name, fields);
        }
    }
}

/// Builds the menu, with its trigger and the outside button, from a test's props.
pub fn mount(props: serde_json::Value) -> Result<Box<dyn crate::Mounted>, String> {
    let p: MenuProps = serde_json::from_value(props).map_err(|e| format!("props: {e}"))?;
    if !["cursor", "control", "submenu", "anchored"].contains(&p.placement.as_str()) {
        return Err(format!("Menu: placement must be one of cursor, control, submenu, anchored, got {}", p.placement));
    }
    if !["content", "task", "tab", "project"].contains(&p.width.as_str()) {
        return Err(format!("Menu: width must be one of content, task, tab, project, got {}", p.width));
    }
    Ok(Box::new(MenuMount { menu: Menu::new(p) }))
}
