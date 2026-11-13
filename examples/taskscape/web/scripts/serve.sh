#!/bin/sh
# Runs Taskscape for the web on your own data (~/.taskscape, shared with the desktop apps):
# http://127.0.0.1:7430 unless PORT says otherwise. Build it first: scripts/build.sh.
web=$(cd "$(dirname "$0")/.." && pwd)
exec "$web/build/cargo/release/taskscape-web" --dist "$web/dist" --port "${PORT:-7430}" "$@"
