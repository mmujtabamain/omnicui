# Todo: one product, two apps, one set of contracts

A todo app built twice by hand, for Android (Kotlin, Jetpack Compose, Material 3) and for the
desktop web (React, TypeScript), and checked by one set of omnicui contracts. `SPEC-APP.md` is the
specification a third or fourth version (iOS, macOS) is built from; `GAPS-todo.txt` lists what
omnicui couldn't express or got wrong, and my own mistakes; `PROGRESS.md` is the log.

```
apps/android/        Gradle project (wrapper): ui/ (the screens, a library: todo:ui), app/ (the activity)
apps/web/            npm package todo-web: src/components/ (the screens, a library), src/main.tsx (the app)
fixtures/seeds.json  the fixture tasks; each app ships a byte-identical copy (SPEC-APP.md §3)
omnicui/             the omnicui project
  contracts/         list, detail, settings, app (.contract.yaml)
  platforms.yaml     named inputs: `back` per platform
  targets/android/   Robolectric adapter (Kotlin, src/test/), setup.sh, target.yaml
  targets/web/       Playwright adapter, from omnicui/targets/web (omnicui add web --template …)
runs/latest.txt      the last full check's console output, and its results.json
```

## Requirements

- Java 21, Node 22.18 or later, Go (to build omnicui). Here: `export PATH=$HOME/.local/node22/bin:$HOME/.local/go/bin:$PATH`.
- The Android SDK, command-line tools only (no emulator, no system image), about 0.5 GB:

  ```sh
  mkdir -p ~/.local/android-sdk/cmdline-tools && cd ~/.local/android-sdk
  curl -LO https://dl.google.com/android/repository/commandlinetools-linux-16111833_latest.zip
  unzip -q commandlinetools-linux-*_latest.zip && mv cmdline-tools cmdline-tools/latest   # (unzip into a temp folder first)
  export ANDROID_HOME=~/.local/android-sdk
  yes | cmdline-tools/latest/bin/sdkmanager --licenses
  cmdline-tools/latest/bin/sdkmanager "platforms;android-36" "build-tools;36.0.0"   # 23 s here
  ```
- omnicui: `cd tools/omnicui && go build -o ../../bin/omnicui ./cmd/omnicui` (1.4 s).

Versions: AGP 8.13.2, Gradle 8.14.4, Kotlin 2.2.21, Compose BOM 2025.10.01, Navigation 2.9.5,
Robolectric 4.16.1 (Android 16, API 36); React 19.3.1, esbuild 0.28.2, Playwright 1.64.1.

## Build and run the apps

| | command (in the app's folder) | time here |
|---|---|---|
| Android: build the APK | `./gradlew :app:assembleDebug` → `app/build/outputs/apk/debug/app-debug.apk` | 1 min 48 s cold (downloads included), seconds warm |
| Android: unit tests (the store, the fixture's SHA-256) | `./gradlew :ui:testDebugUnitTest` | 4 tests |
| web: install | `npm ci` | 5 s |
| web: unit tests (the store, the fixture's SHA-256) | `npm test` | 4 tests, under 1 s |
| web: typecheck | `npx tsc --noEmit` | |
| web: build / serve | `npm run build` (0.1 s) / `npm start` (serves on 127.0.0.1, `PORT=…` to fix it) | |

Both apps start with the `three` fixture tasks. Android: install the APK on a device
(`adb install`); nothing here needs an emulator.

## Set up the targets

```sh
cd examples/todo/omnicui/targets/android && ./setup.sh
# android: built in 3 s; classpath in build/adapter.args
# android: the adapter answered advertise in 8559 ms (start-up, JVM to first reply)

cd ../web && npm ci && npx playwright install chromium
```

**android** does not start Gradle when omnicui runs it. `setup.sh` runs Gradle once
(`./gradlew adapterClasspath`: it compiles the adapter and the app's `ui` library, an included
build, and writes the unit tests' exact runtime classpath into `build/adapter.args`), then starts
the adapter once to fetch Robolectric's Android 16 jar (first time only) and time the start-up.
`target.yaml` then starts one plain JVM per check: `java @build/adapter.args omnicui.android.Main`,
which runs one JUnit "test" (`Serve`) inside one Robolectric sandbox for the whole conversation.
Re-run `setup.sh` after changing the app or the adapter (3 s warm; the first build, with
downloads, took 1 min 10 s).

How the android adapter works (`src/test/kotlin/omnicui/android/`):
- `Components.kt` is the registry: contract id → the app's screen (`TaskListRoute`, …) with a store
  made from the props, and the app's `TodoEvent`s → the contracts' events.
- Each `mount` launches a fresh activity (`runAndroidComposeUiTest`), with the screen set from the
  viewport and the colour scheme by Robolectric qualifiers: `w360dp-h640dp-port-night-mdpi`, so
  1 dp = 1 px and `isSystemInDarkTheme()` follows the scheme.
- Parts are found by `testTag` in the merged semantics tree; `bounds` is `boundsInRoot` plus the
  window's place on the screen; `press` is a touch click, `type` is `performTextInput`, `key` is
  `performKeyInput`, `focus` asks for keyboard input mode and then focus (GAPS 12), `do back` is
  the activity's `OnBackPressedDispatcher` (or the back key to a dialog on top).
- Screenshots are Robolectric native graphics (`@GraphicsMode(NATIVE)`): every window drawn on a
  Skia canvas and composed like the window manager would (GAPS 11); Compose's `captureToImage`
  timed out under Robolectric.
- `trace` and `record` run on Compose's test clock, one 16 ms frame at a time, so motion is
  measured exactly, whatever the machine's speed.

**web** is the reference web target with four changes: the registry (`harness.tsx`), the
`todo.app` part rule (`selector()` in `adapter.ts`), `do back` (`page.goBack()`) and a key with no
part, and `record` stepping the page's animations instead of Chromium's screencast (GAPS 13);
`type` focuses the field before typing (GAPS 16). Its stub generator writes the contracts' types
into `apps/web/src/omnicui/` (`omnicui stubs`); the Android app declares its tags in
`todo.ui.Parts` instead.

## Run the check

From the repository root (`omnicui/`):

```sh
export PATH=$HOME/.local/node22/bin:$PATH ANDROID_HOME=$HOME/.local/android-sdk
bin/omnicui --project examples/todo/omnicui check                       # everything
bin/omnicui --project examples/todo/omnicui check android todo.list     # one target, one component
bin/omnicui --project examples/todo/omnicui check --test "Enter in*"    # some tests
bin/omnicui --project examples/todo/omnicui check --failed              # what failed last time
```

Results go to `examples/todo/omnicui/.omnicui/run/` (`report.html`, `results.json`, screenshots,
clips).

Timings here (20 cores; the targets run side by side):

| | start-up | all four contracts |
|---|---|---|
| android | 7.1–8.6 s (a JVM, the Robolectric sandbox, the first activity) | 38–50 s (last run: 40 s) |
| web | 1.2–1.6 s (esbuild bundle, Chromium) | 1 min 46 s – 1 min 53 s (last: 1 min 50 s) |
| full check, both | | 2 min 2 s – 2 min 9 s (last: 2 min 6 s) |

The web target is the slower one: each generated keyboard test mounts twice in a fresh browser
context and reads every part (about 2 s each).

## What the contracts check

86 tests per target (4 contracts, with the tests `keyboard: true` adds on the three screens; one is for desktops and the web only), at 360×640 on the
phone and 1024×720 on the web (per-platform viewports), light and dark. Android and the web are
different designs (`design: material`, `design: desktop`), so a pair of them is compared by what
each part keeps the same (`same:`): colours and motion for most parts, motion only for checkboxes
and switches, nothing for text fields. Between two phones (Android and a future iOS) the layout
is held too. Navigation back is `{ input: back }` (`platforms.yaml`): Android's system back, the
browser's back. See `runs/latest.txt` for the last full run.
