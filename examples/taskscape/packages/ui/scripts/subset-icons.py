"""Builds the icon font the app ships: Material Symbols Outlined cut down to the icons the code
names (tens of KB instead of 3.8 MB, so an icon never shows as its name while the font loads).

  python3 packages/ui/scripts/subset-icons.py            write src/styles/fonts/material-symbols-subset.woff2
  python3 packages/ui/scripts/subset-icons.py --check    exit 1 if the code names an icon the font lacks

An icon is any string literal in packages/ui/src or apps/web/frontend/src that is also an icon name
in the full font; a few extra glyphs cost little. Run it after using a new icon (the check fails
until you do). Needs fonttools and brotli (pip install fonttools brotli).
"""
import io, os, re, sys

from fontTools import subset
from fontTools.ttLib import TTFont

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "../../.."))
FULL = os.path.join(ROOT, "node_modules/material-symbols/material-symbols-outlined.woff2")
OUT = os.path.join(HERE, "../src/styles/fonts/material-symbols-subset.woff2")
SOURCES = [os.path.join(ROOT, "packages/ui/src"), os.path.join(ROOT, "apps/web/frontend/src")]
LITERAL = re.compile(r"""["'`]([a-z0-9_]{2,60})["'`]""")


def used_names(known):
    names = set()
    for top in SOURCES:
        for d, _, files in os.walk(top):
            for f in files:
                if f.endswith((".ts", ".tsx")):
                    names |= {m for m in LITERAL.findall(open(os.path.join(d, f), encoding="utf-8").read()) if m in known}
    return names


def build(names):
    opts = subset.Options()
    opts.flavor = "woff2"
    opts.layout_features = ["rlig", "rclt", "liga"]
    opts.layout_closure = False  # only the ligatures of the icons asked for, not every icon spelled with these letters
    opts.notdef_outline = True
    opts.name_IDs = ["*"]
    opts.glyph_names = True  # keeps icon names in the font, which the --check reads
    font = TTFont(FULL)
    s = subset.Subsetter(opts)
    letters = "".join(sorted({c for n in names for c in n}))
    s.populate(glyphs=sorted(names | {n + ".fill" for n in names if n + ".fill" in font.getGlyphOrder()}), text=letters)
    s.subset(font)
    buf = io.BytesIO()
    font.flavor = "woff2"
    font.save(buf)
    return buf.getvalue()


def main():
    known = set(TTFont(FULL).getGlyphOrder())
    names = used_names(known)
    if "--check" in sys.argv:
        have = set(TTFont(OUT).getGlyphOrder()) if os.path.exists(OUT) else set()
        missing = sorted(names - have)
        print(f"{len(names)} icons named in code; " + (f"missing from the subset: {', '.join(missing)} (run subset-icons.py)" if missing else "all in the subset"))
        sys.exit(1 if missing else 0)
    data = build(names)
    os.makedirs(os.path.dirname(OUT), exist_ok=True)
    open(OUT, "wb").write(data)
    print(f"{len(names)} icons -> {OUT} ({len(data) // 1024} KB)")


if __name__ == "__main__":
    main()
