# The desktop todo app: structure and palette

One product design for every desktop version of the todo app: the web app (`apps/web`) and a
macOS app. They share **the regions, their order and proportions, and the colours**. Each draws
them **the way its platform does**: the web app looks like a good web app, the Mac app like a Mac
app (its title bar and toolbar, its sidebar, its system controls and font). The contracts hold
`same: [layout, colours, motion]` between them, and `[layout]` only for the native controls
(below); pixels are never compared. The behaviour is in `../SPEC-APP.md`.

Numbers are at the contracts' window size, **1024×720** (the Mac's content area, below its title
bar, which omnicui cuts off). Boxes are `x, y, w, h`, measured from the web app; a Mac version
matches them to within a few points, as its controls' own sizes allow. Layout is compared by
each part's box relative to the component, and by reading order.

## Regions

```
┌──────────────────────────── header / toolbar, 56 high ─────────────────────────────┐
│ Todo                                                                     Settings   │
├──────────── sidebar, 340 wide ───┬──────────────── detail pane, the rest ───────────┤
│ Tasks               [+ Add task] │  Task                                        [×]  │
│ [ All | Active | Done ]          │  Title  [...........................]             │
│ 2 left                           │  Notes  [...........................]             │
│ ☐ Buy milk                       │  Done                                    (  ●)    │
│ ☑ Call the plumber               │  [Delete]                                         │
│ ☐ Book train tickets             │                                                   │
└──────────────────────────────────┴───────────────────────────────────────────────────┘
```

| Region | Box at 1024×720 | Notes |
|---|---|---|
| header (Mac: the window's toolbar) | 0, 0, 1024, 56 | the app's name at the left in the accent colour; **Settings** at the right (901, 10, 107, 36) |
| sidebar | 0, 56, 340, 664 | the task list; a 1 px divider at its right edge; its own background (`sidebar`) |
| · sidebar head | 0, 56, 340, 64 | "Tasks" (a heading, 20, 74) at the left, **Add task** at the right (208, 72, 115, 36) |
| · filter | 16, 120, 307, 34 | All, Active, Done: a segmented control the sidebar's width, less 16 each side |
| · count | 20, 166, 299, 17 | "N left", small, muted |
| · rows | from y 191, 40 high each, 8 in from each side | checkbox, then the title; the open task's row in `selected` |
| detail pane | 340, 56, 684, 664 | the open task; "Choose a task to see its details." centred when none is open |
| · pane head | 372, 76, 560, 36 | "Task" (a heading) at the left, **Close** (an icon button ×, 36×36) at the right |
| · Title field | 372, 149, 560, 43 | a label above it ("Title"); one line |
| · Notes field | 372, 230, 560, 108 | a label above it ("Notes"); four lines |
| · Done | 372, 354, 560, 48 | "Done" at the left, the switch at the right (880, 362, 52, 32) |
| · Delete | 372, 426, 80, 36 | a destructive button, in `danger` |
| settings page (replaces sidebar and pane; the header stays) | 192, 56, 640, 664 | a centred column 640 wide: **Back to tasks** (212, 80), "Settings" (a title, 224, 128), "Theme" with System, Light, Dark (224, 220, 266, 34), "Show completed" with its switch (224, 278, 560, 48) |
| New task dialog (modal, over a scrim) | 312, 249, 400, 223 | centred; "New task", a "Title" field (336, 344, 352, 43), then Cancel and Add at the bottom right |

The pane's content is a column 560 wide, 32 in from the sidebar. Everything keeps its place when
the window grows: the sidebar stays 340 wide, the pane takes the rest.

## Colours

The tokens, light and dark. "System" follows the OS appearance; Light and Dark force it.

| Token | Light | Dark | Used for |
|---|---|---|---|
| `accent` | `#36618E` | `#A0CAFD` | the app's name, Add task, a checked checkbox, an on switch, text buttons, focus |
| `onAccent` | `#FFFFFF` | `#003258` | text and glyphs on `accent` |
| `selected` | `#D7E3F7` | `#3B4858` | the chosen segment, the open task's row |
| `onSelected` | `#101C2B` | `#D7E3F7` | text on `selected` |
| `background` | `#F8F9FF` | `#111418` | the header, the pane, settings |
| `sidebar` | `#F2F3F9` | `#191C20` | the sidebar |
| `text` | `#191C20` | `#E1E2E8` | text |
| `muted` | `#43474E` | `#C3C6CF` | "N left", labels, done tasks' titles, the empty states |
| `raised` | `#E6E8EE` | `#272A2F` | the dialog, an off switch's track |
| `border` | `#73777F` | `#8D9199` | field and segment borders |
| `divider` | `#C3C6CF` | `#43474E` | the header's and the sidebar's 1 px lines |
| `danger` | `#BA1A1A` | `#FFB4AB` | Delete |
| `scrim` | black at 60% | black at 60% | behind the dialog |

On a Mac, the sidebar may use the system's sidebar material if it renders close to `sidebar`, and
the accent should be the app's `accent`, not the user's system accent colour: colours are
compared.

## Controls: drawn by each platform

| Control | Web | macOS | Contract |
|---|---|---|---|
| checkbox (row) | an `<input type=checkbox>` in the app's colours, 18×18, a tick that grows in over 100 ms | `Toggle` with `.checkbox` style (NSButton checkbox) | `same: [layout]` |
| switch (Done, Show completed) | a `role=switch` button, 52×32, the thumb slides over 150 ms | `Toggle` with `.switch` style (NSSwitch) | `same: [layout]` |
| segmented (filter, theme) | three toggle buttons in one bordered group | `Picker` with `.segmented` style (NSSegmentedControl) | `same: [layout]` |
| text fields | `<input>` / `<textarea>` with a border | `TextField` / `TextEditor` (NSTextField) | `same: [layout]` |
| buttons (Add task, Delete, Cancel, Add, Back to tasks, Settings, Close) | the app's own: filled, outlined, text and icon buttons, rounded | standard push buttons; Add task may be `.borderedProminent` in `accent` | layout, colours, motion |
| headings and text | the system UI font stack | the system font (SF Pro) | layout, colours |

Motion that both show:
- **the New task dialog**: it fades in and grows from 90% to 100% over 200 ms, ease-out, over a
  scrim that appears at once (on a Mac, a sheet or a custom modal with that motion);
- changing what the pane shows is instant.
