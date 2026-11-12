#!/bin/sh
# Builds everything the tauri target needs, once: the app's npm packages (from its own
# lockfile), the Material Symbols subsets its build makes (scripts/subset-icons.py, in a venv
# of its own), the harness pages (build.mjs, with the app's own Vite and Tailwind), Chromium, and
# the store server (server/, Rust). omnicui never builds anything per check. Re-run it after
# changing the app, the registry, harness.ts or the server. Then it starts the adapter once and
# prints how long it took to answer advertise (its start-up).
set -eu
here=$(cd "$(dirname "$0")" && pwd)
app="$here/../../../tauri"
t0=$(date +%s)

echo "tauri: npm ci in the app ($app)"
(cd "$app" && npm ci --no-audit --no-fund --loglevel=error)
echo "tauri: npm ci in the target"
(cd "$here" && npm ci --no-audit --no-fund --loglevel=error && npx playwright install chromium)

if [ ! -x "$app/scripts/.venv/bin/python" ]; then
  python3 -m venv "$app/scripts/.venv"
  "$app/scripts/.venv/bin/pip" install -q -r "$app/scripts/requirements.txt"
fi
"$app/scripts/.venv/bin/python" "$app/scripts/subset-icons.py" main
"$app/scripts/.venv/bin/python" "$app/scripts/subset-icons.py" tray

(cd "$here" && node build.mjs)
echo "tauri: building the store server (cargo, release)"
(cd "$here/server" && CARGO_TARGET_DIR="$here/build/cargo" cargo build --release --locked -q)
echo "tauri: built in $(( $(date +%s) - t0 )) s"

# Start-up: from launching the adapter to its reply to advertise.
start=$(node -e 'console.log(Date.now())')
reply=$(printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"advertise","params":{"protocol":"1"}}' | (cd "$here" && node adapter.ts 2>/dev/null | head -1))
end=$(node -e 'console.log(Date.now())')
case "$reply" in
  *'"can"'*) echo "tauri: the adapter answered advertise in $((end - start)) ms (start-up: the store server, Chromium)";;
  *) echo "tauri: the adapter did not answer advertise: $reply" >&2; exit 1;;
esac
