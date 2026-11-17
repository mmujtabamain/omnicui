//! contract: omnicui/contracts/trash.contract.yaml (id: app.trash). The Trash list: the trashed tasks with Restore and Delete permanently on each row, Restore all and Empty, and the Empty Trash confirmation; the content of the Trash window.
use crate::App;
use egui::{pos2, vec2, Align, CornerRadius, Frame, Key, Layout, Margin, Rect, Role, ScrollArea, Sense, Stroke, StrokeKind, UiBuilder, WidgetInfo};
use taskscape_ui::omnicui::button::{parts as button_parts, ButtonEvent, ButtonProps};
use taskscape_ui::omnicui::dialog::{parts as dialog_parts, DialogEvent, DialogProps};
use taskscape_ui::omnicui::emptystate::{parts as emptystate_parts, EmptystateProps};
use taskscape_ui::omnicui::heading::{parts as heading_parts, HeadingProps};
use taskscape_ui::omnicui::icon::IconProps;
use taskscape_ui::omnicui::trash::{parts, TrashEvent, TrashProps};
use taskscape_ui::widgets::button::Button;
use taskscape_ui::widgets::dialog::Dialog;
use taskscape_ui::widgets::emptystate::Emptystate;
use taskscape_ui::widgets::heading::Heading;
use taskscape_ui::widgets::icon::Icon;
use taskscape_ui::{as_part, part, rem, text, Look, Mounted};

/// The md icon button's box (24.5px) and the web's 0.5rem gap between the items of a row.
const ICON_BOX: f32 = 24.5;
const GAP: f32 = rem(0.5);
/// A row: 6px padding around a 13px title and an 11px meta line; rows sit 2px apart.
const ROW_H: f32 = 45.0;
const ROW_PAD: f32 = rem(0.429);
const ROW_SPACE: f32 = rem(0.143);
/// The web's fast transition.
const FAST: f32 = 0.18;
const CHANGED_ELSEWHERE: &str = "That changed elsewhere, so the list has been refreshed.";
const EMPTY_FAILED: &str = "Could not empty the trash. The list has been refreshed.";

/// The two demo rows the contract names, found by title; any other row carries no name.
struct Named {
    title: &'static str,
    row: &'static str,
    meta: &'static str,
    restore: &'static str,
    purge: &'static str,
}

const NAMED: [Named; 2] = [
    Named {
        title: "Call the printer about samples",
        row: parts::ROWPRINTER,
        meta: parts::METAPRINTER,
        restore: parts::RESTOREPRINTER,
        purge: parts::PURGEPRINTER,
    },
    Named {
        title: "Print the old flyer",
        row: parts::ROWFLYER,
        meta: parts::METAFLYER,
        restore: parts::RESTOREFLYER,
        purge: parts::PURGEFLYER,
    },
];

/// The Empty Trash confirmation's parts, under the names the contract gives them.
const SHEET_PARTS: [(&str, &str); 5] = [
    (dialog_parts::CARD, parts::EMPTYSHEET),
    (dialog_parts::TITLE, parts::EMPTYSHEETTITLE),
    (dialog_parts::DESCRIPTION, parts::EMPTYSHEETDETAIL),
    (dialog_parts::CANCEL, parts::CANCELEMPTY),
    (dialog_parts::ACCEPT, parts::CONFIRMEMPTY),
];

/// One trashed row as the list draws it.
struct Entry {
    id: String,
    title: String,
    meta: String,
}

/// What a press asks for. Applied after the frame is drawn, so the list shows the change next frame.
enum Action {
    Restore(Vec<String>),
    Purge(Vec<String>),
    OpenEmpty(usize),
}

/// The Trash list's own state: the inline notice, the Empty Trash confirmation while it is open, and
/// the row whose actions had keyboard focus last frame (its actions stay shown).
pub struct Trash {
    notice: String,
    sheet: Option<Dialog>,
    focus_row: Option<String>,
    /// Set when Empty opened the confirmation, with the number of roots it names; the window reads it.
    pub empty_asked: Option<usize>,
    /// Room kept at the header's left for the OS's window buttons (a real window on macOS).
    pub lead: f32,
}

impl Trash {
    pub fn new() -> Self {
        Self { notice: String::new(), sheet: None, focus_row: None, empty_asked: None, lead: 0.0 }
    }

    /// One frame: draws the Trash list and returns the events it raised.
    pub fn show(&mut self, app: &mut App, ui: &mut egui::Ui, look: Look) -> Vec<TrashEvent> {
        let t = look.t;
        let now = now_secs();
        let entries: Vec<Entry> = app
            .data
            .trash
            .iter()
            .map(|task| {
                let list = app.data.list(&task.list_id).map(|l| l.name.clone()).unwrap_or_default();
                Entry {
                    id: task.id.clone(),
                    title: task.title.clone(),
                    meta: format!("{list} · deleted {}", relative(task.deleted_at.unwrap_or(0) as i64, now)),
                }
            })
            .collect();

        let mut actions: Vec<Action> = vec![];
        let mut events: Vec<TrashEvent> = vec![];
        let mut focus_now: Option<String> = None;

        // The pane: its fill, and the node the pane part names. It takes no room in the layout.
        let full = ui.max_rect();
        ui.painter().rect_filled(full, CornerRadius::ZERO, t.surface_1);
        let pane = ui.interact(full, ui.scope_id().with("trash-pane"), Sense::hover());
        pane.widget_info(|| WidgetInfo::new(Role::GenericContainer));
        part(ui, &pane, parts::PANE);

        ui.spacing_mut().item_spacing.y = 0.0;
        header(ui, look, self.lead);
        if !self.notice.is_empty() {
            notice_bar(ui, look, &self.notice);
        }
        if !entries.is_empty() {
            let ids: Vec<String> = entries.iter().map(|e| e.id.clone()).collect();
            strip(ui, look, &ids, &mut actions);
        }

        ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            Frame::new().inner_margin(Margin::same(8)).show(ui, |ui| {
                if entries.is_empty() {
                    as_part(ui, &[(emptystate_parts::TITLE, parts::EMPTYSTATE)], |ui| {
                        Emptystate::new(EmptystateProps {
                            title: "Trash is empty".into(),
                            icon: "delete".into(),
                            iconSize: 28.0,
                            ..Default::default()
                        })
                        .ui(ui, look)
                    });
                } else {
                    let list = ui.vertical(|ui| {
                        for (i, e) in entries.iter().enumerate() {
                            if i > 0 {
                                ui.add_space(ROW_SPACE);
                            }
                            if self.row(ui, look, e, i == 0, &mut actions) {
                                focus_now = Some(e.id.clone());
                            }
                        }
                    });
                    let node = ui.interact(list.response.rect, ui.scope_id().with("trash-list"), Sense::hover());
                    node.widget_info(|| WidgetInfo::new(Role::GenericContainer));
                    part(ui, &node, parts::LIST);
                }
            });
        });

        // The Empty Trash confirmation, while it is open: over the window, with no scrim.
        let mut sheet_events: Vec<DialogEvent> = vec![];
        if let Some(sheet) = self.sheet.as_mut() {
            sheet_events = as_part(ui, &SHEET_PARTS, |ui| sheet.ui(ui, look));
        }
        let mut accepted = false;
        let mut close_sheet = false;
        for e in sheet_events {
            match e {
                DialogEvent::Accepted { .. } => {
                    accepted = true;
                    close_sheet = true;
                }
                DialogEvent::OpenChanged { .. } => close_sheet = true,
                _ => {}
            }
        }
        if accepted {
            self.notice.clear();
            let count = app.data.trash.len();
            match app.data.empty_trash() {
                Ok(()) => events.push(TrashEvent::Purged { count: count as i64 }),
                Err(_) => self.notice = EMPTY_FAILED.to_owned(),
            }
        }
        if close_sheet {
            self.sheet = None;
        }

        let acted = !actions.is_empty() || accepted || close_sheet;
        for action in actions {
            self.notice.clear();
            match action {
                Action::Restore(ids) => match app.data.restore_tasks(&ids) {
                    Ok(()) => events.push(TrashEvent::Restored { count: ids.len() as i64 }),
                    Err(_) => self.notice = CHANGED_ELSEWHERE.to_owned(),
                },
                Action::Purge(ids) => match app.data.purge_tasks(&ids) {
                    Ok(()) => events.push(TrashEvent::Purged { count: ids.len() as i64 }),
                    Err(_) => self.notice = CHANGED_ELSEWHERE.to_owned(),
                },
                Action::OpenEmpty(count) => {
                    self.sheet = Some(empty_sheet(count));
                    self.empty_asked = Some(count);
                }
            }
        }

        if acted || focus_now != self.focus_row {
            ui.ctx().request_repaint();
        }
        self.focus_row = focus_now;
        events
    }

    /// One trashed row: the title and its meta line, and the Restore and Delete permanently icon
    /// buttons. The buttons show while the row is hovered or one of them has keyboard focus; they
    /// are laid out either way, so they can take focus. Returns whether one of them has focus.
    fn row(&self, ui: &mut egui::Ui, look: Look, e: &Entry, first: bool, actions: &mut Vec<Action>) -> bool {
        let t = look.t;
        let named = NAMED.iter().find(|n| n.title == e.title);
        let (rect, row) = ui.allocate_exact_size(vec2(ui.available_width(), ROW_H), Sense::hover());
        let hovered = ui.rect_contains_pointer(rect);
        let wash = ui.ctx().animate_bool_with_time(row.id.with("wash"), hovered, FAST);
        ui.painter().rect_filled(rect, CornerRadius::same(4), t.wash_1.gamma_multiply(wash));
        if !first {
            ui.painter().hline(rect.x_range(), rect.top(), Stroke::new(1.0, t.edge_1));
        }

        // The row's node carries its name; the first row also carries the positional part on a second node.
        let label = format!("{}, {}", e.title, e.meta);
        row.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, &label));
        if first {
            let positional = ui.interact(rect, row.id.with("first"), Sense::hover());
            positional.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, &label));
            part(ui, &positional, parts::ROW);
        }
        if let Some(n) = named {
            part(ui, &row, n.row);
        }

        // The text column takes the left of the row; the two icon buttons sit at its right end.
        let inner = rect.shrink(ROW_PAD);
        let actions_w = 2.0 * ICON_BOX + GAP;
        let col = Rect::from_min_max(inner.min, pos2(inner.right() - actions_w - GAP, inner.bottom()));
        let buttons = Rect::from_min_max(pos2(inner.right() - actions_w, inner.top()), inner.max);
        ui.scope_builder(UiBuilder::new().max_rect(col).layout(Layout::top_down(Align::Min)), |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            ui.add(egui::Label::new(text(e.title.clone(), 13.0, 400).color(t.content_1)).truncate());
            let meta = ui.add(egui::Label::new(text(e.meta.clone(), 11.0, 400).color(t.content_3)).truncate());
            if let Some(n) = named {
                part(ui, &meta, n.meta);
            }
        });

        let show = hovered || self.focus_row.as_deref() == Some(e.id.as_str());
        let (restore, purge, has_focus) = ui
            .scope_builder(UiBuilder::new().max_rect(buttons).layout(Layout::left_to_right(Align::Center)), |ui| {
                ui.spacing_mut().item_spacing.x = GAP;
                // On the first row the positional part names the button; a named row's own part goes on a
                // second node over the same box when the button already carries the position.
                let restore_part = if first { Some(parts::RESTOREITEM) } else { named.map(|n| n.restore) };
                let restore_named = if first { named.map(|n| n.restore) } else { None };
                let purge_part = if first { Some(parts::PURGEITEM) } else { named.map(|n| n.purge) };
                let purge_named = if first { named.map(|n| n.purge) } else { None };
                let (restore, restore_focus) = row_action(
                    ui,
                    look,
                    show,
                    restore_part,
                    restore_named,
                    icon_button("restore", "Restore", "ghost", "muted"),
                    "Restore",
                );
                let (purge, purge_focus) = row_action(
                    ui,
                    look,
                    show,
                    purge_part,
                    purge_named,
                    icon_button("delete_forever", "Delete permanently", "danger", ""),
                    "Delete permanently",
                );
                (restore, purge, restore_focus || purge_focus)
            })
            .inner;

        if restore {
            actions.push(Action::Restore(vec![e.id.clone()]));
        }
        // Enter never runs a destructive action (decision 10): Delete permanently takes a click or Space.
        if purge && !ui.input(|i| i.key_pressed(Key::Enter)) {
            actions.push(Action::Purge(vec![e.id.clone()]));
        }
        has_focus
    }
}

/// One row action. The icon button is always laid out and in the tree, so it can take focus, and it is
/// covered while its row is not shown (the row is not hovered and no action of it has focus). `part_id`
/// names the button. `named` names a second node over the same box, a press target that also takes
/// Enter and Space, for a named row whose button already carries the first-row position. Returns
/// whether the action was pressed this frame and whether it has keyboard focus.
fn row_action(
    ui: &mut egui::Ui,
    look: Look,
    show: bool,
    part_id: Option<&'static str>,
    named: Option<&'static str>,
    props: ButtonProps,
    name: &str,
) -> (bool, bool) {
    let t = look.t;
    let inner = ui.scope(|ui| {
        let id = ui.next_auto_id();
        let focused = ui.memory(|m| m.has_focus(id));
        let events = match part_id {
            Some(p) => as_part(ui, &[(button_parts::BUTTON, p)], |ui| Button::new(props).ui(ui, look)),
            None => Button::new(props).ui(ui, look),
        };
        (pressed(&events), focused)
    });
    let (mut pressed_now, mut focused) = inner.inner;
    let rect = inner.response.rect;
    if !show {
        // Not drawn while the row is not shown: the box is laid over with the pane's fill.
        ui.painter().rect_filled(rect, CornerRadius::same(5), t.surface_1);
    }
    if let Some(p) = named {
        let node = ui.interact(rect, ui.scope_id().with(p), Sense::click());
        node.widget_info(|| WidgetInfo::labeled(Role::Button, true, name));
        part(ui, &node, p);
        let key = node.has_focus() && ui.input(|i| i.key_pressed(Key::Enter) || i.key_pressed(Key::Space));
        if node.clicked() || key {
            pressed_now = true;
        }
        if node.has_focus() {
            focused = true;
            ui.painter().rect_stroke(rect, CornerRadius::same(4), Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
        }
    }
    (pressed_now, focused)
}

/// The props of a row's icon button: the web's md icon button, named for a screen reader.
fn icon_button(icon: &str, name: &str, variant: &str, tone: &str) -> ButtonProps {
    ButtonProps {
        icon: icon.into(),
        iconSize: 16,
        shape: "icon".into(),
        size: "md".into(),
        width: "content".into(),
        variant: variant.into(),
        tone: tone.into(),
        ariaLabel: name.into(),
        description: name.into(),
        ..Default::default()
    }
}

fn pressed(events: &[ButtonEvent]) -> bool {
    events.iter().any(|e| matches!(e, ButtonEvent::Activated { .. }))
}

/// The header: the delete glyph and the "Trash" title, on a hairline.
fn header(ui: &mut egui::Ui, look: Look, lead: f32) {
    let t = look.t;
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), rem(3.286)), Sense::hover());
    ui.painter().hline(rect.x_range(), rect.bottom(), Stroke::new(1.0, t.edge_1));
    let inner = Rect::from_min_max(rect.min + vec2(rem(1.0) + lead, 0.0), rect.max);
    ui.scope_builder(UiBuilder::new().max_rect(inner).layout(Layout::left_to_right(Align::Center)), |ui| {
        ui.spacing_mut().item_spacing.x = rem(0.357);
        Icon::new(IconProps { name: "delete".into(), size: "lg".into(), tone: "secondary".into(), ..Default::default() })
            .ui(ui, look);
        as_part(ui, &[(heading_parts::TITLE, parts::TITLE)], |ui| {
            Heading::new(HeadingProps { content: "Trash".into(), r#as: "h2".into(), size: "sm".into(), ..Default::default() })
                .ui(ui, look)
        });
    });
}

/// The inline notice under the header, after a restore or purge that the list could not make.
fn notice_bar(ui: &mut egui::Ui, look: Look, notice: &str) {
    let t = look.t;
    let r = Frame::new().inner_margin(Margin::symmetric(9, 5)).show(ui, |ui| {
        let label = ui.label(text(notice.to_owned(), 12.0, 400).color(t.danger_500));
        label.widget_info(|| WidgetInfo::labeled(Role::Status, true, notice));
        part(ui, &label, parts::NOTICE);
    });
    ui.painter().hline(r.response.rect.x_range(), r.response.rect.bottom(), Stroke::new(1.0, t.edge_1));
}

/// The action strip: Restore all, and Empty at its right end (which asks first).
fn strip(ui: &mut egui::Ui, look: Look, ids: &[String], actions: &mut Vec<Action>) {
    let t = look.t;
    let r = Frame::new().inner_margin(Margin::symmetric(9, 7)).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = rem(0.429);
            let restore_all = ButtonProps {
                label: "Restore all".into(),
                icon: "restore".into(),
                iconSize: 14,
                shape: "text".into(),
                size: "md".into(),
                width: "content".into(),
                variant: "ghost".into(),
                tone: "muted".into(),
                ..Default::default()
            };
            let events = as_part(ui, &[(button_parts::BUTTON, parts::RESTOREALL)], |ui| Button::new(restore_all).ui(ui, look));
            if pressed(&events) {
                actions.push(Action::Restore(ids.to_vec()));
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let empty = ButtonProps {
                    label: "Empty".into(),
                    icon: "delete_forever".into(),
                    iconSize: 14,
                    shape: "text".into(),
                    size: "md".into(),
                    width: "content".into(),
                    variant: "danger".into(),
                    ..Default::default()
                };
                let events = as_part(ui, &[(button_parts::BUTTON, parts::EMPTYTRASH)], |ui| Button::new(empty).ui(ui, look));
                if pressed(&events) {
                    actions.push(Action::OpenEmpty(ids.len()));
                }
            });
        });
    });
    let strip_rect = r.response.rect;
    ui.painter().hline(strip_rect.x_range(), strip_rect.bottom(), Stroke::new(1.0, t.edge_1));
    let node = ui.interact(strip_rect, ui.scope_id().with("trash-strip"), Sense::hover());
    node.widget_info(|| WidgetInfo::new(Role::GenericContainer));
    part(ui, &node, parts::ACTIONSTRIP);
}

/// The Empty Trash confirmation: the dialog card with Cancel first (it takes focus on open) and
/// "Empty Trash" as the destructive accept. Enter does not confirm; Space or a press does; Escape cancels.
fn empty_sheet(count: usize) -> Dialog {
    let noun = if count == 1 { "task" } else { "tasks" };
    Dialog::new(DialogProps {
        open: true,
        kind: "confirm".into(),
        tone: "destructive".into(),
        title: "Empty Trash?".into(),
        description: format!("{count} {noun} deleted for good. No undo."),
        cancelLabel: "Cancel".into(),
        acceptLabel: "Empty Trash".into(),
        ..Default::default()
    })
}

/// The short relative time of a row's meta line: "just now", "30m ago", "3h ago", "5d ago". The stored
/// time is unix seconds or milliseconds; a value past year 5000 in seconds can only be milliseconds.
fn relative(deleted: i64, now: i64) -> String {
    let deleted = if deleted > 100_000_000_000 { deleted / 1000 } else { deleted };
    let minutes = (now - deleted).max(0) / 60;
    if minutes < 1 {
        "just now".to_owned()
    } else if minutes < 60 {
        format!("{minutes}m ago")
    } else if minutes < 24 * 60 {
        format!("{}h ago", minutes / 60)
    } else {
        format!("{}d ago", minutes / (24 * 60))
    }
}

fn now_secs() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

/// The test's mount: the app on its data set, with the Trash list alone.
struct Mount {
    app: App,
    screen: Trash,
}

impl Mounted for Mount {
    fn frame(&mut self, ui: &mut egui::Ui, look: Look, emit: &mut dyn FnMut(&'static str, serde_json::Value)) {
        for e in self.screen.show(&mut self.app, ui, look) {
            let (n, f) = e.to_json();
            emit(n, f);
        }
    }
}

pub fn mount(app: App, props: serde_json::Value) -> Result<Box<dyn Mounted>, String> {
    let _: TrashProps = serde_json::from_value(props).map_err(|e| e.to_string())?;
    Ok(Box::new(Mount { app, screen: Trash::new() }))
}
