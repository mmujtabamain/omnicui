"""Turns the inventory, catalogue and comment workflows' results into spec files.

  collect_inventory.py screens <results.json>...   W4/W5 -> spec/inventory/screens.json and <id>.json
  collect_inventory.py families                    widget rows by family -> spec/inventory/rows-<family>.json
  collect_inventory.py catalogue <result.json>     W6 -> spec/catalogue.json
  collect_inventory.py comments <result.json>      W7 -> spec/flow-comments.json (plus the orchestrator's own)
"""
import html, json, os, sys, time

HERE = os.path.dirname(os.path.abspath(__file__))
SPEC = os.path.abspath(os.path.join(HERE, "../../spec"))
INV = os.path.join(SPEC, "inventory")
SCREENS = {s["id"]: s for s in json.load(open(os.path.join(HERE, "screens.json")))["screens"]}

TITLES = {"settings-shortcuts": "Settings: Shortcuts", "settings-capture": "Settings: Capture", "settings-about": "Settings: About", "settings-general": "Settings: General", "capture-bar": "Capture bar", "context-menus": "Menus", "task-list": "Task list"}

FAMILIES = [
    ("buttons", "Buttons", ["button", "toolbar-button", "key-action", "link"], True),
    ("icon-buttons", "Icon buttons", ["icon-button"], True),
    ("choice", "Selection and toggles", ["checkbox", "round-check", "toggle", "segmented-control", "tab", "nav-item"], True),
    ("text", "Text entry", ["text-field", "search-field", "rich-text-editor", "select"], True),
    ("menus", "Menus", ["menu", "menu-item", "context-menu", "dropdown", "popover"], True),
    ("overlays", "Sheets, dialogs and overlays", ["sheet", "dialog", "toast", "lightbox", "tooltip"], True),
    ("keys", "Keys and shortcuts", ["keycap", "hotkey-hint"], False),
    ("status", "Status and feedback", ["badge", "progress-bar", "spinner", "empty-state"], False),
    ("rows", "Rows and settings", ["list-row", "task-row", "setting-row", "settings-group"], False),
    ("labels", "Text, headers and icons", ["label", "section-header", "icon"], False),
    ("structure", "Surfaces and structure", ["title-bar", "window-controls", "surface", "divider", "thumbnail", "other"], False),
]


def unescape(v):
    if isinstance(v, str):
        return html.unescape(v)
    if isinstance(v, list):
        return [unescape(x) for x in v]
    if isinstance(v, dict):
        return {k: unescape(x) for k, x in v.items()}
    return v


def results(paths):
    for p in paths:
        r = json.load(open(p))
        if isinstance(r, dict) and "result" in r and "agentCount" in r:
            r = r["result"]
        yield from (r if isinstance(r, list) else [r])


def cmd_screens(paths):
    os.makedirs(INV, exist_ok=True)
    out = {}
    path = os.path.join(INV, "screens.json")
    if os.path.exists(path):
        out = {s["id"]: s for s in json.load(open(path))["screens"]}
    for r in results(paths):
        if not r or r.get("failed"):
            print("FAILED", r and r.get("id"))
            continue
        s, m = SCREENS[r["id"]], unescape(r["merged"])
        m["links"] = [l for l in m["links"] if l["to"] in SCREENS]
        out[r["id"]] = {"id": r["id"], "title": TITLES.get(r["id"], s["name"].split(":")[0]), "tier": s["tier"], "states": s["states"], **m, "counts": r["counts"]}
        json.dump(out[r["id"]], open(os.path.join(INV, f"{r['id']}.json"), "w"), indent=1, ensure_ascii=False)
        print(r["id"], r["counts"], "merged widgets", len(m["widgets"]), "links", len(m["links"]))
    order = list(SCREENS)
    json.dump({"screens": [out[i] for i in order if i in out]}, open(path, "w"), indent=1, ensure_ascii=False)


def cmd_families():
    screens = json.load(open(os.path.join(INV, "screens.json")))["screens"]
    kind_of = {k: f for f, _, ks, _ in FAMILIES for k in ks}
    rows = {f: [] for f, *_ in FAMILIES}
    for s in screens:
        for w in s["widgets"]:
            rows[kind_of.get(w["kind"], "structure")].append({"screen": s["id"], **{k: w.get(k) for k in ("kind", "component", "variant", "purpose", "text", "states", "behaviour", "look", "count", "region")}})
    fams = []
    for f, title, kinds, high in FAMILIES:
        p = os.path.join(INV, f"rows-{f}.json")
        json.dump(rows[f], open(p, "w"), indent=1, ensure_ascii=False)
        fams.append({"id": f, "title": title, "kinds": kinds, "file": p, "high": high})
        print(f, len(rows[f]), "rows")
    json.dump({"families": fams}, open(os.path.join(HERE, "families.json"), "w"))


def cmd_catalogue(path):
    titles = {f: t for f, t, *_ in FAMILIES}
    kinds = {f: k for f, _, k, _ in FAMILIES}
    out = []
    for r in results([path]):
        if not r or r.get("failed"):
            print("FAILED", r and r.get("id"))
            continue
        e = unescape(r["entry"])
        out.append({"kind": r["id"], "title": titles[r["id"]], "kinds": kinds[r["id"]], "description": e["description"], "variants": e["variants"]})
        print(r["id"], len(e["variants"]), "variants,", r["issues"], "critic issues")
    json.dump(out, open(os.path.join(SPEC, "catalogue.json"), "w"), indent=1, ensure_ascii=False)


ORCH = [
    ("screen:capture-bar", "web", "Not in the web version: the capture bar is a desktop window summoned by a global hotkey. On the web, tasks are added by hand in the main window only."),
    ("screen:sheets", "desktop", "On the desktop version every sheet becomes its own window (new list, new project, rename, confirmations), with no backdrop dimming the main window. The backdrop was a workaround for Tauri, where opening windows lags."),
    ("screen:sheets", "web", "On the web these stay in-page dialogs over a backdrop, which is the platform's own idiom."),
    ("screen:settings-general", "both", "The reference opens Settings as its own window. Desktop: keep it a separate window. Web: a /settings page in the app, not a modal."),
    ("screen:title-bar", "web", "The web has no window controls (traffic lights) and no drag region; the browser owns the window. The project switcher and list tabs stay."),
    ("screen:task-inspector", "web", "Screenshot attachments (the Shot button) are desktop only. On the web, attachments are files uploaded to the server and links."),
    ("screen:settings-capture", "web", "The Capture pane is about screen capture and the capture bar; neither exists on the web, so this pane is dropped there."),
    ("screen:settings-shortcuts", "web", "Global shortcuts (the Global group) cannot exist in a browser, and several macOS chords are taken by the browser. The web keeps in-app shortcuts only, with web-safe defaults."),
    ("screen:task-list", "both", "Option changes the check from select to done. Keep this mode on both versions; on Windows and the web the modifier is Alt, and the hint text must name the platform's key."),
    ("screen:main-window", "desktop", "Desktop windows open as real windows where a desktop app would open one (Settings, sheets, the lightbox), not as layers inside the main window."),
]


def cmd_comments(path):
    out, t = [], int(time.time() * 1000)
    for i, (node, tag, text) in enumerate(ORCH):
        out.append({"id": f"seed-o{i}", "node": node, "parent": None, "author": "Opus (orchestrator)", "tag": tag, "text": text, "resolved": False, "created_at": t})
    for r in results([path]):
        for i, c in enumerate(unescape(r.get("comments", []))):
            out.append({"id": f"seed-{r['id']}-{i}", "node": c["node"], "parent": None, "author": "Haiku critic", "tag": c["tag"], "text": c["text"], "resolved": False, "created_at": t + 1})
    json.dump(out, open(os.path.join(SPEC, "flow-comments.json"), "w"), indent=1, ensure_ascii=False)
    print(len(out), "comments")


if __name__ == "__main__":
    c, rest = sys.argv[1], sys.argv[2:]
    {"screens": lambda: cmd_screens(rest), "families": cmd_families,
     "catalogue": lambda: cmd_catalogue(rest[0]), "comments": lambda: cmd_comments(rest[0])}[c]()
