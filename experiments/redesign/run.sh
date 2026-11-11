#!/bin/sh
# The redesign experiment: does omnicui test behaviour when the UI is different?
#
#   experiments/redesign/run.sh            set up webalt, then every run below, into runs/
#   experiments/redesign/run.sh base       one of them: base, a variant (brand, counter.copy, …), fail-on-palette[-brand],
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
VARIANTS="brand counter.copy counter.srlabel counter.toolbar counter.double counter.divbutton switch.pill switch.noevent switch.nospace"
CONTRACTS="a-optional b-own-test c-no-trace d-record-only e-text-matches"

[ -x "$OMNICUI" ] || { echo "no bin/omnicui: run ./run.sh setup web first" >&2; exit 2; }
[ -d "$HERE/targets/webalt/node_modules" ] || (cd "$HERE/targets/webalt" && npm ci --no-audit --no-fund) || exit 2

# keep <name> <project>: the run's console output, results.json and report.html, in runs/<name>/
keep() {
  mkdir -p "$RUNS/$1"
  cp "$2/.omnicui/run/results.json" "$2/.omnicui/run/report.html" "$RUNS/$1/" 2>/dev/null
  printf '%-18s exit %s · %s\n' "$1" "$3" "$(grep -E '^[0-9]+ pass' "$RUNS/$1/console.txt")"
}

# project <variant>: a project of its own with the reference contracts, one of them replaced by
# contract-variants/<variant>.<counter|switch>.contract.yaml, and both targets. Prints its folder.
project() {
  file=$(ls "$HERE/contract-variants/$1".*.contract.yaml 2>/dev/null | head -1)
  [ -n "$file" ] || { echo "no run $1" >&2; return 2; }
  which=$(basename "$file" .contract.yaml); which=${which##*.}
  P="$HERE/.exp/$1"; rm -rf "$P"; mkdir -p "$P/contracts" "$P/targets"
  cp "$ROOT/omnicui/contracts/"*.contract.yaml "$P/contracts/"
  cp "$file" "$P/contracts/$which.contract.yaml"
  ln -s "$ROOT/omnicui/targets/web" "$P/targets/web"; ln -s "$HERE/targets/webalt" "$P/targets/webalt"
  echo "$P"
}

one() {
  case "$1" in
    base)
      "$OMNICUI" --project "$HERE" check web webalt >"$RUNS/base/console.txt" 2>&1; keep base "$HERE" $? ;;
    fail-on-palette|fail-on-palette-brand)
      # A CI gate on the design language, with the switch's thumb optional (contract variant a):
      # the redesign's own palette fails it, the same layouts in the original's colours pass it.
      P=$(project a-optional) || return 2
      [ "$1" = fail-on-palette-brand ] && export OMNICUI_DEFECTS=webalt.brand
      "$OMNICUI" --project "$P" check web webalt --fail-on pixels --fail-on palette >"$RUNS/$1/console.txt" 2>&1; keep "$1" "$P" $?
      unset OMNICUI_DEFECTS ;;
    brand)
      # The same layouts in the original design language: its colours, its font.
      OMNICUI_DEFECTS=webalt.brand "$OMNICUI" --project "$HERE" check web webalt >"$RUNS/$1/console.txt" 2>&1
      keep "$1" "$HERE" $? ;;
    counter.*|switch.*)
      OMNICUI_DEFECTS="webalt.$1" "$OMNICUI" --project "$HERE" check web webalt "${1%%.*}" >"$RUNS/$1/console.txt" 2>&1
      keep "$1" "$HERE" $? ;;
    *)
      # A contract experiment: the reference contracts with this one changed, in a project of its own.
      P=$(project "$1") || return 2
      file=$(ls "$HERE/contract-variants/$1".*.contract.yaml | head -1)
      which=$(basename "$file" .contract.yaml); which=${which##*.}     # switch or counter
      # e-text-matches is about the copy: run it with the "3 taps" variant.
      [ "$1" = e-text-matches ] && export OMNICUI_DEFECTS=webalt.counter.copy
      "$OMNICUI" --project "$P" check web webalt "$which" >"$RUNS/$1/console.txt" 2>&1; keep "$1" "$P" $?
      unset OMNICUI_DEFECTS
      [ "$1" = d-record-only ] && {
        OMNICUI_DEFECTS=webalt.switch.pill "$OMNICUI" --project "$P" check web webalt switch >"$RUNS/d-record-only-pill/console.txt" 2>&1
        keep d-record-only-pill "$P" $?; } ;;
  esac
}

if [ $# -gt 0 ]; then
  for r in "$@"; do mkdir -p "$RUNS/$r"; [ "$r" = d-record-only ] && mkdir -p "$RUNS/d-record-only-pill"; one "$r"; done
else
  rm -rf "$RUNS"
  for r in base $VARIANTS fail-on-palette fail-on-palette-brand $CONTRACTS; do mkdir -p "$RUNS/$r"; [ "$r" = d-record-only ] && mkdir -p "$RUNS/d-record-only-pill"; one "$r"; done
fi
# The web target's stub generator rewrote apps/web/src/omnicui from these contracts: put it back.
git -C "$ROOT" checkout -q -- apps/web/src/omnicui 2>/dev/null || true
