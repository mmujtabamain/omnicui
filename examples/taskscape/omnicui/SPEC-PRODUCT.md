# Taskscape: the product, for the porters

What the GPUI port (P1) and the web port (P2) build, precisely enough to build it without
reading the React code. It is the Tauri app (`../tauri/`) as it is, except where the design
review (`tauri/docs/main-app-design-review-2026-08-13.md`, "Direction From Your Answers") moves
it on; those places are marked **Direction**. The look is in DESIGN.md. The contracts in
`contracts/` are the checks: when this file and a contract disagree, the contract is what runs,
and the disagreement is a bug in this file (say so in your GAPS file).

## The product

Taskscape is a task manager for one person on a Mac: **projects** hold **lists**, lists hold
**tasks**, and a task holds subtasks (to any depth), rich-text **notes**, and **attachments**
(links, or files copied into the app's data folder). Deleted tasks go to a **Trash**. A
menu-bar **capture bar**, summoned anywhere with ⌘↩, files a task in the list last open.

The design review's direction, which the ports follow:
- **Quiet shell, opinionated core**: calm, dense and desktop-native.
- **Quick capture is the hero.** Empty states point to it; the main window is where captured
  tasks are triaged and deepened.
- **The right rail is a details drawer**, not a second pane.
- **Selection means "I'm looking at this".** A row press focuses and inspects. The round check
  means *done*. Multi-select is ⇧/⌘ press, or the keyboard, entered on purpose.
- **Cleverness only where it saves time**: never in what a primary control means, and never
  hiding what can't be undone.

## What there is, ranked

By how much of the product each carries. The contracts cover the top of the list thoroughly.

| rank | surface | component | what it is |
|---|---|---|---|
| 1 | task pane | `taskscape.pane` | one list's tasks: read, complete, nest, search, act on |
| 2 | preview rail | `taskscape.inspector` | the focused task (title, notes, subtasks, attachments), a draft, or the selection |
| 3 | capture bar | `taskscape.capture` | the hero: file a task from anywhere (desktop only) |
| 4 | title bar | `taskscape.titlebar` | projects, list tabs, the view buttons, the window controls |
| 5 | the window | `taskscape.window` | the surfaces together at 1180×760, and the flows between them |
| 6 | sheets | `taskscape.sheet` | every question the app asks |
| 7 | command palette | `taskscape.palette` | ⌘K: run anything, jump anywhere |
| 8 | Settings | `taskscape.settings` | General, Capture, Shortcuts, About |
| 9 | filter panel | `taskscape.filter` | how the pane sorts and filters |
| 10 | Trash | `taskscape.trash` | restore or purge deleted tasks |
| 11 | toast | `taskscape.toast` | "Moved to Trash · Undo" |
| 12 | attachment lightbox | none | full-size image and text previews; not under contract (GAPS-P0.txt) |
| — | context menus | in the pane | a row's actions (in `taskscape.pane`) |

Also in the app, not under contract: import and export of a list as JSON (they need the file
dialogs), drag and drop (rows, tabs, attachments onto notes), resizing the rail and the split,
@mentions of attachments in notes, the boot overlay.

## The data, and the fixtures

The data model is `taskscape_common` (`tauri/common/src/models.rs`), the same crate every port
uses: GPUI links it, and the web port's server does. Projects → lists → tasks (with
`parent_id` for nesting, `done`, `sort_order`, `deleted_at` for the Trash) → notes (sanitized
HTML, ordered) and attachments (`reference` or `copy`). A task's `notes` field is the plaintext
of its notes, for the row preview and search. Settings (`theme`, `reduced_motion`,
`show_completed`, `screenshot_mode`, `hotkeys`, `last_active_list`, `last_active_project`) live
in a file beside the database.

**Fixtures** are `fixtures/*.json`, loaded by the `taskscape-fixtures` crate
(`fixtures/src/lib.rs`) through `Store` into a fresh data folder: call
`taskscape_fixtures::load(seed, root)` on every mount. It sets `HOME` (the store finds
`~/.taskscape` from it) and returns the open `Store` and `keys`, the fixture's keys mapped to the
store's ids. `taskscape-fixture <seed>` does the same from the command line and prints
`{home, keys}`. Every version reads the same data the same way: never write fixture data any
other way.

| seed | what it holds |
|---|---|
| `standard` | Projects Atlas and Home. Atlas: **Launch plan** (8 tasks, below), **Groceries** (Oat milk; Coffee beans, done; Lemons), **Inbox** (empty). Home: **Chores** (Fix the shelf). Trash: Paint swatches (Groceries), Old launch draft (Launch plan), the first the newest. The last list open: Launch plan. |
| `empty` | Atlas with one empty list, Inbox. |
| `nolists` | Atlas with no lists. |
| `long` | Atlas with Reading: "Read chapter 1" … "Read chapter 30", every fourth one done. |

Launch plan, in order (keys in brackets):

```
Write the launch brief [brief]   note: "Draft due Friday; keep it to one page."
                                 attachments: "Brief outline" (https://example.com/brief),
                                              "whiteboard.png" (a copied image)
  Collect the feature list [features]   done
  Draft the summary [summary]
  Review with Sam [review]
Book the venue [venue]           done; note: "Hall B, forty seats."
Order the banners [banners]
Send the invitations [invites]
  Confirm the guest list [guests]       done
```

The other keys: `atlas`, `home`, `launch`, `groceries`, `inbox`, `chores`, `milk`, `coffee`,
`lemons`, `shelf`, `olddraft`, `swatches` (and `reading`, `ch1`…`ch30` in `long`).

## What every target does

- **Events name things by fixture key.** A registry maps the store's ids back to keys
  (`brief`, `launch`); an id the fixture didn't make (a task created in the test) is passed as
  it is, and tests don't check it. `parentId` is `""` for a top-level task.
- **Domain events come from what the user changed in the store**: one event per change, in the
  order made. The tauri target derives them from the commands the frontend sends (its server,
  `targets/tauri/server/src/main.rs`, is the reference list); a port derives them where its
  stores write.
- **Props that a test leaves out take the contract's default**; the registry applies them.
- **Props set the state, the way the app sets it**, before the component is shown:
  - `seed` loads the fixture;
  - `showCompleted: false` saves the setting first;
  - `list` puts that list in the (left) pane and focuses it;
  - `split` puts a list in the split pane;
  - `preview` opens or closes the rail;
  - `selected` focuses a task;
  - `draft` opens a new-task draft for a list;
  - `multi` selects tasks in the focused list.
- **Parts** are named for meaning, unique across the product, so the window's parts are its
  surfaces' parts by the same names. A part that repeats is numbered by its place on screen:
  `row3` is the third row shown, and `row3Check`, `row3Title`, `row3More` are that row's pieces.
  The same for `tab`, `projectItem`, `command`, `note`, `subtask`, `attachment`, `picked`,
  `trashed`, `submenuItem`. The tauri app tags them `data-part="row"`, `data-part="rowCheck"`;
  a port tags them however its toolkit lets its adapter find them.
- **Text is what is shown**: icons drawn from a font aren't text; `text-transform: uppercase`
  is (the footer reads "3/8 DONE").
- **Platform**: tauri and gpui are `platform: [desktop]`, web is `platform: [web]`. Desktop-only
  tests say `only: [desktop]`. Inputs whose keys differ are named in `platforms.yaml`.

## Components

Each section: what it is, its props, its events, its parts (role, and what they mean), the
rules, the texts, and what a platform is free to do. The contract has the exact parts list.

### `taskscape.pane`: the task pane

One list's tasks, at 860×708.

**Props.** `seed` (standard, empty, long), `list` (key; default `launch`), `showCompleted`
(default true), `multi` (keys, comma-separated).

**Events.** `taskCompleted {id, done}` (one per task that changes), `taskCreated {id, title,
listId, parentId}`, `taskMoved {id, listId, parentId}`, `taskFocused {id}` (the task the
inspector shows), `taskExpanded {id, expanded}`, `confirmAsked {headline}` (the pane asked a
question; the sheet is its own component), `draftOpened {listId}`, `subtaskComposerOpened
{parentId}`, `copied {text}`, `filterToggled {open}`.

**Parts.**

| part | role | meaning |
|---|---|---|
| `search` | textbox | the search field; its name is "Search <list>" |
| `searchOptions` | button | scope (this list, the project, all) and fields (title, notes, both) |
| `matchCount` | status | how many tasks match, shown while searching |
| `clearSearch` | button | ×, while searching |
| `newTask` | button | "+": a new draft for this list |
| `rowN` | button | a task's row; a press focuses it |
| `rowNTitle`, `rowNNotes` | generic | the title; the plaintext of its notes, one line |
| `rowNCheck` | button | the round check (**Direction**: done; the app: select) |
| `rowNToggle` | button | the disclosure chevron, for a task with subtasks |
| `rowNSubtasks` | status | "done/total" of its direct subtasks, e.g. "1/3" |
| `rowNAttachments` | status | how many attachments |
| `rowNMore` | button | "···": the row's menu |
| `subtaskTitle` | textbox | the inline field "Add subtask" opens under the task |
| `menuCopy`, `menuSubtask`, `menuRename`, `menuMove`, `menuPromote`, `menuDelete` | button | the row's menu: Copy, Add subtask, Rename, Move to list (a submenu of the project's lists, `submenuItemN`), Promote to top level (subtasks only), Delete task… |
| `paneEmptyTitle`, `paneEmptySubtitle` | heading, generic | the empty state |
| `doneCount`, `donePercent` | status | the footer: "3/8 DONE", "38%" |
| `viewControl` | button | the footer's sort and filter control; it reads "Filtered" while a view is on |
| `selectedCount` | status | the bulk bar: "2 SELECTED" |
| `bulkDone`, `bulkUndone`, `bulkMove`, `bulkCopy`, `bulkDelete`, `bulkClear` | button | Done, Undone, Move to…, Copy, Delete, Clear |

**Rules.**
- Rows are the list's top-level tasks in their order, each followed by its subtasks, to any
  depth, unless collapsed. A collapsed task's chevron shows; an expanded one shows on hover.
- **Completing** a task completes its whole subtree. When the last open child of a task is
  completed, the parent completes too, and so on up. Reopening a task reopens its subtree and
  every ancestor. A done task's title is muted and struck through, its notes faded.
- **Direction:** pressing the round check completes the task (and the rules above). The Tauri
  app still selects with it (⌥ held: done); the contract's test for it says `except: [tauri]`.
  ⌥Space (the `toggleDone` input) completes the focused task everywhere.
- The footer counts every task in the list, subtasks included, done and not, whatever is
  hidden: "done/total DONE" and the rounded percentage.
- **Show completed** off (a setting) hides done tasks from the rows; they still count.
- **Search** matches titles and notes (by default), case-insensitively, as you type. A match's
  ancestors stay shown, so it shows where it lives; matches are highlighted. Enter and ⇧Enter
  step through the matches. Escape clears. No match: "No matches in <list>".
- A row press focuses the task (`taskFocused`) and the inspector shows it. ↑/↓ move the focus
  through the rows shown; ←/→ collapse and expand, or go to the parent and the first child; Tab
  and ⇧Tab indent under the previous sibling and outdent. The `moveDown` input moves the task
  below its next sibling (`taskMoved`). Typing a letter with no field focused starts a search.
- **Selection** (several tasks): ⌘ and ⇧ press, ⇧↑/↓, or `selectAll`. With a selection, the
  footer is the bulk bar.
- "+" or `newTask` opens a draft in the inspector; nothing is created until the draft has a
  title.
- **Add subtask** opens a field under the task (expanding it); Enter adds the subtask and
  keeps the field for the next; Enter on a blank field or Escape closes it.
- **Delete** (the menu, `deleteTask`, the bulk bar) always asks first (`confirmAsked`); the
  question and the delete are the sheet's. A deleted task goes to the Trash with its subtree.
- **Move to list** moves the task, with its subtree, to the end of that list's top level.
  **Promote** makes a subtask top-level, at the end.
- **Copy** puts a Markdown checklist on the clipboard: `- [ ] Title` (`- [x]` when done), a
  selected task's selected descendants indented two spaces per level, and each note as a
  `> text` line under its task.

**Texts.** Empty: "Nothing here yet" / "Press + or ⌘N to add a task, or ⌘↩ anywhere to capture
one" (the capture shortcut as bound; without it, the sentence stops at "task"). Searching:
"No matches in <list>" / "Try a different search or scope". Search placeholder "Search <list>".
The menu: "Copy", "Add subtask", "Rename", "Move to list", "Promote to top level", "Delete
task…"; for a selection, "Copy N tasks", "Mark N done", "Mark N not done", "Delete N tasks…".
Subtask field: "Subtask title — Enter to add".

**Platform freedom.** Text fields and the menu are the platform's (layout only). The rows,
checks, badges and footer are the product's.

### `taskscape.inspector`: the preview rail

The details drawer, at 320×708.

**Props.** `seed`, `selected` (a task's key), `draft` (a list's key), `multi` (keys).

**Events.** `taskCompleted`, `taskRenamed {id, title}`, `taskCreated`, `taskFocused`,
`noteAdded {taskId, text}`, `noteChanged {taskId, text}` (the note's plaintext),
`confirmAsked`, `previewToggled {open}`, `copied {text}`, `attachmentOpened {location}`.

**Parts.** Empty: `previewEmptyTitle`, `previewEmptySubtitle`. A task: `inspector` (the whole
inspector), `inspectorDone` (checkbox; its name says what a press does: "Mark done" or "Mark not
done"), `inspectorTitle` (textbox: the title, editable in place), `inspectorWhere` ("Atlas /
Launch plan"), `inspectorCreated` ("Created just now"), `inspectorClose`, `inspectorRename`,
`inspectorCopy`, `inspectorDelete`, `notesCount`, `addNote`, `noteN` with `noteNText` (the note,
editable) and `noteNDelete`, `newNote` (the editor for a new note), `subtasksCount` ("1 done /
3"), `subtaskNCheck` and `subtaskNName`, `attachmentNTile` and `attachmentNName`, `attachLink`.
A draft: `draftTitle`, `draftWhere`, `draftNote`. Several tasks: `multiCount` ("2 tasks
selected"), `multiDoneCount` ("1 done / 2"), `multiClear`, `multiDone`, `multiDelete`,
`pickedNName`, `pickedNList`.

**Rules.**
- It shows, in this order of precedence: the focused list's selection when it has two or more
  tasks; else the focused task; else a draft; else the empty state.
- A task: the check, the title (wraps), where it lives, when it was created and updated
  (relative: "just now", "12m ago", "3h ago", "yesterday", "3d ago", "2w ago", then a date), and
  close, rename, copy and delete. Then Notes, Subtasks (its direct children and theirs,
  indented), Attachments.
- The check completes as in the pane (the subtree with it; three `taskCompleted` for the
  brief).
- **Rename**: the title becomes editable in place, selected; Enter or leaving the field saves it
  (`taskRenamed`) if it changed and isn't blank; Escape puts it back.
- **Notes** are rich text (bold, italic, lists, links, code). A note saves 500 ms after typing
  stops, and when it loses focus (`noteChanged`). A note emptied and left is deleted. "Add note"
  opens an editor; the note is created on its first save (`noteAdded`). Deleting a note asks
  ("Delete note?", danger tone): notes skip the Trash.
- Pressing a subtask's name shows the subtask; its check completes it.
- Attachments: tiles, images as thumbnails. A link opens with the system
  (`attachmentOpened`); an image or text file opens the lightbox. "Shot" captures the screen
  (macOS), "Link" asks for a URL or path, "File" picks a file to copy in.
- **A draft** ("+"): an empty, focused title over "Start with: Note · Screenshot". Enter, or
  leaving the field, with a title creates the task (`taskCreated`) and shows it. A blank title,
  Escape, or "Discard draft" drops it with nothing written. "Note" creates "Untitled" and opens
  its note editor; "Screenshot" captures first, and creates nothing if the capture is cancelled.
- **Several tasks**: the count, done of total, Done, Undone, Move, Copy, Delete (asks), and
  the list of them. Clear empties the selection, and the drawer.
- Close (and `togglePreview`) hides the rail.

**Texts.** "No task selected" / "Select a task to inspect it". "Created …", "Updated …". "NOTES",
"SUBTASKS" with "N done / M", "ATTACHMENTS" with "Shot", "Link", "File"; with none: "Copy a
file", "Add a link". "Add note"; the editor's placeholder "Write a note ... (drop an attachment
to @mention it)". The draft: placeholder "Untitled", "START WITH", "Note", "Screenshot", and
"Type a title, add a note, or take a screenshot — the task is created the moment it has
something in it." Several: "N tasks selected", "N done / M", "Done", "Undone", "Move", "Copy",
"Delete", "SELECTED".

**Platform freedom.** The title field and the note editor are the platform's text engines
(layout only). **Motion**: the inspector, draft and multi inspector rise in (110 ms, 3 px, fade)
when they appear or change task.

### `taskscape.capture`: the capture bar (desktop)

The tray agent's only window, 480 px wide. Desktop only: every test says `only: [desktop]`.

**Props.** `seed`. **Events.** `taskCreated`, `noteAdded`, `dismissed`, `mainOpened`,
`screenshotRequested`.

**Parts.** `captureBar` (the card), `captureTitle`, `captureClear`, `captureNotesHint`,
`captureNotes`, `captureTarget`, `captureShot`.

**Rules.**
- It appears at the pointer (⌘↩, global), focused on the title, over any app and any Space.
  In a port it is a native pop-up window that is ready when summoned (WHY.md).
- The footer says where the task will go: "<project> / <list>", the last list the main window
  had focused, else the first list, else a new list. Pressing it opens the main window.
- Enter with a title files the task there (`taskCreated`), with its notes as its first note
  (`noteAdded`) and its screenshots as attachments, then empties the draft. A blank title files
  nothing.
- Tab opens the notes below the title (the panel grows open); ⌘↩ in the notes files the task.
  Emptied and left, the notes close again.
- **The draft survives dismissal**: Escape or a click elsewhere hides the bar and keeps it. Only
  filing it, "Clear" (shown when there is a draft) or `clearDraft` (⌘⇧⌫) empties it.
- The screenshot button captures the screen and attaches it (`screenshotRequested`): a spinner
  while it runs ("Capturing …"), then the count ("1 shot"), or "Capture failed" for 4 s. Each
  press adds another. The bar itself is left out of its own capture.

**Texts.** "Capture a task ...", "Press Tab to add a note", "Notes ...", "Clear", "Screenshot",
"N shot(s)", "Capturing …", "Capture failed".

**Platform freedom.** The window mechanics (a non-activating panel, joining every Space, sizing
to the card) are the platform's. The card is the product's.

### `taskscape.titlebar`: the title bar

The window's top bar, 1180 px wide (mounted at 1180×300, so that the project menu has room).

**Props.** `seed`, `list`, `split`, `preview`. **Events.** `paneShows {listId}` (the list the
left pane shows), `listSelected {id}` (the focused list, saved for the capture bar),
`projectSelected {id}`, `previewToggled`, `splitChanged {listId}` ("" when closed),
`settingsOpened`, `confirmAsked`, `windowAction {action}`.

**Parts.** `windowControls` and `windowClose` (optional: desktop only, `same: none`),
`project` (the pill; its text and name are the project's name), `projectMenu`,
`projectItemNName`, `projectItemN`, `newProject`, `tabN`, `tabNName`, `tabNClose`, `newList`,
`swapPanes`, `splitView`, `togglePreview`, `openSettings`.

**Rules.**
- The pill shows the active project; pressing it opens the menu of projects (the active one
  checked, rename and delete on hover) and "New project…". Choosing one shows its lists.
- The tabs are the project's lists in order. A press shows that list in the focused pane (a
  list already on screen just takes the focus). In split view, markers on the two tabs show
  which pane is which. A tab's × asks before it deletes the list ("Delete “<list>”?", danger
  tone). Double-click renames in place; drag reorders, and a task dropped on a tab moves there.
- `nextList`, `prevList`, `switchList2` (⌘1–9) move between tabs.
- "+" asks for a new list's name (a dice suggests one); "Import JSON…" is the other way in.
- The view buttons: swap panes (in split view only), split view (opens the next list beside;
  "Close split view" when on), the preview rail ("Hide preview panel" / "Show preview panel",
  with its shortcut), Settings. A reduced-motion icon shows when motion is reduced.
- On the desktop the bar is the window's title bar: it drags the window, and double-click zooms
  it. The window controls are at its leading edge.

**Texts.** Names: "Switch project" (the pill's tooltip), "New list", "Split view" / "Close split
view", "Swap panes", "Hide preview panel  ⌘\\", "Settings  ⌘,"; "New project…"; "Delete list".

**Platform freedom.** The window controls are the platform's, and the web has none (the bar is
the page's header there). Everything else is the product's.

### `taskscape.window`: the main window

The surfaces together at 1180×760 (DESIGN.md has the measured structure). **Props.** `seed`.
**Events.** `taskCreated`, `taskTrashed`, `taskRestored`, `taskCompleted`, `settingChanged`.
**Parts.** The surfaces' parts by the same names, plus `windowEmptyTitle` ("No lists yet" /
"Create a list to start adding tasks"; with lists but none chosen: "No list selected" / "Pick a
list tab to view its tasks").

**Rules.**
- It opens on the last project and list, with the rail open unless it was closed last time.
- Pressing a row shows the task in the drawer; a tab switches the pane; split view shows two
  panes. The palette, sheets and the toast open over the window.
- Deleting asks with a sheet, then the toast says "Moved to Trash" with Undo for 6 s; Undo (or
  ⌘Z) puts the task back in its place. Every change is undoable with `undo` and redoable.
- `toggleCompleted` flips Show completed and saves it (`settingChanged`).

**Direction, and WHY.md**: the window paints its first frame ready (no boot veil in a native
port); nothing opens in a window that has to load first.

### `taskscape.sheet`: a question

**Props.** `seed`, `kind` (newList, renameList, deleteList, deleteTask, newProject,
renameProject, deleteProject; the flow that asks), `list`, `task`, `project`.

**Events.** `answered {action: accept|instead|dismiss, text}`, and what accepting does:
`taskTrashed`, `listCreated {id, name, projectId}`, `listRenamed`, `listDeleted`,
`projectCreated {id, name}`, `projectRenamed`, `projectDeleted`.

**Parts.** `sheetHeadline`, `sheetDetail`, `sheetField` (a prompt's field), `sheetDice`,
`sheetAccept`, `sheetCancel`, `sheetInstead`.

**Rules.** One sheet at a time, centred over the dimmed window, focus
kept inside. A glyph badge, the headline (one line), the detail (the consequence), the field
for a prompt, then the footer: the "instead" action on the left, Cancel and the accept verb on
the right. **Enter accepts, Escape dismisses**, wherever focus is in the sheet. A prompt's
accept is disabled while its field is blank; a suggested name is selected, so typing replaces
it. Accent tone for what can be undone, danger for what can't.

**Texts.**

| kind | headline | detail | accept | other |
|---|---|---|---|---|
| deleteTask | Move to Trash? | “<title>” and its N subtask(s) — ⌘Z puts it back. | Move | |
| deleteTask (several) | Move N tasks to Trash? | N tasks including subtasks — ⌘Z puts them back. | Move | |
| deleteList | Delete “<list>”? | Its N task(s) go with it. No undo. (empty: The list is empty. No undo.) | Delete list | danger |
| newList | New list | | Create | field "Name this list", dice; instead "Import JSON…" |
| renameList | Rename list | | Rename | field holds the name |
| newProject | New project | | Create | field "Name this project", dice |
| renameProject | Rename project | | Rename | |
| deleteProject | Delete “<project>”? | Its N list(s) and M task(s) go with it. No undo. | Delete project | danger |
| (inspector) | Delete note? | Notes skip the Trash — this one is gone for good. | Delete | danger |
| (Trash) | Empty Trash? | N tasks deleted for good. No undo. | Empty Trash | danger |
| (inspector) | Link a file or page | | Link | field |
| (note editor) | Add link | | Add link | field "https://…" |

**Motion**: the sheet scales in (140 ms, scale 0.96 → 1, fade). **Platform freedom**: the field is
the platform's.

### `taskscape.palette`: the command palette

**Props.** `seed`, `selected`. **Events.** `commandRun {command}` (the command's id:
`toggle_preview`, `go-list-<key>`, …), `closed`, and what the command does.

**Parts.** `paletteInput`, `paletteEmpty`, `commandGroupN`, `commandN`.

**Rules.** It opens at the top of the window with the field focused and every command, grouped:
View (Toggle preview, Toggle split view, Swap panes, Show / hide completed, Collapse all, Expand
all), Task (New task, Search this list, Undo, Redo; with a task focused, Delete “<title>”),
Manage (New list, New project, Open Trash, Settings), Go to list (the project's lists), Go to
project. Each shows its shortcut. Typing keeps the commands whose label contains the text,
earliest match first. ↑/↓ move the highlight, Enter or a press runs it and closes the palette;
Escape or a press outside closes it. Nothing matching: "No matching commands". Placeholder
"Run a command or jump to…". **Motion**: it scales in like a sheet.

### `taskscape.settings`: Settings

At 640×520. **Props.** `seed`. **Events.** `settingChanged {key, value}`, `hotkeyChanged
{command, accel}` (the accelerator in the catalog's form, `Cmd+Shift+J`), `windowAction`.

**Parts.** `settingsTitlebar` (optional, desktop), `discardChanges`, `settingsSearch`,
`settingsClearSearch`, `navGeneral`, `navCapture`, `navShortcuts`, `navAbout`,
`settingsIntro`, `settingsEmptyTitle`, `settingsEmptySubtitle`, `themeSystem`, `themeLight`,
`themeDark`, `reducedMotion` (switch), `showCompleted` (switch), `captureModeFullscreen`,
`captureModeRegion`, `hotkey<Command>` (a keycap per catalog command, e.g. `hotkeyNewTask`),
`hotkeyError`.

**Rules.**
- Four panes: **General** (Appearance: Theme System/Light/Dark, Reduced motion; Tasks: Show
  completed tasks), **Capture** (Screenshots: Capture mode Full screen/Region, Saved to: the
  folder, opens it), **Shortcuts** (Global, Main window, Capture bar: a keycap per command),
  **About** (Taskscape and its version, the data folder, Restore defaults).
- **Every change applies at once** and is saved (`settingChanged`). The title bar's undo
  ("Discard changes") puts back what the window opened with; a group's own undo does it for
  that group.
- **Search** spans every pane: each pane in the sidebar shows its number of matches, the
  results are grouped "<pane> · <group>". Choosing a pane with matches keeps the search,
  narrowed to it. Nothing: "No settings match" / "Nothing here for “<text>”."
- **Shortcuts**: a press on a keycap records ("Type shortcut…"); the next combination is saved
  (`hotkeyChanged`); Backspace unbinds it, Escape cancels. A combination another command in the
  same scope has is refused, with the reason under the row (naming that command). A changed
  shortcut has a reset button.
- Escape clears the search, else closes Settings. ↑/↓ move between panes.
- Reduced motion is forced on (and the switch disabled) by Low Power Mode or the system's
  setting, and says which.

**Texts.** The intro on Shortcuts: "Click a shortcut to change it — press the new keys,
Backspace to remove, Esc to cancel." Row titles and descriptions as in `tauri/main/src/windows/
settings/*.tsx`.

**WHY.md**: in the Tauri app Settings is a window built fresh on every open. A port draws it in
the main window (a sheet or a panel) or as a native window that opens ready; either way the
contract mounts it on its own at 640×520, and the window-only title bar is optional.

### `taskscape.filter`: sort and filter

In the rail, 320×708. **Props.** `seed`, `list`. **Events.** `viewChanged {listId, sort, dir,
filter}`, `closed`. **Parts.** `filterSummary`, `showAll`, `showActive`, `showCompleted`,
`notesHas`, `filterReset`, `filterDone`, `filterClose`.

**Rules.** Display only: it never changes a task. Sort (Manual, Date created, Date updated,
Alphabetical, Done last; ascending or descending), Show (All, Active, Done), Refine (Notes,
Attachments, Subtasks: Any, With, Without), Date (Created or Updated; Any time, Last 24 hours,
7 days, 30 days, Custom…). A filter keeps the parents of what it shows, for context ("Done"
shows 5 of the launch plan's 8: the 3 done and 2 parents). The footer: "Showing N of M tasks"
(in split view, per pane); Reset (disabled when nothing is on), Done. In split view it edits both
panes. The pane's footer control reads "Filtered" while a view is on.

### `taskscape.trash`: the Trash

In the rail. **Props.** `seed`. **Events.** `taskRestored {id}`, `taskPurged {id}`,
`confirmAsked`, `closed`. **Parts.** `dialogClose` (optional, desktop), `restoreAll`,
`emptyTrash`, `trashEmptyTitle`, `trashedNTitle`, `trashedNWhen` ("Groceries · deleted just
now"), `trashedNRestore`, `trashedNPurge`.

**Rules.** The deleted tasks, newest first; a deleted task's deleted subtasks go with it and
aren't listed. Restore puts a task (and its subtree) back; Delete permanently needs no
question; Restore all; Empty asks first ("Empty Trash?"). Empty: "Trash is empty". **Direction**:
the review would make the Trash a destination of its own rather than a rail tenant; the contract
holds what it shows, not where.

### `taskscape.toast`: the toast

**Props.** `seed`, `trashed` (the task just deleted). **Events.** `taskRestored`. **Parts.**
`toastMessage`, `toastAction`, `toastDismiss`. **Rules.** One at a time, bottom centre, for 6 s:
"Moved to Trash" (or "N moved to Trash") with Undo, and ×. Undo restores. It rises in.

## Keyboard

The catalog is `tauri/common/src/hotkeys.rs` (its frontend twin `common-ui/src/hotkeys.ts`);
every binding can be changed in Settings. `platforms.yaml` names the ones the tests send, and
gives the web its own chords where the browser keeps ⌘N, ⌘T, ⌘W, ⌘L, ⌘1–9 or ⌘⇧N for itself.
Without a modifier, in the main window: ↑/↓/←/→, Tab/⇧Tab, Space (select), ⌥Space (done),
Enter/F2 (rename), Escape (closes the rail's Filter or Trash, else clears the selection, else the
focus), and a letter starts a search.

**Do not copy** two things the Tauri app does with Enter (GAPS-P0.txt): its window-wide Enter
renames the focused task even when a button has the focus, so Enter can't press a button there;
and a sheet's Enter accepts even when Cancel has the focus. The contracts don't test them, and
they don't ask for keyboard checks where the app fails them.

## Platform freedom, in short

The behaviour, the texts, the events and the props are the same everywhere. Each platform draws
its own window, window controls, title bar mechanics, text fields, scrollbars, tooltips and file
pickers. The product's own surfaces (rows, the inspector, the palette, sheets, toasts, empty
states, the capture card) keep the layout, colours and motion in DESIGN.md. The web port is a
desktop-style app in the browser at the window's size, not a responsive one: Taskscape's layout
(a tab strip, a pane and a 320 px drawer) has no small-screen form in the design, and the
review didn't ask for one. If P2 wants a narrower size, it says so in its GAPS file.
