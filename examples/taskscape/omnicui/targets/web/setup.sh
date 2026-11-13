#!/bin/sh
# Builds everything the web target needs, once: the app (examples/taskscape/web) with its omnicui
# page and its server with the test hooks (web/scripts/build.sh), this folder's npm packages and
# Chromium for Playwright. omnicui never builds anything per check. Re-run it after changing the
# app. Then it starts the adapter once and prints how long it took to answer advertise (its
# start-up).
set -eu
here=$(cd "$(dirname "$0")" && pwd)
t0=$(date +%s)
OMNICUI=1 "$here/../../../web/scripts/build.sh"
echo "web: npm ci in the target"
(cd "$here" && npm ci --no-audit --no-fund --loglevel=error && npx playwright install chromium)
echo "web: built in $(( $(date +%s) - t0 )) s"

# Start-up: from launching the adapter to its reply to advertise.
start=$(node -e 'console.log(Date.now())')
reply=$(printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"advertise","params":{"protocol":"1"}}' | (cd "$here" && node adapter.ts 2>/dev/null | head -1))
end=$(node -e 'console.log(Date.now())')
case "$reply" in
  *'"can"'*) echo "web: the adapter answered advertise in $((end - start)) ms (start-up: the server, Chromium)";;
  *) echo "web: the adapter did not answer advertise: $reply" >&2; exit 1;;
esac
