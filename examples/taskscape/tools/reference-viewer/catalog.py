"""Print the reference's hotkey catalog (common/src/hotkeys.rs) as the JSON `list_hotkeys` returns."""
import json, re, sys

src = open(sys.argv[1]).read()
# The first `Name => "token"` table is the wire spelling; later tables are display glyphs.
names = {}
for k, v in re.findall(r'(\w+) => "((?:[^"\\]|\\.)*)"', src):
    names.setdefault(k, v.replace("\\\\", "\\"))
out = []
for body in re.findall(r"Command \{(.*?)\n    \}", src, re.S):
    get = lambda f: re.search(rf'{f}: "([^"]*)"', body).group(1)
    scope = re.search(r"scope: Scope::(\w+)", body).group(1).lower()
    parts = re.search(r"default_accel: accel!\(([^)]*)\)", body).group(1).split("+")
    parts = [p.strip() for p in parts]
    accel = "+".join(parts[:-1] + [names.get(parts[-1], parts[-1])])
    out.append({"id": get("id"), "label": get("label"), "scope": scope, "accel": accel, "default": accel})
json.dump(out, sys.stdout, indent=1, ensure_ascii=False)
