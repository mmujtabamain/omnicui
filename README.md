# omnicui: the reference repository

omnicui checks that hand-written versions of the same UI component, built in different stacks, behave the same, look the same and move the same. Each component has one contract, and every version is checked against it.

This folder is a complete, standalone example of a repository that uses omnicui:
- **three real apps**, each with the same two components (a counter and a switch) written by hand in its own stack: `apps/web` (React), `apps/desktop` (egui) and `apps/desktop-javafx` (JavaFX);
- **the omnicui project** in `omnicui/`: the two contracts, and one target per app that says how the contracts reach that app's components;
- **the omnicui command itself**, written in Go, in `tools/omnicui/`.

The specification is `docs/SPEC.md`; the evidence that everything below works is in `docs/verification/`.

```
apps/                              the real apps; they know nothing about omnicui but two things (below)
  web/                             React: src/main.tsx (the app), src/components/ (exported as a library)
  desktop/                         egui (eframe): src/main.rs (the window), src/{counter,switch}.rs (a library)
  desktop-javafx/                  JavaFX: app/Main.java (the window), app/components/ (a library)
omnicui/                           the omnicui project: omnicui finds it from anywhere in the repository
  contracts/                       counter.contract.yaml, switch.contract.yaml
  targets/                         how the contracts reach each app; no UI code here
    web/                           target.yaml, adapter.ts (Playwright), harness.tsx (the registry), stubs.ts
    egui/                          target.yaml, src/main.rs (egui_kittest + AccessKit), src/components.rs (the registry), stubs/
    javafx/                        target.yaml, Adapter.java + JavaFxTarget.java (TestFX + Monocle), Components.java (the registry)
  .omnicui/run/                    the last check's results (not committed)
shared/fonts/                      DejaVu Sans: the one font every app draws with
tools/omnicui/                     the omnicui command (Go): cmd/, internal/, schema/, its own tests
  integration/                     the tests against the real targets (build tag `real`)
  testdata/fakeadapter/            a fake target for omnicui's own tests, including every kind of failure
  report-test/                     report.html in a real browser: axe (WCAG 2.2 AA) and screenshots
experiments/redesign/              a second design of the web components, checked against the same contracts (README.md)
run.sh                             try it: doctor, setup, app, check, demo, test (macOS, Linux)
Makefile                           the same, with make; and `make release`
```

**The one rule:** `omnicui/targets/*` imports `apps/*`, never the other way round. Each target depends on its app the way its stack does it: an npm `file:` dependency (web), a Cargo path dependency (egui), a Maven dependency (javafx). The apps carry only two traces of omnicui:
- the part tags on their widgets, which is how a target finds "the increment button": `data-part` on the web, the AccessKit `author_id` in egui, the node id in JavaFX;
- `src/omnicui/` in the web and egui apps: the props, events and part ids generated from the contracts. They are committed, so each app builds on its own; `omnicui stubs` refreshes them.

The deliberate defects (below) are in the apps too: they change the components themselves. They are off unless `OMNICUI_DEFECTS` names them.

## Try it on your machine

On macOS or Linux (this proof of concept has no Windows scripts; the omnicui command itself also builds for Windows):

```sh
./run.sh doctor          # what is installed, and what is missing (with how to install it)
./run.sh setup           # build omnicui, the apps and their targets; the first run downloads their dependencies
./run.sh app desktop     # run an app for real: web (your browser), desktop (egui), desktop-javafx
./run.sh check           # run every contract on every target, then open the report
./run.sh demo            # switch on deliberate defects one at a time, and watch each one get caught
./run.sh demo egui       # one target on its own: web, egui, javafx; also setup/doctor <target>
./run.sh test            # every test suite
./run.sh all             # doctor, setup, check, demo, test
```

- The words can come in either order: `./run.sh desktop app` is `./run.sh app desktop`. `show` is another name for `app`.
- Target names: `web`; `egui` (also `rust`, `desktop`); `javafx` (also `java`, `desktop-javafx`).
- `./run.sh app desktop` runs `cargo run --release` in `apps/desktop` and shows cargo's progress. If `setup` hasn't built it yet, the first build takes a few minutes before the window opens.
- Add a deliberate defect to see it by eye: `OMNICUI_DEFECTS=egui.switch.linear ./run.sh app desktop`.
- `./run.sh omnicui …` runs omnicui on this repository's project with any arguments, for example `./run.sh omnicui check egui counter`.
- With `make`: `make install`, `make check`, `make test`, `make release`.

**Requirements** (`doctor` checks them):
- Go 1.21 or later, which fetches the 1.27 toolchain the module asks for by itself;
- Node 22.18 or later, which runs `.ts` files directly;
- Java 21 or later. Maven is not needed: the JavaFX app and target each have the Maven wrapper;
- Rust 1.85 or later.

`setup` downloads the npm packages (from each `package-lock.json`), Chromium for Playwright, Maven and the JavaFX dependencies, and the Rust crates (from each `Cargo.lock`). It installs the JavaFX app into your local Maven repository (`~/.m2`), where the javafx target finds it. Nothing else is installed outside this folder except Playwright's browser cache and the Go, Maven and Cargo download caches.

**egui screenshots need a Vulkan, Metal or DX12 driver.** Any normal Mac or Windows PC has one. On a Linux machine without a GPU, install Mesa's software driver (lavapipe, from the `mesa-vulkan-drivers` package). Without root, unpack that package into a folder of your own (`dpkg-deb -x mesa-vulkan-drivers_*.deb ~/mesa`), set `library_path` in `~/mesa/usr/share/vulkan/icd.d/lvp_icd.json` to the full path of its `libvulkan_lvp.so`, and run with `VK_ICD_FILENAMES` pointing at that file. Without one, the egui target still runs every behaviour test and trace, and reports screenshots and clips as "not advertised", with the reason.

**On Linux, Chromium may need system libraries.** If Playwright says so, run `npx playwright install-deps chromium` in `omnicui/targets/web`.

## The omnicui command

omnicui is one static binary with no runtime dependencies. Get it in one of three ways:

```sh
make release                     # dist/: omnicui_<VERSION>_<os>_<arch>[.exe] for six platforms,
                                 #   omnicui-src-<VERSION>.tar.gz (the command's source) and SHA256SUMS
cd tools/omnicui && go install ./cmd/omnicui      # into $(go env GOPATH)/bin
cd tools/omnicui && go build -o ../../bin/omnicui ./cmd/omnicui   # what setup does
```

```sh
omnicui check                               # every contract, every target
omnicui check egui counter                  # one target, one component
omnicui check --require egui:screenshot     # fail if egui can't take screenshots here
omnicui check --out ci-results              # write the results somewhere else than omnicui/.omnicui/run/
omnicui check --fail-on pixels              # CI: pixel warnings fail the run (also palette, timing, direction, motion, visibility, warnings)
omnicui check --test "each press"           # only the tests whose name has this in it (or a glob, with *)
omnicui check --failed                      # only what failed, broke or warned in the last run
omnicui check --serial                      # one target at a time (or exclusive: true in a target.yaml)
omnicui targets                             # what each target can do on this machine, with its render check
omnicui compare a.png b.png --diff d.png    # the four pixel measures of two saved screenshots
omnicui help contract                       # every field, default and measure, on one screen
omnicui contracts                           # check the contracts only
omnicui stubs                               # run the targets' stub generators
omnicui add <name> [--template <folder or git URL>]
omnicui remove <name>
omnicui version                             # also --version
omnicui --project path/to/omnicui check     # name the project instead of finding it
```

**Finding the project.** The project is the folder with `contracts/` and `targets/`. omnicui looks in the current folder and every folder above it, and in `omnicui/` inside each of them. So it works from anywhere in this repository, including its root. `--project <folder>` or `OMNICUI_PROJECT` names it instead.

**Exit status:** 0 when nothing failed or broke, 1 when something did (or a warning that `--fail-on` names), 2 for a usage or contract error, 130 when interrupted.

**What omnicui is for.** Two things, on every stack an app is built in: the same **behaviour** (the tests, the keyboard, the events), and one **design language** (the look, the colours, the motion), even where the layouts differ by design.

**How screenshots are compared.** Every pair of targets of one design, by four measures, each with its own threshold in the contract (`thresholds: { area: 30%, strength: 28%, shift: 8px, spot: 2% }` are the defaults):
- **area**: how much of the drawn area differs (a pixel differs when nothing within 2 px of it in the other screenshot has its colour);
- **strength**: how far off in colour the recoloured pixels are (pixels merely displaced are left out);
- **shift**: how far the differing pixels moved. Anti-aliasing alone reaches 3–7 px, so by default it is shown but doesn't warn;
- **spot**: how concentrated the difference is in its worst 24×24 px spot. Font rendering spreads thin differences over every edge, and a displaced edge has its colour nearby; a wrong word, colour or opacity is concentrated. This is the measure that finds those, and the diff image boxes the worst spots.

**Different designs.** A target whose layout is meant to differ (a phone layout, a redesign) says `design: <name>` in its target.yaml. Its screenshots are then compared with other designs' by their **design language** instead of pixel by pixel: the colours each draws with. `palette` is the share of one's drawing in colours the other design uses in none of its screenshots (`thresholds.palette`, default 15%), and the warning names them. Behaviour, keyboard and motion are compared between every target as usual. `experiments/redesign/` shows it: a redesign in its own palette warns at 73–100%, and the same layouts in the reference colours measure 0%.

**How motion is compared.** A `trace` follows a point's position, or with `property: opacity` or `scale` a part's opacity or scale, and it finds the part at each frame, so a sheet that slides in is traced from its first frame. A recorded clip (`record: true`) is compared too: each target's frames are reduced to how far the picture has gone from its first frame to its last, over time, and those curves are compared like traces (`thresholds.motion`, default 40 ms). That catches fades, blurs and colour changes on every target, egui included, whatever each target's frame rate. report.html plays the clips side by side, lined up at the input, with a speed control and a slider. `allCombinations: true` takes traces and clips at every viewport and colour scheme. `record: { part: x }` measures a clip over one part's box. A change that goes out and comes back (a spring, a hover in and out) is measured out and back. Two traces that move more than 30° apart (`thresholds.direction`) warn too: `web went right 20 px, webalt up 20 px`.

**Part of a screen.** `screenshot: { mask: [parts] }` (or `mask:` for the whole contract) leaves parts out of the comparison, wherever either target shows them; a part marked `optional: true` is one that only some targets or designs have, such as desktop window controls or a switch's thumb that another design draws as a checkbox. Where a target doesn't have it, traces, clips and checks on it are skipped, with a note, and a test whose steps act on it is "not measured".

**Text.** `text` is what a part shows. A test can ask for it exactly (`text: "3"`), or `{ contains: "3" }`, or `{ matches: "^3\\b" }`, for a design that says more ("3 taps"). `screenshot: { part: toolbar }` compares only that part's box on each target.

**Actions a target defines.** Besides press, focus, key, type, drag and hover, a step can be `{ do: swipe, part: row, with: { direction: left } }`: omnicui passes it to every target that advertises `do.swipe`, unchanged. Scrolling, swiping, the system back and pickers differ by platform, so their targets define them.

**Keyboard.** `keyboard: true` in a contract adds a test per button (Enter, Space), switch and checkbox (Space): the key must do what a pointer press does. A part that isn't on screen when the component mounts (it is in a sheet or a dialog) is "not measured", with the advice to give that sheet a component of its own; so is a press that changes nothing. A target can say which keys its platform uses (`keys` in `advertise`).

`check` prints a table of all four per pair, with ▲ on each one over its threshold, and names the part the worst spot is in. Thresholds far below the defaults make every comparison warn, and the contract check says so.

**Interrupting.** Adapters run in process groups of their own, so a Ctrl+C in the terminal doesn't reach them. On Ctrl+C or SIGTERM (a CI runner cancelling a job), omnicui stops every adapter it started, with everything each one started (a browser, a JVM, cargo's child), then exits with 130.

**Results.** `omnicui check` prints a table, and writes to `omnicui/.omnicui/run/` (or `--out`):
- `results.json`, for CI;
- `report.html`;
- `omnicui.log`, its own log;
- `protocol.log`, every JSON-RPC line sent and received;
- the screenshots, diff images, traces and clips.

## Causing every failure on purpose

Each defect is switched on with `OMNICUI_DEFECTS=name[,name…]`. Most are in the apps' components; three are in the targets, to show what omnicui does when an adapter itself goes wrong. The check must catch every one:

| Defect | Where | What it does | What the check reports |
|---|---|---|---|
| `javafx.switch.no-label` | app | the switch's label doesn't name it | **fail**: `switch.name: expected "Dark mode", got nothing` |
| `egui.switch.no-name` | app | the switch reports an empty name | **fail** |
| `web.switch.no-name` | app | the switch isn't labelled by its text | **fail** |
| `egui.counter.small-text` | app | 13 px text instead of 16 | pixel **warning** |
| `web.counter.padding` | app | a wider, taller button | pixel **warning** |
| `web.switch.slow`, `javafx.switch.linear`, `egui.switch.linear` | app | the thumb slides for 300 ms, linearly | timing **warning** |
| `javafx.host.stretch` | target | the component is centred instead of at (24, 24) | pixel **warning** on every screenshot |
| `egui.render.cursor` | target | the mouse pointer is drawn into screenshots | pixel **warning** |
| `javafx.adapter.stdout` | target | a library's print reaches standard output | javafx **broken**: stray output |
| `web.switch.dim`, `javafx.switch.dim` | app | the thumb also fades to half opacity, over 600 ms | **motion** warning from the clip (a position trace sees nothing), and a pixel warning |
| `javafx.counter.no-enter` | app | the button ignores Enter | **fail**: ``Enter on `increment` didn't do what a press does: …`` |
| `web.harness.late` | target | the component appears 400 ms after mount, and the page says nothing | web **fail**: `no part ui.counter.value` |
| `web.harness.late-busy` | target | the same, but the page says it is busy until then (`window.omnicuiBusy`) | **pass**: the adapter waits |

`make test-real` switches each one on in turn and checks that it is caught.

## Adding a target

```sh
omnicui add swiftui
```

This writes `omnicui/targets/swiftui/target.yaml` and prints the constructs to implement. Then, in that folder:
1. Depend on the app, the way your stack does it (a package, a path dependency, an artifact).
2. Write a registry: for each contract id, how to build the app's component from the test's props, and how its events become the contract's events. `omnicui/targets/javafx/src/main/java/omnicui/javafx/Components.java` is one, in 25 lines.
3. Write an adapter: a program that reads JSON-RPC 2.0 requests from standard input, one per line, and writes one reply per line to standard output. It logs only to standard error. `omnicui/targets/javafx/` is a complete example in about 300 lines.
   Optional constructs: `probe` (the render check's picture, so that `screenshot` is verified before a run), `bounds` (a part's box: enables `visible`, the visibility warning and naming the part a pixel difference is in), and the `role` and `focused` fields of `read`. An adapter waits for the app's busy hook before replying, if the app has one (docs/SPEC.md §4.4).
4. Put the adapter's start command in `adapter:`, starting with `exec`.
5. Tag the parts in the app's components, then run `omnicui check swiftui`.

## Releases

`VERSION` is the version, and `notes/<VERSION>.md` its release notes (front matter with `version`, `date` and a one-sentence `summary`, then `## New`, `## Changed`, `## Fixed`). A merge into `main` raises `VERSION` and adds its note; that is the whole release step.

- `.github/workflows/ci.yml`, on merge requests and on `main`, one job each. Always: Go vet and tests, the smoke test (`scripts/smoke.sh`: the release files built and used as a project uses them, the same as the release job runs before it publishes), and `node scripts/neutral.mjs` (omnicui names no project that uses it). Only when their own files change: the six Go builds (`tools/omnicui/`, `Makefile`), the release scripts' tests (`scripts/`) and the web static server.
- `.github/workflows/notes.yml`: `node scripts/check-notes.mjs` (with `--base-version` from `main` on a merge request): `VERSION` is raised and every note is well formed.
- `.github/workflows/release.yml`, on `main`: when `v<VERSION>` has no release yet, the checks, `make release`, and a GitHub Release with the six binaries, `omnicui-src-<VERSION>.tar.gz` and `SHA256SUMS`. Its text is the note, then how to use each file (`scripts/release-body.mjs`).
- `.github/rulesets/main.json` protects `main` (merge requests only, both checks required); import it once under Settings → Rules → Rulesets.

A project that vendors omnicui takes a release's source tarball, checks it against `SHA256SUMS`, and builds the command with `go build ./cmd/omnicui`. The tarball leaves out the real-target and report tests, which need this repository's apps.
