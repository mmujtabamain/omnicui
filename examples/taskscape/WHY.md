# Why Taskscape is being ported

Taskscape was built as Tauri apps: a React interface in the system webview, with Rust behind it. The interface works. But a webview has costs that a task manager, opened and dismissed all day, can't hide: **load time, and dialogs that open in a window of their own.** Opening the Settings dialog meant creating a new window that loaded the web interface from scratch. The wait between the click and a usable dialog made the whole app feel slow.

## Where it shows in the code

All paths are in `tauri/`.

- **Dialogs in another window.** Settings is a separate window (`open_settings` in `main/src-tauri/src/lib.rs`). Each time it opens, it builds a new webview that loads `main/panels.html` and its own bundle (`main/src/panels.tsx`). Before the first paint there is nothing to show.
- **Workarounds already in place.** Sheets and the filter panel used to be panel windows too. They were moved into the main window (`main/src/components/SheetHost.tsx`, `main/src/components/preview/FilterPanel.tsx`) to avoid that wait. `main/src/panels.tsx` notes that Settings is the only panel window left.
- **Start-up.** The window draws nothing until the page paints. So the app shows a native "boot box" under the webview (`show_boot_box` in `main/src-tauri/src/panels.rs`) and a boot overlay in the page. It then hides the webview and reveals the window in steps, so the user doesn't see a blank or flashing frame.
- **The capture bar.** The menu-bar capture bar has to appear instantly over any app. It keeps its webview window alive and parks it off-screen between uses (`.llm/tray-app.md`), rather than creating it when it is summoned.

Each of these is the app working around the webview, not the product's design.

## What the ports are for

- **GPUI** (`gpui/`): native Rust from the first frame. It has no web interface to load, so:
  - the main window is drawn at once;
  - Settings and every dialog are either drawn in the window or a native window that opens ready;
  - the capture bar is a native pop-up window.

  It links the same Rust crate for storage, so the data and its rules don't change.
- **Web** (`web/`): the same product where a browser is the right place for it. The page loads once, and every dialog after that is drawn in the page; none needs a window of its own.

The design is mostly final and carries over. Each port draws it idiomatically: the layout, colours and motion stay the same, and the chrome and native controls are its own.

## What omnicui checks, and what it measures

- **Behaviour.** The contracts hold all three to the same behaviour: events, texts, states and keyboard.
- **Design.** They also hold all three to the same layout, colours and motion.
- **Time.** Each target's start-up and each test's time are measured: `results.json` has `startMs` per target and `ms` per test, and the console lists the slowest tests. A test that opens Settings or a sheet therefore shows how long that took on each version, measured the same way. omnicui doesn't yet measure the time from a press to the first frame of the dialog; that would be the direct measure of the problem this port solves.
