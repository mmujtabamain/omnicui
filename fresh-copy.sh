#!/bin/sh
# The fresh-copy check: clone the repository into an empty folder, install from the lockfiles,
# build, and run every test, through run.sh.
# Usage: ./fresh-copy.sh [repository: a path or URL; default this one] [log file]
set -eu
here=$(cd "$(dirname "$0")" && pwd)
repo=${1:-$here}
log=${2:-fresh-copy.log}
case "$log" in /*) ;; *) log="$(pwd)/$log" ;; esac
dir=$(mktemp -d)
{
  echo "fresh clone of $repo into $dir, $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "go $(go version | cut -d' ' -f3), node $(node --version), $(java -version 2>&1 | head -1), $(cargo --version)"
  git clone -q "$repo" "$dir/omnicui"
  cd "$dir/omnicui"
  echo "--- ./run.sh all"
  ./run.sh all
  echo "--- fresh copy: everything passed"
} 2>&1 | tee "$log"
# `tee` hides the block's exit status, so the last line decides.
tail -1 "$log" | grep -q "everything passed" || { echo "fresh copy: FAILED (see $log)" >&2; exit 1; }
