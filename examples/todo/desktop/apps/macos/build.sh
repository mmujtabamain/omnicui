#!/bin/sh
set -eu
cd "$(dirname "$0")"
mkdir -p build/Todo.app/Contents/MacOS build/Todo.app/Contents/Resources
todo_build_arch="$(uname -m)"
swiftc -j 1 -parse-as-library -swift-version 5 -target "${todo_build_arch}-apple-macos26.0" \
  Sources/TodoMac/*.swift Sources/Main.swift -o build/Todo.app/Contents/MacOS/Todo
cp Resources/seeds.json build/Todo.app/Contents/Resources/
cat > build/Todo.app/Contents/Info.plist <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleExecutable</key><string>Todo</string>
<key>CFBundleIdentifier</key><string>dev.omnicui.todo.macos</string>
<key>CFBundleName</key><string>Todo</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>LSMinimumSystemVersion</key><string>26.0</string>
<key>NSHighResolutionCapable</key><true/>
</dict></plist>
PLIST
codesign --force --sign - build/Todo.app
