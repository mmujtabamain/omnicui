# Taskscape web: how a screen is written

The web app is React + Vite on the Django API (`../backend/API.md`). It draws **only** with the
widget library `@taskscape/ui`: a lint rule refuses raw interactive, text and media elements
(`button`, `a`, `input`, `p`, `h1`…`h6`, `ul`, `li`, `img`, `svg`, `form`, `label`…), Radix, cmdk,
inline styles and raw colours. Layout tags (`div`, `span`, `main`, `section`, `header`, `nav`,
`aside`, `footer`) with token utilities are fine.

## One screen = a folder and a harness entry

| File | What it holds |
|---|---|
| `src/screens/<screen>/` | The screen: one exported component (`TaskList`), plus any parts of it in the same folder. |
| `src/harness/<contract name>.tsx` | How omnicui mounts it: `export default { id, Component, events }`. |
| `../../../omnicui/contracts/<contract name>.contract.yaml` | Its contract. Never edit it. |

Write only inside your screen's folder and its harness entry. Shared code (`src/api`, `src/state`,
`src/lib`, other screens) is read-only for you: when you need something there, say so.

## What you build with

- Widgets: `import { Button } from "@taskscape/ui/button"`, one path per file in
  `../../../packages/ui/src/components/ui/` (kebab-case file names). Read a widget's file to learn
  its props before using it; never restyle a widget from outside beyond layout (margins, flex,
  width) — if it cannot look right, say so.
- Data: the hooks in `src/api/queries.ts` (`useLists`, `useTasks`…) and the endpoint functions in
  `src/api/endpoints.ts`; writes go through `useWrite(fn)` so every view refreshes. Types are in
  `src/api/types.ts`.
- What the user is looking at (open list, selection, the task in the inspector, the aside, the
  palette, settings): the store in `src/state/ui.ts` (`useUi`).
- Shortcuts: Mod is Ctrl on Windows/Linux and Cmd on macOS; `formatAccel` and `isMac` come from
  `@taskscape/ui/lib/platform`, and the HotkeyHint widget draws a chord.

## Parts: `data-part`

Every part the contract lists is the element with `data-part="app.<contract name>.<part>"`, e.g.
`data-part="app.tasklist.search"`. Widgets pass `data-part` through to their root, so put it on
the widget itself (`<Button data-part="app.trash.emptyTrash" …/>`). The test reads the role, name,
state and text from the accessibility tree, so the widget's own semantics must be right.

## The harness entry

```tsx
import { AppHarness } from "./lib/AppHarness";
import { Trash } from "../screens/trash/Trash";

function TrashHarness(props: { dataset?: string }) {
  return <AppHarness><Trash /></AppHarness>;
}

export default { id: "app.trash", Component: TrashHarness, events: [] };
```

- Before your harness renders, the page has already loaded the demo data set `props.dataset` and
  signed in as the demo account (`spec/screens/web.md` lists the data). `AppHarness` gives a fresh
  query cache, a router and the account's theme, and makes the test wait while requests run.
- Context props (a list or task by name) are resolved by the harness: fetch with the endpoint
  functions, find the row by name, then set the store (`useUi.setState(...)`) or pass the id.
  Render nothing until that is done.

## Checking

```
SCREENS_DATABASE_URL=… TASKSCAPE_PYTHON=… node apps/web/frontend/scripts/check-screen.mjs <contract name>
```

(from `examples/taskscape`; your prompt gives the exact command). It type-checks, lints, runs the
contract in Chromium against the real API and writes contact sheets of every screenshot
(`.checks/screen-<name>/sheet-light.png`, `sheet-dark.png`). Compare them with the reference
screenshots your prompt lists: the screen must look like the reference.
