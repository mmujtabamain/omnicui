//! contract: omnicui/contracts/commandpalette.contract.yaml (id: ui.commandpalette). The command palette: a 490 px card near the top of the window with a search input that keeps focus, a listbox of command rows, and a status message when no row matches.
use crate::omnicui::commandinput::{parts as input_parts, CommandinputEvent};
use crate::omnicui::commandpalette::{parts, CommandpaletteEvent, CommandpaletteProps};
use crate::omnicui::comboboxitem::{parts as row_parts, ComboboxitemEvent};
use crate::widgets::commandinput::CommandInput;
use crate::widgets::comboboxitem::ComboboxItem;
use crate::{as_part, part, rem, weight, Look};
use egui::{pos2, vec2, Align2, Area, Color32, CornerRadius, FontId, Frame, Id, Key, Modifiers, Order, Rect, Role, ScrollArea, Sense, Shadow, Stroke, Vec2, WidgetInfo};

/// The card is 35rem wide (490 px), at most 60vh tall, and its top edge sits at 6.5rem (91 px).
const CARD_W: f32 = rem(35.0);
const CARD_TOP: f32 = rem(6.5);
/// The search input's height (3.143rem), and the list's vertical padding (py-2).
const INPUT_H: f32 = rem(3.143);
const LIST_PAD: f32 = 8.0;
/// The status message: 1.6rem of padding above and below, and a 1.393rem line of 0.929rem text.
const EMPTY_PAD_Y: f32 = rem(1.6);
const EMPTY_LINE: f32 = rem(1.393);
const EMPTY_FONT: f32 = rem(0.929);
const EMPTY_MESSAGE: &str = "No matching commands";

/// One command: the row's id, the label the filter matches, its icon and its shortcut.
#[derive(Clone)]
struct Command {
    id: String,
    label: String,
    icon: &'static str,
    shortcut: &'static str,
}

fn command(id: &str, label: &str, icon: &'static str, shortcut: &'static str) -> Command {
    Command { id: id.to_owned(), label: label.to_owned(), icon, shortcut }
}

/// The harness's command sets, standing in for getCommands: "all" is the web harness's 13 commands,
/// "many" is 40 plain rows, more than the card shows.
fn fixture(set: &str) -> Vec<Command> {
    if set == "many" {
        return (1..=40).map(|i| command(&format!("list-{i}"), &format!("Go to list {i}"), "list", "")).collect();
    }
    vec![
        command("create-task", "Create task", "add", "Mod+N"),
        command("create-project", "Create project", "create_new_folder", ""),
        command("go-today", "Go to today", "today", ""),
        command("go-inbox", "Go to inbox", "inbox", ""),
        command("go-projects", "Go to projects", "folder", ""),
        command("go-settings", "Go to settings", "settings", ""),
        command("switch-theme", "Switch theme", "dark_mode", ""),
        command("toggle-sidebar", "Toggle sidebar", "left_panel_close", ""),
        command("sign-out", "Sign out", "logout", ""),
        command("open-trash", "Open Trash", "delete", ""),
        command("empty-trash", "Empty Trash", "delete_forever", "Mod+Shift+Backspace"),
        command("restore-trash", "Restore from Trash", "restore_from_trash", "Mod+Shift+R"),
        command("move-trash", "Move project to Trash", "delete", "Mod+Delete"),
    ]
}

/// The part a row carries: the first three rows and the last have their own names, the rest keep the row's.
fn row_renames(i: usize, len: usize) -> Vec<(&'static str, &'static str)> {
    let slot = match i {
        0 => parts::FIRSTROW,
        1 => parts::SECONDROW,
        2 => parts::THIRDROW,
        _ if i + 1 == len => parts::LASTROW,
        _ => return Vec::new(),
    };
    vec![(row_parts::ROW, slot)]
}

/// The palette. It holds the filter text and the highlight, and the search input that edits the text.
pub struct CommandPalette {
    p: CommandpaletteProps,
    commands: Vec<Command>,
    /// The filter text, as the input last reported it.
    text: String,
    /// The highlight, an index into the rows the filter leaves (clamped to them when drawn).
    active: usize,
    /// The search input's widget id, learned on the first frame (the input takes focus there) and kept focused.
    search_id: Option<Id>,
    /// The (highlight, row count) the list last scrolled for, so a scroll away from the highlight is kept.
    scrolled_for: Option<(usize, usize)>,
    /// The row under the pointer last frame.
    hover_row: Option<usize>,
}

impl CommandPalette {
    pub fn new(p: CommandpaletteProps) -> Self {
        let commands = fixture(&p.commandSet);
        Self {
            text: p.query.clone(),
            active: p.activeIndex.max(0) as usize,
            commands,
            search_id: None,
            scrolled_for: None,
            hover_row: None,
            p,
        }
    }

    /// One frame: draws the card (when open) and returns the events it raised in this frame.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<CommandpaletteEvent> {
        let mut events = vec![];
        if !self.p.open {
            return events;
        }
        let ctx = ui.ctx().clone();
        // Escape closes the palette, and no row sees it.
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
            self.close("escape", &mut events);
            ctx.request_repaint();
            return events;
        }

        let screen = ctx.content_rect();
        let width = CARD_W.min(screen.width());
        let max_h = screen.height() * 0.6;
        let aria = self.p.ariaLabel.clone();
        let window = self.p.presentation == "window";
        let area = Area::new(ui.scope_id().with("commandpalette.card")).order(Order::Foreground).movable(false);
        let area = if window {
            area.anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
        } else {
            area.anchor(Align2::CENTER_TOP, vec2(0.0, CARD_TOP))
        };
        let _ = area.show(&ctx, |ui| {
            let card = Frame::new()
                .fill(look.t.surface_2)
                .stroke(Stroke::new(1.0, look.t.edge_1))
                .corner_radius(CornerRadius::same(10))
                .shadow(Shadow { offset: [0, 4], blur: 16, spread: 0, color: Color32::from_black_alpha(64) });
            let out = card.show(ui, |ui| {
                // The frame's 1 px stroke takes a pixel from each side of the 490 px card.
                ui.set_width(width - 2.0);
                ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                self.body(ui, look, max_h - 2.0, &mut events);
            });
            let node = ui.interact(out.response.rect, ui.scope_id().with("panel"), Sense::hover());
            node.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, &aria));
            part(ui, &node, parts::PANEL);
        });
        if !events.is_empty() {
            ctx.request_repaint();
        }
        events
    }

    /// The search input, built for this frame from the filter text. Its autoFocus takes the focus on every
    /// frame, so the palette holds it through the first frame of the area (whose widgets are not focusable
    /// until the frame after) and after Tab or a press on a row.
    fn search_input(&self) -> CommandInput {
        CommandInput::new(
            serde_json::from_value(serde_json::json!({
                "value": self.text.clone(),
                "placeholder": "Type a command or search",
                "ariaLabel": "Search commands",
                "leading": "icon",
                "autoFocus": true,
            }))
            .expect("commandinput props"),
        )
    }

    /// The card's contents: the search input, then the rows, or the status message when none match.
    fn body(&mut self, ui: &mut egui::Ui, look: Look, max_h: f32, events: &mut Vec<CommandpaletteEvent>) {
        // The input takes the arrows and Enter it reads, and the typed text.
        let mut input = self.search_input();
        let input_events = as_part(ui, &[(input_parts::INPUT, parts::SEARCH)], |ui| input.ui(ui, look));
        if self.search_id.is_none() {
            self.search_id = ui.memory(|m| m.focused());
        }
        for event in input_events {
            match event {
                CommandinputEvent::QueryChanged { value } => self.change_text(value, events),
                CommandinputEvent::MoveActive { direction } => self.step(direction == "down", events),
                CommandinputEvent::Submit { .. } => {
                    let len = self.matches().len();
                    let cur = self.current(len);
                    self.run(cur, events);
                }
            }
            if !self.p.open {
                return;
            }
        }

        let rows = self.matches();
        let cur = self.current(rows.len());
        if rows.is_empty() {
            self.empty(ui, look);
        } else {
            self.list(ui, look, &rows, cur, max_h - INPUT_H, events);
        }

        // Focus stays in the search: Tab and a click on a row can move it, and it comes back here.
        if let Some(id) = self.search_id {
            if !ui.memory(|m| m.has_focus(id)) {
                ui.memory_mut(|m| m.request_focus(id));
            }
        }
    }

    /// The status message that stands where the rows would be.
    fn empty(&self, ui: &mut egui::Ui, look: Look) {
        let width = ui.available_width();
        let galley = ui.painter().layout_no_wrap(EMPTY_MESSAGE.to_owned(), FontId::new(EMPTY_FONT, weight(400)), look.t.content_2);
        let height = 2.0 * EMPTY_PAD_Y + EMPTY_LINE;
        let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
        response.widget_info(|| WidgetInfo::labeled(Role::Status, true, EMPTY_MESSAGE));
        part(ui, &response, parts::EMPTYMESSAGE);
        let pos = rect.center() - galley.size() * 0.5;
        ui.painter().galley(pos, galley, look.t.content_2);
    }

    /// The listbox: one row per match, in a scroll region at most `max_list` tall that keeps the highlight in view.
    fn list(&mut self, ui: &mut egui::Ui, look: Look, rows: &[usize], cur: usize, max_list: f32, events: &mut Vec<CommandpaletteEvent>) {
        let len = rows.len();
        let items: Vec<Command> = rows.iter().map(|&i| self.commands[i].clone()).collect();
        let key = (cur, len);
        let scroll = self.scrolled_for != Some(key);
        self.scrolled_for = Some(key);
        let prev_hover = self.hover_row;
        let pointer = ui.input(|i| i.pointer.hover_pos());
        let moved = ui.input(|i| i.pointer.delta() != Vec2::ZERO);
        let search = self.search_id.unwrap_or(Id::NULL);
        let mut hovered = None;

        let out = ScrollArea::vertical()
            .id_salt("list")
            .max_height(max_list)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::ZERO;
                ui.add_space(LIST_PAD);
                for (i, cmd) in items.iter().enumerate() {
                    let top = ui.cursor().top();
                    let json = serde_json::json!({
                        "id": cmd.id,
                        "label": cmd.label,
                        "value": cmd.label,
                        "icon": cmd.icon,
                        "shortcut": cmd.shortcut,
                        "context": "palette",
                        "active": i == cur,
                    });
                    let mut row = ComboboxItem::new(serde_json::from_value(json).expect("comboboxitem props"));
                    let renames = row_renames(i, len);
                    let row_events = as_part(ui, &renames, |ui| row.ui(ui, look, search));
                    let bounds = ui.max_rect();
                    let rect = Rect::from_min_max(pos2(bounds.left(), top), pos2(bounds.right(), ui.cursor().top()));
                    if pointer.is_some_and(|p| rect.contains(p)) {
                        hovered = Some(i);
                    }
                    for event in row_events {
                        match event {
                            // A row reports the pointer resting on it too; only a move, or entering the row, counts.
                            ComboboxitemEvent::Highlighted { .. } => {
                                if moved || prev_hover != Some(i) {
                                    self.highlight(i, cur, events);
                                }
                            }
                            ComboboxitemEvent::Picked { .. } => self.run(i, events),
                        }
                    }
                    if scroll && i == cur {
                        ui.scroll_to_rect(rect, None);
                    }
                }
                ui.add_space(LIST_PAD);
            });

        self.hover_row = hovered;
        let list = ui.interact(out.inner_rect, ui.scope_id().with("list-node"), Sense::hover());
        list.widget_info(|| WidgetInfo::new(Role::GenericContainer));
        part(ui, &list, parts::LIST);
    }

    /// The rows the filter leaves, as indices into the commands: a case-insensitive match on the label.
    fn matches(&self) -> Vec<usize> {
        let needle = self.text.trim().to_lowercase();
        self.commands
            .iter()
            .enumerate()
            .filter(|(_, c)| needle.is_empty() || c.label.to_lowercase().contains(needle.as_str()))
            .map(|(i, _)| i)
            .collect()
    }

    /// The highlight as drawn: the stored index, clamped to the rows.
    fn current(&self, len: usize) -> usize {
        if len == 0 {
            0
        } else {
            self.active.min(len - 1)
        }
    }

    /// Typed text: the new filter highlights its first match.
    fn change_text(&mut self, value: String, events: &mut Vec<CommandpaletteEvent>) {
        let cur = self.current(self.matches().len());
        self.text = value;
        if cur != 0 {
            events.push(CommandpaletteEvent::ActiveChange { index: 0 });
        }
        self.active = 0;
    }

    /// Arrow keys: one row down or up, stopping at the ends.
    fn step(&mut self, down: bool, events: &mut Vec<CommandpaletteEvent>) {
        let len = self.matches().len();
        let cur = self.current(len);
        let next = if down { (cur + 1).min(len.saturating_sub(1)) } else { cur.saturating_sub(1) };
        self.highlight(next, cur, events);
    }

    /// Moves the highlight to `next`, reporting it when it changes.
    fn highlight(&mut self, next: usize, cur: usize, events: &mut Vec<CommandpaletteEvent>) {
        if next != cur {
            self.active = next;
            events.push(CommandpaletteEvent::ActiveChange { index: next as i64 });
        }
    }

    /// Runs the row at `index`, then the palette closes.
    fn run(&mut self, index: usize, events: &mut Vec<CommandpaletteEvent>) {
        if index >= self.matches().len() {
            return;
        }
        events.push(CommandpaletteEvent::Select { index: index as i64 });
        self.close("select", events);
    }

    fn close(&mut self, reason: &str, events: &mut Vec<CommandpaletteEvent>) {
        self.p.open = false;
        events.push(CommandpaletteEvent::Close { reason: reason.to_owned() });
    }
}

crate::mounted!(CommandPalette, CommandpaletteProps);
