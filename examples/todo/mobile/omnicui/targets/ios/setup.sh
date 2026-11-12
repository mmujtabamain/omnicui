#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")"
APP="$(cd ../../../apps/ios && pwd)"
"$APP/build.sh"
SDK=$(xcrun --sdk iphonesimulator --show-sdk-path)
export SDKROOT="$SDK"
ARCH=$(uname -m)
mkdir -p build/Host.app/Frameworks
cp -R "$APP/build/TodoUI.framework" build/Host.app/Frameworks/
xcrun swiftc -sdk "$SDK" -target "$ARCH-apple-ios26.0-simulator" -F "$APP/build" -framework TodoUI -Xlinker -rpath -Xlinker @executable_path/Frameworks Host.swift -o build/Host.app/Host
python3 - <<'PY'
import plistlib
with open('build/Host.app/Info.plist', 'wb') as f:
    plistlib.dump(dict(CFBundleIdentifier='org.omnicui.todo.host', CFBundleExecutable='Host', CFBundleName='Todo Host', CFBundlePackageType='APPL', CFBundleVersion='1', CFBundleShortVersionString='1.0', MinimumOSVersion='26.0', UIDeviceFamily=[1], UILaunchScreen={}, UISupportedInterfaceOrientations=['UIInterfaceOrientationPortrait']), f)
PY
codesign --force --sign - build/Host.app/Frameworks/TodoUI.framework >&2
codesign --force --sign - build/Host.app >&2
python3 - <<'PY'
import json,subprocess,pathlib
runtimes=json.loads(subprocess.check_output(['xcrun','simctl','list','runtimes','-j']))['runtimes']
runtime=next(r['identifier'] for r in runtimes if r['isAvailable'] and r['version'].startswith('26.'))
devices=json.loads(subprocess.check_output(['xcrun','simctl','list','devices','-j']))['devices'].get(runtime,[])
device=next((d['udid'] for d in devices if d['name']=='omnicui Todo iOS'),None)
if not device:
    device=subprocess.check_output(['xcrun','simctl','create','omnicui Todo iOS','com.apple.CoreSimulator.SimDeviceType.iPhone-17-Pro',runtime],text=True).strip()
pathlib.Path('build/device').write_text(device+'\n')
print('ios: one reusable Simulator: '+device)
PY
echo 'ios: built app and in-process host; checks never invoke a compiler' >&2
