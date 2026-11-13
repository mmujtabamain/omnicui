"""Writes process/workflows/areas.json: each audit area's files for solver A (code) and
inputs for solver B (recorded states, or docs where an area has no screen)."""
import json, os, re, subprocess, sys

REF = os.environ["TASKSCAPE_REFERENCE"]
REFS = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../spec/references"))
M, C, T = "main/src", "common-ui/src", "tray/src"

AREAS = [
  (1, "Task list", "high", [f"{M}/components/TaskPane.tsx", f"{M}/components/TaskRow.tsx", f"{M}/components/paneContext.ts", f"{M}/components/SelectableText.tsx", f"{M}/lib/selection.ts", f"{M}/lib/paneSelection.ts", f"{M}/lib/scroll.ts"]),
  (2, "Title bar and navigation", "ordinary", [f"{M}/components/TitleBar.tsx", f"{M}/components/WindowControls.tsx", f"{M}/components/windowChrome.tsx", f"{M}/components/ProjectSwitcher.tsx", f"{M}/components/ListTabs.tsx", f"{M}/lib/nameSuggest.ts"]),
  (3, "Task inspector and notes", "high", [f"{M}/components/preview/TaskInspector.tsx", f"{M}/components/preview/PreviewPanel.tsx", f"{M}/components/preview/NoteCard.tsx", f"{M}/components/preview/noteEditing.ts", f"{M}/components/preview/RichTextEditorLazy.tsx", f"{M}/lib/mentions.ts", f"{C}/RichTextEditor.tsx"]),
  (4, "Draft and multi-select inspectors, attachments", "ordinary", [f"{M}/components/preview/DraftInspector.tsx", f"{M}/components/preview/MultiInspector.tsx", f"{M}/components/preview/AttachmentSection.tsx", f"{M}/components/AttachmentLightbox.tsx", f"{C}/fileKind.ts", f"{C}/attachmentMime.ts"]),
  (5, "Filter, sort and search", "ordinary", [f"{M}/components/preview/FilterPanel.tsx", f"{M}/stores/searchStore.ts", f"{M}/lib/searchFocus.ts", f"{M}/stores/visibility.ts", f"{M}/stores/viewStore.ts", f"{M}/stores/derive.ts", f"{M}/commands/view.ts"]),
  (6, "Command palette, menus, toast and sheets", "high", [f"{M}/components/CommandPalette.tsx", f"{M}/commands/palette.ts", f"{M}/components/ContextMenu.tsx", f"{M}/components/contextMenuContext.ts", f"{M}/components/Toast.tsx", f"{M}/stores/toastStore.ts", f"{M}/components/SheetHost.tsx", f"{M}/lib/sheet.ts", f"{M}/stores/sheetStore.ts", f"{C}/components/surfaces/Sheet.tsx", f"{C}/components/surfaces/Overlay.tsx", f"{C}/components/surfaces/Backdrop.tsx", f"{C}/overlay.ts"]),
  (7, "Tasks, Trash and undo", "high", [f"{M}/components/TrashPane.tsx", f"{M}/stores/trashStore.ts", f"{M}/stores/history.ts", f"{M}/stores/actions.ts", f"{M}/stores/taskStore.ts", f"{M}/commands/tasks.ts"]),
  (8, "Keys and modifiers", "high", [f"{M}/hooks/useAppKeyboard.ts", f"{M}/stores/altKeyStore.ts", f"{M}/stores/hotkeyStore.ts", f"{M}/hooks/useMenuEvents.ts", f"{M}/lib/platform.ts", f"{C}/hotkeys.ts", f"{C}/components/buttons/KeyAction.tsx", f"{C}/components/content/Keycap.tsx", f"{C}/components/content/HotkeyHint.tsx"]),
  (9, "Lists, projects and layout", "high", [f"{M}/commands/lists.ts", f"{M}/commands/projects.ts", f"{M}/stores/listStore.ts", f"{M}/stores/projectStore.ts", f"{M}/stores/layoutStore.ts", f"{M}/stores/uiStore.ts", f"{M}/stores/dragStore.ts", f"{M}/stores/bootstrap.ts", f"{M}/lib/boot.ts", f"{M}/App.tsx", f"{M}/panels.tsx"]),
  (10, "Settings window", "high", [f"{M}/windows/SettingsWindow.tsx", f"{M}/windows/settings/"]),
  (11, "Motion, theme and tokens", "ordinary", [f"{M}/motion/", f"{M}/lib/reducedMotion.ts", f"{M}/lib/theme.ts", f"{M}/lib/zLayers.ts", "common-ui/styles/tokens.css", f"{M}/index.css"]),
  (12, "Shared primitives", "ordinary", [f"{C}/components/", f"{C}/cn.ts", f"{C}/Icon.tsx", f"{C}/Spinner.tsx"]),
  (13, "Capture bar UI", "ordinary", [f"{T}/", ".llm/tray-app.md"]),
  (14, "Capture bar, native side", "ordinary", ["tray/src-tauri/src/"]),
  (15, "Main app, native side", "ordinary", ["main/src-tauri/src/"]),
  (16, "Data and storage", "high", ["common/src/storage.rs", "common/src/models.rs", "common/src/entities/", "common/src/migrations.rs", "common/src/names.rs", "common/schema.hcl"]),
  (17, "Shared services", "ordinary", ["common/src/hotkeys.rs", "common/src/server.rs", "common/src/settings.rs", "common/src/attachments.rs", "common/src/screenshot.rs", "common/src/paths.rs"]),
  (18, "Design intent", "ordinary", ["docs/main-app-design-review-2026-08-13.md"]),
  (19, "Command surface", "ordinary", [f"{M}/api.ts", f"{T}/api.ts", f"{C}/sanitizeHtml.ts", f"{M}/time.ts"]),
]
# Solver B reads docs where an area has no recorded screen.
DOCS_B = {
  14: [".llm/tray-app.md", ".llm/architecture.md"],
  15: [".llm/main-app.md", ".llm/architecture.md", ".llm/build-and-run.md"],
  16: [".llm/common-crate.md"],
  17: [".llm/common-crate.md", ".llm/architecture.md"],
  18: [".llm/main-app.md", ".llm/architecture.md", ".llm/conventions.md", ".llm/README.md"],
  19: [".llm/common-crate.md", ".llm/conventions.md"],
}
EXTRA_STATES = {12: ["main-default", "row-selected", "palette", "sheet-new-list", "settings-general", "settings-shortcuts", "filter-panel", "multi-select-shift"]}

states = {}
for d in sorted(os.listdir(REFS)):
    meta = json.load(open(os.path.join(REFS, d, "meta.json")))
    states[d] = meta
src = open(os.path.join(os.path.dirname(__file__), "../../tools/reference-viewer/states.mjs")).read()

out = []
for n, name, tier, files in AREAS:
    st = EXTRA_STATES.get(n) or [d for d, m in states.items() if n in m["areas"]]
    out.append({
        "n": n, "name": name, "tier": tier,
        "filesA": [os.path.join(REF, f) for f in files],
        "statesB": st,
        "docsB": [os.path.join(REF, f) for f in DOCS_B.get(n, [])],
    })
json.dump({"ref": REF, "refs": REFS, "areas": out}, open(os.path.join(os.path.dirname(__file__), "areas.json"), "w"), indent=1)
for a in out:
    print(a["n"], a["tier"], len(a["statesB"]), a["name"])
