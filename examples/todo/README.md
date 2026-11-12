# Todo: two apps, two omnicui projects

The same todo product, built as **two separate apps**, each with its own omnicui project. They
share the idea and the fixture tasks (`fixtures/seeds.json`), and nothing is compared between
them.

| | `mobile/` | `desktop/` |
|---|---|---|
| apps | `apps/android` (Kotlin, Jetpack Compose, Material 3); iOS (SwiftUI) later | `apps/web` (React, TypeScript, esbuild); macOS (SwiftUI or AppKit) later |
| look | entirely native on each phone | one product design (`design/DESIGN-DESKTOP.md`: regions, proportions, palette), each platform's own controls and look |
| contracts compare | behaviour only: `same: none` | behaviour, and `same: [layout, colours, motion]`; `[layout]` for native controls; never pixels |
| target | `omnicui/targets/android`: Robolectric, one JVM (`platform: [mobile]`, `design: material`) | `omnicui/targets/web`: Playwright (`platform: [web]`, `design: web`) |
| spec for the next platform | `mobile/SPEC-APP.md` (for the iOS builder) | `desktop/SPEC-APP.md` (for the macOS builder) |
| viewport | 360×640, light and dark | 1024×720, light and dark |

```
fixtures/seeds.json     the fixture tasks; each app ships a byte-identical copy (SHA-256 in each SPEC-APP.md)
mobile/
  SPEC-APP.md           behaviour, parts and events, for a native iOS version
  apps/android/         Gradle (wrapper): ui/ (the screens, a library: todo:ui), app/ (the activity)
  omnicui/              contracts/ (list, detail, settings, app), platforms.yaml (input back), targets/android/
desktop/
  SPEC-APP.md           behaviour, parts and events, for a macOS version
  design/DESIGN-DESKTOP.md   the regions, their boxes at 1024×720, the colour tokens, the native controls
  apps/web/             npm package todo-web: src/components/ (the panes, a library), src/main.tsx (the app)
  omnicui/              contracts/, platforms.yaml, targets/web/ (from omnicui/targets/web, via omnicui add --template)
runs/                   the last full check of each project: mobile.txt, desktop.txt, and mobile-results.json, desktop-results.json
GAPS-todo.txt           what omnicui couldn't express or got wrong, and my own mistakes
PROGRESS.md             the log
```

## Requirements

- Java 21, Node 22.18 or later, Go (for omnicui). Here: `export PATH=$HOME/.local/node22/bin:$HOME/.local/go/bin:$PATH`.
- For mobile, the Android SDK, command-line tools only (no emulator, no system image), about 0.6 GB:

  ```sh
  mkdir -p ~/.local/android-sdk/cmdline-tools /tmp/sdk && cd /tmp/sdk
  curl -LO https://dl.google.com/android/repository/commandlinetools-linux-16111833_latest.zip
  unzip -q commandlinetools-linux-*_latest.zip && mv cmdline-tools ~/.local/android-sdk/cmdline-tools/latest
  export ANDROID_HOME=~/.local/android-sdk
  yes | $ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager --licenses
  $ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager "platforms;android-36" "build-tools;36.0.0"   # 23 s here
  ```
- omnicui: `cd tools/omnicui && go build -o ../../bin/omnicui ./cmd/omnicui` (1.4 s).

Versions: AGP 8.13.2, Gradle 8.14.4, Kotlin 2.2.21, Compose BOM 2025.10.01, Navigation 2.9.5,
Robolectric 4.16.1 (Android 16, API 36); React 19.3.1, esbuild 0.28.2, Playwright 1.64.1.

## The mobile project

Build and test the Android app (in `mobile/apps/android`):

| | command | time here |
|---|---|---|
| the APK | `./gradlew :app:assembleDebug` → `app/build/outputs/apk/debug/app-debug.apk` | 1 min 48 s cold (downloads included), seconds warm |
| unit tests (the store, the fixture's SHA-256) | `./gradlew :ui:testDebugUnitTest` | 4 tests |

It is plain Material 3: dynamic colour (the baseline scheme before Android 12), the platform
font, the stock top app bar, segmented buttons, list items, FAB, alert dialog, text fields and
switch, Navigation Compose with type-safe routes. The app starts with the `three` fixture tasks.

Set up the target once, then check:

```sh
cd examples/todo/mobile/omnicui/targets/android && ./setup.sh
# android: built in 54 s; classpath in build/adapter.args          (3 s warm)
# android: the adapter answered advertise in 10271 ms (start-up, JVM to first reply)
cd ../../../../../..   # the repository root
ANDROID_HOME=$HOME/.local/android-sdk bin/omnicui --project examples/todo/mobile/omnicui check
```

**The android target does not start Gradle when omnicui runs it.** `setup.sh` runs Gradle once
(`./gradlew adapterClasspath`: it compiles the adapter and the app's `ui` library, an included
build, and writes the unit tests' exact runtime classpath into `build/adapter.args`), then
starts the adapter once to fetch Robolectric's Android 16 jar (first time only) and time its
start-up. `target.yaml` starts one plain JVM per check: `java @build/adapter.args
omnicui.android.Main`, which runs one JUnit "test" (`Serve`) inside one Robolectric sandbox for
the whole conversation. Re-run `setup.sh` after changing the app or the adapter.

How it works (`src/test/kotlin/omnicui/android/`):
- `Components.kt` is the registry: contract id → the app's screen (`TaskListRoute`, …) with a store
  made from the props; the app's `TodoEvent`s → the contracts' events; and the `todo.app` rule
  (its parts are the shown screen's parts of the same name).
- Each `mount` launches a fresh activity (`runAndroidComposeUiTest`), the screen set from the
  viewport and colour scheme by Robolectric qualifiers (`w360dp-h640dp-port-night-mdpi`: 1 dp =
  1 px, and `isSystemInDarkTheme()` follows the scheme).
- Parts are found by `testTag` in the merged semantics tree; `bounds` is `boundsInRoot` plus the
  window's place on the screen; `press` is a touch click, `type` is `performTextInput`, `key` is
  `performKeyInput`, `focus` asks for keyboard input mode and then focus (GAPS 12), `do back` is
  the activity's `OnBackPressedDispatcher`, or the back key to a dialog on top.
- Screenshots use Robolectric native graphics (`@GraphicsMode(NATIVE)`): every window drawn on a
  Skia canvas, placed and dimmed as the window manager would (GAPS 11; `captureToImage` timed out).
- `trace` and `record` run on Compose's test clock, one 16 ms frame at a time.

## The desktop project

Build and test the web app (in `desktop/apps/web`):

| | command | time here |
|---|---|---|
| install | `npm ci` | 5 s |
| unit tests (the store, the fixture's SHA-256) | `npm test` | 4 tests, under 1 s |
| typecheck | `npx tsc --noEmit` | |
| build / serve | `npm run build` (0.1 s) / `npm start` (127.0.0.1; `PORT=…` to fix the port) | |

Set up the target once, then check:

```sh
cd examples/todo/desktop/omnicui/targets/web && npm ci && npx playwright install chromium
cd ../../../../../..   # the repository root
bin/omnicui --project examples/todo/desktop/omnicui check
```

The web target is the reference web target with these changes: the registry (`harness.tsx`), the
`todo.app` part rule (`selector()` in `adapter.ts`), `do back` (`page.goBack()`), a key with no
part, `type` focusing the field first (GAPS 16), and `record` stepping the page's animations
16 ms at a time instead of Chromium's screencast (GAPS 13). Its stub generator writes the
contracts' types into `apps/web/src/omnicui/` (`omnicui stubs`).

## Timings (this machine, 20 cores)

| project | target | start-up | its contracts |
|---|---|---|---|
| mobile | android | 7.4–10.3 s (a JVM, the Robolectric sandbox, the first activity); last run 8.1 s | 46 s, 85 tests (`runs/mobile.txt`) |
| desktop | web | 1.2–1.6 s (esbuild bundle, Chromium); last run 1.6 s | 2 min 1 s, 86 tests (`runs/desktop.txt`) |

The web target is the slower one per test: each generated keyboard test mounts twice in a fresh
browser context and reads every part (about 2 s each).
