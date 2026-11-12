#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")"
SDK=$(xcrun --sdk iphonesimulator --show-sdk-path)
export SDKROOT="$SDK"
ARCH=$(uname -m)
DEVICE=${OMNICUI_IOS_DEVICE:-$(cat ../../omnicui/targets/ios/build/device)}
xcrun swiftc -sdk "$SDK" -target "$ARCH-apple-ios26.0-simulator" -parse-as-library -F "$PWD/build" -framework TodoUI -Xlinker -rpath -Xlinker @executable_path Tests/StoreChecks.swift -o build/StoreChecks
xcrun simctl spawn "$DEVICE" "$PWD/build/StoreChecks" "$PWD/Sources/TodoUI/seeds.json"
