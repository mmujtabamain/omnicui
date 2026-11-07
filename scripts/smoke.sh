#!/usr/bin/env bash
# The smoke test: the release files of one version, used the way a project uses them. Runs on
# every merge request (CI) and in the release job before anything is published.
#
#   scripts/smoke.sh <version>     after `make release VERSION=<version>` (dist/ holds the files)
#
# It checks that the linux/amd64 binary reports its version and runs (help, contracts, targets,
# check on a project with two fake targets: all pass; a deliberate bug fails with exit 1; --failed
# runs only what failed), and that the source tarball, as a vendoring project takes it, builds.
set -euo pipefail

v="${1:?usage: scripts/smoke.sh <version>}"
root="$(cd "$(dirname "$0")/.." && pwd)"
dist="$root/dist"
os="$(go env GOOS)"; arch="$(go env GOARCH)"
bin="$dist/omnicui_${v}_${os}_${arch}"
[ "$os" = windows ] && bin="$bin.exe"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

step() { printf '\n== %s\n' "$*"; }
fail() { printf 'smoke test failed: %s\n' "$*" >&2; exit 1; }
expect() { # expect <exit code> <text the output must contain> <command…>
  local want="$1" text="$2"; shift 2
  local out code=0
  out="$("$@" 2>&1)" || code=$?
  printf '%s\n' "$out" | tail -n 4
  [ "$code" = "$want" ] || fail "\`$*\` exited $code, not $want"
  [ -z "$text" ] || printf '%s\n' "$out" | grep -qF -- "$text" || fail "\`$*\` did not print: $text"
}

step "the binary: $bin"
[ -x "$bin" ] || fail "no $bin (run make release VERSION=$v first)"
expect 0 "omnicui $v (protocol 1" "$bin" version
expect 0 "A contract: contracts/<name>.contract.yaml" "$bin" help contract
expect 0 "contracts are fine" "$bin" --project "$root/omnicui" contracts

step "a project with two fake targets"
(cd "$root/tools/omnicui" && go build -o "$work/fake" ./testdata/fakeadapter)
proj="$work/project"
mkdir -p "$proj/contracts"
cp "$root"/omnicui/contracts/*.contract.yaml "$proj/contracts/"
for t in a b; do
  mkdir -p "$proj/targets/$t"
  : > "$proj/targets/$t/fake.txt"
  printf "name: %s\nlanguage: go\nadapter: 'exec \"%s\" fake.txt'\n" "$t" "$work/fake" > "$proj/targets/$t/target.yaml"
done
cd "$proj"
expect 0 "" "$bin" targets
expect 0 "20 pass · 0 fail · 0 broken · 0 not advertised · 0 warnings" "$bin" check
echo "FAKE_BUG=double" > "$proj/targets/b/fake.txt"
expect 1 "fail ·" "$bin" check
expect 1 "" "$bin" check --failed
grep -q '"test": "starts at the start value"' .omnicui/run/results.json && fail "--failed ran a test that had passed"

step "the source tarball, as a vendoring project builds it"
mkdir -p "$work/vendor"
tar -xzf "$dist/omnicui-src-$v.tar.gz" -C "$work/vendor"
(cd "$work/vendor/omnicui" && go build -o "$work/vendored" ./cmd/omnicui)
expect 0 "omnicui dev (protocol 1" "$work/vendored" version

printf '\nsmoke test passed: omnicui %s\n' "$v"
