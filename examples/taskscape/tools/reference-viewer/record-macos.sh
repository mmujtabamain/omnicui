#!/bin/bash
# Records every reference state on macOS: the reference app's own frontend, run in Chromium on fixture data
# (prepare.mjs, serve.mjs, capture.mjs), into spec/references/<state>/ (shot.png, elements.json, tree.txt,
# meta.json). On macOS, system-ui is SF, the font the reference app draws with; recorded elsewhere it falls
# back to a wider face and every text run differs from the targets.
#
# usage: tools/reference-viewer/record-macos.sh <path to the reference app's repo> [state-id …]
#   With no state ids, every state in states.mjs. Needs Node 20+ and python3 (brew install node python).
#   Ends with spec/references packed as ~/Desktop/taskscape-references-macos.zip.
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../.." && pwd)"
ref="${1:-${TASKSCAPE_REFERENCE:-}}"
[ $# -gt 0 ] && shift

if [ "$(uname)" != Darwin ] && [ -z "${ALLOW_OTHER_OS:-}" ]; then
  echo "Run this on macOS: elsewhere system-ui is a different font and the recordings don't match the app." >&2
  exit 2
fi
if [ -z "$ref" ] || [ ! -d "$ref/common-ui" ] || [ ! -d "$ref/main" ]; then
  echo "usage: $0 <path to the reference app's repo (the one with main/, tray/, common-ui/)> [state-id …]" >&2
  exit 2
fi
command -v node >/dev/null || { echo "Node 20 or later is needed: brew install node" >&2; exit 2; }
[ "$(node -p 'process.versions.node.split(".")[0]')" -ge 20 ] || { echo "Node 20 or later is needed (found $(node -v)): brew upgrade node" >&2; exit 2; }
command -v python3 >/dev/null || { echo "python3 is needed: brew install python" >&2; exit 2; }
for port in 1420 1421; do
  if command -v lsof >/dev/null && lsof -nP -iTCP:$port -sTCP:LISTEN >/dev/null 2>&1; then
    echo "Port $port is in use (a reference dev server already running?). Stop it and run this again." >&2
    exit 2
  fi
done

export TASKSCAPE_REFERENCE="$(cd "$ref" && pwd)"
cd "$here"

echo "== 1/4 the recorder's own packages"
npm ci --no-audit --no-fund --loglevel=error
npx playwright install chromium

echo "== 2/4 a copy of the reference's frontend (outside the repo), with its packages"
node prepare.mjs

echo "== 3/4 the reference's dev servers"
log="${TMPDIR:-/tmp}/taskscape-reference-serve.$$.log"
node serve.mjs >"$log" 2>&1 &
server=$!
trap 'kill $server 2>/dev/null; wait $server 2>/dev/null || true' EXIT
for url in http://localhost:1420/viewer.html http://localhost:1421/viewer.html; do
  for i in $(seq 1 120); do
    curl -sf "$url" >/dev/null && break
    if ! kill -0 $server 2>/dev/null; then echo "The dev servers stopped:" >&2; tail -40 "$log" >&2; exit 1; fi
    [ "$i" = 120 ] && { echo "No answer from $url after 2 minutes:" >&2; tail -40 "$log" >&2; exit 1; }
    sleep 1
  done
done

echo "== 4/4 recording"
node capture.mjs "$@" | tee "$root/spec/references-record.log"

shots=$(find "$root/spec/references" -name shot.png | wc -l | tr -d ' ')
zip="$HOME/Desktop/taskscape-references-macos.zip"
mkdir -p "$(dirname "$zip")"
rm -f "$zip"
(cd "$root/spec" && zip -qr "$zip" references references-record.log)
echo
echo "Recorded $shots states into spec/references, packed as $zip."
echo "Send that zip in the chat (or commit spec/references on your branch)."
