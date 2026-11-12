# Tasks for iOS 26

The app is SwiftUI: a NavigationStack, native List with swipe-to-delete,
segmented Pickers, switches, system colours and Liquid Glass toolbars. State is
in memory. TodoUI is a framework with the store and four independently mountable
components; Todo is the standalone app. The framework knows nothing about the
omnicui target. Its only testing metadata is the accessibility identifiers.

Open `Todo.xcodeproj`, select Todo and an iOS 26+ Simulator, and Run. A physical
device also needs your development signing team. No third-party dependencies.
`make-project.py` regenerates the checked-in project deterministically.

For a command-line Simulator build (uses Xcode's active SDK):

```sh
./build.sh
# build/Todo.app, build/TodoUI.framework
```

To build the verification host, from `../../omnicui/targets/ios` run `./setup.sh`.
It builds this app/framework too and selects one reusable Simulator. After it has
booted, `./test.sh` here runs the store checks in that Simulator, including the
canonical fixture's SHA-256. See the target's README for protocol verification.
