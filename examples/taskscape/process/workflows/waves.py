"""Writes the widget build waves (args for widgets.js): python3 waves.py <out-dir>. A widget comes after the widgets it composes."""
import json, os, re, sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "../.."))
SPECS = os.path.join(ROOT, "spec/library/widgets")
CONTRACTS = os.path.join(ROOT, "omnicui/contracts")

USES = {
    "Keycap": ["hotkey-hint", "button"], "SectionHeader": ["text", "button"], "SearchField": ["input-group", "input", "button"],
    "PaneFooter": ["footer"], "DialogButton": ["button"], "ConfirmButton": ["button"], "DisclosureButton": ["button"],
    "Dialog": ["overlay", "scrim", "button"], "Lightbox": ["overlay", "scrim", "button"], "CommandInput": ["input-group"],
    "CommandList": ["scroll-area"], "ComboboxItem": ["match-highlight", "hotkey-hint"], "ListRow": ["menu-item"],
    "SettingRow": ["switch", "text"], "SidebarTab": ["text"], "ListTab": ["list-tab-label"], "TitleBar": ["tab-list", "button"],
    "SubtaskRow": ["checkbox", "button"], "TaskRow": ["checkbox", "button", "row"], "InlineComposer": ["input", "button"],
    "RichTextEditor": ["toolbar", "toggle", "mention-chip", "character-counter"], "Menu": ["menu-item", "separator"],
    "MenuButton": ["button", "menu"], "TaskTreeRow": ["checkbox", "button", "menu"],
    "CommandPalette": ["overlay", "command-input", "command-list", "combobox-item"],
}
WAVE0 = ["Icon"]
WAVE3 = ["MenuButton", "TaskTreeRow", "CommandPalette"]
NOTES = {
    "Icon": "Icon already exists and Button uses it as <Icon name size={px number} weight data-slot=…/>. Keep that working: size also accepts a number of px, and a data-slot passed by the caller replaces your own.",
}

def kebab(n): return re.sub(r"(?<!^)(?=[A-Z])", "-", n).lower()

names = sorted(f[:-5] for f in os.listdir(SPECS) if f.endswith(".json"))
have = {f.split(".")[0] for f in os.listdir(CONTRACTS)}
items = {}
for n in names:
    c = n.lower()
    if c not in have or n == "Button":
        continue
    items[n] = {"name": n, "contract": c, "file": kebab(n), "spec": os.path.join(SPECS, f"{n}.json"),
                **({"uses": USES[n]} if n in USES else {}), **({"note": NOTES[n]} if n in NOTES else {})}
wave = {n: 0 if n in WAVE0 else 3 if n in WAVE3 else 2 if n in USES else 1 for n in items}
out = sys.argv[1]
root = ROOT
for k in range(4):
    ws = [items[n] for n in sorted(items) if wave[n] == k]
    for i in range(0, len(ws), 22):
        part = ws[i:i + 22]
        path = os.path.join(out, f"widgets-{k}{chr(97 + i // 22)}.json")
        json.dump({"root": root, "widgets": part}, open(path, "w"), indent=1)
        print(path, len(part), " ".join(w["name"] for w in part))
