#!/bin/sh
# Builds Taskscape for the web: the npm packages (from package-lock.json), the icon subset (in a
# venv of its own, scripts/.venv is not shared with the Tauri app's), the frontend into dist/, and
# the server into build/cargo/release/taskscape-web.
#   scripts/build.sh            the app
#   OMNICUI=1 scripts/build.sh  the app with omnicui.html and the server's test hooks (the
#                               omnicui target's setup.sh does this)
set -eu
web=$(cd "$(dirname "$0")/.." && pwd)
cd "$web"
npm ci --no-audit --no-fund --loglevel=error
if [ ! -x .venv/bin/python ]; then
  python3 -m venv .venv
  .venv/bin/pip install -q -r ../tauri/scripts/requirements.txt
fi
.venv/bin/python scripts/subset-icons.py
npx vite build --logLevel warn
features=""
[ "${OMNICUI:-}" = 1 ] && features="--features test-hooks"
(cd server && CARGO_TARGET_DIR="$web/build/cargo" cargo build --release --locked -q $features)
echo "web: built dist/ and build/cargo/release/taskscape-web${features:+ (with the test hooks)}"
