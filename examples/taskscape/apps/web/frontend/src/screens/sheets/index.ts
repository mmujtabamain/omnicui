// The sheets screen (contract app.sheets): the one modal question card over a soft scrim, and the
// "Moved to Trash" toast.
//
// Other screens open a sheet with openSheet(spec). It resolves once with the answer:
//   { action: "accept", text?, id? } | { action: "dismiss" } | { action: "instead", id }
// Build the spec from the app's own action: { kind: "newList", projectId }, { kind: "newProject" },
// { kind: "renameList", list }, { kind: "renameProject", project }, { kind: "deleteList", list },
// { kind: "deleteProject", project }, { kind: "deleteConfirm", task }.
// Accept performs the write (create, rename, delete, Move to Trash) before resolving.
// Only one sheet is up at a time: a new one dismisses the one already open.
//
// Mount <SheetHost /> once, above the main window (the app shell does this).
export { SheetHost } from "./SheetHost";
export { openSheet, useSheets, type SheetResult, type SheetSpec } from "./store";
