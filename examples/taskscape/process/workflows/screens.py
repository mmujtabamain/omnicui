"""Writes process/workflows/screens.json: each screen or surface of the reference, its tier,
the code files for solver A and the recorded states both solvers use."""
import json, os

REF = os.environ["TASKSCAPE_REFERENCE"]
M, C, T, S = "main/src", "common-ui/src", "tray/src", "main/src/windows/settings"
SCREENS = [
  ("main-window", "Main window: layout, panes, split view, preview panel", "high",
   [f"{M}/App.tsx", f"{M}/panels.tsx", f"{M}/components/preview/PreviewPanel.tsx"],
   ["main-default", "main-dark", "split-view", "preview-hidden", "narrow-window", "no-lists"]),
  ("title-bar", "Title bar: window controls, project switcher, list tabs, toolbar buttons", "high",
   [f"{M}/components/TitleBar.tsx", f"{M}/components/WindowControls.tsx", f"{M}/components/windowChrome.tsx", f"{M}/components/ProjectSwitcher.tsx", f"{M}/components/ListTabs.tsx"],
   ["main-default", "cmd-held", "project-menu", "tab-hover", "list-switch-3", "no-lists", "narrow-window"]),
  ("task-list", "Task list pane: search bar, task rows, subtasks, footer stats, bulk bar", "high",
   [f"{M}/components/TaskPane.tsx", f"{M}/components/TaskRow.tsx"],
   ["main-default", "row-hover", "row-selected", "option-held-hover", "multi-select-shift", "rename-inline", "collapsed-all", "empty-list", "search-typed"]),
  ("task-inspector", "Task inspector: header, notes, subtasks, attachments", "high",
   [f"{M}/components/preview/TaskInspector.tsx", f"{M}/components/preview/NoteCard.tsx", f"{M}/components/preview/AttachmentSection.tsx", f"{C}/RichTextEditor.tsx"],
   ["row-selected", "inspector-notes", "note-editing", "row-with-subtasks-selected", "done-row-selected", "row-selected-dark"]),
  ("draft-inspector", "Draft inspector: a new task before it has content", "ordinary",
   [f"{M}/components/preview/DraftInspector.tsx"], ["new-task-draft", "new-task-typed"]),
  ("multi-inspector", "Multi-select inspector: several tasks selected", "ordinary",
   [f"{M}/components/preview/MultiInspector.tsx"], ["multi-select-shift", "multi-select-cmd-two"]),
  ("filter-panel", "Filter & Sort panel", "ordinary",
   [f"{M}/components/preview/FilterPanel.tsx"], ["filter-panel", "completed-hidden"]),
  ("command-palette", "Command palette", "high",
   [f"{M}/components/CommandPalette.tsx", f"{M}/commands/palette.ts"], ["palette", "palette-search", "palette-no-match"]),
  ("context-menus", "Context and dropdown menus: task row, several tasks, list tab, project", "high",
   [f"{M}/components/ContextMenu.tsx", f"{M}/components/contextMenuContext.ts", f"{C}/components/menus/MenuItem.tsx"],
   ["row-context-menu", "row-more-menu", "multi-context-menu", "tab-context-menu", "project-menu"]),
  ("sheets", "Sheets: new list, new project, rename, delete and other confirmations", "high",
   [f"{M}/components/SheetHost.tsx", f"{M}/lib/sheet.ts", f"{C}/components/surfaces/Sheet.tsx", f"{C}/components/surfaces/Backdrop.tsx"],
   ["sheet-new-list", "sheet-new-project", "sheet-rename-list", "sheet-delete-list", "delete-confirm"]),
  ("toast", "Toast: undoable action notice", "ordinary", [f"{M}/components/Toast.tsx"], ["toast-after-delete", "after-undo"]),
  ("trash", "Trash pane", "ordinary", [f"{M}/components/TrashPane.tsx"], ["trash", "trash-selected"]),
  ("attachment-lightbox", "Attachment lightbox", "ordinary", [f"{M}/components/AttachmentLightbox.tsx"], ["attachment-lightbox"]),
  ("settings-general", "Settings window: frame, section nav, search, General pane", "high",
   [f"{M}/windows/SettingsWindow.tsx", f"{S}/SettingsTitleBar.tsx", f"{S}/generalPane.tsx", f"{C}/components/settings/SettingRow.tsx", f"{C}/components/settings/SettingsGroup.tsx"],
   ["settings-general", "settings-dark", "settings-search"]),
  ("settings-shortcuts", "Settings: Shortcuts pane", "ordinary",
   [f"{S}/shortcutsPane.tsx", f"{S}/HotkeyControl.tsx", f"{S}/useHotkeyEditor.ts"],
   ["settings-shortcuts", "settings-shortcuts-scrolled", "settings-shortcut-recording"]),
  ("settings-capture", "Settings: Capture pane", "ordinary", [f"{S}/capturePane.tsx", f"{S}/PathButton.tsx"], ["settings-capture"]),
  ("settings-about", "Settings: About pane", "ordinary", [f"{S}/aboutPane.tsx", f"{S}/ResetEverything.tsx"], ["settings-about"]),
  ("capture-bar", "Capture bar (desktop only): the quick-add window", "ordinary",
   [f"{T}/App.tsx", f"{T}/components/TitleRow.tsx", f"{T}/components/NotesSection.tsx", f"{T}/components/Footer.tsx"],
   ["capture-bar", "capture-bar-typed", "capture-bar-notes", "capture-bar-dark"]),
]
REFS = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../spec/references"))
out = [{"id": i, "name": n, "tier": t, "files": [os.path.join(REF, f) for f in fs], "states": st} for i, n, t, fs, st in SCREENS]
for s in out:
    for f in s["files"]:
        assert os.path.exists(f), f
    for st in s["states"]:
        assert os.path.isdir(os.path.join(REFS, st)), st
json.dump({"refs": REFS, "screens": out}, open(os.path.join(os.path.dirname(__file__), "screens.json"), "w"), indent=1)
print(len(out), sum(s["tier"] == "high" for s in out))
