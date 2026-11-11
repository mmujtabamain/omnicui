# omnicui: specification

omnicui checks that hand-written versions of the same UI component, built in different stacks, behave the same, look the same and move the same. This repository contains three things:
- the complete specification (this file, `docs/SPEC.md`);
- a small standalone reference project that proves it, which is the rest of this repository: three real apps (web in React, desktop in egui/Rust, desktop in JavaFX/Java), each with the same two components (a counter and a switch) written by hand in its own stack; an omnicui project with the two contracts and one target per app; and the `omnicui` command itself, in Go;
- the evidence that it works, in `docs/media/` and `docs/verification/`.

This document is complete on its own. A reader with only this repository can rebuild omnicui, add a target for any stack, or audit another implementation against these rules.

**How rules are tagged:**
- **[B] baseline**: what the design document said. omnicui did not exist before this repository, so there is no product code to cite. The baseline is the design document as it stood on 6 November 2026. Its relevant content is restated in §1 and §2, so this document does not depend on it.
- **[R] reference**: what this repository's implementation does. Each one cites the test that proves it.
- **[REC] recommended**: not built. Each one says why, and how it would be verified.

Contents:
- 0 Scope
- 1 Summary
- 2 Model
- 3 Lifecycle
- 4 Interfaces
- 5 Behaviour rules
- 6 Errors and recovery
- 7 Traceability and observability
- 8 Security and permissions
- 9 Performance
- 10 User-facing surface
- 11 Edge cases
- 12 Acceptance checklist
- 13 Gaps and drift
- 14 Improvements
- 15 Verification
- 16 Code map
- 17 Found while building
- 18 Evidence

---

## 0. Scope

**FEATURE:** omnicui. This document answers four questions:
1. How does a contract describe a component?
2. How is a target added, and how does omnicui talk to an arbitrary stack?
3. What is checked, and how are differences reported?
4. How does it fail, and how does it recover?

**SOURCE:** omnicui did not exist before this repository. The baseline is the design document, version of 6 November 2026, which is restated in §1 and §2. Every [B] rule refers to that version.

**Decisions taken before building:**

| Question | Decision |
|---|---|
| Which targets | web (React), egui (a standard Rust GUI library that supports what is needed), JavaFX |
| Message format | JSON-RPC 2.0, as used by LSP and MCP |
| omnicui's own language | Go. Rust and Zig were also evaluated: Go has mature libraries for YAML with line numbers and for JSON Schema, makes process supervision short, and cross-compiles trivially. Zig has no mature YAML or JSON Schema library. |
| Names to remove | the product, company and service names of the environment this was built in, people's names, and any branding |

**Out of scope, and why:**
- **Checking that a target implements each construct correctly.** The design assumes targets are correct; see [REC-10] for a conformance kit.
- **Masks for intentional platform differences.** They are designed but not built; see [REC-2].
- **Differential fuzzing.** It is designed but not built; see [REC-3].
- **Any server, account or network service.** omnicui is a local command.

---

## 1. Summary

**What it is.** Some UI must exist on several stacks at once, such as React on the web and native desktop toolkits. There are two common approaches, and both fail:
- **Generating every stack from one source.** This gives up what each stack does best, and forces one model of state onto all of them.
- **Writing each stack by hand with no shared check.** The versions drift apart.

omnicui keeps the hand-written versions and adds the missing check:
- every version is written by hand, in its own stack;
- a **contract** (YAML) describes each component once;
- `omnicui check` runs every version against it and prints where they differ.

The model is React Native's platform-specific files extended to whole languages. No single compiler can check versions written in different languages, so the contract's tests take the compiler's place.

**Q1: How does a contract describe a component?** A contract is a YAML file in the project's `contracts/` folder (in the reference repository, `omnicui/contracts/`), typed by a JSON Schema. It holds:
- the component's `props` and `events`, with JSON Schema types;
- its `parts`: the pieces a test can act on and check, each with a WAI-ARIA role;
- a list of `tests`. Each test has steps (user actions), then `expect` (fields of parts, and events), and can also ask for a `screenshot`, a `trace` of one point on a part over time, or a `record`ed clip;
- settings: `viewports`, `colorSchemes` and `thresholds`.

Mistakes are caught before any target runs, with the file and line and a suggested correction. The schema catches fields and types, and omnicui's own checks catch cross-references, such as a part that does not exist or an action its role does not accept.

**Q2: How is a target added, and how does omnicui talk to an arbitrary stack?** The components live in the apps that use them, written in each app's own idiom and with their own API. A target is a folder in the project's `targets/` folder that reaches one app's components. It holds:
- a dependency on the app, in that stack's usual way (an npm `file:` dependency, a Cargo path dependency, a Maven artifact);
- a **registry**: for each contract id, how to build the app's component from a test's props, and how the component's events become the contract's events;
- an **adapter**: a program that performs a fixed set of steps, the *fixture language*, in that stack;
- a `target.yaml` naming the shell command that starts the adapter.

**The rule:** targets import apps, never the reverse. An app carries only two traces of omnicui: the part tags on its widgets (how a target finds "the increment button") and, if its target has a stub generator (web and egui in the reference repository), the types generated from the contracts (§4.5). Each app builds and runs on its own.

omnicui starts the adapter once per check and keeps it running. It sends one JSON-RPC 2.0 request per line on the adapter's standard input and reads one reply per line from its standard output. Logs go to standard error. `omnicui add <name>` writes only `target.yaml`: omnicui knows no stack, so it ships no scaffolds. `--template <folder|git URL>` copies an existing target instead. A target advertises its **capabilities** (constructs), not its components; a missing component fails its tests at mount.

**Q3: What is checked, and how are differences reported?** There are three checks:
- **Tests** are exact. A mismatch **fails** the run.
- **Pixels:** each screenshot is compared between every pair of targets **of one design** that took it, by four measures (§5): **area** (how much of the drawn area differs, allowing for 2 px of placement noise), **strength** (how far off in colour), **shift** (how far pixels moved) and **spot** (how concentrated the difference is in its worst 24×24 px spot). Any measure above the contract's threshold for it prints a **warning**.
- **Colours (design language):** targets of different designs (`design:` in target.yaml: a phone layout and a desktop one, a redesign) aren't compared pixel by pixel, which would only say that the layout differs, but by the colours they draw with: **palette** is the share of one's drawing in colours the other design uses in none of its screenshots at that colour scheme. Above `thresholds.palette` it prints a **warning**.
- **Visibility:** a part whose text, name, value or checked state was read while it had no size on screen, or was outside the viewport, is a **warning** (targets that report `bounds`).
- **Motion:** each trace (a point's position, or a part's opacity) is compared between targets and with the test's named timing function, if it has one. Each recorded clip is reduced to its **visual progress** (how far the picture has gone from its first frame to its last, over time) and compared between targets the same way, which covers fades, blurs and colour changes that have no position to trace. Above the threshold (`timing` for traces, `motion` for clips) it prints a **warning**. Two traced points that move in directions more than `thresholds.direction` apart (a thumb sliding up against one sliding across) are a **warning** too.
- **Keyboard:** with `keyboard: true`, every button, switch and checkbox gets a generated test per key its role is activated by; the key must do what a pointer press does. A difference **fails**.

A warning never fails the run unless `check --fail-on` says so (for CI), and nothing is sent anywhere: the author reads it and decides. omnicui keeps no state between runs, and has no approval step and no baselines.

**Q4: How does it fail, and how does it recover?** Every failure is visible and classified:
- an error reply **fails** that test;
- a timeout, a crash, closed output or stray output marks the target **broken**, with the last lines of its standard error. Its remaining tests are "not run", and other targets are unaffected.
- a capability the target lacks is **not advertised**, with the target's reason. It does not fail unless the run uses `--require`.

There is nothing to recover: each run starts clean and replaces the last run's folder. An interrupted run (Ctrl+C, or SIGTERM from a CI runner) stops every adapter it started, with everything each one started, and exits with 130.

---

## 2. Model

### Entities

| Entity | Fields | Identifier | Owner |
|---|---|---|---|
| **Project** | `contracts/`, `targets/`, `.omnicui/run/` | its folder: named by `--project <folder>` or `OMNICUI_PROJECT`, or else found by walking up from the working directory, taking at each level the folder itself if it has `contracts/`, or its `omnicui/` subfolder if that has `contracts/` (in the reference repository, `omnicui/`) | the user |
| **Contract** | `id`, `props`, `events`, `parts`, `tests`, `viewports` (each with optional `props`), `colorSchemes`, `thresholds`, `keyboard` | `id`, e.g. `ui.counter` (unique in the project) | `contracts/<name>.contract.yaml` |
| **Field** | `type` (string, integer, number, boolean), optional `enum`, optional `default` | its name within props or within one event | a contract |
| **Part** | `role` | `<contract id>.<part name>`, e.g. `ui.counter.increment` | a contract |
| **Role** | its checkable fields and the actions it accepts (table below) | WAI-ARIA name | omnicui |
| **Test** | `name`, `props`, `steps`, `expect`, `screenshot`, `trace`, `record`, `allCombinations` | `name` (unique in its contract) | a contract |
| **Step** | `action`, `part`, `times`, and `key`, `text` or `dx`/`dy` | its position | a test |
| **Expectation** | either `part` and fields, or `event` with `times` and/or `last` | its position | a test |
| **Trace spec** | `part`, `property` (`position`, the default, or `opacity`), `point` (`center` or `topLeft`; positions only), optional `timing` (`duration`, `easing`) | its position | a test |
| **Target** | `name`, `language`, `adapter`, `stubs`, `stepTimeout`, `startTimeout`, `exclusive`, `design` | `name` (equals its folder name) | `targets/<name>/target.yaml` |
| **Advertisement** | `protocol`, `can` (constructs), `cannot` (construct → reason) | none: one per target per run | the adapter |
| **Run** | run id, the results of every test on every target, the comparisons, the summary | `<UTC time>-<12 hex digits>` | `.omnicui/run/` in the project, or the folder given by `check --out` |
| **Test result** | contract, test, target, status, messages, screenshots, traces, clip, not-advertised capabilities | (contract, test, target) | a run |
| **Comparison** | kind (`pixels`, `palette`, `timing`, `direction`, `motion` or `visibility`), A, B (a target, or `timing`), combination, part, value, threshold, warning, diff image, note | (contract, test, kind, A, B, part) | a run |

**Roles** [R] (`internal/contract/roles.go`, tested by `TestErrors/role refuses action` and `TestErrors/role has no field`):

| Role | Fields that can be checked | Actions it accepts |
|---|---|---|
| button | text, name, disabled | press, focus, key, hover |
| switch, checkbox | text, name, disabled, checked | press, focus, key, hover |
| slider | name, disabled, value | press, focus, key, drag, hover |
| textbox | text, name, disabled | press, focus, key, type, hover |
| status, heading | text, name | hover |
| generic | text | hover |

Every role also has `role` (the role assistive technology reports, compared with the value given; capability `role`) and `visible` (the part has a size on screen and some of it is inside the viewport; omnicui works it out from `bounds`). The focusable roles (button, switch, checkbox, slider, textbox) have `focused` (capability `focused`). A test that checks one of them on a target that doesn't advertise its capability is "not advertised" there. *(TestRoleAndFocused)*

### Invariants

- [R] A contract's `id` is unique among the project's contracts. *(TestDuplicateIDs)*
- [R] Every part, prop, event and event field a test names exists in its contract. Every action fits the part's role, and every expected field is one the role has and is of its type. *(TestErrors: unknown part, role refuses action, role has no field, unknown event, unknown event field, event field type, unknown prop, prop type)*
- [R] A prop's default has the prop's type. *(TestErrors/default type)*
- [R] Test names are unique within a contract. *(TestErrors/duplicate test)*
- [R] A test that traces has steps. *(TestErrors/trace without steps)*
- [R] A target's `name` equals its folder name. *(`project.Targets`; an error otherwise)*
- [R] Within a run, each (contract, test, target) has exactly one result. *(TestAllPass counts 14 results for 2 targets × 7 tests)*
- [R] No contract error leaves a run folder behind, because no target runs. *(TestContractErrorsStop)*

---

## 3. Lifecycle

### A run

```
load contracts ──errors──▶ print every problem, exit 2 (no target runs, no run folder)
      │ ok
      ▼
replace the run folder (.omnicui/run/, or --out) ─▶ for every target, in parallel:
                           stubs (if target.yaml has `stubs:`) ──fails──▶ target broken
                           start the adapter ──can't start──▶ target broken
                           advertise ──no valid reply in startTimeout──▶ target broken
                           for each contract, each test, in file order:
                               for each viewport × colour scheme:
                                   mount → steps (or trace) → expect → screenshot
                               first combination only (every one with allCombinations): further traces, record
                           stop the adapter
      ▼
compare screenshots and traces between targets ─▶ results.json, report.html ─▶ print ─▶ exit 0 or 1

at any point: SIGINT or SIGTERM ─▶ kill every running adapter's process group ─▶ exit 130
```

### A target within a run

| State | Entered when | Allowed | Leaves to |
|---|---|---|---|
| starting | omnicui runs the `adapter:` command | the `advertise` request only | ready, broken |
| ready | a valid `advertise` reply | any advertised construct, one request at a time | broken, stopped |
| broken | timeout, exit, closed output, stray output, a wrong reply id, a reply of the wrong shape, failed stubs | nothing: every remaining test is "not run" | stopped |
| stopped (final) | the run ends, or the target broke | nothing | none |

**Resources in the final states** [R]:
- omnicui closes the adapter's standard input.
- It waits up to 3 s for the adapter to exit.
- It then kills the adapter's whole process group, so wrappers such as `npm` or `cargo run` go with it.

A hung adapter is gone after its timeout *(TestTimeout checks that the process no longer exists)*.

- [R] omnicui keeps a list of every adapter that is still running. When omnicui itself receives SIGINT or SIGTERM, it kills each one's process group (its process tree on Windows), prints how many it stopped, and exits with 130. This is needed because each adapter runs in a process group of its own and so never receives the terminal's Ctrl+C. *(TestKillAll: two fake adapters are running, both are killed, and neither process exists afterwards)*

### A test result

| Status | Meaning | Fails the run? |
|---|---|---|
| pass | every expectation held | no |
| fail | an expectation disagreed, a step acted on a part that does not exist, or the target has no such component | **yes** |
| not advertised | the behaviour needs a construct the target did not advertise (the test can't run there) | no |
| not measured | the test ran but had nothing to measure: a generated keyboard test whose part isn't on screen when the component mounts, or whose press changes nothing a test can read (§5); a test whose steps act on an optional part this target's design doesn't have, or everything it checks is on such parts (§4.2) | no |
| broken | the target broke during this test, or before it | **yes** |

**Warnings** belong to comparisons, not to test results. A pass with a missing capability (for example, no `screenshot`) stays a pass, and the missing capability is counted in the summary as "not advertised". The **worst status over all combinations** is the test's status, in this order: broken > fail > not advertised, not measured > pass. "not measured" is counted in the summary only when there is one: `… · 2 not advertised · 3 not measured · 0 warnings`.

---

## 4. Interfaces

### 4.1 The command line

Every command takes the form `omnicui [--project <folder>] <command> …`.

| Command | Does | Exit status |
|---|---|---|
| `omnicui check [target…] [component…] [--test <name>]… [--failed] [--serial] [--require target:capability]… [--out <folder>] [--fail-on warnings\|pixels\|palette\|timing\|direction\|motion\|visibility]…` | runs every test of every contract (or only those named) on every target (or only those named); writes the results to `.omnicui/run/` in the project, or to `--out`. One progress line per finished test, with its time, goes to standard error: `[web 12/38] counter › each press adds one  pass  0.4s`; while a target starts, a line every 10 s (§4.3) | 0 when nothing failed or broke; 1 when something did, a required capability is missing, or there is a warning of a kind `--fail-on` names; 2 for a usage or contract error |
| `omnicui targets` | starts each adapter, asks `advertise`, runs the render check (§4.4) on each target that advertises `screenshot` and `probe`, and prints its capabilities with reasons and the render check's outcome | 0; 2 if a target.yaml is invalid |
| `omnicui compare <a.png> <b.png> [--diff <out.png>] [--json]` | measures two screenshots the way `check` does and prints the four measures against the default thresholds, for looking into one comparison or trying thresholds on saved screenshots | 0; 2 if a file can't be read |
| `omnicui help contract` | one screen: every field of a contract, the defaults, the roles and their fields, and how each comparison is measured | 0 |
| `omnicui stubs [target…]` | runs the targets' stub generators | 0; 1 if one fails |
| `omnicui contracts` | loads and checks the contracts only | 0; 2 on any error |
| `omnicui add <name> [--template <folder or git URL>]` | creates `targets/<name>/target.yaml`, or copies a template and renames it | 0; 2 if the name is invalid or exists |
| `omnicui remove <name>` | deletes `targets/<name>/` | 0; 2 if there is no such target |
| `omnicui version`, `omnicui --version` (or `-v`) | prints `omnicui <version> (protocol 1, <os>/<arch>, <Go version>)`, e.g. `omnicui dev (protocol 1, linux/amd64, go1.27…)` | 0 |
| `omnicui`, `omnicui help`, `--help`, `-h`, `<command> --help` | prints the usage | 2 with no command at all; 0 otherwise |

**Exit status, over all commands:** 0 when nothing failed or broke; 1 when something did; 2 for a usage, project or contract error; 130 when interrupted.

**Running less** [R]. A full check of a large app takes long, mostly in mounts and captures. `--test <name>` (repeatable) keeps the tests whose name contains it, ignoring case, or, with `*` or `?`, matches the whole name as a glob (`--test 'keyboard:*'`). No match exits 2 with the tests that share a word with it. `--failed` keeps the tests that failed, broke or had a warning in the last run's `results.json` (in `--out`'s folder, if given): ``nothing to rerun: in the last run no test failed, broke or warned`` exits 0, and no earlier run exits 2. Both narrow what the target and component names chose. The run still replaces the run folder, so a second `--failed` reruns what still fails. *(TestFilterTests, TestLastProblems, TestCheckSelection)*

**Time** [R]. Each result has `ms`, each target `ms` and `startMs` (until it answered `advertise`) in `results.json`, and the summary is preceded by the five slowest tests (of a second or more) and each target's total and start-up time. *(TestTimesAndStderr)*

**`--serial`** [R] runs one target at a time, as if each had `exclusive: true` (§4.3). *(TestExclusive)*

A name given to `check` or `stubs` is matched against target names first, then against contract ids and their last segment (`counter` for `ui.counter`). [R] An unknown name exits 2, listing the known names. *(TestUsageErrors)*

**Finding the project** [R]. `--project <folder>` (before the command), or else the `OMNICUI_PROJECT` environment variable, names the project: that folder if it has `contracts/`, or else its `omnicui/` subfolder if that has `contracts/`. Otherwise omnicui walks up from the working directory and, at each level, takes the folder itself if it has `contracts/`, or its `omnicui/` subfolder if that has `contracts/`. So in a repository that keeps its project in `omnicui/`, next to its apps, omnicui works from the root and from anywhere below it. With no project found, or a named folder that is not one, it exits 2 with ``no contracts/ folder (or omnicui/contracts/) here or in any parent folder; run omnicui inside a project, or name one with --project <folder>`` or ``<folder> is not an omnicui project: it has no contracts/ folder (or omnicui/contracts/)``. *(TestProjectAndOut: found from a repository's root; `--project` from an unrelated folder; `OMNICUI_PROJECT`; a folder that is not a project)*

**`check --out <folder>`** [R] writes the run there instead of `.omnicui/run/`, and the printed results and report paths name it. A relative folder is taken from the working directory. Like `.omnicui/run/`, the folder is deleted and recreated by every run. *(TestProjectAndOut)*

**The version** [R] comes from, in order: the value stamped at build time with `-ldflags "-X main.Version=<version>"` (what `make release` does); the module version that `go install …@v<version>` records in the binary's build information; otherwise `dev`. The protocol version is the one in `advertise` (§4.4). *(TestProjectAndOut checks `omnicui dev (protocol 1, …`)*

**Interruption** [R]. On SIGINT (Ctrl+C) or SIGTERM, omnicui kills the process group of every adapter still running, prints ``omnicui: interrupt; stopped <n> adapter(s)`` (or `terminated`) on standard error, and exits with 130 (§3). *(TestKillAll)*

**Example**, from the reference repository's root (the project is found in `omnicui/`):

```
$ OMNICUI_DEFECTS=web.counter.padding bin/omnicui check counter
[javafx 1/5] counter › starts at the start value  pass               # progress, on standard error
…
ui.counter                                               egui      javafx    web
  starts at the start value                              pass      pass      pass
    screenshots, worst combination per pair:
                                 area      strength  shift     spot
      egui ↔ javafx 320x240-dark 16%       19%       4 px      0.2%      pass
      egui ↔ web 320x240-dark    74% ▲     61% ▲     4 px      9.7% ▲    warning: area, strength, spot above the threshold (worst spot 24×24 at 78,18, in `increment`)
      javafx ↔ web 320x240-light 75% ▲     61% ▲     4 px      9.6% ▲    warning: area, strength, spot above the threshold (worst spot 24×24 at 78,18, in `increment`)
      thresholds                 30%       28%       8 px      2%
  …
  keyboard: Enter on `increment` does what a press does  pass      pass      pass
  keyboard: Space on `increment` does what a press does  pass      pass      pass

15 pass · 0 fail · 0 broken · 0 not advertised · 4 warnings
results: omnicui/.omnicui/run/results.json   report: omnicui/.omnicui/run/report.html
```

The two paths are printed relative to the working directory when the run folder is inside it, and in full otherwise.

### 4.1a Run scripts

`run.sh` is for trying the reference repository on your own machine. It is in the repository's root. It runs omnicui as `bin/omnicui --project omnicui/`.

| Command | Does |
|---|---|
| `./run.sh doctor [target]` | checks Go ≥ 1.21, Node ≥ 22.18, Java ≥ 21 and Cargo ≥ 1.85 (with a target, only what it needs), and says how to install what is missing (on Linux, also whether a Vulkan driver is present) |
| `./run.sh setup [target]` | builds omnicui into `bin/omnicui`, then, for each app, the app first and then its target: web (npm in `apps/web`, then npm and Chromium for Playwright in `omnicui/targets/web`, then `omnicui stubs web`, then `tsc --noEmit` in both); javafx (`./mvnw install` in `apps/desktop-javafx`, which puts the app's jar in `~/.m2`, then `./mvnw package` in `omnicui/targets/javafx`); egui (`omnicui stubs egui`, then `cargo build --release --locked` in `apps/desktop`, then in `omnicui/targets/egui`). With no target, it then installs the report tests and prints `omnicui targets` |
| `./run.sh app <app>` (also `show`) | runs a real app, to use by hand: `web` builds `apps/web` and serves it on 127.0.0.1, then opens your browser (`node serve.ts`); `desktop` runs `cargo run --release --locked` in `apps/desktop`, with cargo's build progress visible, then opens the egui window; `desktop-javafx` runs `java -jar target/desktop-javafx.jar` in `apps/desktop-javafx`. Each app shows the counter and the switch, a dark colour scheme toggle and the events they report. `OMNICUI_DEFECTS` works here too. omnicui never opens these; they are for people |
| `./run.sh check [args]` | `omnicui check`, then opens `omnicui/.omnicui/run/report.html` |
| `./run.sh demo [target]` | five deliberate defects, one at a time, each with what to expect, then a clean run. With a target: only that target, on its own. That target's defects are a failed test, a timing warning against the contract and (javafx) a broken adapter |
| `./run.sh test` | the four suites of §15, each run from `tools/omnicui`. The real-target suite's whole output goes to `omnicui/.omnicui/test-real.log` |
| `./run.sh all` | doctor, setup, check, demo and test, in order |
| `./run.sh omnicui …` | omnicui on the repository's project, with any arguments |

- **Names.** Targets: `web`; `egui` (also `rust` or `desktop`); `javafx` (also `java` or `desktop-javafx`). Apps: `web`, `desktop`, `desktop-javafx`, or any of the target names, which map to their app.
- **Word order.** The two words can come in either order: `./run.sh desktop app` is `./run.sh app desktop`.

The scripts are for macOS and Linux (bash or any POSIX shell); this proof of concept has no Windows scripts. omnicui itself builds and runs on Windows.

On Windows, omnicui runs `adapter:` with `cmd /C` and drops a leading `exec`, so the same `target.yaml` works everywhere [R]:
- JavaFX starts with `java -jar`, and its manifest carries the classpath;
- egui starts with `cargo run`;
- an adapter that hangs, or that is still running when omnicui is interrupted, is ended with `taskkill /T /F`, which takes its children too.

### 4.2 The contract file

A contract, modelled on the reference repository's `omnicui/contracts/counter.contract.yaml`, with every kind of entry shown once:

```yaml
# yaml-language-server: $schema=../../tools/omnicui/schema/contract.schema.json
id: ui.counter

props:
  start: { type: integer, default: 0 }

events:
  changed:
    count: { type: integer }

parts:
  value:     { role: status }
  increment: { role: button }

tests:
  - name: each press adds one
    props: {}                                         # optional
    steps:
      - { action: press, part: increment, times: 3 }  # press, focus, key (key:), type (text:), drag (dx:, dy:), hover
    expect:
      - { part: value, text: "3" }                    # a part and the fields to check; text: { contains: "3" } or { matches: "^3\\b" }
      - { event: changed, times: 3, last: { count: 3 } }
    screenshot: true                                  # at every viewport and colour scheme; or { mask: [part…], part: <part> }
    trace:                                            # follow one point while the steps run
      - { part: value, point: center, timing: { duration: 200ms, easing: ease-out } }
      - { part: value, property: opacity }            # or follow its opacity as drawn, or its scale
    record: true                                      # a clip, compared by its visual progress; or { part: <part> }
    allCombinations: true                             # traces and the clip at every viewport × colour scheme

viewports:    [{ width: 320, height: 240, props: {} }] # default: 320×240; props: every test's at that size (a test's own win)
colorSchemes: [light, dark]                           # default: light
keyboard:     true                                    # Enter/Space must do what a press does, per role
mask:         []                                      # parts left out of every screenshot comparison
thresholds:   { area: 30%, strength: 28%, shift: 8px, spot: 2%, timing: 20ms, motion: 40ms, palette: 15%, direction: 30deg }   # the defaults
```

**Screenshots of part of a screen** [R]:
- `screenshot: { mask: [a, b] }` on a test, and `mask: [a, b]` for every test of the contract, leave those parts' boxes out of the comparison. Every box either target of a pair reports is left out of both images, so what one target draws there and the other doesn't is not compared: masked pixels count as not drawn, and show grey in the diff image.
- A part can be `optional: true`: shown by some targets or designs only, such as window controls that a desktop toolbar has and the web one doesn't. A mask or a crop naming it covers it where it is, and is skipped where it isn't.
- `screenshot: { part: toolbar }` compares only that part's box on each target, wherever it sits on each; boxes of different sizes are compared from their top-left corners, and the difference in size counts.
- The whole screenshots' measures stay in the comparison (`raw` in `results.json`) and in its note: `masked: keyboard (whole screenshots: area 41%, strength 30%, shift 2 px, spot 63%)`. A target that doesn't advertise `bounds` can't be masked: the note says so, and the whole screenshots are compared.
*(TestMeasureView, TestMaskOptionalPart, TestScreenshotPart, TestMaskContractErrors)*

**Parts some designs don't have** [R]. The same behaviour is often drawn differently: a switch with a sliding thumb on one target, a native checkbox with a tick on another (experiments/redesign). A part only some designs have is `optional: true`, and where a target replies "no part" (-32002) for it:
- a trace of it is skipped, and the test's steps are run again, plainly, on a fresh component (the trace ran them once already);
- a clip of it (`record: { part: x }`) is skipped when the part is there neither before nor after the steps;
- what a test expects of it (`text`, `visible`, …) is skipped;
- a test whose steps act on it is **not measured**, as is a test that skipped everything it checks, and a generated keyboard test on it.

Each skip is noted under the test, on the console (`webalt: no `thumb` in webalt's design (an optional part): its trace is skipped`) and in `results.json` (`absent`); the rest of the test counts as usual. A part that isn't optional and that a target doesn't have fails, and the message says how to mark it: ``…if webalt's design has no `thumb`, mark the part `optional: true` in the contract``. *(TestOptionalPartAbsent, TestOptionalPartSteps)*

**Looser text** [R]. `text` and `name` are a string the value must equal, or `{ contains: "3" }`, or `{ matches: "^3\\b" }` (a regular expression, RE2), for a design that shows more than the contract cares about ("3 taps"). A regular expression that doesn't compile is a contract error. `text` is what the part shows: the web adapter leaves out text that only a screen reader gets (an element clipped to a 1×1 box); JavaFX and egui read their labels' text. *(TestTextMatch, TestTextMatchContractError)*

**Steps.** A step has `action` (press, focus, key, type, drag, hover) and a `part`, or `do` and its `with` (§4.4). Neither, or both, is an error that says which to write, and an unknown action suggests `do`: ``tests.0.steps.0.action: `swipe` is not one of `press`, `focus`, `key`, `type`, `drag`, `hover`; for an action the targets define, write `do: swipe` (with its parameters under `with:`)``.

**Standards it follows:**
- typing: JSON Schema 2020-12, through the `yaml-language-server` first line, a path relative to the contract file (the schema is `tools/omnicui/schema/contract.schema.json`, and is also embedded in the omnicui binary);
- type names: JSON Schema's;
- roles and fields: WAI-ARIA;
- viewport and colour scheme: Playwright's terms;
- durations and easings: CSS (`linear`, `ease`, `ease-in`, `ease-out`, `ease-in-out`, `cubic-bezier(…)`);
- keys: names such as `Enter`, `Space`, `Shift+Tab`.

**Errors**, one per line, as `file:line  error|warning: message`. The file is relative to the working directory when it is inside it; here omnicui was run from the repository's root:

```
omnicui/contracts/counter.contract.yaml:24  error: part `incremnt` does not exist; did you mean `increment`?
omnicui: fix the contracts first; no target was run
```

Other messages, as `TestErrors` checks them:
- ``unknown field `expectt`; did you mean `expect`?``
- ``tests.0.expect.0.text: expected string, got number``
- ``` `value` is a status, which does not accept `press` ```
- ``` `value` is a status, which has no field `checked` ```
- ``event `chnged` does not exist; did you mean `changed`?``
- ``prop `strat` does not exist; did you mean `start`?``
- ``the default of `start` is a string, but its type is integer``
- ``part `extra` is never used by any test`` (a warning)

### 4.3 `target.yaml`

`omnicui/targets/web/target.yaml`, without its comments:

```yaml
name: web                    # must equal the folder name
language: typescript         # informational
adapter: exec node adapter.ts   # run with /bin/sh -c in the target's folder (cmd /C on Windows)
stubs: node stubs.ts ../../../apps/web/src/omnicui   # optional; every contract as JSON on stdin
stepTimeout: 15s             # default 10s: any one request
startTimeout: 120s           # default 120s: starting and answering advertise
exclusive: false             # true: run this target with no other target alongside
design: redesign             # optional: a design of its own (default: the one most targets share)
```

**Starting** [R]. While a target starts, a line every 10 s on standard error says so, with the time left and the last line its adapter wrote to standard error: `[android] starting: 40s of startTimeout 20m0s · adapter: > Task :adapter:compileDebugUnitTestKotlin`, and then `[android] started in 52s`. *(TestStartProgress)*

**`design:`** [R] names a target's design when it isn't the one most targets share: a phone layout against a desktop one, a redesign. Screenshots are compared pixel by pixel only between targets of one design; between designs, by their colours (§5, Colours across designs). Behaviour, keyboard checks, traces, clips and visibility are compared between every target, whatever its design. The console says so when there is more than one: `designs: default: web · redesign: webalt (pixels compared within a design, colours across designs)`. *(TestDesigns)*

**`exclusive: true`** [R] is for a target that needs the whole machine (an emulator, a simulator, a large JVM): it starts and runs with no other target alongside, and the others wait (`[ios] waiting for an exclusive target to finish`). Targets without it run side by side, as before. *(TestExclusive)*

Both commands run in the target's folder, so a relative path in them starts there. The other two reference targets:

| Target | `adapter:` | `stubs:` | `stepTimeout`, `startTimeout` |
|---|---|---|---|
| egui | `exec cargo run -q --release --locked --bin adapter` | `cargo run -q --release --locked -p omnicui-egui-stubs -- ../../../apps/desktop/src/omnicui` | 15s, 300s |
| javafx | `exec java -jar target/omnicui-target-javafx.jar` | none | 15s, 60s |

[R] Start `adapter:` with `exec`. The shell then hands its place to the adapter, so that the adapter closing its standard output is seen at once, instead of only at the timeout (see §17).

### 4.4 The protocol: JSON-RPC 2.0 over standard input and output

**Rules:**
- one request per line, one reply per line;
- requests are answered in order, one at a time;
- standard output carries replies only, and anything else on it breaks the target;
- logs go to standard error;
- images and clips are written to files at paths omnicui chooses;
- an adapter replies to a step only once its UI has settled: nothing animating, all events processed, and the app not busy. **Busy** is the app saying it is still waiting for something outside the UI (a network reply, a file). Each reference adapter asks it the way its stack does: web, `window.omnicuiBusy()` if the page defines it; egui, `Mounted::busy()` (default false); JavaFX, a `BooleanSupplier` under `Component.BUSY` in the component root's properties. Busy is waited for in real time, up to 10 s, then the step fails. *(real: `web.harness.late` fails, `web.harness.late-busy` passes)*
- an adapter whose own runtime dies (the web adapter's Chromium) exits, so that omnicui reports the target broken once, with its standard error, instead of failing each later request. *(docs/verification/banking-review/browser-death.txt)*

| Method | Params | Result | Required |
|---|---|---|---|
| `advertise` | `{protocol: "1"}` | `{protocol, can: [construct…], cannot: {construct: reason}}` | yes, the first request |
| `mount` | `{component, props, viewport: {width, height}, colorScheme}` | `null` | yes |
| `press`, `focus`, `hover` | `{part}` | `null` | yes |
| `key` | `{part, key}` | `null` | yes |
| `type` | `{part, text}` | `null` | yes |
| `drag` | `{part, dx, dy}` | `null` | yes |
| `do` | `{name, part?, with: {…}}` | `null` | optional: an action the target defines, advertised as `do.<name>` (see below) |
| `read` | `{part, field}` | the value (string, boolean, number or null) | yes |
| `events` | `{}` | `[{name, fields}…]` since the last call (mount clears) | yes |
| `screenshot` | `{path}` | the path, after writing a PNG there | if it can render |
| `trace` | `{part, point, steps: [{method, …params}…]}`, or `{part, property: "opacity" \| "scale", steps}` | `[{ms, x, y}…]`, sampled every frame while the steps run; for opacity `[{ms, value}…]`, the part's opacity times its ancestors' (capability `trace.opacity`); for scale, its width as drawn over its layout width, ancestors' transforms included (capability `trace.scale`). [R] The part is looked up at each frame, and samples start when it is first there: a part that appears during the steps (a sheet sliding in) is traced from its first frame. Not there at any frame: error -32002 | if it can render |
| `record` | `{path, steps}` | `{frames: [{ms, file}…], inputs: [ms…], held}`: the frames written into `path`, when each step's input was delivered on the frames' clock, and whether frames come only when the picture changes (`held: true`, Chromium's screencast) rather than every pulse. A bare `[{ms, file}…]` (the first protocol 1 adapters) is still accepted | optional |
| `probe` | `{path, viewport: {width, height}}` | the path, after writing the render check's picture there | optional; asked right after `advertise` when the target advertises `screenshot` and `probe` |
| `bounds` | `{part}` | `{x, y, width, height}`: the part's box on screen; 0×0 when it isn't rendered | optional |

**The render check** [R]. Advertising `screenshot` is a claim. Before any test, omnicui asks each target that advertises `screenshot` and `probe` to draw, at the largest width and height of the run's viewports: a white background, a black 32×32 square at 8,8, and a red (#ff0000) 32×32 square whose bottom-right corner is 8 px from the viewport's. It checks the picture's size, the white middle, and both squares. If the check fails, `screenshot` is not advertised for that target in this run (behaviour still runs), the reason is printed first (`web: render check failed: the picture is 640×400, not the 1408×868 asked for: …`), and `--require web:screenshot` fails the run. *(TestProbePasses, TestProbeFails, TestProbeNotAdvertised)*

**`read` fields beyond the role table:** `role` (a WAI-ARIA role name; the reference adapters map Chromium's, AccessKit's and JavaFX's roles to it, and name any other role by its own name, e.g. JavaFX `text`), `focused` (a boolean) and, on a button, `selected` (a boolean: a chosen chip, tab or segment; capability `selected`; web reads `aria-pressed`, `aria-selected` or `aria-checked`, JavaFX a selected `Toggle`, egui AccessKit's selected state).

**`advertise` may also say `keys`** [R]: per role, the keys that activate it on this platform, `{"keys": {"button": ["Enter"], "switch": []}}`. A generated keyboard test (§5) with a key the target leaves out is not advertised there, with the reason; a role it doesn't name gets the usual keys. Compose, for one, doesn't activate a button with Space. *(TestKeyboardTargetKeys)*

**Actions the targets define (`do`)** [R]. omnicui builds in press, focus, key, type, drag and hover. Anything else a platform has, a swipe, a scroll, pull-to-refresh, the system back, a date picker, means something different on each and depends on how its target is built, so omnicui doesn't define it. A contract writes `{ do: swipe, part: row1, with: { direction: left } }`; omnicui sends `do` with `{name: "swipe", part: "<id>.row1", with: {direction: "left"}}`, unchanged, and `part` may be left out (`{ do: back }`). A target that can, advertises `do.swipe`; one that can't reports the test not advertised, with its `cannot` reason. These are the shapes the first mobile targets needed, as a convention to start from, not a rule:

| `do` | `part` | `with` | what the target does |
|---|---|---|---|
| `scroll` | the scrolling container | `{to: "<part>"}` or `{by: <px>}` | scrolls, with the platform's own physics, until `to` is fully on screen, or by `by` |
| `swipe` | the part to swipe | `{direction: left\|right\|up\|down, distance?: <px>}` | a real swipe gesture, as a finger does it |
| `refresh` | the list | `{}` | pull to refresh |
| `back` | none | `{}` | the system back (Android's back, iOS's edge swipe or back button) |
| `pick` | the picker | `{value: "2026-11-14"}` | chooses a value in the platform's own picker |

*(TestDoStep, TestDoStepContract)*

**Error codes:**

| Code | Meaning |
|---|---|
| -32001 | no component |
| -32002 | no part |
| -32003 | the step failed |
| -32601 | unknown method |
| -32700 | parse error |

**Example**, from a real run's `protocol.log`:

```
javafx > {"jsonrpc":"2.0","id":60,"method":"mount","params":{"colorScheme":"light","component":"ui.switch","props":{},"viewport":{"height":240,"width":320}}}
javafx < {"jsonrpc":"2.0","id":60,"result":null}
javafx > {"jsonrpc":"2.0","id":61,"method":"focus","params":{"part":"ui.switch.switch"}}
javafx < {"jsonrpc":"2.0","id":61,"result":null}
javafx > {"jsonrpc":"2.0","id":62,"method":"key","params":{"key":"Space","part":"ui.switch.switch"}}
javafx < {"jsonrpc":"2.0","id":62,"result":null}
javafx > {"jsonrpc":"2.0","id":63,"method":"read","params":{"field":"checked","part":"ui.switch.switch"}}
javafx < {"jsonrpc":"2.0","id":63,"result":true}
javafx > {"jsonrpc":"2.0","id":64,"method":"events","params":{}}
javafx < {"jsonrpc":"2.0","id":64,"result":[{"fields":{"on":true},"name":"toggled"}]}
```

### 4.5 The stub generator

If `target.yaml` has `stubs:`, omnicui runs it in the target's folder before starting the adapter, with every contract in the project on standard input as JSON. That is the normalised model of §2: `id`, `file`, `props`, `events`, `parts` with their ids, and `tests`. It is run on `check` and on `omnicui stubs`. It writes whatever its language's components import. A non-zero exit marks the target broken, with its output.

The components that import the stubs are in the app, so in the reference repository each generator takes its output folder as its one argument, and `target.yaml` passes the app's `src/omnicui/`. The generator empties that folder and writes one file per contract. The generated files are committed with the app, so that the app builds on its own without omnicui; `omnicui stubs` refreshes them. A contract change that a component no longer fits is then a compile error in the app.

The reference generators write:
- **web** (`omnicui/targets/web/stubs.ts`, into `apps/web/src/omnicui/`): `CounterProps` (`start?`, `onChanged?`), `CounterChanged`, `CounterDefaults`, `Parts` and `Events`, in `counter.ts`; the same for the switch in `switch.ts`; and `index.ts`, with `EVENTS`, the events of each contract id, which the web target's harness listens to. The app exports the folder as `web-app/omnicui`.
- **egui** (the `omnicui-egui-stubs` crate in `omnicui/targets/egui/stubs/`, into `apps/desktop/src/omnicui/`): a `CounterProps` struct with serde defaults, a `CounterEvent` enum with `to_json()`, and `parts::VALUE` / `parts::INCREMENT`, in `counter.rs`; the same for the switch in `switch.rs`; and `mod.rs`.
- **javafx:** none. The JavaFX components declare their part ids as constants, and the registry maps props by name (§4.8).

### 4.6 Files a run writes, in `.omnicui/run/` (replaced by every run)

The folder is `.omnicui/run/` inside the project (in the reference repository, `omnicui/.omnicui/run/`, which is not committed), or the folder given by `check --out`.

| File | Holds |
|---|---|
| `results.json` | the whole report: contracts, targets with their advertisement, every test result, every comparison, the summary |
| `report.html` | the same, for people: statuses, screenshots side by side, diff images, trace charts, clips |
| `omnicui.log` | structured log, one JSON object per line (§7) |
| `protocol.log` | every JSON-RPC line sent (`>`) and received (`<`), prefixed by the target |
| `<target>/<component>/<test>/<w>x<h>-<scheme>.png` | screenshots |
| `<target>/<component>/<test>/frames-<combination>/`, `clip-<combination>.png` | recorded frames and the lossless animated PNG made from them |
| `diff/<component>/<test>/<a>-<b>-<combo>.png` | differences between two targets: red where they differ |

### 4.7 Environment

| Variable | Used by |
|---|---|
| `OMNICUI_PROJECT` | omnicui: the project folder, when `--project` is not given (§4.1) |
| `OMNICUI_RUN_ID` | set by omnicui for every adapter: the run id, for the adapter's own logs |
| `OMNICUI_DEFECTS` | the reference apps and targets only: deliberate defects (§15). Read by the apps' components (and by the apps when run by hand) and by the targets' harnesses |
| `OMNICUI_BIN` | the real-target tests: the omnicui binary to drive (default `bin/omnicui` in the repository) |
| `OMNICUI_EVIDENCE` | the real-target tests: a folder to keep each scenario's console output and `results.json` in |
| `PLAYWRIGHT_BROWSERS_PATH` | the web target, if Chromium is installed somewhere else |
| `VK_ICD_FILENAMES` | the egui target: which Vulkan driver wgpu uses (for example Mesa's software driver) |
| `PORT`, `NO_OPEN` | the web app's `serve.ts` only: a fixed port instead of an ephemeral one; `NO_OPEN=1` doesn't open a browser |


### 4.8 Apps, targets and registries

In the reference repository the components belong to three real apps, and each omnicui target imports one app. **Targets import apps, never the reverse.** Each app builds and runs without omnicui, and carries only two traces of it: the part tags, and (web and egui) the generated `src/omnicui/` types of §4.5.

| | web | egui | javafx |
|---|---|---|---|
| App | `apps/web`, npm package `web-app`; exports `./components` and `./omnicui` | `apps/desktop`, crate `desktop`: a library, plus the window binary behind the default feature `app` (eframe) | `apps/desktop-javafx`, Maven artifact `app:desktop-javafx`: package `app.components`, plus the window `app.Main` |
| How the target depends on it | `"web-app": "file:../../../apps/web"` in `package.json` | `desktop = { path = "../../../apps/desktop", default-features = false }`, so eframe is never built | a Maven dependency on `app:desktop-javafx:0.1.0`, which `./mvnw install` in the app puts in `~/.m2` |
| Part tag | the `data-part` attribute | the widget's AccessKit `author_id`, written by `desktop::part` | the node's id (`setId`) |
| Registry | `harness.tsx`: a map from contract id to component; event handlers (`onChanged`, …) are made from `EVENTS` | `src/components.rs`: a trait `Mounted` that each app component implements, and `create(component, props)` | `Components.java`: a map from contract id to a `Component` |

**The components' own APIs.** Each is written the way its stack writes components, not to omnicui's protocol:
- **web:** React function components with props and callbacks: `<Counter start={0} onChanged={…} />`, `<Switch on={false} label="Dark mode" onToggled={…} />`, typed by the generated `CounterProps` and `SwitchProps`.
- **egui:** a struct built from the generated props and drawn once per frame, returning what happened in that frame: `Counter::new(CounterProps)`, `fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Option<CounterEvent>`; `Switch::new(SwitchProps)`, the same `ui` returning `Option<SwitchEvent>`. `Look::of("light" | "dark")` holds the colours.
- **javafx:** `HBox` subclasses with constructors and handlers: `new Counter(int start)` with `setOnChanged(IntConsumer)`; `new SwitchControl(boolean on, String label)` with `setOnToggled(Consumer<Boolean>)`. Both also have a no-argument constructor with the contract's defaults.

**The registries adapt them to the protocol.** The JavaFX one, `omnicui/targets/javafx/src/main/java/omnicui/javafx/Components.java`, in full:

```java
package omnicui.javafx;

import app.components.Counter;
import app.components.SwitchControl;
import java.util.Map;

// The registry: which of the app's components answers to which contract id, and how the test's
// props and the component's events become the contract's. One entry per component.
final class Components {
    // contract: the key is the contract's id (counter.contract.yaml → id: ui.counter).
    static final Map<String, Component> ALL = Map.of(
        "ui.counter", (props, emit) -> {
            var c = new Counter(((Number) props.getOrDefault("start", 0)).intValue());
            c.setOnChanged(count -> emit.emit("changed", Map.of("count", count)));
            return c;
        },
        "ui.switch", (props, emit) -> {
            var s = new SwitchControl((Boolean) props.getOrDefault("on", false), (String) props.getOrDefault("label", "Dark mode"));
            s.setOnToggled(on -> emit.emit("toggled", Map.of("on", on)));
            return s;
        }
    );

    private Components() {}
}
```

`Component` is `Parent create(Map<String, Object> props, Emitter emit)`, and `Emitter` is `void emit(String event, Map<String, Object> fields)`. The egui registry, `omnicui/targets/egui/src/components.rs`:

```rust
pub trait Mounted {
    /// Draws one frame. `emit` reports an event with its contract name and fields.
    fn frame(&mut self, ui: &mut egui::Ui, look: Look, emit: &mut dyn FnMut(&'static str, Value));
}

impl Mounted for Counter {
    fn frame(&mut self, ui: &mut egui::Ui, look: Look, emit: &mut dyn FnMut(&'static str, Value)) {
        if let Some(e) = self.ui(ui, look) {
            let (name, fields) = e.to_json();
            emit(name, fields);
        }
    }
}
// … the same for Switch …

pub fn create(component: &str, props: Value) -> Result<Box<dyn Mounted>, String> {
    let bad = |e: serde_json::Error| format!("props for {component}: {e}");
    match component {
        "ui.counter" => Ok(Box::new(Counter::new(serde_json::from_value(props).map_err(bad)?))),
        "ui.switch" => Ok(Box::new(Switch::new(serde_json::from_value(props).map_err(bad)?))),
        other => Err(format!("no component {other}")),
    }
}
```

**One React.** The web target bundles the app's components into its harness page, and they must run with the app's own React, and only one copy of it. An esbuild plugin in `adapter.ts` resolves every `react` and `react-dom` import from the app's folder, and `tsconfig.json` points the same imports at the app's type definitions.

**One font.** Every app draws with DejaVu Sans from `shared/fonts/DejaVuSans.ttf`: the web app through a CSS `@font-face` that esbuild bundles; the egui app with `include_bytes!` (`desktop::FONT`); the JavaFX app's build copies the file into its jar, next to the theme, and `Theme.loadFont()` loads it.

---

## 5. Behaviour rules

- [R] **Order.** Targets run in parallel, except one with `exclusive: true` (or every target with `check --serial`), which runs with no other target alongside. Within a target, the contracts run in file-name order, then the tests in file order, then the combinations, viewports first. *(TestAllPass; `check.Combos`)*
- [R] **One request at a time** per target. The next is sent only after the previous reply. *(TestConversation: ids 1…4 in order)*
- [R] **A fresh component for every test and every combination.** `mount` is sent before each one. *(protocol.log in every scenario)*
- [R] **`times` is expanded by omnicui.** `press ×3` is three `press` requests. *(TestConversation, TestFailMessage)*
- [R] **Traces and clips.** Traces and the clip are taken at the first combination only, or at every combination when the test says `allCombinations: true`, and are then compared combination by combination. The first trace replaces the plain steps there. Each further trace, and the recording, uses a fresh `mount`. *(TestAllPass: one trace, one clip, two screenshots; TestAllCombinations)*
- [R] **Viewport props.** A viewport's `props` are given to every mount at that viewport, under the test's own props, which win; they are type-checked like a test's. *(TestViewportProps)*
- [R] **Pixels.** For every test with `screenshot: true`, every pair of targets that took it, and every combination, omnicui computes four measures (`internal/compare/pixels.go`):
  - the drawn area is every pixel that is not the top-left (background) colour in either image;
  - a pixel **differs** when no pixel within 2 px of it in the other image is within the colour tolerance, which is YIQ Δ ≤ 0.1 on pixelmatch's scale, in either direction;
  - **area**: the share of the drawn area that differs (how often);
  - **strength**: the mean, over differing pixels that have no pixel of their colour within 4 px (recoloured, not displaced), of the colour distance to the closest colour within 2 px in the other image, as a percentage of black against white (how far off in colour);
  - **shift**: the median, over differing pixels that have one within 8 px, of the distance to the nearest pixel of the same colour (how far it moved). Anti-aliasing alone reaches 3–7 px, so its default is the most it measures, 8 px: it is shown, and warns only under a contract's own threshold;
  - **spot**: the share of the worst 24×24 px square's pixels that differ strongly (colour distance above 35%) and have no pixel of their colour within 4 px (how concentrated). Displaced edges, such as a button drawn 3 px wider by one toolkit, have their colour nearby and don't count. Anti-aliasing differences spread thinly over every edge; a wrong word, colour or opacity is concentrated, and this is the measure that tells them apart. The five worst spots are boxed in the diff image; when the targets report `bounds`, the worst is put down to the smallest part that holds it.

  The worst combination is reported per pair (the one with the most measures over their thresholds, then the largest area), with all four measures; each measure above its threshold is a warning. The defaults were set from screenshots of correct and defective renderings (docs/verification/banking-review/pixel-measures.md): area 30%, strength 28%, shift 8 px, spot 2%. *(TestPixelsIdentical, TestPixelsDrawnAreaAndReach, TestMeasures, TestSpotTellsNoiseFromAConcentratedDifference, TestPixelWarning)*
- [R] **`thresholds.pixels` was renamed `area`** and is refused, with the new spelling: ``thresholds.pixels was renamed to area (the share of the drawn area that differs): write `area: 30%` ``. A threshold under a third of its default is allowed, with a warning that says what correct renderings usually measure. *(TestPixelsRenamedToArea, TestStrictThresholdWarns)*
- [R] **Visibility.** On a target that advertises `bounds`, a part whose text, name, value or checked state was read is asked for its box; one with no size, or outside the viewport, is a `visibility` warning, unless the test checks `visible` itself. *(TestInvisibleReadWarns)*
- [R] **Different sizes are 100% different**, with a note. *(TestPixelsSizes)*
- [R] **Colours across designs** (`internal/compare/palette.go`). Two targets of different designs are compared, at each combination both took a screenshot at, by their **palettes**, and the worst combination is reported per pair:
  - a palette is made of the solid pixels only, those whose 2×2 block is one colour (anti-aliased edges are blends every toolkit makes its own way), grouped into colours within the colour tolerance; the largest is the background, and a colour belongs to the palette when it covers at least 3% of the drawing (the solid pixels that aren't the background) and 0.1% of the picture;
  - a colour of one is **used** by the other design when at least 0.05% of the other screenshot's pixels, solid or not, are within the tolerance of it, or of any other screenshot that design took in the run at the same colour scheme: a design language is the colours an app uses anywhere (the accent of a switch counts for a counter's button), and a colour one design fills a box with and another draws as lines and text is shared;
  - **palette** is the larger, of the two, of the share of one's drawing in colours the other design doesn't use; different backgrounds are 100%.

  Above `thresholds.palette` (default 15%) it is a warning that names the colours: `webalt uses #f4a261 (68%), #c77dff (22.2%), which web doesn't use`. Masks and a screenshot part apply as for pixels. Measured on the reference apps: one design drawn by web, egui and JavaFX, 0% on every screenshot; the redesign with a palette of its own, 73–100%; the same redesign in the reference palette and font, 0% (experiments/redesign). A run of one component knows only that component's colours, so a colour another component uses may warn there and not in a full run. Text colours drawn only as glyphs are seldom solid, so they rarely enter a palette. *(TestComparePalettes, TestDesigns)*
- [R] **Round trips.** A trace, opacity or scale that goes out and comes back (a save star scaling 1 → 1.25 → 1, a hover in and out) is measured the same way as a round-trip clip, out and back. A round trip against a change that ends somewhere else is an infinite difference: ``one went out and came back, the other ended somewhere else``. *(TestRoundTripTraces, TestRoundTripAgainstOneWay, TestScaleTrace)*
- [R] **Opacity and scale.** An opacity trace is analysed the same way, as the times its opacity covered 10%, … 90% of its change; a change under 0.02 is no change. A target that can't read opacity (egui paints it into colours) says so in `cannot`, and the test is "not advertised" there. *(TestAnalyzeValues, TestOpacityTrace)*
- [R] **Clips** (`internal/compare/frames.go`). Each target records on its own clock (web: Chromium's screencast, a frame only when the page changes; JavaFX: a snapshot per pulse; egui: simulated frames at exactly 60 fps) and from its own moment, so clips are not compared frame by frame. Each clip becomes its **visual progress**: for every frame, 1 − its distance to the last frame / the first frame's distance to the last, over the pixels that change at all in the clip (the mean colour distance). When the target says its frames are held, frames more than 40 ms apart are a still picture, not a ramp; otherwise levels are interpolated across a gap (a slow pulse), as for traces. The curve is then compared like a trace; above `thresholds.motion` (default 40 ms, wider than `timing` because a clip shows every change on the screen and correct renderings measured up to 30 ms apart) it is a warning. A clip that ends close to where it started (a round trip) is measured out and back: up to 50% of its progress on the way out to the frame farthest from the first, the rest on the way back to the last, and says so in its note. `record: { part: x }` measures the clip over that part's box (before and after the steps, and 8 px around it) instead of the whole frame, so a small part's motion isn't lost in a phone screen. *(TestVisualProgress, TestRecordPartAndSelected)* On the reference switch, over 20 correct runs, clips measured 1.5–30 ms apart, 14 ms or less in 18 of them (position traces 0.0–0.5 ms); a 300 ms linear slide 76–104 ms; a 600 ms fade that only one target has, which no position trace sees, 182–189 ms. *(TestVisualProgress, TestClipMotionWarning, TestOldRecordReply; real: web.switch.dim, javafx.switch.dim)*
- [R] **Clips are lossless animated PNGs** (`compare.APNG`, no external encoder), each frame shown for its own duration. report.html shows each combination's clips with a chart of their visual progress and a player that shows every target's frame for the same moment, lined up at its first input (or where its picture starts to change). *(TestAPNG, TestClipMotionWarning)*
- [R] **Keyboard checks.** `keyboard: true` adds, for every part with role button (Enter, Space), switch or checkbox (Space), a test named ``keyboard: <key> on `<part>` does what a press does``: mount and press the part; mount, focus it and press the key; the events and every part's text, checked, value, disabled and (where advertised) selected must be the same. A difference fails, saying what each gave. Two cases are **not measured**, not passed or failed: the part isn't on screen when the component mounts (no box, or "no part"), with ``…isn't on screen when <id> mounts… If it is shown by a sheet, a dialog or another state, make that a component of its own (or a prop that mounts it shown), and give that component keyboard: true``; and the press changes nothing a test can read (choosing the segment already chosen), which leaves nothing to compare the key with. A target's `keys` (§4.4) leaves out the keys its platform doesn't use. *(TestKeyboardChecks, TestKeyboardPartNotShown, TestKeyboardInertPress, TestKeyboardTargetKeys; real: javafx.counter.no-enter)*
- [R] **Motion.** A trace becomes the times at which the point reached 10%, 20%, … 90% of its movement, interpolated between samples. Two traces are first shifted to line up as well as possible, by the mean difference, and then compared by their largest remaining difference in ms. This compares the shape and the length of the movement (timing function and duration), not when each target reacted. A named timing is compared the same way with the CSS curve. Above `thresholds.timing` it is a warning. *(TestAnalyzeAlignsStart, TestExpectedMatchesEasing, TestDifferMeasures, TestTimingWarning)*
- [R] **Direction.** Timing is measured along each trace's own movement, so it can't tell a thumb sliding up from one sliding across. Each pair of position traces is also compared by where the point went (from the first sample to the last, or to the farthest for a round trip): above `thresholds.direction` (default 30°) apart it is a warning, `web went right 20 px, webalt up 20 px`. Traced properties have no direction. *(TestDirection)*
- [R] **Movement under 1 px is no movement.** Moving against still is an infinite difference, and still against still is 0. *(TestDifferMeasures)*
- [R] **No retries.** A step that fails is reported, never repeated: a flaky target should show. *(the code has no retry path; TestFailMessage)*
- [R] **No state between runs.** Each run replaces `.omnicui/run/` entirely, and running the same check twice gives the same results. *(TestRepeatable, TestRepeatable for the real targets)*
- [R] **Limits:**
  - `times` is 1–100, and viewports are 16–4096 px *(schema)*;
  - a step's timeout is `stepTimeout` (default 10 s);
  - the start timeout is `startTimeout` (default 120 s);
  - an adapter's reply line may be up to 64 MiB;
  - the stderr kept for diagnosis is the last 40 lines; a failing or broken test keeps the last 20 lines written while it ran, printed under it (5) and in `results.json` (`stderr`). *(TestTimesAndStderr)*
- [R] **Stub generators always get every contract**, even when only some are checked, because they empty their output folder and rewrite it from scratch. *(TestStubsGetAllContracts; this was a bug, §17)*

---

## 6. Errors and recovery

| Class | Example | What the user sees | Status |
|---|---|---|---|
| bad input: contract | `part: incremnt` | `file:line  error: … did you mean …?`, every problem at once | exit 2, no target runs |
| bad input: usage | `omnicui check nope` | ``omnicui: `nope` is neither a target nor a component (known: …)`` | exit 2 |
| bad input: no project | run outside a project, or `--project` names a folder without `contracts/` | ``omnicui: no contracts/ folder (or omnicui/contracts/) here or in any parent folder; …`` or ``omnicui: <folder> is not an omnicui project: …`` | exit 2 |
| a step fails | `{"error":{"code":-32002,"message":"no part ui.counter.nope"}}` | `fail`, with the message, e.g. `press ui.counter.nope: no part ui.counter.nope` | fail |
| missing component | mount of a component the target lacks | `mount: no component ui.switch`, once for all combinations | fail |
| a dependency times out | no reply within `stepTimeout` | `broken: timed out after 500ms on press ui.counter.increment`; the process group is killed | broken |
| a crash | the adapter exits | `broken: the adapter exited with code 3 during mount ui.counter`, plus the last stderr lines | broken |
| a dropped connection | the adapter closes standard output | `broken: the adapter closed its standard output during advertise` | broken |
| a protocol violation | stray output, a wrong id, the wrong shape | `broken: stray output on standard output during mount ui.counter: "[javafx] mounting ui.counter"` | broken |
| can't start | the command doesn't exist | `broken: did not answer advertise: the adapter exited with code 127 during advertise` | broken, for every test |
| missing capability | no display for screenshots | `screenshot: not advertised by egui (no GPU adapter for wgpu)` | not advertised; fails only with `--require` |
| cancellation | Ctrl-C, or SIGTERM from a CI runner | `omnicui: interrupt; stopped 2 adapter(s)` (the adapters still running at that moment). Each adapter runs in its own process group, so it doesn't get the terminal's signal; omnicui kills every running adapter's process group (its process tree on Windows), with whatever it started (a browser, a JVM, cargo's child), and the partial run folder stays | exit 130 |
| partial completion | a target broke midway | its earlier results stand; later tests are `not run: the target broke earlier (…)`; other targets complete | broken |

- [R] **Retried:** nothing (§5).
- [R] **Kept for diagnosis:**
  - the last 40 lines of each adapter's standard error, in `results.json` and `report.html`;
  - every protocol line, in `protocol.log`;
  - structured events, in `omnicui.log`.

  *(TestCrashMidRun checks the stderr tail.)*
- [R] **After a crash or restart,** nothing is lost and nothing is done twice. A run has no external effects, only its own folder, and the next run replaces that folder. *(TestRepeatable writes a stale file and checks it is gone.)*

Tests: TestErrorReply, TestCrash, TestClosedOutput, TestTimeout, TestStrayOutput, TestWrongID, TestBadCommand, TestKillAll, TestCrashMidRun, TestHang, TestCannotStart, TestMissingComponent, TestContractErrorsStop, TestRequire, TestProjectAndOut.

---

## 7. Traceability and observability

- [R] **Correlation:**
  - every run has an id (`20261106T204140Z-8109aa172483`), which is in every `omnicui.log` line;
  - the id is passed to every adapter as `OMNICUI_RUN_ID`;
  - every request has a JSON-RPC id, unique per target per run;
  - a log line about a call carries `run`, `target`, `contract`, `test`, `combo`, `method`, `id` and `ms`.

  A reply in `protocol.log` is joined to its log line by target and id. *(TestLogFields)*
- [R] **Logs:**
  - `omnicui.log` is JSON lines from Go's `log/slog`, with the fields `time`, `level` (DEBUG, INFO, WARN, ERROR), `msg` and the fields above;
  - it never holds props or text values *(TestLogFields checks that no label text appears)*;
  - `protocol.log` holds the full messages, which carry contract data. It is a debugging artifact of the run, and the contracts contain no secrets.
- **Metrics:** the summary is the metric: counts of pass, fail, broken, not advertised and warnings, in `results.json` and on the last line of output.
- **Alerts:** not applicable, because omnicui is a local command and sends nothing anywhere. The exit status is the signal for CI, and warnings are lines in the output.

---

## 8. Security and permissions

- [R] **Adapter and stub commands are trusted, like a Makefile.** omnicui runs what `target.yaml` says, in the target's folder, with the user's permissions. Review a template before `omnicui add --template` copies or clones it.
- [R] **Contracts are data.** A YAML library and a JSON Schema read them, with no evaluation. YAML aliases resolve, but anchors can't create code.
- [R] **No network service.**
  - omnicui listens on nothing.
  - The reference web adapter serves its bundle on `127.0.0.1` with an ephemeral port, for its own browser only.
  - The web app's `serve.ts` (`run.sh app web`, for people, not omnicui) also serves only on `127.0.0.1`, on an ephemeral port unless `PORT` is set.
  - `add --template <git URL>` is the only command that reaches the network.
- [R] **No secrets** are read or written. `OMNICUI_DEFECTS` changes only the reference apps and targets.
- [R] **Paths.** The screenshot and record paths are chosen by omnicui inside the run folder. The web adapter's server, and the web app's, refuse paths outside their `dist/` folder.
- [R] **The run folder is deleted before every run**, including a folder named with `check --out`. Point `--out` at a folder of its own, never at one holding anything else.
- **Abuse limits:** not applicable, because there is no multi-user surface.

---

## 9. Performance

Measured on Linux x86-64 with 20 cores and no GPU (software Vulkan for egui, software rendering for JavaFX, headless Chromium), with `time` around the command and warm builds.

| Budget | Measured | How |
|---|---|---|
| A full check: 2 contracts × 7 tests × 2 colour schemes × 3 targets | 17.5 s wall time | `time bin/omnicui check` (in parallel; the slowest target, JavaFX, sets it) |
| The web target alone | 10.3 s | `time bin/omnicui check web` |
| The JavaFX target alone | 16.9 s | `time bin/omnicui check javafx` |
| The egui target alone, with rendering | under 2 s | `time bin/omnicui check egui` |
| Loading and checking both contracts | under 30 ms | `go test ./internal/contract` |
| omnicui's own unit and failure-injection suites | about 75 s, mostly deliberate timeouts | `make test-unit` |

Budgets for a project to set:
- a full check under a minute for a component library of this size;
- each request well under `stepTimeout`.

A run here uses about 200 requests per target.

---

## 10. User-facing surface

omnicui has two surfaces: the terminal output (§4.1) and `report.html`.

**`report.html`:**
- **States:** all passing; failures, with messages; broken targets, with their stderr; not-advertised capabilities, with reasons; `--require` failures; warnings, with diff images; no screenshots; one or many targets.
- **Keyboard:** a "Skip to the results" link comes first. Every table is in a focusable, labelled scroll region (`role="region"`, `tabindex="0"`), so wide tables scroll inside the region, not the page.
- **Accessibility, to WCAG 2.2 AA:**
  - one `h1`, sections labelled by their `h2`, and a `main` landmark;
  - every table has a `caption` and `th` scopes;
  - every image has a text alternative, and charts are `role="img"` with a description;
  - status is text, never colour alone;
  - contrast meets AA in light and dark;
  - **forced colours:** charts use `CanvasText`, and lines also differ by dash pattern;
  - **reduced motion:** clips show their first frame instead of animating (`<picture>` with `prefers-reduced-motion`); the player moves only when asked to.
- **Phone:** at 390 px wide the page does not scroll sideways, and long paths wrap.

[R] axe-core 4.14 finds 0 violations of WCAG 2.0, 2.1 and 2.2 A and AA in light, dark, phone and forced colours. *(`tools/omnicui/report-test/report.test.ts`; output in `docs/verification/report-a11y.log`)*

**The example components** (the counter and the switch) have the same look in all three apps:
- DejaVu Sans 16 px, from one font file, `shared/fonts/DejaVuSans.ttf` (§4.8);
- the same colours and sizes. Each target mounts them at the same position, (24, 24).

Their own accessibility is what the contracts test: names, checked state, and keyboard operation by Enter and Space.

---

## 11. Edge cases

| Case | Expected | Verified by |
|---|---|---|
| a contract with no viewports, colour schemes or thresholds | 320×240, light; area 30%, strength 28%, shift 8 px, spot 2%; timing 20 ms, motion 40 ms | TestDefaults |
| no `*.contract.yaml` files | exit 2, "there are no *.contract.yaml files" | `loadContracts` |
| no targets | exit 2, "add one with `omnicui add <name>`" | `doCheck` |
| a target.yaml with an empty adapter | `targets` shows "no adapter command in target.yaml yet" | TestAddRemove |
| two contracts with the same id | error, and the second is ignored | TestDuplicateIDs |
| YAML syntax error | `not valid YAML`, with the line | TestErrors/not yaml |
| a part that no test uses | a warning; the check still runs | TestUnusedPartWarns |
| `key` without a `key:` value | schema error | TestErrors/missing key for key action |
| a component the target lacks | fail once, "mount: no component …" | TestMissingComponent |
| a target that lacks a step's construct | that test is "not advertised" | TestNotAdvertised |
| a target that can't take screenshots | the tests pass; screenshots are "not advertised" with the reason; there are fewer pairs to compare | TestNotAdvertised |
| only one target took a screenshot | no pixel comparison for that test | TestNotAdvertised (1 comparison left) |
| screenshots of different sizes | 100%, "sizes differ" | TestPixelsSizes |
| nothing drawn in either screenshot | 0%, "nothing is drawn in either image" | TestPixelsDrawnAreaAndReach |
| a traced part that didn't move on one target | infinite difference, a warning | TestDifferMeasures |
| a target slower to react than another | no difference (traces are lined up) | TestAnalyzeAlignsStart |
| a steep easing sampled at 60 fps | measured to within 4 ms | TestExpectedMatchesEasing |
| the same message on every combination | shown once, without the combination | TestCollapse |
| checking one component | stub generators still get every contract | TestStubsGetAllContracts |
| a template with `node_modules/`, `target/`, `generated/`, `dist/`, `build/` or `.git/` | left out of the copy (TestAddTemplate checks `node_modules/`) | TestAddTemplate |
| a name with capitals or spaces | `add` refuses it | TestAddRemove |
| omnicui run from a repository's root, with the project in `omnicui/` | the project is found there | TestProjectAndOut |
| run from a folder outside any project, with `--project` or `OMNICUI_PROJECT` | that project is used | TestProjectAndOut |
| `--project` names a folder that is not a project | exit 2, "is not an omnicui project" | TestProjectAndOut |
| no project here or above | exit 2, "no contracts/ folder (or omnicui/contracts/) here or in any parent folder" | `project.Find` |
| `check --out <folder>` | the results and report are written there, and the printed paths name it | TestProjectAndOut |
| omnicui interrupted with adapters running | every adapter's process group is killed; exit 130 | TestKillAll |

---

## 12. Acceptance checklist

| # | Item | Tests | Result |
|---|---|---|---|
| 1 | Contracts load, typed by the JSON Schema; every mistake has its line and a suggestion | TestExampleContractsLoad, TestModel, TestDefaults, TestErrors (21 cases), TestUnusedPartWarns, TestDuplicateIDs, TestClosest | pass |
| 2 | Contract errors stop the check before any target runs | TestContractErrorsStop | pass |
| 3 | JSON-RPC 2.0, one line each way, ids matched | TestConversation, TestWrongID | pass |
| 4 | An error reply fails the test; the target stays usable | TestErrorReply, TestFailMessage | pass |
| 5 | A crash, a dropped connection, a timeout, stray output or a bad command breaks the target visibly, with stderr; hung processes are killed | TestCrash, TestClosedOutput, TestTimeout, TestStrayOutput, TestBadCommand, TestCrashMidRun, TestHang, TestCannotStart | pass |
| 6 | Capabilities are advertised; components are not | TestNotAdvertised, TestMissingComponent, R-REAL-1 | pass |
| 7 | `--require` makes a missing capability a failure | TestRequire, R-REAL-4 | pass |
| 8 | Pixels: the share of the drawn area, 2 px reach, warnings with diff images | TestPixelsIdentical, TestPixelsDrawnAreaAndReach, TestPixelsSizes, TestPixelWarning | pass |
| 9 | Motion: traces lined up and compared, also against CSS timings | TestAnalyzeAlignsStart, TestExpectedMatchesEasing, TestDifferMeasures, TestTimingWarning | pass |
| 10 | Clips are recorded and never compared | TestClip, TestAllPass | pass |
| 11 | No state: the same run twice gives the same result; the run folder is replaced | TestRepeatable, R-REAL-2 | pass |
| 12 | Stub generators get every contract; a failing one breaks its target | TestStubs, TestStubsGetAllContracts | pass |
| 13 | A contract change is a compile error in the apps that use the stubs: TypeScript in `apps/web` (`tsc --noEmit`) and Rust in `apps/desktop` (`cargo check --lib --no-default-features`) | R-REAL-5 | pass |
| 14 | The CLI's commands, exit codes and messages | TestUsage, TestUsageErrors, TestCheckExitCodes, TestAddRemove, TestAddTemplate, TestTargets, TestProjectAndOut | pass |
| 15 | Logs are structured, correlated, and free of text values | TestLogFields | pass |
| 16 | All three real targets pass every test, with no warning | R-REAL-1 | pass |
| 17 | Every deliberate defect is caught, on the right target only | R-REAL-3 (10 defects) | pass |
| 18 | report.html: 0 axe violations (WCAG 2.2 AA) in light, dark, phone and forced colours | `tools/omnicui/report-test/report.test.ts` | pass |
| 19 | omnicui builds for Linux, macOS and Windows on amd64 and arm64 | `make cross`; `make release` builds the same six | pass |
| 20 | A fresh copy installs from the lockfiles and passes every test, through `run.sh` | `fresh-copy.sh` | pass |
| 21 | Interrupted (SIGINT or SIGTERM), omnicui kills every running adapter's process group and exits 130 | TestKillAll (R-PROTO-12); a real check interrupted: 7 adapter processes during it, 0 after, exit 130 | pass |
| 22 | The project is found from a repository's root (`omnicui/contracts/`), or named with `--project` or `OMNICUI_PROJECT`; `check --out` writes the results elsewhere; `--version` prints the version and protocol | TestProjectAndOut (R-CLI-9) | pass |

---

## 13. Gaps and drift

The baseline is the design document (6 November 2026). Each row is a place where building it showed that the design was wrong or incomplete.

| # | Severity | The design said [B] | What is true | Evidence | Fix in this repository |
|---|---|---|---|---|---|
| G1 | **high** | Pixels: the "% of pixels changed" across the screenshot, threshold 2% | Too blunt for small components. A JavaFX counter drawn 80 px too low measured **0.6%**, and 13 px text instead of 16 measured **0.35%**: both under 2%, so both were missed | `docs/verification/metric-study.txt` | [R] the share of the **drawn area**, allowing 2 px of placement; default threshold 30%. The same defects now measure 100% and 38–62%, against 3–26% for correct versions |
| G2 | medium | A focused JavaFX `Button` ignores Enter, so the JavaFX counter needs an extra key handler | A focused JavaFX 21 `Button` fires on Enter, as a browser `<button>` does | with the handler removed, `the keyboard works too` passes on JavaFX | the handler was removed. The deliberate JavaFX defect is now an unnamed switch (`javafx.switch.no-label`) |
| G3 | medium | "A shell command starts the conversation" | Under `/bin/sh -c`, a shell such as dash keeps the adapter's standard output open, so the adapter closing it is seen only at the timeout | TestClosedOutput failed until `exec` was added | [R] start `adapter:` with `exec` (§4.3) |
| G4 | medium | Stubs are regenerated when a contract changes | They must always get **every** contract. A check of one component made the web stub generator delete the others' types, and the web build then failed | §17, TestStubsGetAllContracts | [R] stubs always get every contract |
| G5 | medium | Traces are compared "for duration, delay and easing" | Comparing raw times mixes in when each target reacted. Ease-out sampled at 60 fps also biased a crossing-based start time by 4 ms | TestExpectedMatchesEasing failed with a start-crossing alignment | [R] traces are shifted to line up before comparing. Delay is not compared ([REC-6]) |
| G6 | low | Each reply waits for the UI to settle; without that, screenshots are taken mid-animation | True as a rule, but in this suite it could not be shown to matter. With settling switched off on web and on JavaFX, no result changed: Playwright's and TestFX's own waits outlast the 200 ms slide | the experiment is in §17 | the rule stays [R] (§4.4); no fix to prove |
| G7 | low | The Rust target is GPUI, with a per-component `Inspect` trait whose answers are "the component's own word" | With egui, every field is read from AccessKit, the tree a screen reader reads, so no component reports itself | `omnicui/targets/egui/src/main.rs` | the egui target needs no `Inspect` trait |
| G8 | low | Clips become replicas for a product's docs | Branding; omnicui is standalone | — | clips are "for documentation or demos" |
| G9 | low | egui_kittest checks accessibility itself | Its check panics on an unnamed widget, so the adapter answered "mount panicked" instead of the contract's message | scenario `egui.switch.no-name`, first run | [R] the adapter turns kittest's own check off and lets the contract judge, and a panic's message is now reported |

---

## 14. Improvements

Ordered by value against cost. None of these is built.

1. **[REC-1] Per-pair tolerance.** The JavaFX/egui counter differs by 26% when correct, against 30% for the threshold, which is the tightest margin in the suite. A contract could set a threshold per pair, or omnicui could report how far each pair is from its threshold. *Verify:* the deliberate defects are still caught with the narrower margins.
2. **[REC-2] Masks** are built: per test and per contract, with optional parts and part-cropped screenshots (§4.2). Still open: masks a target declares itself for its own system UI (a status bar, the soft keyboard), so that contracts needn't name them. *Verify:* a target's declared region is left out without a contract mask.
3. **[REC-3] Differential fuzzing.** Random action sequences built from the parts, run on every target, reporting the first step where they diverge and saving it as a test. *Verify:* the `double` fake bug is found without a hand-written test.
4. **[REC-4] A coverage report:** which parts and actions each target's tests reach. *Verify:* with the unused-part warning as a start.
5. **[REC-5] Several adapter instances per target**, so that one target's tests run in parallel. JavaFX sets the wall time today. *Verify:* identical results at N = 1 and N = 4.
6. **[REC-11] Keyboard checks from the tests.** The generated keyboard tests mount the component and press one part; a part inside a sheet is "not measured" and the advice is a component of its own. Deriving them from a test's own steps (swap its last press for focus and a key) would reach such parts without splitting the contract. *Verify:* a sheet's button is checked without a component of its own.
7. **[REC-6] Compare delay too**, as a separate number: how long after the input the movement starts. *Verify:* a fake adapter with a 100 ms delay warns, and one with a 10 ms delay doesn't.
8. **[REC-7] Batch requests** (JSON-RPC batches) for steps that don't need their replies one by one. *Verify:* the same results with fewer round trips.
9. **[REC-8] Check roles as well:** built as the `role` field, which a test checks explicitly. Still open: checking every part's role automatically against the contract's. *Verify:* a `div` styled as a button fails.
10. **[REC-9] Clips** are now lossless animated PNGs. Still open: WebM or MP4 next to them, for smaller files. *Verify:* the frame count and timing match.
11. **[REC-10] A conformance kit for adapter authors:** a set of fake components and contracts that an adapter must pass, to check the adapter itself. *Verify:* each reference adapter passes it, and an adapter with a broken `read` fails it.

By platform:
- **web:** run Firefox and WebKit as further targets, with the same adapter.
- **egui:** verify on real GPUs (Metal, DX12).
- **JavaFX:** verify on a real display, with LCD text smoothing.

---

## 15. Verification

| Suite | What it covers | Passed / total | Log |
|---|---|---|---|
| omnicui unit and failure injection (`make test-unit`, run in `tools/omnicui`) | contracts, protocol (crash, closed output, timeout, stray output, wrong id, bad command, killing every adapter on interruption), the check (statuses, warnings, `--require`, repeatability, stubs, logs), the CLI (including finding the project, `--project`, `OMNICUI_PROJECT`, `--out` and `--version`) | 67 / 67 | `docs/verification/fresh-copy-run-sh.log` (the `ok` lines) |
| real targets (`make test-real`, run in `tools/omnicui` on the project `omnicui/`) | baseline, repeatability, 10 deliberate defects, `--require`, stubs catching a contract change (compiled in `apps/web` with `tsc` and in `apps/desktop` with `cargo check --lib --no-default-features`) | 15 / 15 (5 tests; 10 defect cases) | `docs/verification/test-real.log`, `docs/verification/scenarios/` |
| report accessibility (`make test-report`) | axe WCAG 2.2 AA, plus structure, in 4 settings; on a passing and on a failing report | 8 / 8 (0 axe violations) | `docs/verification/report-a11y.log` |
| cross-compile (`make cross`) | linux, darwin and windows, amd64 and arm64 | 6 / 6 | `docs/verification/cross-compile.log` |
| web typechecks (`npx tsc --noEmit` in `apps/web`, then in `omnicui/targets/web`) | the app's components against their stubs; the target's adapter, harness and stub generator against the app | pass | in `make install` and `run.sh setup` |
| interrupting a real check (SIGINT during `omnicui check`) | every adapter process is gone afterwards, and the exit status | 7 adapter processes during it, 0 after; exit 130 | `docs/verification/interrupt.txt` |
| fresh copy through `run.sh all` (`fresh-copy.sh`) | clone into an empty folder, then doctor, setup (from the lockfiles), check, demo, and every suite above | pass | `docs/verification/fresh-copy-run-sh.log` |
| Windows builds of the tests (`GOOS=windows go test -c`) | the rpc, check and CLI test packages compile for Windows | 3 / 3 | — |

**Integration against real dependencies:**
- real adapter processes;
- real Chromium (Playwright 1.64.1);
- real egui 0.37 rendered by wgpu on Mesa's lavapipe;
- real JavaFX 21.0.13 on Monocle with software rendering.

**Failure injection**, each with its outcome asserted:
- a worker stops mid-task: TestCrash, TestCrashMidRun;
- a connection drops: TestClosedOutput;
- a dependency times out: TestTimeout, TestHang;
- omnicui itself is interrupted: TestKillAll, plus an interrupted real check;
- a restart: TestRepeatable. omnicui is stateless, so a restart is a new run;
- the same request twice: TestRepeatable, plus R-REAL-2 on the real targets.

**Proof that each fix matters.** Each fix was switched off, its test failed, and then it was switched back on:

| Fix | Switched off by | Then | Switched back on |
|---|---|---|---|
| JavaFX components keep their size (not stretched) | `OMNICUI_DEFECTS=javafx.host.stretch` | 10 pixel warnings, all involving javafx | 0 warnings (`99-baseline-again`) |
| JavaFX sends `System.out` to stderr | `javafx.adapter.stdout` | javafx broken: stray output | pass |
| egui doesn't draw the pointer | `egui.render.cursor` | pixel warning | pass |
| the drawn-area measure (G1) | the old whole-image measure | 3 of 4 layout defects missed | all caught |
| `exec` in `adapter:` (G3) | `exec` removed from the test adapter | TestClosedOutput timed out | pass |
| stubs get every contract (G4) | the selection passed to stubs | the web target broke on `check switch` | TestStubsGetAllContracts passes |
| trace alignment (G5) | the start-crossing alignment | TestExpectedMatchesEasing: 4.2 ms error | pass |

**Tests on what ships:**
- the real-target suite drives the built `bin/omnicui`;
- the web adapter serves a minified production bundle of the app's own components (`web-app/components`), with the app's React;
- JavaFX runs the target's packaged jar, built against the app's jar as installed in `~/.m2`;
- egui runs the target's release binary, built against the app's library (`apps/desktop`, without its window).
- The apps themselves build on their own: `apps/web` typechecks, `apps/desktop` builds with `cargo build --release --locked`, and `apps/desktop-javafx` with `./mvnw install`, in every `setup`.

**Not verified, and why:**
- **omnicui, the targets and the run scripts on macOS and Windows.** No such machine was available.
  - omnicui and its tests are cross-compiled for both.
  - No Windows run scripts are included; on Windows, omnicui itself is only cross-compiled.
  - On Windows, the `cmd /C` path (dropping `exec`, `taskkill /T`, also on interruption) is untested.
  - Pixel scores between targets may differ on macOS, where Chromium draws text with the system's renderer.
- **egui on a real GPU, and JavaFX on a real display.** Only software rendering was available here.
- **Device pixel ratios other than 1, right-to-left text and long labels.** None of the example contracts uses them.

---

## 16. Code map

**The baseline:** there is no product code. The design document is restated in §1 and §2.

**The reference project (the repository's root):**

```
apps/                     the real apps; each builds and runs without omnicui
  web/                    React (npm package web-app)
  desktop/                egui (crate desktop: a library and the window)
  desktop-javafx/         JavaFX (Maven artifact app:desktop-javafx)
omnicui/                  the omnicui project, found from anywhere in the repository
  contracts/              counter.contract.yaml, switch.contract.yaml
  targets/{web,egui,javafx}/   one target per app: target.yaml, the adapter, the registry
  .omnicui/run/           the last check's results (not committed)
shared/fonts/             DejaVu Sans, the one font every app draws with
tools/omnicui/            the omnicui command, in Go, and its own tests
run.sh, Makefile, fresh-copy.sh, README.md, docs/ (SPEC.md, media/, verification/)
```

Targets import apps, never the reverse (§4.8).

**The apps:**

| File | What it does |
|---|---|
| `apps/web/src/main.tsx` | the web app's page: the counter and the switch, a dark colour scheme toggle, the events they report |
| `apps/web/src/components/Counter.tsx`, `Switch.tsx`, `components.css` | the React counter and switch, with their `data-part` tags; the shared look, with the font from `shared/fonts` |
| `apps/web/src/components/defects.ts`, `index.ts` | the app's deliberate defects; the library export (`web-app/components`) |
| `apps/web/src/omnicui/` | the types generated from the contracts (committed; exported as `web-app/omnicui`) |
| `apps/web/serve.ts`, `package.json` | builds the app with esbuild, serves it on 127.0.0.1 and opens the browser; the package, exporting `./components` and `./omnicui` |
| `apps/desktop/src/lib.rs` | the egui library: `part` (the AccessKit `author_id` tag), `defect`, `Look`, `fonts`, the CSS ease-out curve; `FONT`, `include_bytes!` of `shared/fonts/DejaVuSans.ttf` |
| `apps/desktop/src/counter.rs`, `switch.rs` | the egui counter and switch (`new(props)`, `ui(&mut self, ui, look) -> Option<event>`) |
| `apps/desktop/src/omnicui/` | the types generated from the contracts (committed) |
| `apps/desktop/src/main.rs`, `Cargo.toml` | the eframe window, behind the default feature `app`; the crate `desktop` |
| `apps/desktop-javafx/src/main/java/app/Main.java` | the JavaFX window |
| `apps/desktop-javafx/src/main/java/app/components/Counter.java`, `SwitchControl.java`, `Theme.java`, `Defects.java`, and their CSS in `src/main/resources/app/components/` | the JavaFX counter and switch, with their node ids as part tags; the look and the font; the app's deliberate defects |
| `apps/desktop-javafx/pom.xml`, `mvnw` | the Maven project `app:desktop-javafx`; copies the font from `shared/fonts` into the jar; `./mvnw install` puts it in `~/.m2` |

**The omnicui project:**

| File | What it does |
|---|---|
| `omnicui/contracts/*.contract.yaml` | the counter and the switch |
| `omnicui/targets/web/adapter.ts` | Playwright adapter: a production esbuild bundle of the harness (with the app's React), a local server, Chromium, the CDP accessibility tree, rAF traces, screencast recording |
| `omnicui/targets/web/harness.tsx`, `harness.html` | the registry, and the page the adapter opens: mounts any of the app's components, fresh, and records their events |
| `omnicui/targets/web/stubs.ts`, `package.json`, `tsconfig.json` | the TypeScript stub generator (its output folder is its argument); the dependency on `web-app`; React's types taken from the app |
| `omnicui/targets/egui/src/main.rs` | egui_kittest adapter: AccessKit lookup by `author_id`, simulated input, wgpu screenshots, frame-by-frame traces; the target's own defect `egui.render.cursor` |
| `omnicui/targets/egui/src/components.rs` | the registry: the trait `Mounted` over the app's components, and `create` |
| `omnicui/targets/egui/Cargo.toml`, `stubs/` | the path dependency on `apps/desktop` without its default features; the Rust stub generator (its output folder is its argument) |
| `omnicui/targets/javafx/src/main/java/omnicui/javafx/Adapter.java`, `JavaFxTarget.java` | the JavaFX adapter: the JSON-RPC loop, and one method per construct with TestFX on Monocle; the target's own defects `javafx.adapter.stdout` and `javafx.host.stretch` |
| `omnicui/targets/javafx/src/main/java/omnicui/javafx/Component.java`, `Components.java` | the component interface; the registry |
| `omnicui/targets/javafx/pom.xml` | depends on `app:desktop-javafx`; builds `target/omnicui-target-javafx.jar` |

**The omnicui command (`tools/omnicui/`):**

| File | What it does |
|---|---|
| `cmd/omnicui/main.go` | the command: `check` (with `--require` and `--out`), `targets`, `stubs`, `contracts`, `add`, `remove`, `version`; `--project`; the name selection; on SIGINT or SIGTERM, `rpc.KillAll` and exit 130 |
| `schema/contract.schema.json`, `embed.go` | the contract's JSON Schema (2020-12), embedded in the binary; also what editors use |
| `internal/contract/load.go` | YAML → values with line numbers → schema → model → cross-reference checks, with did-you-mean suggestions |
| `internal/contract/model.go`, `roles.go` | the contract model and its defaults; the role table |
| `internal/rpc/client.go`, `proc_*.go` | starts an adapter in its own process group; JSON-RPC client with timeouts; broken detection; stderr tail; wire log; the list of running adapters and `KillAll` |
| `internal/project/project.go` | finds the project (`Find`, walking up, accepting `contracts/` or `omnicui/contracts/`; `Open`, for `--project`); the run folder (`.omnicui/run/` or `--out`); loads `target.yaml`; `add` (with templates) and `remove` |
| `internal/check/run.go` | runs every test on every target; advertising; statuses; `--require`; stubs; the summary |
| `internal/check/comparisons.go` | pixel and timing comparisons between targets |
| `internal/check/print.go`, `report.go` | the terminal table; `report.html` |
| `internal/compare/pixels.go`, `trace.go`, `clip.go` | the drawn-area pixel measure and diff image; trace analysis and CSS timing curves; GIF clips |
| `internal/testutil/testutil.go` | helpers for the tests: the repository and project roots, the fake adapter built once, throwaway projects |
| `testdata/fakeadapter/main.go` | a fake target for omnicui's own tests, which can be told to fail in every way |
| `integration/real_test.go` | the real targets, the defects, `--require`, stubs catching contract changes (build tag `real`) |
| `report-test/report.test.ts` | report.html in Chromium: axe and screenshots |

**The rest:**

| File | What it does |
|---|---|
| `run.sh` | try it on macOS or Linux: doctor, setup, app, check, demo, test (§4.1a) |
| `Makefile` | the same steps for `make` (`install`, `check`, `test`, `test-unit`, `test-real`, `test-report`, `cross`); `make release` (the version from `VERSION`, or `VERSION=<version>`): omnicui for linux, darwin and windows on amd64 and arm64, built with `CGO_ENABLED=0 -trimpath -ldflags "-s -w -X main.Version=<version>"`, plus `omnicui-src-<version>.tar.gz` (the command's source, without the real-target and report tests), and `SHA256SUMS`, in `dist/` |
| `VERSION`, `notes/<version>.md`, `scripts/`, `.github/` | the version and its release notes; `check-notes.mjs` (the notes check, with its tests), `release-body.mjs` (a release's text), `neutral.mjs` (omnicui names no project that uses it); CI on merge requests and `main`, the notes check, a release `v<VERSION>` with the files of `make release` whenever `main` names a version that has none, and the ruleset protecting `main` |
| `fresh-copy.sh`, `README.md` | the fresh-copy check; how to use |

---

## 17. Found while building

1. **dash doesn't hand over to the command.** Under `/bin/sh -c "./adapter"` on Debian, the shell stays as the parent and keeps the adapter's standard output open. *Fix:* `exec` in `adapter:` (G3).
2. **Go's `cmd.StdoutPipe()` can lose the last replies.** `Wait` closes the pipe as soon as the process exits, which can be before the reader has read everything. *Fix:* a pipe of omnicui's own (`os.Pipe`), closed in the parent after `Start`.
3. **Stubs given only the selected contracts** deleted the other components' generated types (G4). *Fix:* stubs always get every contract.
4. **egui 0.37 renamed `WidgetType` to `Role`** (`accesskit::Role`), and `Id::new` to `Id::unique`. The switch now reports `Role::Switch` directly.
5. **egui has no part ids.** Parts are tagged by writing the part id into the widget's AccessKit node as its `author_id` (`Context::accesskit_node_builder`). The adapter finds parts there.
6. **egui_kittest turns animations off** through `style.animation_time`. Animations with an explicit time (`animate_bool_with_time_and_easing`) still run, on simulated time, one `step_dt` per frame, so egui's traces are exact on every run.
7. **egui_kittest's own accessibility check panics on unnamed widgets** (G9). *Fix:* turned off, and panic messages are reported.
8. **egui_kittest draws the mouse pointer into renders.** *Fix:* `with_render_cursor(false)` (the `egui.render.cursor` defect turns it back on).
9. **wgpu needs a Vulkan driver.** On a machine without a GPU, Mesa's lavapipe works if the Vulkan loader can find it (`VK_ICD_FILENAMES`, and the loader library on `LD_LIBRARY_PATH`). Without it, the adapter probes at start-up and advertises "no GPU adapter for wgpu".
10. **A JavaFX `StackPane` stretches its child** to fill the window, which centred the components vertically. *Fix:* children keep their preferred size (the `javafx.host.stretch` defect).
11. **JavaFX draws LCD subpixel text** (colour fringes) by default, which was the largest source of pixel noise. *Fix:* `-fx-font-smoothing-type: gray`.
12. **JavaFX shows the focus ring after a mouse click.** *Fix:* `:focus-visible`, available since JavaFX 19.
13. **TestFX's robot API names hamcrest types,** so hamcrest must be on the compile path.
14. **Monocle headless needs `prism.order=sw`,** and `java.awt.headless=true` for `SwingFXUtils` and `ImageIO`.
15. **A focused JavaFX `Button` fires on Enter** (G2).
16. **Playwright names the space bar `" "`, not `Space`.** The web adapter maps it.
17. **The web adapter reads names and states from Chromium's own accessibility tree** (CDP `Accessibility.getPartialAXTree`), not from attributes, so that `aria-labelledby` and roles are resolved as a screen reader would resolve them.
18. **`getBoundingClientRect` includes CSS transforms,** so a `requestAnimationFrame` loop traces a CSS transition exactly. The web trace matches CSS `ease-out` to within 0.2 ms.
19. **A sampled ease-out biases a start-crossing alignment** by up to 4 ms at 60 fps (G5). *Fix:* align by the mean difference across levels.
20. **Settling could not be shown to matter here** (G6). With it switched off, no result changed on web or JavaFX, because their drivers' own waits outlast a 200 ms transition.
21. **The whole-image pixel percentage missed displaced and resized components** (G1). A metric study over four deliberate layout defects led to the drawn-area measure with a 2 px reach.
22. **Making one `target.yaml` work on Windows.** `cmd` has no `exec`, so omnicui drops a leading one there. A `java -cp a:b` classpath only works on Unix, so the jar's manifest now carries it. `./target/release/adapter` isn't a valid command in `cmd`, so egui starts through `cargo run`. Windows PowerShell 5.1 turns a native program's redirected stderr into errors, so `run.ps1` checks exit codes and not stderr.
23. **Log lines about calls weren't linked to their test, and named the target twice.** The JSON-RPC client logged with the target's logger. *Fix:* the client is handed each test's logger (`SetLog`). TestLogFields now checks that every call after `advertise` carries its contract, test and combination, and names the target once.
24. **Moving the components out of the targets and into real apps.** Each target then needs the app as a dependency, and each app must not need omnicui:
    - the stub generators take their output folder as an argument and write into the app's `src/omnicui/`, and the output is committed, so each app builds on its own;
    - the egui target depends on `apps/desktop` with `default-features = false`, so that it never builds eframe and the window; the stub test checks the app with `cargo check --lib --no-default-features` for the same reason;
    - the web target bundles the app's components, so it must use the app's React and only one copy of it: an esbuild plugin resolves `react` and `react-dom` from the app, and `tsconfig.json` paths point their types there;
    - the JavaFX target finds the app as a Maven artifact, so the app is installed into `~/.m2` (`./mvnw install`) before the target is packaged;
    - the font moved to `shared/fonts/`, and each app takes it from there in its own way (§4.8).
25. **An interrupted omnicui relied on each adapter noticing its standard input close.** Each adapter runs in a process group of its own, so the terminal's Ctrl+C never reaches it, and nothing made an adapter that was busy in a step, or what it had started, exit. *Fix:* omnicui keeps a list of running adapters and kills each one's process group on SIGINT or SIGTERM, then exits with 130 (TestKillAll).
26. **A repository keeps its omnicui project in `omnicui/`, next to its apps,** so walking up to the first folder with `contracts/` found nothing from the repository's root. *Fix:* discovery also accepts `omnicui/contracts/` at each level, and `--project` or `OMNICUI_PROJECT` names the project outright (TestProjectAndOut).

---

## 18. Evidence

The files are in `docs/media/` and `docs/verification/`.

- **Screenshots** of both components on all three targets, light and dark, side by side: `docs/media/screenshots/`.
- **Diff images** for a passing run and for each layout defect: `docs/media/diffs/`.
- **Clips** of the switch sliding, recorded by each target: `docs/media/clips/`. A headless driver has no visible pointer, so the clip shows the result of the press, and its caption names the step.
- **The trace chart** of the thumb on all three targets, against CSS `ease-out` 200 ms: `docs/media/report/`.
- **report.html** at desktop, phone, dark and forced colours: `docs/media/report/`.
- **Terminal transcripts** of every scenario (baseline, each defect, `--require`, the stub compile errors): `docs/verification/scenarios/*.txt`.
- **One unit of work, from start to failure to recovery:** the JavaFX target breaking on stray output, then the next run passing. This includes its `omnicui.log` and `protocol.log`: `docs/verification/unit-of-work/`.
