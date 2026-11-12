# Todo: the app specification

One todo product, built by hand on each platform, and checked by one set of omnicui contracts
(`omnicui/contracts/`). This document is what a version on a new platform (iOS in SwiftUI, macOS in
SwiftUI or AppKit) must do to pass those contracts. It was written from the Android
(`apps/android`, Jetpack Compose) and web (`apps/web`, React) versions, which both pass them.

Words used here: a **component** is what omnicui mounts and checks (one per screen, plus the whole
app); a **part** is a widget a test acts on or reads, found by its tag; an **event** is what the
component reports through a callback.

Contents: 1 What is the same and what is free · 2 Data and rules · 3 Fixtures · 4 Components
(`todo.list`, `todo.detail`, `todo.settings`, `todo.app`) · 5 Tagging parts · 6 Look and motion ·
7 Keyboard and focus · 8 Platform notes for iOS and macOS · 9 Checking a new version.

## 1. What is the same and what is free

**The same on every platform** (the contracts check it):
- the data rules, the fixtures, every event with its fields, and when each is reported (§2–4);
- every part named in §4: it exists when the section says it is shown, with that role, text, name,
  checked, selected, disabled and focused state;
- the texts in §4, exactly (they are what the tests read);
- navigation as the events and the parts show it (§4.4);
- the colours (§6.1), so that every platform speaks one design language; and the motion of the
  checkbox, the dialog and the switch (§6.2).

**Free on each platform:** the layout (a phone stacks screens, a desktop shows panes), the
controls' own drawing (a Material checkbox, a native one), the type, the icons, the spacing,
elevation, ripples and hover states, and how the platform goes back (a system back, an edge
swipe, a keyboard shortcut). The contracts hold the pixels of no pair of platforms the same; a
pair of different designs is compared by colours, motion and, between two phones, layout
(`same:` in each contract says it part by part, with the reason).

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
for byte** (Android `ui/src/main/res/raw/seeds.json`, web `src/seeds.json`; the canonical copy is
`fixtures/seeds.json`). Its SHA-256 is
`c1ace19804fea36d7f4a6e1b96c2e183c518741010339f09c058615992a51d2e` (2577 bytes, UTF-8, LF line
ends, a final newline); both apps' unit tests check it.

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
out take the defaults below. Every screen is drawn in the app's theme (§6.1), so a screen mounted on
its own is themed too. Props are primitives only.

Roles are WAI-ARIA's as omnicui reads them: on Android from Compose semantics, on the web from the
accessibility tree, on Apple platforms from the accessibility element (traits). **text** is what a
part shows; **name** is what a screen reader announces.

### 4.1 `todo.list`: the task list

The list of tasks, with the filter, the count and the way to add one. On a phone it is the first
screen (a top app bar "Tasks", the filter, the count, the rows, a floating add button). On the
desktop web it is the left sidebar.

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
| `newTaskDialog` | generic | the dialog's surface (its box is what is traced: §6.2) |
| `newTitle` | textbox | the dialog's title field; name "Title"; focused when the dialog opens; empty |
| `confirmAdd` | button | text "Add"; **disabled** while the title is blank after trimming |
| `cancelAdd` | button | text "Cancel" |

Behaviour:
- **Empty states** (the `empty` part's text): under Done, "Nothing done yet"; otherwise, if there
  are no tasks at all, "No tasks yet"; otherwise (all hidden or done), "Nothing left to do".
- **The dialog** ("New task", a modal): `add` opens it with `newTitle` focused. Add (or Enter in
  the field, when the title isn't blank) adds the task, closes the dialog and reports `added`
  once. Cancel, the system back on a phone, or Escape on a desktop closes it with nothing added.
  The list keeps its filter; a new task under Done is added but not shown.
- Pressing a row reports `opened` and nothing else (the list doesn't navigate on its own; the app
  does: §4.4). The checkbox does not open the task.
- A done task's title may be drawn differently (both apps strike it through in a muted colour).

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
| `delete` | button | name "Delete" (an icon button on Android, a text button on the web) |
| `detailBack` | button | leaves the detail. Name **"Back"** on phones (the top bar's up arrow) and **"Close"** elsewhere (the desktop pane's close button). This is a declared platform exception in the contract |

Behaviour: typing appends where the caret is, and **focusing a text field from the keyboard or
from code puts the caret at the end of its text** (Compose's TextFieldState does; the web app moves
it there on focus). A test types " today" into "Buy milk" and expects "Buy milk today". Delete
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
| `settingsBack` | button | leaves settings; name **"Back"** on phones, **"Back to tasks"** elsewhere (a platform exception) |

The chosen theme applies to this screen at once (choosing Dark darkens it, whatever the system).

### 4.4 `todo.app`: the whole app

| Prop | Type | Default |
|---|---|---|
| `seed` | string: `empty`, `three`, `many` | `three` |

Events: `navigated {screen, id}` (below), and the screens' `opened`, `added`, `toggled`,
`renamed`, `deleted`, `themeChanged`, `showCompletedChanged` with the fields above (an app may
report its screens' other events too; the contract ignores them).

Parts: `openSettings` (button, name "Settings": on a phone an icon in the list's top app bar, on
the desktop in the app's header), and the screens' parts by the same names: `tasksHeading`,
`left`, `add`, `newTitle`, `confirmAdd`, `row1`, `row2`, `row1Check`, `title`, `done`, `delete`,
`detailBack`, `themeSystem`, `themeDark`, `showCompleted`, `settingsBack` (§5 says how they are
found).

Navigation:
- The app starts on the list, with no task open, and reports nothing for that.
- Pressing a row opens that task (reports `opened`, then `navigated {screen: "detail", id}`).
- `openSettings` opens settings (`navigated {screen: "settings", id: ""}`).
- **Back returns to the list** from a task or from settings, whichever way: the screen's own
  control (`detailBack`, `settingsBack`), the platform's back (Android's system back, the
  browser's back button, iOS's back button or edge swipe, macOS's Cmd+[), and Delete in the
  detail. Each reports `navigated {screen: "list", id: ""}`.
- `navigated` is reported once for every change of the screen shown after the first; `id` is the
  task's id on the detail and "" otherwise.
- Phones show one screen at a time. The desktop web shows the list always, the open task in a
  pane beside it (the open row is highlighted), and settings as a page of its own with the
  header; opening another task while one is open replaces it, so back still returns to the list.
  A desktop app must show the list and the open task side by side too: the contract's
  `only: [web, desktop]` test checks that both are visible at once.
- Changes made on one screen show on the others: a title edited in the detail is the row's new
  text, Done in the detail checks the row, Show completed off hides done tasks under All.

## 5. Tagging parts

- Each part carries the tag `<component>.<part>`: `todo.list.row1Check`, `todo.detail.title`,
  `todo.settings.themeDark`, `todo.app.openSettings`. Android: `Modifier.testTag`; web: the
  `data-part` attribute; SwiftUI: `.accessibilityIdentifier(...)`; AppKit:
  `setAccessibilityIdentifier`.
- Rows are tagged by **position among the rows shown**: the top row is `todo.list.row1`, its
  checkbox `todo.list.row1Check`, and so on for every row; after a filter or a toggle the tags
  follow the rows, not the tasks.
- **`todo.app`'s parts** are the screens' parts: a target resolves `todo.app.<name>` to the
  element tagged `todo.app.<name>` if there is one, else to the shown element tagged
  `todo.<screen>.<name>` (part names are unique across the screens for that reason). The app
  shell's own `openSettings` is tagged `todo.app.openSettings`. Both reference targets implement
  this rule in their registry (`Components.tagMatches` on Android, `selector()` in the web
  adapter).
- A tag goes on the element that carries the role: on the clickable row (not its text), on the
  checkbox itself, on the switch's toggleable row or button, on the text field. When the
  element's box is traced or recorded (`newTaskDialog`), the tag must be **inside** any scale
  transform, so that its box is the box as drawn.
- What a part's name is made of: its accessibility label (content description,
  `aria-label`, `accessibilityLabel`), else its label element, else its text.

## 6. Look and motion

### 6.1 Colours

One Material 3 scheme from a single seed blue, fixed (not the device's dynamic colours). Use these
exact values; screens are compared by the colours they fill areas with.

| Role | Light | Dark | Used for |
|---|---|---|---|
| primary | `#36618E` | `#A0CAFD` | the add button, a checked checkbox, an on switch, the settings label |
| onPrimary | `#FFFFFF` | `#003258` | text and icons on primary |
| primaryContainer | `#D1E4FF` | `#194975` | the phone's FAB |
| onPrimaryContainer | `#001D36` | `#D1E4FF` | the FAB's icon |
| secondaryContainer | `#D7E3F7` | `#3B4858` | the chosen segment, the open row |
| onSecondaryContainer | `#101C2B` | `#D7E3F7` | text on it |
| surface / background | `#F8F9FF` | `#111418` | screens |
| onSurface | `#191C20` | `#E1E2E8` | text |
| onSurfaceVariant | `#43474E` | `#C3C6CF` | secondary text, done titles, "N left" |
| surfaceContainerLow | `#F2F3F9` | `#191C20` | the desktop sidebar |
| surfaceContainer | `#ECEEF4` | `#1D2024` | |
| surfaceContainerHigh | `#E6E8EE` | `#272A2F` | the dialog, an off switch's track |
| outline | `#73777F` | `#8D9199` | segment and field borders |
| outlineVariant | `#C3C6CF` | `#43474E` | dividers |
| error | `#BA1A1A` | `#FFB4AB` | Delete on the web |

The scrim behind the dialog is black at 60% (Android's dialog window dims by 0.6; the web's
`::backdrop` is `rgb(0 0 0 / 0.6)`).

### 6.2 Motion

| What | Motion | Checked by |
|---|---|---|
| The New task dialog appearing | fades in (opacity 0→1) and grows from 90% (scale 0.9→1), together, over **200 ms, ease-out** (CSS `ease-out`, `cubic-bezier(0, 0, 0.58, 1)`); the scrim appears at once | `trace` of `newTaskDialog`'s scale against that timing; a clip of its box |
| A checkbox being checked | the box fills and the tick comes in over about 100 ms | a clip of `row1Check`'s box |
| A switch flipping | the thumb slides over about 150 ms | a clip of `done`'s box |
| Changing screens | free (Android's Navigation fades; the web doesn't animate) | not checked |

### 6.3 Viewports

The contracts run phones (`platform: [mobile]`) at 360×640 and everything else at 1024×720, in
light and dark. On Android 1 dp = 1 px (mdpi), and an iOS target should draw at 1× (1 pt = 1 px)
at 360×640 points.

## 7. Keyboard and focus

- Every button, switch and checkbox can take keyboard focus and is activated by **Space** and by
  **Enter**, as a pointer press activates it (the contracts generate a test per part and key).
  A target whose platform doesn't use one of those keys for a role says so in `advertise`
  (`keys`); then that key isn't checked there.
- Space on a focused row checkbox toggles it; Enter on a focused theme segment chooses it; Space
  on the focused Done switch flips it, and the part stays focused.
- Enter in the dialog's title field adds the task (when the title isn't blank).

## 8. Platform notes for iOS and macOS

What the contracts and `platforms.yaml` already expect:
- **target.yaml**: iOS `platform: [mobile]` and its own `design:` (for example `cupertino`); macOS
  `platform: [desktop]` and its own design. The adapter reports `"platform": {"os": "ios"}` (or
  `"macos"`) in `advertise`.
- **Back**: `platforms.yaml` maps `input: back` to `{ do: back }` on iOS (the navigation bar's back
  button or the edge swipe: advertise `do.back`) and to the key `Meta+BracketLeft` (Cmd+[) on
  macOS, which must return to the list.
- **Names that differ by form**: `detailBack` and `settingsBack` are named "Back" on `mobile`.
  SwiftUI's default back button announces the previous screen's title, so give it the
  accessibility label "Back" (or a custom toolbar button named so). On macOS, "Close" and "Back to
  tasks" as on the web.
- **Segmented controls**: SwiftUI's `Picker(.segmented)` exposes its segments as buttons with a
  selected trait: tag each segment, and read `selected` from the trait.
- **The dialog**: an `.alert` with a text field, or a sheet; whichever, its surface must be tagged
  `todo.list.newTaskDialog` and fade and grow in as §6.2 says. If the platform's own presentation
  can't be retimed, keep it and expect a motion warning; don't fake the part.
- **Keyboard on iOS**: if the target can't send keys or focus, it says so in `cannot`, and the
  keyboard tests are reported "not advertised" there.
- **macOS**: a desktop layout like the web's (a sidebar list, the task beside it, settings as its
  own view or window) at 1024×720.

## 9. Checking a new version

1. Build the app with these screens, the fixtures file and the tags.
2. Write a target that mounts `todo.list`, `todo.detail`, `todo.settings` and `todo.app` from
   props (its registry) and reports events by their contract names (`README.md` shows the two
   reference targets).
3. `bin/omnicui --project examples/todo/omnicui check <target>` and fix what fails; then the same
   with every target, to see the colour and motion comparisons.
