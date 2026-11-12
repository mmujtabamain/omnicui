# Todo for macOS

SwiftUI screens for macOS 26+, with AppKit text fields, checkboxes, switches,
segmented controls and buttons. The same screen library is used by the app and
by the in-process omnicui adapter. State is in memory only.

Build from the repository root (Xcode's macOS SDK and Swift compiler required):

```sh
examples/todo/desktop/apps/macos/build.sh
open examples/todo/desktop/apps/macos/build/Todo.app
```

The build uses one compiler job. Run it when other builds have finished on
memory-constrained machines. The app bundle is ad-hoc signed locally.

The fixture copy in `Resources/seeds.json` is byte-identical to
`examples/todo/fixtures/seeds.json`.

The native wrappers provide per-segment accessibility elements and responder
handling. Identifiers follow SPEC-APP.md; the omnicui adapter reads their actual
native frames and states. The screen layout and palette live in `Screens.swift`
and `Model.swift`.
