# Taskscape: the design, for the porters

This is the design every version of Taskscape keeps. It is taken from the Tauri app
(`../tauri/`): its tokens (`tauri/common-ui/styles/tokens.css`), its components, and the
window measured at its default size. The rule is **consistent but idiomatic**: the layout, the
colours and the motion are the product's and stay the same; the chrome and the native controls
are each platform's own.

The contracts hold it with `same: [layout, colours, motion]` (never pixels: the versions are
drawn by different engines with different fonts). A part a platform draws as its own control
keeps only its layout; a part only some platforms have is `optional` and keeps nothing visual.
The table at the end says which is which.

Where the Tauri app and the design review (`tauri/docs/main-app-design-review-2026-08-13.md`,
"Direction From Your Answers") disagree, SPEC-PRODUCT.md says what the ports do. This file
describes the look, which the review keeps.

## Units

The app sets the root font size to **14 px** (`main/src/index.css`), and every `rem` in the
tokens and in Tailwind's spacing scale is relative to it. So tokens.css's comments (written for
a 16 px root) overstate the sizes by 8/7. The numbers below are what is drawn, in CSS pixels
(logical points on macOS). The capture bar is the tray app's, which keeps the 16 px default:
there the tokens are as tokens.css writes them.

## Colour

Every colour is a token with a light and a dark value. These are tokens.css's `oklch()` values
converted to sRGB (the conversion script is in the P0 report, GAPS-P0.txt). Eight-digit values
carry alpha: they are drawn over the surface beneath.

| token | light | dark | used for |
|---|---|---|---|
| `surface-0` | `#ebecee` | `#0f1012` | recessed wells: the search well, the project pill, segmented controls |
| `surface-1` | `#f3f4f6` | `#161719` | the window and the title bar; the preview rail |
| `surface-2` | `#fdfdfe` | `#1e1f22` | the task pane; sheets; the palette |
| `surface-3` | `#ffffff` | `#292b2e` | raised: menus, the toast, cards |
| `edge-1` | `#2d333d13` | `#f5f9ff0f` | row separators, faint hairlines |
| `edge-2` | `#2d333d24` | `#f5f9ff1c` | pane borders, the title bar's bottom edge |
| `edge-3` | `#2d333d4c` | `#f5f9ff3d` | strong borders: the round check's ring |
| `content-1` | `#21242a` | `#e9ebee` | primary text |
| `content-2` | `#545860` | `#a8abb0` | secondary text: notes previews |
| `content-3` | `#82868e` | `#74787e` | muted text, icons |
| `accent-500` | `#007df3` | `#3a9bfd` | the accent: progress, the inspector's tick, focus |
| `accent-600` | `#006ce7` | `#53abff` | accent, hover |
| `accent-700` | `#005bd6` | `#268beb` | accent, pressed |
| `on-accent` | `#fafcff` | `#fafcff` | text and glyphs on the accent |
| `selection-1` | `#007df31a` | `#3a9bfd29` | a selected row's wash |
| `focus-1` | `#007df380` | `#3a9bfd99` | the focus ring (2 px, 1 px offset) |
| `wash-1` | `#2d333d0d` | `#f5f9ff0b` | hover wash on rows and tabs |
| `wash-2` | `#2d333d17` | `#f5f9ff14` | hover wash on buttons and the pill |
| `danger-500` | `#cc2827` | `#f75d59` | destructive actions |
| `danger-600` | `#bb0916` | `#ff6f69` | destructive, hover |
| `danger-100` | `#cc28271a` | `#f75d5926` | destructive soft fill |
| `success-500` | `#319751` | `#62bb78` | success |
| `done-lamp-1` | `#4c88d3` | `#2f82d6` | the done check's fill |

The window's native background is `surface-1` (`#f4f4f6` / `#161719` in `main/src-tauri/src/lib.rs`):
a native window shows it before the first frame. The tray's capture bar is one opaque card in
`surface-2` with an `edge-2` border.

Dark is chosen by the saved Theme setting (System, Light or Dark), else the system appearance.

## Type

One family: the **system UI font** (`system-ui`; SF Pro on macOS). There is no display face:
`--font-display` is the same font, and hierarchy comes from size and weight. A port uses the
platform's system font; the contracts never compare pixels, so the fonts may differ.

| role | size | weight | other |
|---|---|---|---|
| body, a task's title in a row | 14 | 500 (title), 400 (body) | line 19 |
| a row's notes preview | 12.5 | 400 | line 17, `content-2` (done: muted, 55 % opacity) |
| a done task's title | 14 | 500 | `content-3`, struck through |
| list tab | 13 | 500 | tracking 0.01em; `content-1` focused, `content-3` otherwise |
| project pill | 14 | 600 | |
| inspector title | 18 | 600 | line 24, wraps |
| inspector metadata | 11.5 | 400 | tabular numbers |
| section header (NOTES, SUBTASKS) | 10.5–11 | 600 | upper case, wide tracking |
| footer stats ("3/8 DONE") | 11 | 600 | upper case, tracking 0.08em, tabular |
| sheet headline | 15 | 600 | detail 12.5 |
| palette group | 10.5 | 600 | upper case, widest tracking |
| empty state | 15 title, 12.5 subtitle | 500 / 400 | centred under a 28–30 px icon |

Icons are Material Symbols Outlined (`material-symbols` 0.45.6, the app ships a subset), at
weight 350 in light and 300 in dark (light glyphs on dark render heavier).

## Spacing, radii, elevation

Spacing steps (`--spacing-space-N`), as drawn at the 14 px root:

| step | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 |
|---|---|---|---|---|---|---|---|---|
| px | 1.75 | 2.6 | 3.5 | 5.25 | 7 | 8.75 | 12.25 | 17.5 |

Tailwind's own numeric spacing is 3.5 px per step (`p-4` is 14 px).

| radius | px | for |
|---|---|---|
| `field` | 4.4 | inputs, small chips, the done check |
| `control` | 7 | buttons, tabs, menu items, the pill |
| `panel` | 10.5 | cards, sheets, menus, the palette, the capture bar |

Elevation is two shadows, both in a dark blue-grey (`oklch(0.2 0.02 262)`, `#14171c`):
- **menu**: `0 10px 28px` at 20 % and `0 1px 3px` at 12 %: menus, sheets, the palette;
- **lift**: `0 2px 8px` at 20 %: the toast, a note's delete button.

## Motion

Durations and easings, from tokens.css:

| name | duration | easing | what moves | where |
|---|---|---|---|---|
| rise | 110 ms | `cubic-bezier(0.2, 0, 0, 1)` | up 3 px and in from transparent | the inspector, draft and multi inspectors, the toast |
| panel-in | 120 ms | `cubic-bezier(0.2, 0, 0, 1)` | in from 6 px right, from transparent | a Settings pane |
| sheet-in | 140 ms | `cubic-bezier(0.2, 0, 0, 1)` (`--ease-standard`) | scale 0.96 → 1, from transparent | sheets, the palette, the project menu |
| scale-in | 280 ms | `cubic-bezier(0.22, 1, 0.36, 1)` (`--ease-out`) | scale 0.97 → 1, from transparent | the Settings window |
| check draw | 160 ms after 80 ms | `cubic-bezier(0.2, 0, 0, 1)` | the tick's stroke drawn in | the done check |
| notes open | none | | the capture bar's notes replace the Tab hint at once, and the window fits the card | the capture bar (`.llm/tray-app.md` describes a grid-row transition the code no longer has) |

Also defined: `--duration-fast` 180 ms, `--duration-base` 280 ms, `--duration-slow` 500 ms,
`--ease-exit` `cubic-bezier(0.3, 0, 1, 1)`.

**Reduced motion** (the system's setting, the app's own Settings switch, or macOS Low Power
Mode) cuts every animation and transition to nothing. Toggles never animate at all.

## The window at 1180×760

The default size (`main/src-tauri/tauri.conf.json`; minimum 880×560). Measured from the Tauri
app; a port keeps these proportions (the contracts' `layout` measure allows 10 % of the width).

```
0                                                                     860       1180
┌─────────────────────────────────────────────────────────────────────────────────┐ 0
│ ● ● ●  ⠿ [Atlas ⇕] │Launch plan ×│Groceries ×│Inbox ×│ +        ⫼  ▣  ⚙        │ title bar
├────────────────────────────────────────────────────────────────────┬────────────┤ 46
│ [🔍 Search Launch plan                                     ⚙  │ + ]│            │
│ ○ Write the launch brief                          1/3  📎2  ···    │  No task   │
│   ○ Collect the feature list                                ···    │  selected  │
│   ○ Draft the summary                                       ···    │            │
│ …                                                                  │ preview    │
│                                                                    │ rail       │
├────────────────────────────────────────────────────────────────────┤            │
│ ⚙ │ 3/8 DONE                                        ▬▬▬▬──── 38% │            │ 729
└────────────────────────────────────────────────────────────────────┴────────────┘ 760
```

- **Title bar**: the full width, 46 px, `surface-1`, an `edge-2` line under it. From the left:
  the window controls (84 px; desktop only), a drag glyph, the project pill (28 px tall,
  `surface-0`, 7 px radius), the list tabs (each the bar's full height, 13 px labels, a delete ×,
  thin separators; the focused tab is filled `surface-2`), "+" for a new list, a flexible drag
  strip, then the view buttons, 28×28 each, 5 px apart: swap panes (in split view only), split
  view, the preview rail, Settings. The whole bar drags the window.
- **Task pane**: the window's width less the rail (860 px), `surface-2`.
  - The search well: 35 px tall, 14 px in from the sides, 10.5 px from the top, `surface-0`,
    holding the search field, the search options, a divider and "+". In split view a ×
    closes the pane.
  - The rows: at least 31.5 px, 46 px with a notes line. Each depth indents 24 px. Left to right:
    20 px of gutter, the round check (16 px), the title and its notes line, then the subtask
    count and attachment badges, and "···". The separator under a row starts at the text column
    (24 px per depth + 44 px), never under the check.
  - The footer: 32 px, an `edge-2` line above: the sort and filter control, "3/8 DONE", and a
    progress bar with its percentage at the right. With tasks selected it is the bulk bar instead.
- **Preview rail**: 320 px (the user resizes it between 280 and 420), `surface-1`, an `edge-2`
  line on its left. It holds the inspector, the draft, the multi inspector, the filter panel or
  the Trash. Hidden, the pane takes the whole width.
- **Split view**: two panes side by side, half each (the user drags it between 25 % and 75 %),
  an `edge-2` line between.
- **Over the window**: sheets, centred over a dimmed window (at most 26 rem wide); the palette, at the
  top (at most 35 rem wide, at most 60 % of the height); the toast, centred 17.5 px above the
  bottom; menus at the pointer or under their button.
- **Settings**: 640×520 (fixed): its own 46 px title bar, a 154 px sidebar (search, then the
  four panes), and the pane's groups on `surface-2`.
- **The capture bar**: 480 px wide, as tall as its content: the title row (40 px), the notes
  (behind Tab), a hairline, and the footer with where the task goes and the screenshot button.

## What each platform draws

"Tokens" means drawn by the app from the tokens above, the same everywhere. "Platform" means the
platform's own control or chrome; the contracts then compare its layout only, or nothing.

| control | tauri (macOS, webview) | gpui (desktop) | web | contracts |
|---|---|---|---|---|
| window frame, shadow, corners, resizing | platform | platform | the browser | — |
| title bar | tokens: the app's own bar in the window's title area | tokens: the same bar | tokens: the page's header, no window behind it | `layout, colours, motion` |
| window controls (traffic lights) | drawn to match macOS's | the platform's own (macOS traffic lights; others theirs) | none | `same: none`, `optional` |
| text fields (search, titles, sheet fields) | HTML inputs styled from tokens | GPUI's own, styled from tokens | HTML inputs | `layout` |
| rich-text notes | Tiptap editor | the port's editor | the port's editor | `layout` |
| buttons, icon buttons, tabs, chips, badges | tokens | tokens | tokens | `layout, colours, motion` |
| the round check, checkboxes, switches, segmented controls | tokens | tokens | tokens | `layout` (the segments and switches); the check: `layout, colours, motion` |
| task rows, the inspector, the palette, sheets, the toast, empty states | tokens | tokens | tokens | `layout, colours, motion` |
| context menus | tokens (drawn in the page) | tokens, or the platform's menu | tokens | `layout` |
| scrollbars | WebKit's, thin and themed | the platform's | the browser's | — |
| tooltips | the webview's (`title`) | the platform's or GPUI's | the browser's (`title`) | — |
| file pickers | the platform's (Tauri dialog) | the platform's | the browser's | — |
| fonts | the system UI font | the system UI font | the browser's system UI font | never pixels |

Viewports: the contracts mount each surface at the size it has in the window (the pane 860×708,
the rail 320×708, the title bar 1180 wide, the window 1180×760, Settings 640×520, the capture
bar 480 wide). The web port is a desktop-style web app at the same sizes; it is not responsive,
so there is no narrower web size (SPEC-PRODUCT.md says why).
