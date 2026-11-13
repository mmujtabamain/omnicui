#!/usr/bin/env python3
"""The web app's Material Symbols subset: the icons named in the shared sources
(tauri/main/src, tauri/common-ui/src) and in this app's own (src/), cut from
this app's material-symbols package. The subsetting is the Tauri app's own
script (tauri/scripts/subset-icons.py), loaded and pointed at these folders.

usage: subset-icons.py   (run by scripts/build.sh; needs fonttools and brotli)
"""
import importlib.util
import sys

# Load the Tauri app's script without leaving bytecode in its folder.
sys.dont_write_bytecode = True
import pathlib

WEB = pathlib.Path(__file__).resolve().parent.parent
TAURI = WEB.parent / "tauri"

spec = importlib.util.spec_from_file_location("subset_icons", TAURI / "scripts" / "subset-icons.py")
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)

mod.SRC_FONT = WEB / "node_modules" / "material-symbols" / "material-symbols-outlined.woff2"
out = WEB / "src" / "assets" / "fonts" / "material-symbols-subset.woff2"
scan = [str(TAURI / "main" / "src"), str(TAURI / "common-ui" / "src"), str(WEB / "src")]

inputs = mod.source_files(scan) + [mod.SRC_FONT, pathlib.Path(__file__)]
if mod.is_stale(out, inputs):
    wanted = mod.scan_candidate_names(scan) & mod.real_icon_names(mod.TTFont(str(mod.SRC_FONT)))
    data, glyphs = mod.build_font(wanted)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_bytes(data)
    print(f"[subset-icons] web: {len(wanted)} icons -> {len(data) // 1024} KB ({glyphs} glyphs)")
else:
    print(f"[subset-icons] web: up to date ({out.stat().st_size // 1024} KB)")
