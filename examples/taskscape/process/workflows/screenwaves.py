"""Writes the screen build waves (args for screenbuild.js): python3 screenwaves.py <out-dir> <pg> <python>.
A screen comes after the screens it composes; contextmenus is a flow over the whole main window."""
import json, os, sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "../.."))
REF = {s["id"]: s for s in json.load(open(os.path.join(ROOT, "spec/inventory/screens.json")))["screens"]}

def screen(cid, ref, folder, component, **kw):
    r = REF.get(ref, {})
    return {"id": cid, "folder": folder, "component": component,
            **({"inventory": f"{ref}.json", "features": f"features-{ref}.txt", "notes": f"notes-{ref}.txt"} if r else {}),
            "refs": [st for st in r.get("states", []) if os.path.exists(os.path.join(ROOT, "spec/references", st, "shot.png"))], **kw}

WAVES = [
    [
        screen("signin", None, "sign-in", "SignIn", note="The placeholder SignIn.tsx in your folder is yours to replace; App.tsx renders <SignIn mode=… /> on /signin and /signup."),
        screen("sheets", "sheets", "sheets", "SheetHost", note="Other screens open sheets through what your folder exports (for example an openSheet() function or a hook with a store): export it from sheets/index.ts and document it at the top of that file."),
        screen("toasthost", "toast", "toast", "ToastHost", note="Other screens show a toast (with Undo) through what your folder exports: export it from toast/index.ts and document it at the top of that file."),
        screen("attachmentlightbox", "attachment-lightbox", "lightbox", "AttachmentLightbox"),
        screen("filterpanel", "filter-panel", "filter-panel", "FilterPanel"),
        screen("draftinspector", "draft-inspector", "draft-inspector", "DraftInspector"),
        screen("settingsgeneral", "settings-general", "settings", "SettingsDialog",
               own=["settings/SettingsDialog.tsx", "settings/GeneralPage.tsx", "settings/shared.tsx"],
               note="The Shortcuts and About pages are other builders' (settings/ShortcutsPage.tsx and settings/AboutPage.tsx, placeholders now): render them as they are, as the Shortcuts and About pages of your dialog. Shared parts of the dialog (search box, groups, rows) go in settings/shared.tsx for them to reuse."),
    ],
    [
        screen("trash", "trash", "trash", "TrashPane", uses=["sheets", "toast"]),
        screen("palette", "command-palette", "palette", "AppPalette"),
        screen("multiinspector", "multi-inspector", "multi-inspector", "MultiInspector", uses=["sheets", "toast"]),
        screen("appbar", "title-bar", "title-bar", "TitleBar", uses=["sheets", "toast", "settings", "palette", "trash"]),
        screen("tasklist", "task-list", "task-list", "TaskList", uses=["sheets", "toast", "filter-panel"]),
        screen("taskinspector", "task-inspector", "task-inspector", "TaskInspector", uses=["sheets", "toast", "lightbox"]),
        screen("settingsshortcuts", "settings-shortcuts", "settings", "ShortcutsPage", own=["settings/ShortcutsPage.tsx"], uses=["settings"]),
        screen("settingsabout", "settings-about", "settings", "AboutPage", own=["settings/AboutPage.tsx"], uses=["settings"]),
    ],
    [
        screen("mainwindow", "main-window", "main-window", "MainWindow",
               uses=["title-bar", "task-list", "task-inspector", "draft-inspector", "multi-inspector", "filter-panel", "trash", "palette", "sheets", "toast", "settings", "lightbox"],
               note="The placeholder MainWindow.tsx is yours to replace; App.tsx renders it for every signed-in route. You own the app-wide keyboard shortcuts (decision 18, the user's bindings from useShortcuts) and the layout of the panes."),
    ],
    [
        screen("contextmenus", "context-menus", "main-window", "MainWindow", own=[],
               note="This contract tests the menus across the whole main window: your harness mounts <MainWindow />. The menus live in the title bar and task list screens, which are not yours: if a test fails because of them, report exactly what is missing in needs."),
    ],
]

out, pg, py = sys.argv[1:4]
for i, w in enumerate(WAVES):
    path = os.path.join(out, f"screens-wave{i + 1}.json")
    json.dump({"root": ROOT, "pg": pg, "python": py, "screens": w}, open(path, "w"))
    print(path, len(w), " ".join(s["id"] for s in w), [len(s["refs"]) for s in w])
