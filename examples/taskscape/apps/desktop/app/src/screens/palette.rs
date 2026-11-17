//! contract: omnicui/contracts/palette.contract.yaml (id: app.palette). The command palette: a floating panel inside the main window with no backdrop (decision 22), a search input over the commands and the jump rows, grouped under View, Task, Manage, Go to list and Go to project.
use crate::app::Sheet;
use crate::state::Aside;
use crate::App;
use egui::{pos2, vec2, Align2, Area, Color32, CornerRadius, FontId, Frame, Id, Key, Modifiers, Order, Rect, Role, ScrollArea, Sense, Shadow, Stroke, Vec2, WidgetInfo};
use taskscape_ui::omnicui::commandinput::{parts as input_parts, CommandinputEvent};
use taskscape_ui::omnicui::comboboxitem::{parts as row_parts, ComboboxitemEvent};
use taskscape_ui::omnicui::palette::{parts, PaletteEvent, PaletteProps};
use taskscape_ui::widgets::commandinput::CommandInput;
use taskscape_ui::widgets::comboboxitem::ComboboxItem;
use taskscape_ui::{as_part, part, rem, weight, Look, Mounted};

/// The card is 35rem wide (490 px), its top edge sits at 6.5rem (91 px), and it is at most 60% of the window tall.
const CARD_W: f32 = rem(35.0);
const CARD_TOP: f32 = rem(6.5);
/// The search input's height (3.143rem), and the list's top padding (py-2).
const INPUT_H: f32 = rem(3.143);
const LIST_PAD: f32 = 8.0;
/// A group heading: 3.5 px above it (mt-space-3), 8.75 px at its sides (px-space-6), an 11 px semibold label on a 16 px line.
const GROUP_MT: f32 = 3.5;
const GROUP_PX: f32 = 8.75;
const GROUP_FONT: f32 = rem(0.786);
const GROUP_LINE: f32 = rem(1.143);
/// Rows are 32 px apart. A row with no shortcut is 30 px tall, so it is followed by 2 px.
const ROW_GAP: f32 = 2.0;
/// The status message that stands where the rows would be (as the widget's: 1.6rem padding, a 1.393rem line of 0.929rem text).
const EMPTY_PAD_Y: f32 = rem(1.6);
const EMPTY_LINE: f32 = rem(1.393);
const EMPTY_FONT: f32 = rem(0.929);
const EMPTY_MESSAGE: &str = "No matching commands";
const PLACEHOLDER: &str = "Run a command or jump to…";

/// The five groups, in the order the palette lists them. Each heading is the group's label.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Group {
    View,
    Task,
    Manage,
    GoToList,
    GoToProject,
}

impl Group {
    fn label(self) -> &'static str {
        match self {
            Group::View => "View",
            Group::Task => "Task",
            Group::Manage => "Manage",
            Group::GoToList => "Go to list",
            Group::GoToProject => "Go to project",
        }
    }

    fn part(self) -> &'static str {
        match self {
            Group::View => parts::GROUPVIEW,
            Group::Task => parts::GROUPTASK,
            Group::Manage => parts::GROUPMANAGE,
            Group::GoToList => parts::GROUPGOLIST,
            Group::GoToProject => parts::GROUPGOPROJECT,
        }
    }
}

/// What a row does when it runs.
#[derive(Clone, Debug, PartialEq)]
enum Action {
    TogglePreview,
    ToggleSplit,
    SwapPanes,
    ShowCompleted,
    NewTask,
    Undo,
    NewList,
    NewProject,
    OpenTrash,
    Settings,
    /// Move to Trash, asked first (decision 10).
    DeleteTask(String),
    OpenList(String),
    OpenProject(String),
    /// Rows the web leaves as no-ops (Collapse all, Expand all, Search this list, Redo).
    Noop,
}

/// One row: its group, the visible label the filter matches and a run reports, its icon, its shortcut
/// (empty for none), the contract part it carries (when it has its own) and what it does.
#[derive(Clone, Debug)]
struct Row {
    group: Group,
    label: String,
    icon: &'static str,
    accel: &'static str,
    slot: Option<&'static str>,
    action: Action,
}

fn cmd(group: Group, label: impl Into<String>, icon: &'static str, accel: &'static str, slot: Option<&'static str>, action: Action) -> Row {
    Row { group, label: label.into(), icon, accel, slot, action }
}

/// The rows the app offers now: the commands, then the Delete row for the one selected task, then the
/// lists of the open project, then the projects. The web's order and labels.
fn catalog(app: &App) -> Vec<Row> {
    use Group::*;
    let mut rows = vec![
        cmd(View, "Toggle preview", "right_panel_open", "Mod+\\", None, Action::TogglePreview),
        cmd(View, "Toggle split view", "vertical_split", "Mod+Alt+S", None, Action::ToggleSplit),
        cmd(View, "Swap panes", "swap_horiz", "", Some(parts::ROWSWAPPANES), Action::SwapPanes),
        cmd(View, "Show / hide completed", "visibility", "Mod+Shift+H", None, Action::ShowCompleted),
        cmd(View, "Collapse all", "unfold_less", "Mod+Alt+-", None, Action::Noop),
        cmd(View, "Expand all", "unfold_more", "Mod+Alt+=", None, Action::Noop),
        cmd(Task, "New task", "add", "Mod+N", None, Action::NewTask),
        cmd(Task, "Search this list", "search", "Mod+F", None, Action::Noop),
        cmd(Task, "Undo", "undo", "Mod+Z", None, Action::Undo),
        cmd(Task, "Redo", "redo", "Mod+Shift+Z", None, Action::Noop),
        cmd(Manage, "New list", "playlist_add", "Mod+Shift+N", None, Action::NewList),
        cmd(Manage, "New project", "create_new_folder", "", None, Action::NewProject),
        cmd(Manage, "Open Trash", "delete", "", None, Action::OpenTrash),
        cmd(Manage, "Settings", "settings", "Mod+Comma", None, Action::Settings),
    ];

    // The Delete row: only for the one selected task of the focused list.
    if let Some(list) = app.active_list() {
        if app.ui.selection.len() == 1 {
            let selected = &app.ui.selection[0];
            if let Some(task) = app.data.tasks_of(&list).into_iter().find(|t| &t.id == selected) {
                rows.push(cmd(
                    Task,
                    format!("Delete “{}”", task.title),
                    "delete",
                    "",
                    Some(parts::ROWDELETE),
                    Action::DeleteTask(task.id.clone()),
                ));
            }
        }
    }

    if let Some(project) = app.ui.project.as_deref() {
        for list in app.data.lists_of(project) {
            rows.push(cmd(GoToList, list.name.clone(), "list", "", None, Action::OpenList(list.id.clone())));
        }
    }
    for project in &app.data.projects {
        let slot = (project.name == "Spring launch").then_some(parts::ROWSPRINGLAUNCH);
        rows.push(cmd(GoToProject, project.name.clone(), "folder", "", slot, Action::OpenProject(project.id.clone())));
    }
    rows
}

/// The rows whose label contains the query, ignoring case and the query's ends (the web's filter).
fn filter(rows: Vec<Row>, query: &str) -> Vec<Row> {
    let needle = query.trim().to_lowercase();
    rows.into_iter().filter(|r| needle.is_empty() || r.label.to_lowercase().contains(&needle)).collect()
}

/// The highlight as drawn: the stored index, clamped to the rows.
fn current(active: usize, len: usize) -> usize {
    if len == 0 {
        0
    } else {
        active.min(len - 1)
    }
}

/// The part a row carries: its own part, else the first row is the contract's "row".
fn row_renames(row: &Row, index: usize) -> Vec<(&'static str, &'static str)> {
    match row.slot.or((index == 0).then_some(parts::ROW)) {
        Some(slot) => vec![(row_parts::ROW, slot)],
        None => Vec::new(),
    }
}

/// Runs a row's command. The palette closes first, then the command runs (the web's close, then run).
fn run(app: &mut App, row: &Row, events: &mut Vec<PaletteEvent>) {
    app.ui.palette_open = false;
    perform(app, &row.action);
    events.push(PaletteEvent::Ran { label: row.label.clone() });
}

fn perform(app: &mut App, action: &Action) {
    match action {
        Action::TogglePreview => {
            app.ui.aside = if app.ui.aside == Aside::Preview { Aside::None } else { Aside::Preview };
        }
        Action::ToggleSplit => {
            let on = app.ui.panes.len() < 2;
            app.ui.split(on);
        }
        Action::SwapPanes => app.ui.panes.reverse(),
        Action::ShowCompleted => {
            let shown = app.data.settings.show_completed;
            if let Err(e) = app.data.set_setting("show_completed", if shown { "false" } else { "true" }) {
                app.refused(e);
            }
        }
        Action::NewTask => {
            if let Some(list) = app.active_list() {
                match app.data.create_task(&list, "Untitled", None, None) {
                    Ok(task) => app.ui.preview = Some(task.id),
                    Err(e) => app.refused(e),
                }
            }
        }
        Action::Undo => app.undo(),
        Action::NewList => app.sheet = Some(Sheet::NewList),
        Action::NewProject => app.sheet = Some(Sheet::NewProject),
        Action::OpenTrash => app.trash_window = true,
        Action::Settings => app.settings_window = Some("general".to_owned()),
        Action::DeleteTask(id) => app.sheet = Some(Sheet::DeleteTask(id.clone())),
        Action::OpenList(id) => app.ui.open_list(Some(id.clone()), None),
        Action::OpenProject(id) => {
            app.ui.set_project(Some(id.clone()));
            let first = app.data.lists_of(id).first().map(|l| l.id.clone());
            app.ui.open_list(first, Some(0));
        }
        Action::Noop => {}
    }
}

/// A group heading: the label in capitals (the web's CSS uppercase). The node keeps the label as written.
fn heading(ui: &mut egui::Ui, look: Look, group: Group, first: bool) {
    ui.add_space(GROUP_MT);
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(vec2(width, GROUP_LINE), Sense::hover());
    response.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, group.label()));
    if first {
        part(ui, &response, group.part());
    }
    let galley = ui.painter().layout_no_wrap(group.label().to_uppercase(), FontId::new(GROUP_FONT, weight(600)), look.t.content_2);
    let pos = pos2(rect.left() + GROUP_PX, rect.center().y - galley.size().y / 2.0);
    ui.painter().galley(pos, galley, look.t.content_2);
}

/// The palette. It holds the filter text and the highlight, and the search input that edits the text.
pub struct Palette {
    /// The filter text, as the search input last reported it.
    query: String,
    /// The highlight, an index into the rows the filter leaves (clamped to them when drawn).
    active: usize,
    /// The search input's widget id, learned on the first frame of an opening and kept focused.
    search_id: Option<Id>,
    /// The (highlight, row count) the list last scrolled for, so a scroll away from the highlight is kept.
    scrolled_for: Option<(usize, usize)>,
    /// The row under the pointer last frame.
    hover_row: Option<usize>,
}

impl Default for Palette {
    fn default() -> Self {
        Self::new()
    }
}

impl Palette {
    pub fn new() -> Self {
        Self { query: String::new(), active: 0, search_id: None, scrolled_for: None, hover_row: None }
    }

    /// One frame, on the app: draws the palette while `app.ui.palette_open` is set, and returns the events it raised.
    pub fn show(&mut self, app: &mut App, ui: &mut egui::Ui, look: Look) -> Vec<PaletteEvent> {
        let mut events = Vec::new();
        if !app.ui.palette_open {
            // Closed: the next opening starts from an empty query, as the web's dialog mounts anew.
            self.reset();
            return events;
        }
        let ctx = ui.ctx().clone();
        // Escape closes the palette and runs nothing.
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
            app.ui.palette_open = false;
            ctx.request_repaint();
            return events;
        }

        let screen = ctx.content_rect();
        let width = CARD_W.min(screen.width());
        let max_h = screen.height() * 0.6;
        let mut panel = None;
        let area = Area::new(ui.scope_id().with("palette.card"))
            .order(Order::Foreground)
            .movable(false)
            .anchor(Align2::CENTER_TOP, vec2(0.0, CARD_TOP));
        let _ = area.show(&ctx, |ui| {
            let card = Frame::new()
                .fill(look.t.surface_2)
                .stroke(Stroke::new(1.0, look.t.edge_1))
                .corner_radius(CornerRadius::same(10))
                .shadow(Shadow { offset: [0, 4], blur: 16, spread: 0, color: Color32::from_black_alpha(64) });
            let out = card.show(ui, |ui| {
                // The frame's 1 px stroke takes a pixel from each side of the card.
                ui.set_width(width - 2.0);
                ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                self.body(&mut *app, ui, look, max_h - 2.0, &mut events);
            });
            let node = ui.interact(out.response.rect, ui.scope_id().with("panel"), Sense::hover());
            node.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, "Command palette"));
            part(ui, &node, parts::PANEL);
            panel = Some(out.response.rect);
        });

        // There is no backdrop to press, so a press outside the panel closes it (the web's dismiss on the scrim).
        if let Some(rect) = panel {
            let pressed_at = ctx.input(|i| if i.pointer.primary_pressed() { i.pointer.interact_pos() } else { None });
            if pressed_at.is_some_and(|pos| !rect.contains(pos)) {
                app.ui.palette_open = false;
            }
        }
        if !events.is_empty() {
            ctx.request_repaint();
        }
        events
    }

    fn reset(&mut self) {
        self.query.clear();
        self.active = 0;
        self.search_id = None;
        self.scrolled_for = None;
        self.hover_row = None;
    }

    /// The rows the filter leaves, for the app as it is now.
    fn rows(&self, app: &App) -> Vec<Row> {
        filter(catalog(app), &self.query)
    }

    /// The card's contents: the search input, then the rows, or the status message when none match.
    fn body(&mut self, app: &mut App, ui: &mut egui::Ui, look: Look, max_h: f32, events: &mut Vec<PaletteEvent>) {
        // The input takes the typed text, Enter and the arrows. It takes focus on every frame (autoFocus),
        // so Tab, which moves focus on the next pass, does not keep it away.
        let mut input = CommandInput::new(
            serde_json::from_value(serde_json::json!({
                "value": self.query.clone(),
                "placeholder": PLACEHOLDER,
                "ariaLabel": "Search commands",
                "leading": "icon",
                "autoFocus": true,
            }))
            .expect("commandinput props"),
        );
        let input_events = as_part(ui, &[(input_parts::INPUT, parts::SEARCH)], |ui| input.ui(ui, look));
        if self.search_id.is_none() {
            self.search_id = ui.memory(|m| m.focused());
        }
        for event in input_events {
            match event {
                CommandinputEvent::QueryChanged { value } => {
                    self.query = value;
                    self.active = 0;
                }
                CommandinputEvent::MoveActive { direction } => {
                    let len = self.rows(app).len();
                    let cur = current(self.active, len);
                    self.active = if direction == "down" { (cur + 1).min(len.saturating_sub(1)) } else { cur.saturating_sub(1) };
                }
                CommandinputEvent::Submit { .. } => {
                    let rows = self.rows(app);
                    if let Some(row) = rows.get(current(self.active, rows.len())) {
                        run(app, row, events);
                    }
                }
            }
            if !app.ui.palette_open {
                return;
            }
        }

        let rows = self.rows(app);
        let cur = current(self.active, rows.len());
        if rows.is_empty() {
            self.empty(ui, look);
        } else {
            self.list(app, ui, look, &rows, cur, max_h - INPUT_H, events);
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
        response.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, EMPTY_MESSAGE));
        part(ui, &response, parts::EMPTYMESSAGE);
        let pos = rect.center() - galley.size() * 0.5;
        ui.painter().galley(pos, galley, look.t.content_2);
    }

    /// The listbox: the group headings and one row per match, in a scroll region at most `max_list` tall
    /// that keeps the highlight in view.
    fn list(&mut self, app: &mut App, ui: &mut egui::Ui, look: Look, rows: &[Row], cur: usize, max_list: f32, events: &mut Vec<PaletteEvent>) {
        let len = rows.len();
        let scroll = self.scrolled_for != Some((cur, len));
        self.scrolled_for = Some((cur, len));
        let prev_hover = self.hover_row;
        let pointer = ui.input(|i| i.pointer.hover_pos());
        let moved = ui.input(|i| i.pointer.delta() != Vec2::ZERO);
        let search = self.search_id.unwrap_or(Id::NULL);

        // The listbox's own text: its headings and labels, as the web's text content.
        let mut text = String::new();
        let mut shown = None;
        for row in rows {
            if shown != Some(row.group) {
                shown = Some(row.group);
                text.push_str(row.group.label());
                text.push(' ');
            }
            text.push_str(&row.label);
            text.push(' ');
        }

        let mut hovered = None;
        let mut highlight = None;
        let mut picked = None;
        let out = ScrollArea::vertical()
            .id_salt("results")
            .max_height(max_list)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::ZERO;
                ui.add_space(LIST_PAD);
                let mut group = None;
                for (i, row) in rows.iter().enumerate() {
                    if group != Some(row.group) {
                        group = Some(row.group);
                        let first = rows.iter().position(|r| r.group == row.group) == Some(i);
                        heading(ui, look, row.group, first);
                    }
                    let top = ui.cursor().top();
                    let json = serde_json::json!({
                        "id": format!("palette-{i}"),
                        "label": row.label,
                        "value": row.label,
                        "icon": row.icon,
                        "shortcut": row.accel,
                        "context": "palette",
                        "active": i == cur,
                    });
                    let mut item = ComboboxItem::new(serde_json::from_value(json).expect("comboboxitem props"));
                    let item_events = as_part(ui, &row_renames(row, i), |ui| item.ui(ui, look, search));
                    let bottom = ui.cursor().top();
                    let bounds = ui.max_rect();
                    let rect = Rect::from_min_max(pos2(bounds.left(), top), pos2(bounds.right(), bottom));
                    if pointer.is_some_and(|p| rect.contains(p)) {
                        hovered = Some(i);
                    }
                    for event in item_events {
                        match event {
                            // A row reports the pointer resting on it too; only a move, or entering the row, counts.
                            ComboboxitemEvent::Highlighted { .. } => {
                                if moved || prev_hover != Some(i) {
                                    highlight = Some(i);
                                }
                            }
                            ComboboxitemEvent::Picked { .. } => picked = Some(i),
                        }
                    }
                    if scroll && i == cur {
                        ui.scroll_to_rect(rect, None);
                    }
                    if row.accel.is_empty() {
                        ui.add_space(ROW_GAP);
                    }
                }
                ui.add_space(LIST_PAD);
            });

        self.hover_row = hovered;
        if let Some(i) = highlight {
            self.active = i;
        }
        let node = ui.interact(out.inner_rect, ui.scope_id().with("results-node"), Sense::hover());
        node.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, &text));
        part(ui, &node, parts::RESULTS);
        if let Some(i) = picked {
            run(app, &rows[i], events);
        }
    }
}

/// The mount: the app on the props' data set, with the palette open where the props say it starts.
struct Mount {
    app: App,
    screen: Palette,
}

impl Mounted for Mount {
    fn frame(&mut self, ui: &mut egui::Ui, look: Look, emit: &mut dyn FnMut(&'static str, serde_json::Value)) {
        for e in self.screen.show(&mut self.app, ui, look) {
            let (n, f) = e.to_json();
            emit(n, f);
        }
    }
}

pub fn mount(mut app: App, props: serde_json::Value) -> Result<Box<dyn Mounted>, String> {
    let p: PaletteProps = serde_json::from_value(props).map_err(|e| e.to_string())?;
    // The web harness: Errands in Home is the focused list, and the task prop, when set, is selected the way a click selects it.
    app.open_named(Some("Home"), Some("Errands"));
    let home = app.data.project_named("Home").map(|pr| pr.id.clone()).ok_or("palette harness: no Home project")?;
    let errands = app.data.list_named(Some(&home), "Errands").map(|l| l.id.clone()).ok_or("palette harness: no Errands list in Home")?;
    app.ui.set_project(Some(home));
    app.ui.open_list(Some(errands.clone()), Some(0));
    if !p.task.is_empty() {
        let picked = app
            .data
            .tasks_of(&errands)
            .into_iter()
            .find(|t| t.title == p.task)
            .map(|t| t.id.clone())
            .ok_or_else(|| format!("palette harness: no task \"{}\" in Errands", p.task))?;
        app.ui.selection = vec![picked.clone()];
        app.ui.preview = Some(picked);
    }
    app.ui.palette_open = true;
    Ok(Box::new(Mount { app, screen: Palette::new() }))
}
