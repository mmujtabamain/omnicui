# The Taskscape omnicui project

The contracts that hold the three versions of Taskscape (`../tauri`, `../gpui`, `../web`) to one
behaviour and one design, and the `tauri` target that runs them on the Tauri app.

```
contracts/          eleven components: window, titlebar, pane, inspector, capture, sheet,
                    palette, settings, filter, trash, toast
platforms.yaml      the inputs whose keys differ by platform (from the app's hotkey catalog)
fixtures/           the data every version loads: *.json, and the taskscape-fixtures crate that
                    writes them through taskscape_common's Store (`cargo test` checks it)
targets/tauri/      the Tauri app's real frontend in Chromium; its commands answered by server/
SPEC-PRODUCT.md     the product, for the ports: every component, its props, events, parts, rules
DESIGN.md           the design: tokens (sRGB hex), type, spacing, motion, the window's structure
GAPS-P0.txt         what the contracts and omnicui couldn't say, the decisions made, my mistakes
PROGRESS.md         how it was built, with times
```

## Run it

From the repository root, with Node 22, Go and Rust:

```sh
cd tools/omnicui && go build -o ../../bin/omnicui ./cmd/omnicui && cd ../..
examples/taskscape/omnicui/targets/tauri/setup.sh       # once: npm, icons, the harness, the server
bin/omnicui --project examples/taskscape/omnicui check tauri
bin/omnicui --project examples/taskscape/omnicui check tauri pane --test "search*"
```

`setup.sh` installs the app's npm packages from its own lockfile, makes the icon subsets the
app's build makes, builds the harness pages with the app's own Vite and Tailwind, installs
Chromium for Playwright, and builds the store server. Re-run it after changing the app.

## How the tauri target reaches the app

- The app's components are mounted from a registry it carries,
  `tauri/main/src/omnicui/registry.tsx` (and the capture bar's in `tauri/tray/src/omnicui/`):
  for each contract id, the stores loaded as the app loads them and the surface built as App
  builds it. The app's only other change is `data-part` tags.
- `harness.ts` replaces Tauri's runtime: Tauri's own `mockIPC` sends every `invoke` to
  `server/`, a Rust server that answers each command as the app's Rust side does, with
  `taskscape_common::Store` on a database the fixture loader wrote for that mount. Window
  plumbing (presenting and theming windows) is answered in the page.
- The page says it is a Mac (WKWebView's user agent), so the app takes its macOS paths.
- The server reports what each command changed as the contracts' domain events, named by
  fixture key (`taskCompleted {id: brief, done: true}`).
