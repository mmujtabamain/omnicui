//! contract: omnicui/contracts/confirmwindows.contract.yaml (id: app.confirmwindows). Each confirmation and prompt sheet as its own window over the main window (New list, rename, delete, Move to Trash, Empty Trash, Delete permanently): the sheet's card, with no scrim and no press-outside dismissal.
use crate::app::Sheet;
use crate::data::Id;
use crate::screens::sheets::Sheets;
use crate::App;
use egui::{pos2, Rect, Role, Sense, WidgetInfo};
use taskscape_ui::omnicui::confirmwindows::{parts, ConfirmwindowsEvent, ConfirmwindowsProps};
use taskscape_ui::omnicui::sheets::{parts as sheets_parts, SheetsEvent};
use taskscape_ui::{as_part, part, rem, Look, Mounted};

/// The card's width (26rem). The window is as wide as its card.
const CARD_W: f32 = rem(26.0);
/// The footer row: its 1px divider, 9px above and below the lg button (2rem tall).
const FOOTER_H: f32 = 1.0 + 9.0 + rem(2.0) + 9.0;

/// The sheet's own parts, under this window's names.
const RENAMES: &[(&str, &str)] = &[
    (sheets_parts::GLYPH, parts::BADGE),
    (sheets_parts::CARD, parts::CARD),
    (sheets_parts::HEADLINE, parts::HEADLINE),
    (sheets_parts::DETAIL, parts::DETAIL),
    (sheets_parts::NAMEFIELD, parts::FIELD),
    (sheets_parts::ROLL, parts::ROLL),
    (sheets_parts::IMPORTJSON, parts::INSTEAD),
    (sheets_parts::CANCEL, parts::CANCEL),
    (sheets_parts::ACCEPT, parts::ACCEPT),
];

/// The window's own state: the card's state is in the sheet screen, and `more` holds the tasks after the
/// first of a Move to Trash of several (the sheet trashes the first; see `trash_more`).
pub struct ConfirmWindows {
    sheets: Sheets,
    more: Vec<Id>,
}

impl Default for ConfirmWindows {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfirmWindows {
    pub fn new() -> Self {
        Self { sheets: Sheets::new(), more: vec![] }
    }

    /// One frame of the window: the card of `app.sheet`, which the window was opened for. Returns how the
    /// sheet answered, and the data change an accept made.
    pub fn show(&mut self, app: &mut App, ui: &mut egui::Ui, look: Look) -> Vec<ConfirmwindowsEvent> {
        let mut events = vec![];
        let Some(sheet) = app.sheet.clone() else { return events };
        // What an accept changes, read before the write removes it (the name of a deleted list).
        let (kind, named) = change_of(app, &sheet);
        chrome(ui, look);
        // The main window's toast is not this window's: the sheet draws the toast it raised, so the one
        // waiting in the main window is kept out of this frame and put back after it.
        let saved = app.toast.take();
        let answers = as_part(ui, RENAMES, |ui| self.sheets.show(app, ui, look));
        let mut accepted = false;
        for answer in answers {
            let SheetsEvent::Answered { action, text, id } = answer;
            let accept = action == "accept";
            accepted |= accept;
            // A prompt's name is the typed one; a confirmation's name is the one `change_of` read.
            let name = named.clone().unwrap_or_else(|| text.clone());
            events.push(ConfirmwindowsEvent::Answered { action, text, id });
            if accept {
                events.push(ConfirmwindowsEvent::Changed { kind: kind.into(), name });
            }
        }
        if accepted && !self.more.is_empty() {
            self.trash_more(app);
        }
        if app.toast.is_none() {
            app.toast = saved;
        }
        events
    }

    /// Move to Trash of several tasks: the sheet has trashed the first; the rest go under one toast that
    /// names them all and undoes them all.
    fn trash_more(&mut self, app: &mut App) {
        let rest = std::mem::take(&mut self.more);
        let first_undo = app.toast.take().map(|t| t.undo).unwrap_or_default();
        let total = rest.len() + 1;
        app.trash(&rest);
        if let Some(toast) = app.toast.as_mut().filter(|t| t.error.is_none()) {
            let mut undo = first_undo;
            undo.append(&mut toast.undo);
            toast.undo = undo;
            toast.message = format!("{total} moved to Trash");
        }
    }
}

/// The window's background and its footer's part. The sheet draws the footer's divider and buttons; the
/// footer is the card's bottom row, which the window's height fills (the window is sized to its card).
fn chrome(ui: &mut egui::Ui, look: Look) {
    let area = ui.max_rect();
    ui.painter().rect_filled(area, 0.0, look.t.surface_1);
    let card_w = CARD_W.min(area.width());
    let footer = Rect::from_min_max(pos2(area.center().x - card_w / 2.0, area.bottom() - FOOTER_H), pos2(area.center().x + card_w / 2.0, area.bottom()));
    let node = ui.interact(footer, egui::Id::unique("app.confirmwindows.footer"), Sense::hover());
    node.widget_info(|| WidgetInfo::new(Role::GenericContainer));
    part(ui, &node, parts::FOOTER);
}

/// The data change an accept of this sheet makes, as the contract names it: its kind, and its name when
/// the name is known before the write (None for a prompt, whose name is the typed one).
fn change_of(app: &App, sheet: &Sheet) -> (&'static str, Option<String>) {
    match sheet {
        Sheet::NewList => ("list-created", None),
        Sheet::NewProject => ("project-created", None),
        Sheet::RenameList(_) => ("list-renamed", None),
        Sheet::RenameProject(_) => ("project-renamed", None),
        Sheet::DeleteList(id) => ("list-deleted", app.data.list(id).map(|l| l.name.clone())),
        Sheet::DeleteProject(id) => ("project-deleted", app.data.project(id).map(|p| p.name.clone())),
        Sheet::DeleteTask(id) => ("task-trashed", app.data.task(id).map(|t| t.title.clone())),
        Sheet::EmptyTrash => ("trash-emptied", Some("Trash".to_owned())),
        Sheet::PurgeTasks(ids) => ("purged", Some(if ids.len() == 1 { "1 item".to_owned() } else { format!("{} items", ids.len()) })),
    }
}

/// The window's mount in a test: the app on the props' data set, with the sheet the props name open.
struct Mount {
    app: App,
    window: ConfirmWindows,
}

impl Mounted for Mount {
    fn frame(&mut self, ui: &mut egui::Ui, look: Look, emit: &mut dyn FnMut(&'static str, serde_json::Value)) {
        for e in self.window.show(&mut self.app, ui, look) {
            let (name, fields) = e.to_json();
            emit(name, fields);
        }
    }
    fn close(&mut self) {
        // The OS closed the window: the sheet is cancelled.
        self.app.sheet = None;
    }
}

pub fn mount(mut app: App, props: serde_json::Value) -> Result<Box<dyn Mounted>, String> {
    let p: ConfirmwindowsProps = serde_json::from_value(props).map_err(|e| e.to_string())?;
    // The main window's project is Home, which the reference's targets are read in.
    app.open_named(Some("Home"), None);
    let (sheet, more) = sheet_for(&app, &p)?;
    app.sheet = Some(sheet);
    let mut window = ConfirmWindows::new();
    window.more = more;
    window.sheets.initial_name = Some(p.initialName.clone());
    Ok(Box::new(Mount { app, window }))
}

/// The sheet the props open, and the tasks after the first for a Move to Trash of several.
fn sheet_for(app: &App, p: &ConfirmwindowsProps) -> Result<(Sheet, Vec<Id>), String> {
    let home = app.ui.project.clone();
    // The reference target of a name-based window: the given name, or the default.
    let named = |given: &str, default: &str| if given.is_empty() { default.to_owned() } else { given.to_owned() };
    let list_id = |name: String| app.data.list_named(home.as_deref(), &name).map(|l| l.id.clone()).ok_or_else(|| format!("no list named \"{name}\""));
    let project_id = |name: String| app.data.project_named(&name).map(|p| p.id.clone()).ok_or_else(|| format!("no project named \"{name}\""));
    // Inbox in the empty data set, Reading otherwise.
    let default_list = if p.dataset == "empty" { "Inbox" } else { "Reading" };
    match p.open.as_str() {
        "newList" => Ok((Sheet::NewList, vec![])),
        "newProject" => Ok((Sheet::NewProject, vec![])),
        "renameList" => Ok((Sheet::RenameList(list_id(named(&p.target, default_list))?), vec![])),
        "renameProject" => Ok((Sheet::RenameProject(project_id(named(&p.target, "Home"))?), vec![])),
        "deleteList" => Ok((Sheet::DeleteList(list_id(named(&p.target, default_list))?), vec![])),
        "deleteProject" => Ok((Sheet::DeleteProject(project_id(named(&p.target, "Home"))?), vec![])),
        "moveToTrash" => trash_targets(app, p),
        "emptyTrash" => Ok((Sheet::EmptyTrash, vec![])),
        "deletePermanently" => purge_targets(app, p),
        other => Err(format!(
            "open \"{other}\" is not one of newList, newProject, renameList, renameProject, deleteList, deleteProject, moveToTrash, emptyTrash, deletePermanently"
        )),
    }
}

/// Move to Trash: the task named by `target` (default "Book bike service" in Errands), then, for a count
/// above one, the open tasks of Errands after it.
fn trash_targets(app: &App, p: &ConfirmwindowsProps) -> Result<(Sheet, Vec<Id>), String> {
    let errands = app
        .data
        .list_named(app.ui.project.as_deref(), "Errands")
        .map(|l| l.id.clone())
        .ok_or_else(|| "no list named \"Errands\"".to_owned())?;
    let first = if p.target.is_empty() {
        app.data
            .tasks_of(&errands)
            .into_iter()
            .find(|t| t.title == "Book bike service")
            .map(|t| t.id.clone())
            .ok_or_else(|| "no task named \"Book bike service\" in Errands".to_owned())?
    } else {
        app.data.task_named(&p.target).map(|t| t.id.clone()).ok_or_else(|| format!("no task named \"{}\"", p.target))?
    };
    let count = p.count.max(1) as usize;
    let open: Vec<Id> = app.data.tasks_of(&errands).into_iter().filter(|t| !t.done).map(|t| t.id.clone()).collect();
    let mut ids = vec![first.clone()];
    ids.extend(open.into_iter().filter(|id| *id != first).take(count - 1));
    if ids.len() < count {
        return Err(format!("Errands has only {} open tasks, not {count}", ids.len()));
    }
    let more = ids.split_off(1);
    Ok((Sheet::DeleteTask(first), more))
}

/// Delete permanently of two or more Trash rows: the two the contract names first, then the rest of the Trash.
fn purge_targets(app: &App, p: &ConfirmwindowsProps) -> Result<(Sheet, Vec<Id>), String> {
    if p.count < 2 {
        return Err(format!("deletePermanently is opened for 2 or more rows, not {}", p.count));
    }
    let count = p.count as usize;
    let mut ids: Vec<Id> = ["Print the old flyer", "Call the printer about samples"]
        .iter()
        .filter_map(|title| app.data.trash.iter().find(|t| t.title == *title).map(|t| t.id.clone()))
        .collect();
    for t in &app.data.trash {
        if ids.len() >= count {
            break;
        }
        if !ids.contains(&t.id) {
            ids.push(t.id.clone());
        }
    }
    if ids.len() < count {
        return Err(format!("the Trash holds {} rows, not {count}", app.data.trash.len()));
    }
    ids.truncate(count);
    Ok((Sheet::PurgeTasks(ids), vec![]))
}
