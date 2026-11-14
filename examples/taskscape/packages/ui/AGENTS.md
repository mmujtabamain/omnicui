# @taskscape/ui: how a widget is written

Every piece of UI the Taskscape apps draw comes from this package. The apps may not use raw
elements (`<button>`, `<input>`, `<p>`…), Radix, cmdk or raw colours: a lint rule refuses them
(`packages/eslint-plugin`). So each widget here has to be complete enough for the screens.

## One widget = three files

| File | What it holds |
|---|---|
| `src/components/ui/<kebab-name>.tsx` | The widget, shadcn style (below). |
| `src/harness/<contract name>.tsx` | How omnicui mounts it: `export default { id, Component, events }`. |
| `../../omnicui/contracts/<contract name>.contract.yaml` | Its contract: props, events, parts, tests. Never edit it. |

The contract name is the file name without `.contract.yaml` (`button`, `panefooter`, `listtab`).

## The shadcn conventions

- One file per widget, named in kebab case (`pane-footer.tsx`), exporting named components
  (`PaneFooter`, plus any sub-parts) and, where it has variants, its `cva` function
  (`paneFooterVariants`).
- Variants with `class-variance-authority`; class lists merged with `cn()` from `../../lib/utils`.
- Behaviour from Radix primitives where one fits (the widget spec names the base). Import them as
  `import * as DialogPrimitive from "@radix-ui/react-dialog"`. Installed: alert-dialog, checkbox,
  collapsible, context-menu, dialog, dropdown-menu, popover, progress, radio-group, scroll-area,
  select, separator, slot, switch, tabs, toast, toggle, toggle-group, toolbar, tooltip; also
  `cmdk` (command palette), `react-resizable-panels` (panes) and TipTap (`@tiptap/react`,
  `@tiptap/starter-kit`, `@tiptap/extension-character-count`, `-mention`, `-placeholder`). Ask (in your summary) before needing another.
- `React.forwardRef` on the root element when a caller may need it; spread the remaining props
  onto the root.
- Compose the widgets that already exist instead of redrawing them: `Icon` (`./icon`), `Button`
  (`./button`), `Tooltip` (`./tooltip`), and any other file in `src/components/ui/`.
- Icons are Material Symbols names rendered by `<Icon name="add" size={16} />`. Never an SVG.

## Parts: `data-slot`

Every part the contract lists is the element whose `data-slot` is the part's name, exactly as
spelled in the contract (`data-slot="trailingIcon"`). The test runner finds parts that way, and
reads their role, name, state and text from the accessibility tree: use the real element or ARIA
role the contract's comment names (`aria: tab`, `aria: checkbox`…), not the simplified role the
contract's `role:` field had to use. Optional parts are rendered only when they apply.

When a widget nests another widget that has a part of the same name, give your own part
`data-part="ui.<contract name>.<part>"` as well; it wins over `data-slot`.

## Tokens and sizes

- 1rem is 14px. Write sizes in rem (`h-[1.75rem]` is 24.5px) or Tailwind's spacing scale.
- Colours only from the tokens in `src/styles/tokens.css`: `bg-surface-0…3`, `text-content-1…3`,
  `border-edge-1…3`, `bg-wash-1/2`, `accent-500/600/700`, `danger-100/500/600`, `success-500`,
  `done-lamp-1`, `focus-1`, `selection-1`, `on-accent`. They switch for dark mode by themselves
  (`.dark`). `white`/`black` with an opacity are allowed only over photos (media surfaces).
- Corners: `rounded-(--radius-field)`, `rounded-(--radius-control)`, `rounded-(--radius-panel)`,
  `rounded-full`. Shadows: `shadow-menu`, `shadow-lift`. Motion: `ease-standard`, `ease-exit`,
  `duration-(--duration-fast)`, `animate-rise`, `animate-panel-in`. Layers:
  `z-(--z-index-popover)` and the other `--z-index-*` values.
- Inline `style` is for computed geometry only (a glyph size, a measured offset), never colour.

## The harness entry

```tsx
import { useRef } from "react";
import { Button } from "../components/ui/button";

function ButtonHarness({ emit, ...props }: any) {
  const count = useRef(0);
  return <Button {...props} onClick={() => emit("activated", { count: ++count.current })} />;
}

export default { id: "ui.button", Component: ButtonHarness, events: ["activated"] };
```

- The harness receives the contract's props exactly as the test sets them (defaults are not
  filled in: give your component the contract's defaults) and `emit(name, fields)`.
- Emit each contract event with exactly the fields the contract declares, with the same types.
- Keep state the contract expects to change (a checkbox's `checked` after a press) in the harness
  with `useState`, seeded from the prop, the way a real caller would.
- When the widget needs content the contract cannot pass (children, items), build realistic demo
  content in the harness from the props.
- A tooltip provider with no delay already wraps every harness.

## Checking

```
node packages/ui/scripts/check-widget.mjs <contract name>     # from examples/taskscape
```

It type-checks, lints, runs the contract in Chromium and writes two contact sheets of every
screenshot (`.checks/<name>/sheet-light.png`, `sheet-dark.png`). Look at the sheets: a test can
pass while the widget looks wrong. Add `--test "<part of a test name>"` to run a few tests.
