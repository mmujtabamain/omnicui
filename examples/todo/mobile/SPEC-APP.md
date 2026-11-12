# Todo on phones: the app specification

The todo app for phones: Android (`apps/android`, Jetpack Compose) today, iOS (SwiftUI) next.
This is what an iOS version must do to pass the same contracts (`omnicui/contracts/`). Written
from the Android version, which passes them.

## 1. Native, and the same behaviour

Each phone app is **entirely native**. Android is plain Material 3 as Google ships it (dynamic
colour, the platform font, stock components and motion); iOS should be the iOS system look in
the same way (system colours, SF, standard controls, a navigation stack, alerts or sheets). The
contracts say `same: none`: **no pixels, colours, layout or motion are compared** between the
phones. Screenshots are taken for people to look at, not compared. Nothing is copied from the
desktop app either: the desktop todo app is a separate project (`../desktop`) with no checks
against the phones.

What must be the same is **behaviour**: the data rules and fixtures (§2–3), every event with its
fields and when it is reported, every part named in §4 with its role, text, name and state, the
texts in §4 exactly, and navigation as the events and parts show it.

Words: a **component** is what omnicui mounts (one per screen, plus the whole app); a **part** is
a widget a test acts on or reads, found by its tag; an **event** is what the component reports.

## 2. Data and rules

A **task** is `{ id: string, title: string, notes: string, done: boolean }`.

- **Ids** are `t` and a number. A new task's id is `t` followed by one more than the highest
  number used so far in this store, seeds included and deleted tasks included: with the `three`
  seed the first new task is `t4`; delete it and the next is `t5`; with `empty`, `t1`.
- **Adding** trims the title (leading and trailing white space). A title that is empty after
  trimming adds nothing and reports nothing. A new task is not done, has empty notes, and goes at
  the **end** of the list.
- **Order** is the seed's order, then the order of adding. Toggling or editing never reorders.
- **Filter**: `all`, `active` (not done) or `done`. Under `all`, done tasks are shown only while
  **Show completed** is on; `active` and `done` ignore that setting.
- **Left** is the number of tasks not done, whatever the filter: "N left" (also "1 left", "0 left").
- **Theme**: `system` (follow the device's light or dark appearance), `light` or `dark`. It applies
  at once, to every screen, settings included.
- **Events are reported for real changes only**: choosing the filter or theme already chosen,
  setting a task's done to what it is, or renaming to the same title reports nothing.
- **Editing**: title and notes are saved at every change of the text (every keystroke), each one
  reported. The title is saved as typed (not trimmed).
- **Deleting** removes the task at once (no confirmation, no undo).
- The state lives in memory for the app's lifetime; nothing is persisted.

## 3. Fixtures

The `seed` prop chooses the tasks a component starts with. Every version ships this file **byte
for byte** (the canonical copy is `examples/todo/fixtures/seeds.json`; Android ships it as
`ui/src/main/res/raw/seeds.json`, the web app as `src/seeds.json`). Its SHA-256 is
`c1ace19804fea36d7f4a6e1b96c2e183c518741010339f09c058615992a51d2e` (2577 bytes, UTF-8, LF line
ends, a final newline); the Android and web apps' unit tests check it.

- `empty`: no tasks.
- `three`: Buy milk (active), Call the plumber (done), Book train tickets (active): 2 left.
- `many`: 20 tasks, `t1`–`t20`, every third one done (`t3`, `t6`, …, `t18`): 14 left. It is longer
  than a phone screen, so the list scrolls.

```json
{
  "empty": [],
  "three": [
    {
      "id": "t1",
      "title": "Buy milk",
      "notes": "2 litres, semi-skimmed",
      "done": false
    },
    {
      "id": "t2",
      "title": "Call the plumber",
      "notes": "",
      "done": true
    },
    {
      "id": "t3",
      "title": "Book train tickets",
      "notes": "Friday 18:05 to Leeds",
      "done": false
    }
  ],
  "many": [
    {
      "id": "t1",
      "title": "Renew passport",
      "notes": "",
      "done": false
    },
    {
      "id": "t2",
      "title": "Water the plants",
      "notes": "",
      "done": false
    },
    {
      "id": "t3",
      "title": "Pay the electricity bill",
      "notes": "",
      "done": true
    },
    {
      "id": "t4",
      "title": "Return library books",
      "notes": "",
      "done": false
    },
    {
      "id": "t5",
      "title": "Book a dentist appointment",
      "notes": "",
      "done": false
    },
    {
      "id": "t6",
      "title": "Clean the fridge",
      "notes": "",
      "done": true
    },
    {
      "id": "t7",
      "title": "Send birthday card to Sam",
      "notes": "",
      "done": false
    },
    {
      "id": "t8",
      "title": "Fix the bike light",
      "notes": "",
      "done": false
    },
    {
      "id": "t9",
      "title": "Back up the laptop",
      "notes": "",
      "done": true
    },
    {
      "id": "t10",
      "title": "Order printer ink",
      "notes": "",
      "done": false
    },
    {
      "id": "t11",
      "title": "Plan the weekend hike",
      "notes": "",
      "done": false
    },
    {
      "id": "t12",
      "title": "Cancel the gym trial",
      "notes": "",
      "done": true
    },
    {
      "id": "t13",
      "title": "Defrost the freezer",
      "notes": "",
      "done": false
    },
    {
      "id": "t14",
      "title": "Reply to the landlord",
      "notes": "",
      "done": false
    },
    {
      "id": "t15",
      "title": "Buy a present for Ana",
      "notes": "",
      "done": true
    },
    {
      "id": "t16",
      "title": "Change the bed sheets",
      "notes": "",
      "done": false
    },
    {
      "id": "t17",
      "title": "Update the CV",
      "notes": "",
      "done": false
    },
    {
      "id": "t18",
      "title": "Take out the recycling",
      "notes": "",
      "done": true
    },
    {
      "id": "t19",
      "title": "Sharpen the kitchen knives",
      "notes": "",
      "done": false
    },
    {
      "id": "t20",
      "title": "Call grandma",
      "notes": "",
      "done": false
    }
  ]
}
```

## 4. Components

Each component is mounted on its own, with a fresh state made from its props; props a test leaves
out take the defaults below. Every screen is drawn in the app's theme (light, dark or the system's), so a screen mounted
on its own is themed too. Props are primitives only.

Roles are WAI-ARIA's as omnicui reads them: on Android from Compose semantics, on iOS from the
accessibility element (traits). **text** is what a
part shows; **name** is what a screen reader announces.

### 4.1 `todo.list`: the task list

The first screen: the list of tasks, with the filter, the count and the way to add one. Android
draws a top app bar "Tasks", the filter as a segmented button, the count, the rows and a floating
add button; iOS draws it the iOS way (a navigation title, a segmented picker, a list, an add
button in the toolbar, for example).

| Prop | Type | Default |
|---|---|---|
| `seed` | string: `empty`, `three`, `many` | `three` |
| `showCompleted` | boolean | `true` |

| Event | Fields | When |
|---|---|---|
| `added` | `id`, `title` | a task was added (title trimmed) |
| `toggled` | `id`, `done` | a row's checkbox changed a task |
| `opened` | `id` | a row was pressed (not its checkbox) |
| `filterChanged` | `filter`: `all` / `active` / `done` | another filter was chosen |

| Part | Role | What it is; what the tests read |
|---|---|---|
| `tasksHeading` | heading | the screen's title; text "Tasks" |
| `filterAll`, `filterActive`, `filterDone` | button | the filter, a single-choice segmented control; texts "All", "Active", "Done"; `selected` true on the chosen one (All at first) |
| `left` | status | text "N left" (§2) |
| `add` | button | opens the "New task" dialog; name "Add task" (an icon-only button names itself so) |
| `row1`, `row2`, `row3` | button | the first, second, third row shown, whichever task each holds (rows are parts **by position**: tag every row, `row4` and on too); text: the task's title; pressing it reports `opened` |
| `row1Check`, `row2Check`, `row3Check` | checkbox | that row's checkbox; name: the task's title; checked: done; pressing it toggles the task and reports `toggled` |
| `empty` | status | shown instead of the rows when none is shown (below) |
| `newTitle` | textbox | the dialog's title field; name "Title"; focused when the dialog opens; empty |
| `confirmAdd` | button | text "Add"; **disabled** while the title is blank after trimming |
| `cancelAdd` | button | text "Cancel" |

Behaviour:
- **Empty states** (the `empty` part's text): under Done, "Nothing done yet"; otherwise, if there
  are no tasks at all, "No tasks yet"; otherwise (all hidden or done), "Nothing left to do".
- **The dialog** ("New task", a modal): `add` opens it with `newTitle` focused. Add (or Enter in
  the field, when the title isn't blank) adds the task, closes the dialog and reports `added`
  once. Cancel, or the platform's back or dismiss gesture, closes it with nothing added.
  The list keeps its filter; a new task under Done is added but not shown.
- Pressing a row reports `opened` and nothing else (the list doesn't navigate on its own; the app
  does: §4.4). The checkbox does not open the task.
- A done task's title may be drawn differently (Android strikes it through in a muted colour).

### 4.2 `todo.detail`: one task

| Prop | Type | Default |
|---|---|---|
| `seed` | string: `empty`, `three`, `many` | `three` |
| `taskId` | string | `t1` |

| Event | Fields | When |
|---|---|---|
| `renamed` | `id`, `title` | every change of the title field's text, with the whole new title |
| `notesChanged` | `id`, `notes` | every change of the notes field's text |
| `toggled` | `id`, `done` | the Done switch flipped |
| `deleted` | `id` | Delete was pressed (the task is removed) |
| `closed` | `id` | the screen's own back/close control was pressed |

| Part | Role | What it is |
|---|---|---|
| `title` | textbox | the title, editable in place; name "Title" |
| `notes` | textbox | the notes, multi-line; name "Notes"; empty text when there are none |
| `done` | switch | name "Done"; checked: done |
| `delete` | button | name "Delete" (an icon button in Android's top app bar) |
| `detailBack` | button | the navigation bar's back control; name **"Back"** |

Behaviour: typing appends where the caret is, and **focusing a text field from the keyboard or
from code puts the caret at the end of its text** (Compose's TextFieldState does). A test types " today" into "Buy milk" and expects "Buy milk today". Delete
reports `deleted` and not `closed`; the back control reports `closed` and not `deleted`. A
`taskId` that doesn't exist shows no task (not tested).

### 4.3 `todo.settings`

| Prop | Type | Default |
|---|---|---|
| `theme` | string: `system`, `light`, `dark` | `system` |
| `showCompleted` | boolean | `true` |

| Event | Fields | When |
|---|---|---|
| `themeChanged` | `theme` | another theme was chosen |
| `showCompletedChanged` | `show` | the switch flipped |
| `closed` | (none) | the screen's back control was pressed |

| Part | Role | What it is |
|---|---|---|
| `themeSystem`, `themeLight`, `themeDark` | button | a single-choice segmented control under a "Theme" label; texts "System", "Light", "Dark"; `selected` on the chosen one |
| `showCompleted` | switch | name "Show completed"; checked: on |
| `settingsBack` | button | the navigation bar's back control; name **"Back"** |

The chosen theme applies to this screen at once (choosing Dark darkens it, whatever the system).

### 4.4 `todo.app`: the whole app

| Prop | Type | Default |
|---|---|---|
| `seed` | string: `empty`, `three`, `many` | `three` |

Events: `navigated {screen, id}` (below), and the screens' `opened`, `added`, `toggled`,
`renamed`, `deleted`, `themeChanged`, `showCompletedChanged` with the fields above (an app may
report its screens' other events too; the contract ignores them).

Parts: `openSettings` (button, name "Settings"), and the screens' parts by the same names: `tasksHeading`,
`left`, `add`, `newTitle`, `confirmAdd`, `row1`, `row2`, `row1Check`, `title`, `done`, `delete`,
`detailBack`, `themeSystem`, `themeDark`, `showCompleted`, `settingsBack` (§5 says how they are
found).

Navigation (one screen at a time, a navigation stack):
- The app starts on the list and reports nothing for that.
- Pressing a row pushes that task's detail (reports `opened`, then `navigated {screen: "detail", id}`).
- `openSettings` (Android: an icon in the list's top app bar; iOS: a toolbar button) pushes
  settings (`navigated {screen: "settings", id: ""}`).
- **Back returns to the list** from a task or from settings: the screen's back control
  (`detailBack`, `settingsBack`), the platform's back (`input: back`: Android's system back; on
  iOS whatever the target implements as `do: back`, the back button or the edge swipe), and
  Delete in the detail. Each reports `navigated {screen: "list", id: ""}`.
- `navigated` is reported once for every change of the screen shown after the first; `id` is the
  task's id on the detail and "" otherwise.
- Changes made on one screen show on the others: a title edited in the detail is the row's new
  text, Done in the detail checks the row, Show completed off hides done tasks under All.

## 5. Tagging parts

- Each part carries the tag `<component>.<part>`: `todo.list.row1Check`, `todo.detail.title`,
  `todo.settings.themeDark`, `todo.app.openSettings`. Android: `Modifier.testTag`; SwiftUI:
  `.accessibilityIdentifier(...)`.
- Rows are tagged by **position among the rows shown**: the top row is `todo.list.row1`, its
  checkbox `todo.list.row1Check`, and so on for every row; after a filter or a toggle the tags
  follow the rows, not the tasks.
- **`todo.app`'s parts** are the screens' parts: a target resolves `todo.app.<name>` to the
  element tagged `todo.app.<name>` if there is one, else to the shown element tagged
  `todo.<screen>.<name>` (part names are unique across the screens for that reason). The app
  shell's own `openSettings` is tagged `todo.app.openSettings`. The Android target implements this rule
  in its registry (`Components.tagMatches`).
- A tag goes on the element that carries the role: on the clickable row (not its text), on the
  checkbox itself, on the switch's toggleable row, on the text field.
- What a part's name is made of: its accessibility label (content description,
  `aria-label`, `accessibilityLabel`), else its label element, else its text.

## 6. Look

- Free: each phone's own (§1). The `theme` setting chooses light, dark or the system's
  appearance, and applies at once to every screen, settings included.
- The contracts run at **360×640**, light and dark. On Android 1 dp = 1 px (mdpi); an iOS target
  should draw at 1× (1 pt = 1 px) at 360×640 points. A status bar or home indicator that the
  rendering includes is reported as a system region in the screenshot reply.

## 7. Keyboard and focus

- With a hardware keyboard, every button, switch and checkbox can take focus and is activated by
  **Space** and by **Enter**, as a tap activates it (the contracts generate a test per part and
  key; Compose does both). A target whose platform doesn't use one of those keys for a role says
  so in `advertise` (`keys`); one that can't send keys or move focus at all says so in `cannot`,
  and those tests are "not advertised" there.
- Space on a focused row checkbox toggles it; Enter on a focused theme segment chooses it; Space
  on the focused Done switch flips it, and the part stays focused.
- Enter in the dialog's title field adds the task (when the title isn't blank).

## 8. Notes for iOS

- **target.yaml**: `platform: [mobile]`, `design: cupertino`; the adapter reports
  `"platform": {"os": "ios"}` in `advertise`.
- **Back**: `platforms.yaml` maps `input: back` to `{ do: back }` on iOS: advertise `do.back` and
  perform it as the user does (the navigation bar's back button, or the edge swipe).
- **Back's name**: SwiftUI's default back button announces the previous screen's title, so give
  `detailBack` and `settingsBack` the accessibility label "Back" (a custom toolbar button, or
  `.accessibilityLabel`). If that is truly impossible, the contract can take a per-platform value
  with a `why:`; that is the only kind of text exception the mobile contracts allow.
- **Segmented controls**: tag each segment of the `Picker(.segmented)` and expose `selected`.
- **The add dialog**: an `.alert` with a text field, or a sheet; tag its field `newTitle` and its
  buttons `confirmAdd` and `cancelAdd`. Add must be disabled while the title is blank.

## 9. Checking a new version

1. Build the app with these screens, the fixtures file and the tags.
2. Write a target that mounts `todo.list`, `todo.detail`, `todo.settings` and `todo.app` from
   props (its registry) and reports events by their contract names (`../README.md` describes the
   Android target).
3. `bin/omnicui --project examples/todo/mobile/omnicui check <target>`, and fix what fails.
