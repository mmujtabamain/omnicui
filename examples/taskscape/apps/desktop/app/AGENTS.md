# The desktop app's screens (egui): conventions

A screen is the desktop form of one screen contract (`omnicui/contracts/<name>.contract.yaml`, id `app.<name>`), drawn with the widget library (`apps/desktop/ui`, read its AGENTS.md too) on the app's data.

## One screen, one file

- A screen is `src/screens/<contract name>.rs`. While it is being built it lives in `drafts/<contract name>.rs`. You write only your draft.
- Its first line names its contract:
  ```rust
  //! contract: omnicui/contracts/tasklist.contract.yaml (id: app.tasklist). <one line on what it is>
  ```
- The props, events and part ids are generated: `taskscape_ui::omnicui::<name>::{parts, <Name>Props, <Name>Event}`.

## The shape of a screen

```rust
use crate::App;
use taskscape_ui::omnicui::tasklist::{parts, TasklistEvent, TasklistProps};
use taskscape_ui::{part, Look, Mounted};

/// The screen's own state (what it keeps between frames: a search query, an open menu…).
pub struct TaskList { pane: usize, query: String, /* … */ }

impl TaskList {
    pub fn new(pane: usize) -> Self { … }
    /// One frame, on the app: what the main window calls, and what the test mounts.
    pub fn show(&mut self, app: &mut App, ui: &mut egui::Ui, look: Look) -> Vec<TasklistEvent> { … }
}

/// The test's mount: the app on the props' data set, put where the props say the screen starts.
struct Mount { app: App, screen: TaskList }
impl Mounted for Mount {
    fn frame(&mut self, ui: &mut egui::Ui, look: Look, emit: &mut dyn FnMut(&'static str, serde_json::Value)) {
        for e in self.screen.show(&mut self.app, ui, look) { let (n, f) = e.to_json(); emit(n, f); }
    }
}
pub fn mount(mut app: App, props: serde_json::Value) -> Result<Box<dyn Mounted>, String> {
    let p: TasklistProps = serde_json::from_value(props).map_err(|e| e.to_string())?;
    app.open_named(Some(&p.project), Some(&p.list));   // start where the props say
    Ok(Box::new(Mount { app, screen: TaskList::new(0) }))
}
```

- **The screen is one part of a window,** never a whole window of its own. The main window composes `show` functions; the test mounts the same `show` alone. (The web build went wrong by building screens as whole windows.)
- **Data:** `app.data` (`src/data.rs`) holds every live task, list, project, the Trash and the settings, read from SQLite and written through. Each write updates the screen at once.
  - Use its methods: `create_task`, `update_task`, `move_task`, `trash_tasks`, `rename_list`… Never open the database yourself.
  - Ids are strings.
- **What the user is looking at:** `app.ui` (`src/state.rs`): the project, the panes and the active pane, the selection, the previewed task, Filter & Sort views, the aside, the draft.
  - Change it the way the web's `state/ui.ts` does (`app.ui.open_list`, `app.ui.split`…).
- **Shared:**
  - `app.toast` and `app.trash(ids)` (Move to Trash with Undo);
  - `app.sheet` (a confirmation or prompt, drawn by the confirmwindows screen as its own window);
  - `app.trash_window` and `app.settings_window` (opened from the title bar or the palette);
  - `app.refused(e)` when a save fails.
- **Props that set up a test** (`trash: "Cut a spare key"`, `open: "rename"`) are applied in `mount` before the first frame, through the same app methods a user's action would use. Each prop's meaning is in its comment in the contract; the web harness `apps/web/frontend/src/harness/<name>.tsx` shows how the web applied it.

## Port the web screen

The web screen is `apps/web/frontend/src/screens/<folder>/` (its harness names it). Port its layout, sizes, copy and behaviour, with the widgets from `taskscape_ui::widgets::<widget>` where the web uses a widget. Every part the contract names is tagged with `part(ui, &response, parts::X)`. Follow the desktop decisions (`spec/decisions.md`): no scrim or backdrop, the OS draws window controls, key labels as text (⌘N on macOS, "Ctrl N" elsewhere), the platform modifier is `modifiers.command`.

## Check

From `examples/taskscape`:

```
python3 apps/desktop/scripts/check-widget.py <name> --slot <slot> [--test "<part of a test name>"]
```

It builds your draft with the finished widgets and screens, and runs the contract on the desktop target. Tests tagged `only: [web]` are "not applicable". Fix until it exits 0, then look at a few screenshots.

## A library widget under the screen's part names

A library widget tags its own parts (`ui.button.button`). Where the contract names the same thing as a screen part (`app.toasthost.undo`), draw the widget inside `taskscape_ui::as_part`:

```rust
use taskscape_ui::{as_part, omnicui::button::parts as button_parts};
as_part(ui, &[(button_parts::BUTTON, parts::UNDO)], |ui| self.undo.ui(ui, look));
```

The node is then found by the screen's part id. This is the web's `data-part="ui.button.button app.toasthost.undo"`.
