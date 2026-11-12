# macOS target

Build once, then run the long-lived adapter without invoking a compiler:

```sh
./setup.sh
python3 smoke.py
```

From the repository root:

```sh
bin/omnicui --project examples/todo/desktop/omnicui check macos web
```

The adapter runs the app's SwiftUI screens in an NSHostingView in a real macOS
window. Requests run on the main thread while NSApplication's event loop is
active. Standard output carries only JSON-RPC replies. Key input is delivered as
NSEvents through NSApplication; Cmd+[ is handled by the app's window.

Screenshots cache the window's view hierarchy into an explicit scale-1 bitmap.
The titlebar region and all part frames use that bitmap's top-left coordinates.
The render probe uses the same capture path. Trace and recording timestamps use
real monotonic time; they do not synthesize animation samples.

`build/` contains local executables and smoke screenshots and is ignored.
