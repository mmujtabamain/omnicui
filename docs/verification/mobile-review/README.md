# The mobile stress test (Trailmate): what it found

An agent built Trailmate twice, in Jetpack Compose (Android, rendered with Robolectric) and SwiftUI (iOS Simulator). It used a Django backend and vendored omnicui v0.2.0 unchanged; the copy is byte-identical to the release. It ran for 4.6 hours (15:17–19:52 UTC) on an 8 GB Mac before the user stopped it.

**Last completed check:** `118 pass · 39 fail · 0 broken · 80 not advertised · 30 warnings`.
- Android: 74 pass, 39 fail. 38 of the failures are generated keyboard tests; the other is a trace of a sheet that doesn't exist yet when the trace starts.
- iOS: 44 pass, 69 not advertised. XCTest gives it no key, focus or per-frame trace.
- Both deliberate defects (Save not toggling) were caught.

This review is from the run's source, its GAPS.txt and its transcript; its captures, logs and git history were not part of the handoff. The time figures come from the transcript's timestamps.

## Why it took so long

| UTC | what | time |
|---|---|---|
| 15:17–15:42 | vendoring, backend, both apps | 25 min |
| 15:42–15:50 | first `omnicui targets`: **no output for over 3 minutes** while Gradle and the Simulator started cold, then interrupted | 8 min |
| 15:50–16:57 | per-target and per-component checks, fixing its own adapters | 67 min |
| 16:57–17:25 | first full check; stopped at the user's request | 28 min |
| 17:25–18:09 | gitignore, APK, run scripts, Device Hub, masks question | 44 min |
| 18:09–18:42 | full check, interrupted to fix the iOS viewport | 33 min |
| 18:42–19:12 | **the completed baseline** | about 30 min |
| 19:12–19:42 | fixes and focused smoke scripts, then a full check, stopped | 30 min |
| 19:42–19:52 | packaging | 10 min |

What made it slow, in order:

1. **A full check takes 25–30 minutes, and omnicui can't run less than a target or a component.** The agent wrote four scripts of its own (`viewport-smoke.py`, `native-scenarios.py`, `standalone-keyboard.py` and `protocol-smoke.py`) to run single cases, because there is no `--test` filter and no "rerun what failed". It knew about `check <target>` and `check <target> <component>` and used them.
2. **The generated keyboard tests are expensive and mostly wrong.** Each one mounts twice and reads every field of every part. On iOS each read is an XCTest snapshot. They start from the first screen, so a button inside a sheet or dialog always fails (38 false failures). A press that changes nothing, such as choosing the segment that is already selected, passes without proving anything.
3. **Its own adapter bugs**, about 30 entries marked "OWN": store lifecycle, focus mode, scroll reveal, the iOS safe area, the keyboard pan, slider thumb geometry, a stale XCTest runner, and stale evidence files. Each one cost a rebuild and often a full run. The agent says it should have frozen sooner.
4. **The host:** 8 GB of memory, with Robolectric and the Simulator running at once (omnicui starts all targets in parallel). It added a lock of its own and raised `startTimeout` to 3600 s to wait for it.
5. **Cold dependencies:** Robolectric downloaded 360 MB with no output (more than 580 s); xcodebuild and the Simulator boot took minutes.

omnicui can fix 1, 2 and part of 4 and 5. Item 3 is the agent's.

## The gaps, sorted

### Real omnicui gaps, small and clearly worth fixing
| # | gap | evidence | fix |
|---|---|---|---|
| A | No way to run one test or rerun the failures | four home-made smoke scripts | `check --test <glob>` and `check --failed` (from the last `results.json`) |
| B | No time per test anywhere | the agent couldn't say where time went | progress lines show each test's time; `results.json` gets `ms`; the summary lists the slowest tests |
| C | Silent start-up | over 3 min with no output, interrupted | a "starting android… 40 s" line every 10 s, with the adapter's last stderr line and the time left before `startTimeout` |
| D | Keyboard tests press parts that aren't on screen yet | 38 of 39 Android failures | derive keyboard checks from the tests: replay a test's steps and swap the last `press` for `focus` + key. Parts no test presses are skipped and listed |
| E | A keyboard check of a press that changes nothing passes | units "km" and theme "Light" | call such a check "not measured: the press changed nothing", not a pass |
| F | Keyboard keys are desktop rules (button = Enter and Space) | Compose ignores Space | a target can declare its keys per role in `advertise` (`keys: {button: [Enter]}`); the report says so |
| G | Generated keyboard tests log RPC calls under the previous test's name | its DEBUG log | set the call log in `runKeyboard`; a one-line bug |
| H | Adapter stderr is kept only when start-up breaks | empty diagnostics in `results.json` | keep the stderr lines written during a failing test in that test's messages |
| I | Targets always start in parallel | 8 GB host swapping | `exclusive: true` in `target.yaml`, so that target runs alone; or `check --serial` |
| J | omnicui commits 36 MB of `report-test/node_modules` | every vendored copy carries it | remove it from git and ignore it; CI runs `npm ci` |

### Real gaps in the contract language, which need design
| # | gap | proposal |
|---|---|---|
| K | `trace` and `record` find their part before the steps run, so an entering sheet or hero can't be traced | look the part up lazily, at its first frame |
| L | No `selected` field and no tab, radio or segment roles | add `selected` to button, plus roles `tab` and `radio` |
| M | Only opacity traces, so the save star's 1.25× spring can't be checked | `property: scale` (and width/height from bounds) |
| N | A clip of the whole screen can't see a small animation, and a round trip isn't measured | `record: { part: x }` crops the clip to the part's bounds |
| O | No masks for system material (iOS Liquid Glass, the keyboard, pickers) | `screenshot: { mask: [part…] }` or a target-declared `system` region; the raw numbers stay in the report |
| P | No scroll, swipe, pull-to-refresh, back, pickers, IME, safe areas, rotation or text size | step 1: `scroll` (to a part), `swipe` (part, direction) and `back`; text size and orientation as viewport props (the agent already used `fontScale` as a prop). Pickers, the soft keyboard and system dialogs stay platform-specific for now |

### Platform limits, not omnicui's
- Neither Compose semantics nor XCTest exposes drawn opacity. XCTest has no general key or focus input, and no per-frame positions. These belong in `cannot`, which is what the agent did.
- Robolectric's content-view capture leaves out dialog windows and the soft keyboard; the Simulator includes them.
- Simulator video is H.264, so iOS clips are lossy and carry codec noise.

### The agent's own mistakes

It marked about 30 entries "OWN". Below are those that cost the most, and then the ones it reported as omnicui or platform gaps that were really its own choices.

**Its contracts: whole screens as components.** omnicui compares a *component*, mounted on its own. The agent made each screen one component and put into it the parts of its sheet, its dialog, its error state, and other screens.
- `trailmate.explore` has 21 parts. These include the Filters sheet's `loop`, `apply`, `reset` and `distance`, the error state's `retry`, and `detailTitle`, which belongs to another screen (mapped by its own `registry.json`).
- `trailmate.profile` has 16, including the sign-out dialog's `cancel` and `confirm`.
- With `keyboard: true` that became **68 generated tests next to its 45 own (113)**. 38 of those 68 fail because their part isn't on screen when the component mounts. The fix is in the contract: a `trailmate.filters` component, a `trailmate.signOut` dialog, and the error state as a prop. The task prompt suggested "screens or flows as components", so part of this came from the prompt.

**Its Android target starts Gradle for every check.** `adapter.py` runs `gradle :adapter:testDebugUnitTest --no-daemon` and relays JSON over a socket to a Robolectric test. So every `targets` or `check` pays a cold Gradle and Robolectric start, with an internal 1180 s deadline and `startTimeout: 3600s` on top. Building the test classpath once and starting the JVM directly, as the JavaFX reference target does (and the prompt asked it to copy), would start in seconds.

**Its iOS target: XCTest for everything.**
- Every action and every read goes through a long-polled HTTP queue to an XCTest runner, and each read is an accessibility snapshot. That is the slowest of the options the prompt allowed.
- That choice is also why iOS "cannot" trace or read opacity. An in-app test server could read the presentation layer every frame (`CALayer.presentation()` on a display link) and draw the hosting view at scale 1.

**Its iOS screenshots add their own differences.**
- It captures the whole Simulator screen of a 402×874 device (the size is hard-coded), crops it and shrinks @3x to 1x with Lanczos. Android draws natively at 1x.
- So every iOS image is resampled, and the 393×852 viewport sat inside a larger device's safe area: the 22 pt footer offset and a full rerun.
- Clips came from H.264 video, so they are lossy.

**Scroll "reveal" inside `press`.**
- Its adapters scroll a part into view before pressing it, and each platform scrolls a different distance.
- At 360×780 that alone produced a 12 pt offset: 57% spot on "expand description", which it reported as a native accessibility limit.

**Tests that check little.**
- "dark mode", "light mode" and "system theme" only expect the name to read "Alex Rivera". The theme is checked only by the screenshot.
- "filter by difficulty" relies on which trail happens to be first.
- One oracle was wrong (Alpine Lake), and it fixed that one itself.

**How it worked.**
- It ran both native runtimes at once on 8 GB, then added a lock inside the adapters, which uses up start-up time.
- It wrote four smoke scripts and an extra `nativeSwipe` method outside omnicui, and produced a 4.65 GB zip of 4691 files.
- It verified fixes with full runs where `check android detail` would do. It says itself that it should have frozen sooner.

**Its other "OWN" items:**
- the store lifecycle;
- touch versus keyboard focus mode;
- the iOS keyboard panning a fixed surface;
- an input marker taken before XCTest's preflight, which gave a 2872 ms "motion difference";
- slider drags started off the thumb;
- a stale XCTest runner;
- stale `status.json` and `failure.txt` files;
- PATCH unsupported by `HttpURLConnection`;
- SwiftUI identifiers swallowed by containers;
- the spring at twice its length.

**What omnicui should still do, given those mistakes:**
- **The keyboard checks need a clear message.** Today a part that isn't on screen shows up as a failure. It should be "not run: `apply` isn't shown when `trailmate.explore` mounts; give it its own component, or a prop that shows it".
- **Start-up should print progress.** The cold Gradle start ran for minutes with no output.
- **The render check should catch hosting mistakes.** It passed while mounted screens were offset, so it should check the mounted size too.
- **A timing difference over a second should be called out.** It is almost certainly the input clock, and the report should say so.

## What 0.2.1 changed because of it

| finding | in 0.2.1 |
|---|---|
| A, B | `check --test`, `check --failed`; `ms` per result and target, the slowest tests in the summary |
| C | a start-up line every 10 s with the adapter's last stderr line |
| D, E, F | generated keyboard tests are "not measured" for a part not on screen at mount (with the advice above) and for a press that changes nothing; `advertise` may say `keys` per role |
| G | keyboard tests' calls logged under their own name |
| H | a failing test keeps its adapter's stderr |
| I | `exclusive: true`, `check --serial` |
| J | `report-test/node_modules` out of git |
| K | traces look their part up at each frame (protocol and reference targets) |
| L | `selected` on buttons |
| M | `property: scale`; round trips measured out and back |
| N | `record: { part: x }`; round-trip clips measured |
| O | `screenshot: { mask, part }`, contract `mask`, `optional` parts |
| P | `do: <name>` steps the targets define and advertise; SPEC lists conventions (scroll, swipe, refresh, back, pick) |

What is still open is in GAPS.txt, 18–22.
