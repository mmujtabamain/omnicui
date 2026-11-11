#!/bin/sh
# The redesign experiment: does omnicui test behaviour when the UI is different?
#
#   experiments/redesign/run.sh            set up webalt, then every run below, into runs/
#   experiments/redesign/run.sh base       one of them: base, a variant (counter.copy, …), fail-on-pixels,
#                                          or a contract experiment (a-optional, b-own-test, …)
#
# Needs ./run.sh setup web first. The project here has two targets: web (a link to the reference
# repository's web target, the original design) and webalt (the second design), and the reference
# contracts (a link). The variants are switched on with OMNICUI_DEFECTS, on top of the redesign.
set -u
HERE=$(cd "$(dirname "$0")" && pwd)
ROOT=$(cd "$HERE/../.." && pwd)
OMNICUI="$ROOT/bin/omnicui"
RUNS="$HERE/runs"
VARIANTS="counter.copy counter.srlabel counter.toolbar counter.double counter.divbutton switch.pill switch.noevent switch.nospace"
CONTRACTS="a-optional b-own-test c-no-trace d-record-only"

[ -x "$OMNICUI" ] || { echo "no bin/omnicui: run ./run.sh setup web first" >&2; exit 2; }
[ -d "$HERE/targets/webalt/node_modules" ] || (cd "$HERE/targets/webalt" && npm ci --no-audit --no-fund) || exit 2

# keep <name> <project>: the run's console output, results.json and report.html, in runs/<name>/
keep() {
  mkdir -p "$RUNS/$1"
  cp "$2/.omnicui/run/results.json" "$2/.omnicui/run/report.html" "$RUNS/$1/" 2>/dev/null
  printf '%-18s exit %s · %s\n' "$1" "$3" "$(grep -E '^[0-9]+ pass' "$RUNS/$1/console.txt")"
}

one() {
  case "$1" in
    base)
      "$OMNICUI" --project "$HERE" check web webalt >"$RUNS/base/console.txt" 2>&1; keep base "$HERE" $? ;;
    fail-on-pixels)
      "$OMNICUI" --project "$HERE" check web webalt counter --fail-on pixels >"$RUNS/fail-on-pixels/console.txt" 2>&1
      keep fail-on-pixels "$HERE" $? ;;
    counter.*|switch.*)
      OMNICUI_DEFECTS="webalt.$1" "$OMNICUI" --project "$HERE" check web webalt "${1%%.*}" >"$RUNS/$1/console.txt" 2>&1
      keep "$1" "$HERE" $? ;;
    *)
      # A contract experiment: the reference contracts with this switch contract, in a project of its own.
      [ -f "$HERE/contract-variants/$1.switch.contract.yaml" ] || { echo "no run $1" >&2; return 2; }
      P="$HERE/.exp/$1"; rm -rf "$P"; mkdir -p "$P/contracts" "$P/targets"
      cp "$ROOT/omnicui/contracts/counter.contract.yaml" "$P/contracts/"
      cp "$HERE/contract-variants/$1.switch.contract.yaml" "$P/contracts/switch.contract.yaml"
      ln -s "$ROOT/omnicui/targets/web" "$P/targets/web"; ln -s "$HERE/targets/webalt" "$P/targets/webalt"
      "$OMNICUI" --project "$P" check web webalt switch >"$RUNS/$1/console.txt" 2>&1; keep "$1" "$P" $?
      [ "$1" = d-record-only ] && {
        OMNICUI_DEFECTS=webalt.switch.pill "$OMNICUI" --project "$P" check web webalt switch >"$RUNS/d-record-only-pill/console.txt" 2>&1
        keep d-record-only-pill "$P" $?; } ;;
  esac
}

if [ $# -gt 0 ]; then
  for r in "$@"; do mkdir -p "$RUNS/$r"; [ "$r" = d-record-only ] && mkdir -p "$RUNS/d-record-only-pill"; one "$r"; done
else
  rm -rf "$RUNS"
  for r in base $VARIANTS fail-on-pixels $CONTRACTS; do mkdir -p "$RUNS/$r"; [ "$r" = d-record-only ] && mkdir -p "$RUNS/d-record-only-pill"; one "$r"; done
fi
# The web target's stub generator rewrote apps/web/src/omnicui from these contracts: put it back.
git -C "$ROOT" checkout -q -- apps/web/src/omnicui 2>/dev/null || true
