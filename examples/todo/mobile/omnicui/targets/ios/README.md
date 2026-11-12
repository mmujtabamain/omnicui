# iOS target

Requires macOS, Xcode with the iOS Simulator SDK, an installed iOS 26 runtime,
Python 3, and the repository's built `bin/omnicui`. No XCTest or third-party
packages are used for reads, actions, or screenshots.

```sh
./setup.sh
# From the repository root:
bin/omnicui --project examples/todo/mobile/omnicui check ios
```

Run setup again after editing the app or host. Checks launch prebuilt code and
reuse the same `omnicui Todo iOS` Simulator; they never build or create devices.
`build/device` stores its UDID. The target is exclusive so another target does
not compete for the machine during a check. The Simulator stays booted between
checks; the host app is terminated when stdin closes or the adapter is stopped.

`adapter.py` relays JSON-RPC stdin/stdout to one UIKit app over a loopback socket.
`Host.swift` imports TodoUI, creates a fresh store and UIHostingController on each
mount, maps its callback events, walks UIKit's accessibility tree, and reads live
labels, traits, input values and view frames. `todo.app.*` resolves to the shown
screen's same-named part. Native UIControl actions and accessibility activation
operate the actual controls. Text is inserted one character at a time through
UITextInput, with the caret placed at the end. Back activates the screen's named
navigation control; it dismisses a presented add sheet first.

SwiftUI creates accessibility nodes lazily. The Simulator-only host enables
Apple's automation accessibility runtime using `_AXSSetAutomationEnabled`, then
primes the hierarchy during mount. This private symbol is confined to the test
host; the shipping app and TodoUI contain no private API calls. The host fails
if the runtime cannot supply the required parts.

The hosting surface and window are 360×640 points for these contracts. Screenshots
use UIGraphicsImageRenderer at scale 1 and draw the native view hierarchy directly;
there is no Simulator screenshot, resizing, or interpolation. The probe uses the
same renderer. The rendering excludes SpringBoard's status bar and home indicator;
those absent regions are not claimed in the reply. Any keyboard region actually
rendered is reported in screenshot coordinates.

Hardware key injection, arbitrary control focus, pointer/drag injection, and
per-frame trace/record are explicitly unadvertised with reasons. `role` maps the live accessibility traits and backing native controls to the
protocol roles. `focused` reads the native text input's first-responder state, and `selected` reads the segment's
accessibility trait. See `../../../GAPS-ios.txt` for evidence and remaining limits.

Supplementary checks (after setup and Simulator boot):

```sh
../../../apps/ios/test.sh
python3 test_protocol.py
```

The checked full-run result and representative 1× screenshots are in `evidence/`.
The implementation report is [GAPS-ios.txt](../../../GAPS-ios.txt).
