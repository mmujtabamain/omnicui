#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")"
SDK=$(xcrun --sdk iphonesimulator --show-sdk-path)
export SDKROOT="$SDK"
ARCH=$(uname -m)
DEST="$PWD/build"
mkdir -p "$DEST/TodoUI.framework/Modules/TodoUI.swiftmodule" "$DEST/Todo.app/Frameworks"
xcrun swiftc -sdk "$SDK" -target "$ARCH-apple-ios26.0-simulator" -parse-as-library -emit-library -emit-module -module-name TodoUI -module-link-name TodoUI -emit-module-path "$DEST/TodoUI.framework/Modules/TodoUI.swiftmodule/$ARCH-apple-ios-simulator.swiftmodule" -Xlinker -install_name -Xlinker @rpath/TodoUI.framework/TodoUI Sources/TodoUI/*.swift -o "$DEST/TodoUI.framework/TodoUI"
cp Sources/TodoUI/seeds.json "$DEST/TodoUI.framework/"
cat > "$DEST/TodoUI.framework/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?><!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd"><plist version="1.0"><dict><key>CFBundleIdentifier</key><string>org.omnicui.todo.ui</string><key>CFBundleExecutable</key><string>TodoUI</string><key>CFBundlePackageType</key><string>FMWK</string></dict></plist>
PLIST
xcrun swiftc -sdk "$SDK" -target "$ARCH-apple-ios26.0-simulator" -parse-as-library -F "$DEST" -framework TodoUI -Xlinker -rpath -Xlinker @executable_path/Frameworks Sources/TodoApp/*.swift -o "$DEST/Todo.app/Todo"
cp -R "$DEST/TodoUI.framework" "$DEST/Todo.app/Frameworks/"
python3 - "$DEST/Todo.app/Info.plist" <<'PY'
import plistlib,sys
with open(sys.argv[1], 'wb') as f:
    plistlib.dump(dict(CFBundleIdentifier='org.omnicui.todo', CFBundleExecutable='Todo', CFBundleName='Tasks', CFBundlePackageType='APPL', CFBundleVersion='1', CFBundleShortVersionString='1.0', MinimumOSVersion='26.0', UIDeviceFamily=[1,2], UIApplicationSceneManifest={'UIApplicationSupportsMultipleScenes':False,'UISceneConfigurations':{}}, UILaunchScreen={}, UISupportedInterfaceOrientations=['UIInterfaceOrientationPortrait']), f)
PY
codesign --force --sign - "$DEST/Todo.app/Frameworks/TodoUI.framework" >&2
codesign --force --sign - "$DEST/Todo.app" >&2
echo "Built $DEST/Todo.app" >&2
