"""Checks one desktop widget against its contract, in a build slot of its own:

    python3 apps/desktop/scripts/check-widget.py <contract name> [--slot N] [--test <name>]

The slot (.checks/desktop-slots/N) is a copy of the widget library and the desktop target with the
finished widgets (ui/src/widgets/) plus this one: its draft (ui/drafts/<name>.rs) if there is one, else
its finished file. Builders working on different widgets at once each use their own slot, so one
draft that does not compile never breaks another's build. Prints BUILD, then CONTRACT with the
counts and every failing test. Exit code 0 when it builds and every test passes.
"""
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]          # examples/taskscape
UI = ROOT / "apps" / "desktop" / "ui"
TARGET = ROOT / "omnicui" / "targets" / "desktop"
OMNICUI = ROOT.parents[1] / "bin" / "omnicui"

args = sys.argv[1:]
name = args[0]
slot = args[args.index("--slot") + 1] if "--slot" in args else "0"
tests = [a for i, a in enumerate(args) if i > 0 and args[i - 1] == "--test"]

dest = ROOT / ".checks" / "desktop-slots" / slot
ui, target = dest / "ui", dest / "target-src"
for d in (ui / "src", target):
    shutil.rmtree(d, ignore_errors=True)
shutil.copytree(UI / "src", ui / "src")
for f in ("Cargo.toml", "build.rs"):
    shutil.copy(UI / f, ui / f)
if not (ui / "fonts").exists():
    shutil.copytree(UI / "fonts", ui / "fonts")
draft = UI / "drafts" / f"{name}.rs"
if draft.exists():
    shutil.copy(draft, ui / "src" / "widgets" / f"{name}.rs")
shutil.copytree(TARGET / "src", target / "src")
toml = (TARGET / "Cargo.toml").read_text().replace('path = "../../../apps/desktop/ui"', 'path = "../ui"').replace('members = ["stubs"]', "members = []")
(target / "Cargo.toml").write_text(toml)
shutil.copy(TARGET / "Cargo.lock", target / "Cargo.lock")

env = {**os.environ, "CARGO_TARGET_DIR": str(dest / "build"), "PATH": f"{Path.home()}/.cargo/bin:{Path.home()}/.local/node22/bin:{os.environ['PATH']}"}
build = subprocess.run(["cargo", "build", "-q", "--release", "--bin", "adapter"], cwd=target, env=env, capture_output=True, text=True)
errors = [l for l in build.stderr.splitlines() if l.startswith(("error", "warning"))]
if build.returncode != 0:
    print("BUILD: errors")
    print("\n".join(build.stderr.splitlines()[:60]))
    sys.exit(1)
own = [l for l in build.stderr.split("\n\n") if f"widgets/{name}.rs" in l and l.startswith("warning")]
print("BUILD: ok" + (f" ({len(own)} warnings in {name}.rs)" if own else ""))
for w in own[:10]:
    print(w)

out = ROOT / ".checks" / f"desktop-{name}"
env["DESKTOP_ADAPTER"] = str(dest / "build" / "release" / "adapter")
env.setdefault("VK_ICD_FILENAMES", str(Path.home() / ".local" / "mesa" / "lvp_icd.json"))
cmd = [str(OMNICUI), "--project", str(ROOT / "omnicui"), "check", "desktop", name, "--out", str(out)]
for t in tests:
    cmd += ["--test", t]
run = subprocess.run(cmd, cwd=ROOT, env=env, capture_output=True, text=True)
try:
    results = json.loads((out / "results.json").read_text())["results"]
except Exception:
    print("CONTRACT: did not run")
    print("\n".join((run.stdout + run.stderr).splitlines()[-25:]))
    sys.exit(1)
count = {}
for r in results:
    count[r["status"]] = count.get(r["status"], 0) + 1
print("CONTRACT: " + ", ".join(f"{v} {k}" for k, v in sorted(count.items())))
for r in results:
    if r["status"] in ("fail", "broken"):
        print(f"- {r['status'].upper()} {r['test']}: {' / '.join(r.get('messages') or [])[:400]}")
print(f"SCREENSHOTS: {out}/desktop/{name}/<test>/<size>-<scheme>.png")
sys.exit(0 if all(r["status"] not in ("fail", "broken") for r in results) else 1)
