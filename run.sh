#!/bin/sh
# Try the omnicui reference repository on your own machine (macOS or Linux).
#
#   ./run.sh doctor [target]   what is installed, and what is missing
#   ./run.sh setup [target]    build omnicui, the apps and their targets (or one: web, egui, javafx)
#   ./run.sh app <app>         run an app for real: web (in your browser), desktop (egui), desktop-javafx
#   ./run.sh check [args]      run every contract on every target, then open the report
#   ./run.sh demo [target]     switch on deliberate defects one at a time and watch each get caught;
#                              with a target (web, egui, javafx), only that one, on its own
#   ./run.sh test              every test suite: unit, failure injection, real targets, report accessibility
#   ./run.sh all               doctor, setup, check, demo, test
#   ./run.sh omnicui …         run omnicui with any arguments, e.g. ./run.sh omnicui check egui counter
#
# Targets: web, egui (also: rust, desktop), javafx (also: java, desktop-javafx). The words can come
# in either order: ./run.sh app desktop and ./run.sh desktop app are the same.
set -eu
cd "$(dirname "$0")"
ROOT=$(pwd)
OMNICUI="$ROOT/bin/omnicui"

say() { printf '\n\033[1m%s\033[0m\n' "$*"; }
fail() { printf '\n\033[31m%s\033[0m\n' "$*" >&2; exit 1; }
have() { command -v "$1" >/dev/null 2>&1; }
# omnicui on this repository's project, omnicui/
omni() { "$OMNICUI" --project "$ROOT/omnicui" "$@"; }

# version_at_least 22.18.0 22.9.1 → true when the second is at least the first
version_at_least() {
  [ "$(printf '%s\n%s\n' "$1" "$2" | sort -t. -k1,1n -k2,2n -k3,3n | head -1)" = "$1" ]
}

# web | egui | javafx | all: the omnicui target, and the app it checks
target_of() {
  case "${1:-all}" in
    web) echo web ;; egui|rust|desktop) echo egui ;; javafx|java|desktop-javafx) echo javafx ;; all|"") echo all ;;
    *) fail "Unknown target '$1': use web, egui (or rust, desktop) or javafx (or desktop-javafx)." ;;
  esac
}
wants() { [ "$T" = all ] || [ "$T" = "$1" ]; }

doctor() {
  T=$(target_of "${1:-all}")
  say "Checking what is installed${1:+ for $T}"
  ok=1
  need() { # name, found version or "", minimum, hint
    if [ -z "$2" ]; then
      printf '  %-8s missing          needs %s or later. %s\n' "$1" "$3" "$4"; ok=0
    elif version_at_least "$3" "$2"; then
      printf '  %-8s %-16s ok\n' "$1" "$2"
    else
      printf '  %-8s %-16s too old: needs %s or later. %s\n' "$1" "$2" "$3" "$4"; ok=0
    fi
  }
  case "$(uname -s)" in
    Darwin) hint_go="brew install go"; hint_node="brew install node"; hint_java="brew install openjdk@21"; hint_rust="brew install rustup && rustup-init" ;;
    *) hint_go="https://go.dev/dl/"; hint_node="https://nodejs.org/ (or nvm install 22)"; hint_java="e.g. apt install openjdk-21-jdk"; hint_rust="https://rustup.rs/" ;;
  esac
  go_v=$(have go && go env GOVERSION 2>/dev/null | sed 's/^go//' || true)
  need go "$go_v" 1.21.0 "$hint_go (Go 1.21+ downloads the 1.27 toolchain this pack asks for by itself)"
  if wants web || [ "$T" = all ]; then
    node_v=$(have node && node -p process.versions.node 2>/dev/null || true)
    need node "$node_v" 22.18.0 "$hint_node (22.18+ runs .ts files directly)"
  fi
  if wants javafx; then
    java_v=$(have java && java -XshowSettings:properties -version 2>&1 | sed -n 's/^ *java.specification.version = //p' || true)
    need java "${java_v:+$java_v.0.0}" 21.0.0 "$hint_java (Maven is not needed: the JavaFX app and target have the Maven wrapper)"
  fi
  if wants egui; then
    rust_v=$(have cargo && cargo --version 2>/dev/null | cut -d' ' -f2 || true)
    need cargo "$rust_v" 1.85.0 "$hint_rust"
  fi
  if wants egui && [ "$(uname -s)" = Linux ]; then
    if [ -z "${VK_ICD_FILENAMES:-}" ] && ! ls /usr/share/vulkan/icd.d/*.json >/dev/null 2>&1; then
      echo "  vulkan   no driver found: egui will still run every test, but can't take screenshots."
      echo "           Install one (e.g. apt install mesa-vulkan-drivers, which includes a software driver)."
    fi
  fi
  [ $ok = 1 ] || fail "Install what is missing, then run ./run.sh setup${1:+ $1}"
  echo "  Everything needed is here."
}

setup() {
  T=$(target_of "${1:-all}")
  say "omnicui (Go): tools/omnicui → bin/omnicui"
  (cd tools/omnicui && go build -trimpath -o "$OMNICUI" ./cmd/omnicui)
  if wants web; then
    say "web: the app (apps/web), then its target (omnicui/targets/web) and Chromium for Playwright"
    (cd apps/web && npm ci --no-audit --no-fund)
    (cd omnicui/targets/web && npm ci --no-audit --no-fund && npx playwright install chromium)
    omni stubs web
    (cd apps/web && npx tsc --noEmit)
    (cd omnicui/targets/web && npx tsc --noEmit)
  fi
  if wants javafx; then
    say "javafx: the app (apps/desktop-javafx), then its target; the first run downloads Maven and the dependencies"
    (cd apps/desktop-javafx && ./mvnw -q -B install)
    (cd omnicui/targets/javafx && ./mvnw -q -B package)
  fi
  if wants egui; then
    say "egui: the app (apps/desktop), then its target; the first build of each takes a few minutes"
    omni stubs egui
    (cd apps/desktop && cargo build --release --locked)
    (cd omnicui/targets/egui && cargo build --release --locked)
  fi
  if [ "$T" = all ]; then
    say "report tests"
    (cd tools/omnicui/report-test && npm ci --no-audit --no-fund)
    say "what each target can do on this machine"
    omni targets
    say "Set up. Next: ./run.sh app desktop (or web, desktop-javafx), or ./run.sh check"
  else
    say "Set up. Next: ./run.sh app $(app_of "$T"), or ./run.sh demo $T"
  fi
}

app_of() { case "$1" in web) echo web ;; egui) echo desktop ;; javafx) echo desktop-javafx ;; esac; }

need_setup() { [ -x "$OMNICUI" ] || fail "Run ./run.sh setup first."; }

# An app, for real: the one people use. omnicui never opens these; it checks the same components headlessly.
app() {
  [ -n "${1:-}" ] || fail "Which app? ./run.sh app web, ./run.sh app desktop (egui) or ./run.sh app desktop-javafx"
  T=$(target_of "$1")
  case "$T" in
    web)
      [ -d apps/web/node_modules ] || fail "Run ./run.sh setup web first."
      say "apps/web: building it, then serving it and opening your browser (Ctrl+C here to stop)"
      cd apps/web && exec node serve.ts ;;
    egui)
      say "apps/desktop: cargo builds it if needed (the first build takes a few minutes), then the window opens"
      cd apps/desktop && exec cargo run --release --locked ;;
    javafx)
      [ -f apps/desktop-javafx/target/desktop-javafx.jar ] || fail "Run ./run.sh setup javafx first."
      say "apps/desktop-javafx: the window opens (close it to stop)"
      cd apps/desktop-javafx && exec java -jar target/desktop-javafx.jar ;;
    all) fail "Which app? ./run.sh app web, ./run.sh app desktop (egui) or ./run.sh app desktop-javafx" ;;
  esac
}

open_report() {
  report="$ROOT/omnicui/.omnicui/run/report.html"
  [ -f "$report" ] || return 0
  echo "Report: $report"
  if [ -t 1 ]; then
    if have open; then open "$report"; elif have xdg-open; then xdg-open "$report" >/dev/null 2>&1 || true; fi
  fi
}

check() {
  need_setup
  say "omnicui check $*"
  set +e
  omni check "$@"
  code=$?
  set -e
  open_report
  return $code
}

run_defect() { # defect, what it does, omnicui check arguments…
  defect=$1; what=$2; shift 2
  say "Defect $defect: $what"
  OMNICUI_DEFECTS=$defect omni check "$@" || true
}

# One target on its own. Pixel defects need a second target to compare with, so these are the
# defects a single target shows by itself: failed tests, timing against the contract, a broken adapter.
demo_one() {
  case "$1" in
    web)
      run_defect web.switch.no-name "the app's switch isn't named by its label (expect: fail)" web
      run_defect web.switch.slow "the app's switch thumb slides for 300 ms, linearly, not 200 ms ease-out (expect: a timing warning)" web ;;
    egui)
      run_defect egui.switch.no-name "the app's switch reports no name to AccessKit (expect: fail)" egui
      run_defect egui.switch.linear "the app's switch thumb slides for 300 ms, linearly, not 200 ms ease-out (expect: a timing warning)" egui ;;
    javafx)
      run_defect javafx.switch.no-label "the app's switch label doesn't name it (expect: fail)" javafx
      run_defect javafx.switch.linear "the app's switch thumb slides for 300 ms, linearly, not 200 ms ease-out (expect: a timing warning)" javafx
      run_defect javafx.adapter.stdout "a library prints on the target adapter's standard output (expect: broken)" javafx ;;
  esac
  say "$1 with no defect: everything passes"
  check "$1"
}

demo() {
  need_setup
  T=$(target_of "${1:-all}")
  if [ "$T" != all ]; then demo_one "$T"; return; fi
  run_defect javafx.switch.no-label "the JavaFX app's switch label doesn't name it (expect: fail on javafx)" switch
  run_defect egui.counter.small-text "the egui app draws the counter at 13 px instead of 16 (expect: pixel warnings involving egui)" counter
  run_defect javafx.host.stretch "the JavaFX target centres the component instead of placing it at (24, 24) (expect: pixel warnings involving javafx)" counter
  run_defect web.switch.slow "the web app's switch thumb slides for 300 ms, linearly (expect: a timing warning involving web)" switch
  run_defect javafx.adapter.stdout "a library prints on the JavaFX adapter's standard output (expect: javafx broken)" counter
  say "And with no defect: everything passes again"
  check
}

tests() {
  need_setup
  say "1/4  omnicui: unit tests and failure injection"
  (cd tools/omnicui && go vet ./... && go test -count=1 ./...)
  say "2/4  the real targets: agreement, every deliberate defect, --require, stubs (a few minutes)"
  mkdir -p omnicui/.omnicui
  set +e
  (cd tools/omnicui && OMNICUI_BIN="$OMNICUI" go test -count=1 -tags real -timeout 40m -v ./integration/) > omnicui/.omnicui/test-real.log 2>&1
  code=$?
  set -e
  grep -E '^(--- |    --- |ok|FAIL)' omnicui/.omnicui/test-real.log || true
  [ $code = 0 ] || fail "The real-target tests failed; the whole output is in omnicui/.omnicui/test-real.log"
  say "3/4  report.html in a browser: axe (WCAG 2.2 AA), on a failing and a passing report"
  OMNICUI_DEFECTS=javafx.adapter.stdout,egui.counter.small-text,web.switch.slow omni check >/dev/null || true
  (cd tools/omnicui/report-test && REPORT_LABEL=failing node report.test.ts)
  omni check >/dev/null
  (cd tools/omnicui/report-test && REPORT_LABEL=passing node report.test.ts)
  say "4/4  omnicui builds for every platform"
  (cd tools/omnicui && for p in linux/amd64 linux/arm64 darwin/amd64 darwin/arm64 windows/amd64 windows/arm64; do
    GOOS=${p%/*} GOARCH=${p#*/} go build -trimpath -o /dev/null ./cmd/omnicui && echo "built for $p"; done)
  say "Every suite passed."
}

is_command() { case "$1" in doctor|setup|app|show|check|demo|test|all|omnicui) return 0 ;; *) return 1 ;; esac; }

cmd=${1:-help}
[ $# -gt 0 ] && shift
# Either word order: `./run.sh desktop app` is `./run.sh app desktop`.
if ! is_command "$cmd" && [ $# -gt 0 ] && is_command "$1"; then
  name=$cmd; cmd=$1; shift; set -- "$name" "$@"
fi
case "$cmd" in
  doctor) doctor "$@" ;;
  setup) doctor "$@" && setup "$@" ;;
  app|show) app "$@" ;;
  check) check "$@" ;;
  demo) demo "$@" ;;
  test) tests ;;
  all) doctor && setup && check && demo && tests ;;
  omnicui) need_setup && exec "$OMNICUI" --project "$ROOT/omnicui" "$@" ;;
  *) sed -n '2,15p' "$0" | sed 's/^# \{0,1\}//' ;;
esac
