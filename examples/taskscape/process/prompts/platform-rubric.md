# Platform tags: the rubric

The Taskscape contracts were written for the web build first. They now also run on a desktop build. Your job is to say, test by test, which platforms each test is for, and what each platform's form of it is. The binding product decisions are in `spec/decisions.md` ("decision N" below).

## The targets and the selectors

| Target | Form | Design | OS |
|---|---|---|---|
| `web`, `web-app` (React in a browser, Django behind it) | `web` | `web` | whatever the browser runs on |
| `desktop` (an egui app: offline, SQLite, real OS windows) | `desktop` | `desktop` | `macos`, `windows` or `linux` |

A test may carry one tag: `only: [<selectors>]` or `except: [<selectors>]`. The selectors are only `web`, `desktop`, `macos`, `windows` and `linux`. With no tag, a test runs everywhere. There is no mobile version: a test only for `ios`, `android`, `mobile` or `tablet` is removed.

## Web only (`only: [web]`)

- Sign-in, accounts, sign out, the account menu, CSRF, and anything about a server or the network:
  - a write the server refuses, "Couldn't save that change", Try again (decisions 11 and 14);
  - reloading after a failure;
  - uploads to a server.
- Settings stored per account (decision 17).
- The browser-reserved chord remaps (decision 18): New task Alt+N, New list Alt+Shift+N, Search "/", lists Alt+1…9, tabs Alt+Shift+Left/Right, projects Alt+Shift+[ and ].
  - The desktop keeps the reference defaults: New task Cmd+N, New list Cmd+Shift+N, Search Cmd+F, lists Cmd+1…9, tabs Cmd+Alt+Left/Right, projects Cmd+Shift+[ and ]. Here Cmd is Meta on macOS and Control on Windows and Linux.
- Shortcuts that refuse a chord because the **browser** keeps it (decision 15). This is web only, never `macos` or `windows`/`linux`. The desktop refuses only chords the OS keeps, or a bare key without a modifier.
- The Trash in the side panel or aside (decision 8). On the desktop, the Trash is its own window.
- Settings shown as a dialog inside the main window. On the desktop, Settings is its own window (decision 19).
- Any backdrop, scrim, dim layer or in-window overlay behind a dialog, sheet, palette or lightbox. On the desktop, a confirmation, a prompt sheet (new list, rename) or the lightbox is a real separate window, with no backdrop.
  - A test of "press outside closes it" or "the scrim dims the window" is web only.
  - Its desktop form, where one is needed, says the window opens, takes focus, and closes on Escape or Cancel.
- The draft's Image slot: paste or pick an image (decision 5). The desktop has Screenshot instead.
- The visible search-and-commands button that opens the palette (decision 7). This applies unless the test or its source says the reference has it on the desktop too.
- Mounting through the browser: `asChild`, links that navigate, `className`.

## Desktop only (`only: [desktop]`, or one OS)

- The global shortcut group and global hotkeys (decision 15).
- The capture bar and everything in it:
  - its title and notes fields;
  - Cmd+Enter submits from them (decision 20);
  - Escape dismisses it;
  - screenshots and capture modes.
- Settings → Capture.
- The data folder row, and opening a local folder or file in the OS (Reveal in Finder is `macos`).
- Settings as a window that opens at 640 × 520 and is never smaller (decision 19).
- Trash as a window (decision 8).
- Dialogs as windows.
- Native menus.
- The tray or menu-bar agent.
- Window drag regions.
- Window focus (a window that is focused or blurred).
- The Screenshot slot in the draft.

## No platform: remove the test

- **The app never draws window controls** (decision 2). This covers close, minimize, maximize and zoom buttons, traffic lights, and a red close light, and whether they are lit or grey. The OS draws them on the desktop, and the browser owns the window on the web. A test that expects the app to draw them is removed.
  - A test of the app's own close button in a pane (an ordinary ×) stays.
  - A double-click on the title bar to maximize is the OS's doing on the desktop: remove that test too.
- A test only for `ios`, `android`, `mobile` or `tablet`.

## Desktop forms of the same behaviour

- **The platform modifier.** It is Command (Meta) on macOS and Control on Windows, Linux and the web.
  - A key step that hard-codes `Control+X` or `Meta+X` for a platform-modifier chord should become a named input, `{ input: <name> }`.
  - Use one of the inputs in `platforms.yaml`, or propose a new input with its forms: `default` (Windows, Linux), `macos` and, if different, `web`.
  - Example: `newTask: { default: { key: Control+N }, macos: { key: Meta+N }, web: { key: Alt+N } }`.
- **Option or Alt** switches the round check from select to done: the `modifierHold` input. Held modifiers use `selectHold` and `modifierHold`, never `do: holdSelect` or `do: holdModifier` in a contract.
- **Key labels** are text, never icons. The reference drew Material icons, and that was rejected.
  - On macOS the labels are the glyphs ⌘ ⌥ ⇧ ⌃ and ↩ ⌫ ⇥ ⎋ ← → ↑ ↓.
  - On Windows and Linux they are the words `Ctrl`, `Alt` and `Shift`, then the key.
  - An expected label or accessible name that differs by platform is written per platform, with a reason: `text: { default: "Ctrl N", macos: "⌘N" }, why: "…"`. A per-platform value without `why:` is a contract error.
  - Where a part shows glyphs or text instead of an icon on the desktop, the test says what it shows, not which icon.
- **A part that only one platform has** (the capture field, a scrim, an Image slot, the data-folder button) is `optional: true` in `parts`.

## What not to change

- **Props and events never vary by platform.** Never propose a change to `props` or `events`.
- **Leave a test that applies to both alone.** Tag only what the rules above make platform-specific.
- **Keep the desktop forms the contracts already have**, even ones marked "proposed" or "provisional", when they agree with the decisions. Correct only those that contradict them.
- **Do not invent behaviour.** If the decisions and your sources do not settle a test, leave it and say so in an `unsure` entry.

## What you return

A list of changes. Each change names its contract and test exactly as written: copy the `name:` value. Each change also has a `reason` naming the rule or decision and, where you used one, a source (`file:line`).

| kind | fields | meaning |
|---|---|---|
| `tag` | `test`, `tag` | Set the test's tag: `only: [web]`, `only: [desktop]`, `only: [macos]`, `except: [macos]`, `only: [windows, linux]`…; `none` removes a tag so it runs everywhere |
| `remove` | `test` | Delete the test (window controls, mobile only) |
| `replace` | `test`, `yaml` | The test's whole new YAML, one list item starting `- name:` at column 0: named inputs instead of hard-coded keys, per-platform values with `why:`. Keep its name unless the old one is now wrong |
| `add` | `after`, `yaml` | A new test after the named one: the desktop form beside a web-only test, or the web form beside a desktop-only one |
| `optional` | `part` | Mark a part `optional: true` |
| `input` | `input`, `forms` | A new named input for `platforms.yaml`, with its forms as YAML flow (`{ default: { key: Control+N }, macos: { key: Meta+N }, web: { key: Alt+N } }`) |
| `unsure` | `test` | A test you could not settle, and why |

New and replaced YAML uses only what the contract already uses: its parts, props, events, actions and the format's fields. A key step names its part, as the contract's other key steps do.
