#!/usr/bin/env python3
from __future__ import annotations

"""Build a per-app subset of the Material Symbols icon font.

The full `material-symbols-outlined.woff2` is ~3.8 MB — every one of its ~3600
icons. Each app uses only a few dozen, so shipping the whole thing is the single
biggest drag on cold-boot (the webview downloads and parses multiple megabytes
before icons can paint). This script scans an app's source for the icon names it
actually references and emits a tiny subset font (tens of KB) containing just
those glyphs.

It produces one font per app (main + tray), since their icon sets differ. Wired
to run before Vite via each app's `predev` / `prebuild` npm hook, so the subset
is always current; it self-skips when nothing changed.

Subsetting note: Material Symbols renders an icon from its *name* via an OpenType
ligature (`<span>settings</span>` → the gear glyph), and each glyph is named for
its icon. A plain `pyftsubset --text=...` does NOT shrink the font: once the
subset contains every letter (which any real icon set does), ligature closure
re-adds every icon. So we instead prune the GSUB ligature table down to the
wanted names first, then subset — keeping name→icon rendering intact.

Requires the Python packages listed in `scripts/requirements.txt`. If they are
missing, the script exits with a clear install command instead of modifying the
environment itself.
"""

import os
import pathlib
import re
import sys
from importlib.util import find_spec
from typing import Any

ROOT = pathlib.Path(__file__).resolve().parent.parent
REQUIREMENTS_FILE = ROOT / "scripts" / "requirements.txt"
VENV = ROOT / "scripts" / ".venv"

USE_COLOR = sys.stderr.isatty() and os.environ.get("TERM") != "dumb"
RED = "\033[0;31m" if USE_COLOR else ""
YELLOW = "\033[0;33m" if USE_COLOR else ""
BLUE = "\033[0;34m" if USE_COLOR else ""
BOLD = "\033[1m" if USE_COLOR else ""
RESET = "\033[0m" if USE_COLOR else ""


def _rel_from_cwd(path: pathlib.Path) -> str:
    return os.path.relpath(path, pathlib.Path.cwd())


def _print_stderr(line: str = "") -> None:
    print(line, file=sys.stderr)


def _step(message: str) -> None:
    _print_stderr(f"{BOLD}{BLUE}==> {message}{RESET}")


def _warn(message: str) -> None:
    _print_stderr(f"{YELLOW}{message}{RESET}")


def _fail(message: str) -> None:
    _print_stderr(f"{RED}{message}{RESET}")


def ensure_dependencies() -> None:
    missing = [
        package_name
        for module_name, package_name in (
            ("fontTools", "fonttools>=4.38"),
            ("brotli", "brotli>=1.1"),
        )
        if find_spec(module_name) is None
    ]
    if not missing:
        return

    joined = ", ".join(missing)
    in_scripts_venv = VENV.exists() and pathlib.Path(sys.prefix).resolve() == VENV.resolve()

    _fail(f"[subset-icons] missing Python dependencies in this interpreter: {joined}")
    _print_stderr()

    if VENV.exists() and not in_scripts_venv:
        _step("Activate the repo's scripts virtualenv, then rerun your command:")
        _print_stderr(f"  source {_rel_from_cwd(VENV / 'bin' / 'activate')}")
        _print_stderr("  npm run dev")
        _print_stderr()
        _warn(
            "This usually means npm picked the system python3 instead of "
            "scripts/.venv/bin/python."
        )
    else:
        _step("Install the script dependencies, then rerun your command:")
        if REQUIREMENTS_FILE.exists():
            _print_stderr(f"  python3 -m pip install -r {_rel_from_cwd(REQUIREMENTS_FILE)}")
        else:
            _print_stderr(f"  python3 -m pip install {' '.join(missing)}")

    raise SystemExit(1)


ensure_dependencies()

from fontTools.subset import Options, Subsetter
from fontTools.ttLib import TTFont

SRC_FONT = ROOT / "node_modules" / "material-symbols" / "material-symbols-outlined.woff2"

# Each app's subset covers its own source plus the shared UI library (whose
# components — the editor toolbar, file-kind icons — it also renders).
APPS = {
    "main": {
        "scan": ["main/src", "common-ui/src"],
        "out": "main/src/assets/fonts/material-symbols-subset.woff2",
    },
    "tray": {
        "scan": ["tray/src", "common-ui/src"],
        "out": "tray/src/assets/fonts/material-symbols-subset.woff2",
    },
}

# Any pure snake_case quoted string is a candidate icon name; we intersect with
# the font's real icon names below, so non-icon strings fall away harmlessly.
# Over-including is safe (a spare glyph); missing one would render as raw text.
NAME_LITERAL = re.compile(r"""['"`]([a-z][a-z0-9_]+)['"`]""")


def scan_candidate_names(dirs: list[str]) -> set[str]:
    names: set[str] = set()
    for d in dirs:
        base = ROOT / d
        for f in list(base.rglob("*.ts")) + list(base.rglob("*.tsx")):
            names.update(NAME_LITERAL.findall(f.read_text(encoding="utf-8")))
    return names


def source_files(dirs: list[str]) -> list[pathlib.Path]:
    out: list[pathlib.Path] = []
    for d in dirs:
        base = ROOT / d
        out += list(base.rglob("*.ts")) + list(base.rglob("*.tsx"))
    return out


def _unwrap(subtable: Any) -> Any:
    return subtable.ExtSubTable if subtable.__class__.__name__ == "ExtensionSubst" else subtable


def build_font(wanted: set[str]) -> tuple[bytes, int]:
    """Prune the ligature table to `wanted`, subset, return (woff2 bytes, glyph count)."""
    font = TTFont(str(SRC_FONT))
    glyph_to_char = {gn: chr(cp) for cp, gn in font.getBestCmap().items()}

    for lookup in font["GSUB"].table.LookupList.Lookup:
        for subtable in lookup.SubTable:
            st = _unwrap(subtable)
            if st.__class__.__name__ != "LigatureSubst":
                continue
            for first, ligs in list(st.ligatures.items()):
                st.ligatures[first] = [
                    lig
                    for lig in ligs
                    if "".join(glyph_to_char.get(g, "\x00") for g in [first, *lig.Component])
                    in wanted
                ]

    options = Options()
    options.flavor = "woff2"
    options.layout_features = ["*"]  # keep `liga` so name→icon still renders
    options.name_IDs = ["*"]
    options.notdef_outline = True
    subsetter = Subsetter(options=options)
    # The ligature inputs are the letters/digits/underscore of the wanted names.
    subsetter.populate(text="".join(sorted({c for n in wanted for c in n})))
    subsetter.subset(font)

    import io

    buf = io.BytesIO()
    font.save(buf)
    return buf.getvalue(), len(font.getGlyphOrder())


def is_stale(out_path: pathlib.Path, inputs: list[pathlib.Path]) -> bool:
    if not out_path.exists():
        return True
    out_mtime = out_path.stat().st_mtime
    newest_in = max(p.stat().st_mtime for p in inputs if p.exists())
    return newest_in > out_mtime


def real_icon_names(font: TTFont) -> set[str]:
    glyph_to_char = {gn: chr(cp) for cp, gn in font.getBestCmap().items()}
    names: set[str] = set()
    for lookup in font["GSUB"].table.LookupList.Lookup:
        for subtable in lookup.SubTable:
            st = _unwrap(subtable)
            if st.__class__.__name__ != "LigatureSubst":
                continue
            for first, ligs in st.ligatures.items():
                for lig in ligs:
                    names.add("".join(glyph_to_char.get(g, "\x00") for g in [first, *lig.Component]))
    return names


def main() -> int:
    if not SRC_FONT.exists():
        print(f"[subset-icons] source font missing: {SRC_FONT} (run npm install)", file=sys.stderr)
        return 1

    which = [a for a in sys.argv[1:] if a in APPS] or list(APPS)
    real_names = real_icon_names(TTFont(str(SRC_FONT)))

    for app in which:
        cfg = APPS[app]
        out_path = ROOT / cfg["out"]
        inputs = source_files(cfg["scan"]) + [SRC_FONT, pathlib.Path(__file__)]
        if not is_stale(out_path, inputs):
            print(f"[subset-icons] {app}: up to date ({out_path.stat().st_size // 1024} KB)")
            continue

        wanted = scan_candidate_names(cfg["scan"]) & real_names
        data, glyphs = build_font(wanted)
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_path.write_bytes(data)
        print(f"[subset-icons] {app}: {len(wanted)} icons → {len(data) // 1024} KB ({glyphs} glyphs)")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
