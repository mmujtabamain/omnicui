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

**egui screenshots need a Vulkan, Metal or DX12 driver.** Any normal Mac or Windows PC has one. On a Linux machine without a GPU, install Mesa's software driver (lavapipe, from the `mesa-vulkan-drivers` package). Without one, the egui target still runs every behaviour test and trace, and reports screenshots and clips as "not advertised", with the reason.

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
omnicui targets                             # what each target can do on this machine
omnicui contracts                           # check the contracts only
omnicui stubs                               # run the targets' stub generators
omnicui add <name> [--template <folder or git URL>]
omnicui remove <name>
omnicui version                             # also --version
omnicui --project path/to/omnicui check     # name the project instead of finding it
```

**Finding the project.** The project is the folder with `contracts/` and `targets/`. omnicui looks in the current folder and every folder above it, and in `omnicui/` inside each of them. So it works from anywhere in this repository, including its root. `--project <folder>` or `OMNICUI_PROJECT` names it instead.

**Exit status:** 0 when nothing failed or broke, 1 when something did, 2 for a usage or contract error, 130 when interrupted.

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

`make test-real` switches each one on in turn and checks that it is caught.

## Adding a target

```sh
omnicui add swiftui
```

This writes `omnicui/targets/swiftui/target.yaml` and prints the constructs to implement. Then, in that folder:
1. Depend on the app, the way your stack does it (a package, a path dependency, an artifact).
2. Write a registry: for each contract id, how to build the app's component from the test's props, and how its events become the contract's events. `omnicui/targets/javafx/src/main/java/omnicui/javafx/Components.java` is one, in 25 lines.
3. Write an adapter: a program that reads JSON-RPC 2.0 requests from standard input, one per line, and writes one reply per line to standard output. It logs only to standard error. `omnicui/targets/javafx/` is a complete example in about 300 lines.
4. Put the adapter's start command in `adapter:`, starting with `exec`.
5. Tag the parts in the app's components, then run `omnicui check swiftui`.

## Releases

`VERSION` is the version, and `notes/<VERSION>.md` its release notes (front matter with `version`, `date` and a one-sentence `summary`, then `## New`, `## Changed`, `## Fixed`). A merge into `main` raises `VERSION` and adds its note; that is the whole release step.

- `.github/workflows/ci.yml`, on merge requests and on `main`: vet, the unit and failure-injection tests, the six builds, the release scripts' tests (`node --test scripts/`) and `node scripts/neutral.mjs` (omnicui names no project that uses it).
- `.github/workflows/notes.yml`: `node scripts/check-notes.mjs` (with `--base-version` from `main` on a merge request): `VERSION` is raised and every note is well formed.
- `.github/workflows/release.yml`, on `main`: when `v<VERSION>` has no release yet, the checks, `make release`, and a GitHub Release with the six binaries, `omnicui-src-<VERSION>.tar.gz` and `SHA256SUMS`. Its text is the note, then how to use each file (`scripts/release-body.mjs`).
- `.github/rulesets/main.json` protects `main` (merge requests only, both checks required); import it once under Settings → Rules → Rulesets.

A project that vendors omnicui takes a release's source tarball, checks it against `SHA256SUMS`, and builds the command with `go build ./cmd/omnicui`. The tarball leaves out the real-target and report tests, which need this repository's apps.
