// data-part names for the context menus (contract app.contextmenus), in the main window. The child screens
// tag their own elements (app.appbar.*, app.tasklist.*, app.sheets.*, app.toasthost.*), and the main window's
// own tagger (parts.ts) renames some of them to app.mainwindow.*. SOURCES gives every name a part can carry
// the contract's name. The parts with no name of their own (the More buttons, the task rows, the submenu,
// the project row that is last) are claimed by structure.
//
// parts.ts claims some elements by structure too (the first row's More, the first tab, the Reading tab, the
// sheet buttons "Create", "Delete list" and "Move to Trash"). An element it holds is not taken here: the two
// taggers would take turns on it for ever, so that part is left to the main window's name.
const P = "app.contextmenus.";

/** Names a part carries before this tagger renames it: the child screen's, and the main window's rename of it. */
const SOURCES: Record<string, string> = {
  // Project switcher and project dropdown
  "app.appbar.projectSwitcher": "projectSwitch",
  "app.mainwindow.projectSwitcher": "projectSwitch",
  "app.appbar.projectMenu": "projectMenu",
  "app.appbar.projectRow": "projectHome",
  "app.mainwindow.projectMenuHome": "projectHome",
  "app.appbar.projectRowSpring": "projectSpringLaunch",
  "app.mainwindow.projectMenuSpring": "projectSpringLaunch",
  "app.appbar.projectNew": "projectNew",
  "app.mainwindow.projectNew": "projectNew",
  "app.appbar.projectMore": "projectMore",
  "app.appbar.projectMoreSpring": "projectMoreSpring",
  "app.appbar.menuProjectRename": "menuProjectRename",
  "app.appbar.menuProjectDelete": "menuProjectDelete",

  // List tabs
  "app.appbar.listTab": "firstTab",
  "app.appbar.listTabReading": "tabReading",
  "app.appbar.listTabGarden": "tabGarden",
  "app.appbar.listTabCloseReading": "tabCloseReading",
  "app.appbar.tabMoreReading": "tabMoreReading",
  "app.appbar.tabMenu": "tabMenu",
  "app.appbar.tabMenuRename": "menuTabRename",
  "app.appbar.tabMenuSplit": "menuTabSplit",
  "app.appbar.tabMenuExport": "menuTabExport",
  "app.appbar.tabMenuDelete": "menuTabDelete",

  // Task rows: the menu, its items, the Move to list target, the title field and the subtask composer
  "app.tasklist.rowMenu": "rowMenu",
  "app.mainwindow.rowMenu": "rowMenu",
  "app.tasklist.menuCopy": "menuCopy",
  "app.tasklist.menuAddSubtask": "menuAddSubtask",
  "app.tasklist.menuRename": "menuRename",
  "app.tasklist.menuMoveToList": "menuMoveToList",
  "app.tasklist.menuDelete": "menuDelete",
  "app.tasklist.moveTargetReading": "moveReading",
  "app.tasklist.renameField": "titleInput",
  "app.mainwindow.rowTitleEdit": "titleInput",
  "app.tasklist.subtaskComposer": "subtaskInput",

  // Sheets and the toast they raise
  "app.sheets.headline": "sheetHeadline",
  "app.mainwindow.dialogHeading": "sheetHeadline",
  "app.sheets.detail": "sheetDetail",
  "app.mainwindow.dialogDetail": "sheetDetail",
  "app.sheets.nameField": "sheetInput",
  "app.mainwindow.dialogName": "sheetInput",
  "app.sheets.accept": "sheetAccept",
  "app.sheets.cancel": "sheetCancel",
  "app.mainwindow.dialogCancel": "sheetCancel",
  "app.sheets.toast": "toast",
  "app.mainwindow.toast": "toast",
  "app.sheets.toastUndo": "toastUndo",
  "app.mainwindow.toastUndo": "toastUndo",
  "app.sheets.toastClose": "toastDismiss",
  "app.toasthost.message": "toast",
  "app.toasthost.undo": "toastUndo",
  "app.toasthost.dismiss": "toastDismiss",
};

/** The names parts.ts gives by claiming an element. A claim does not take an element that carries one. */
const MAIN_CLAIMS = new Set([
  "app.mainwindow.rowMore",
  "app.mainwindow.row",
  "app.mainwindow.rowTitle",
  "app.mainwindow.rowSelect",
  "app.mainwindow.listTab",
  "app.mainwindow.listTabReading",
  "app.mainwindow.listTabGarden",
  "app.mainwindow.listTabNew",
  "app.mainwindow.deleteList",
  "app.mainwindow.deleteListReading",
  "app.mainwindow.dialogCreate",
  "app.mainwindow.dialogDelete",
  "app.mainwindow.dialogMoveToTrash",
  "app.mainwindow.rowBike",
  "app.mainwindow.rowKeysTitle",
  "app.mainwindow.rowGroceries",
  "app.mainwindow.rowHeroRewrite",
  "app.mainwindow.subtaskOatMilk",
  "app.mainwindow.matchRow",
  "app.mainwindow.commandInput",
  "app.mainwindow.paletteOpenTrash",
]);

const buttonsIn = (root: ParentNode | null | undefined) =>
  Array.from(root?.querySelectorAll<HTMLElement>('[role="button"][aria-label], button[aria-label]') ?? []);

const text = (el: Element | null | undefined) => (el?.textContent ?? "").replace(/\s+/g, " ").trim();

/** Gives `part` to `target` (or to nothing when it is absent) and takes it from every other element. */
function claim(part: string, target: Element | null | undefined): void {
  const full = P + part;
  if (target && MAIN_CLAIMS.has(target.getAttribute("data-part") ?? "")) return;
  for (const el of Array.from(document.querySelectorAll(`[data-part="${full}"]`))) {
    if (el !== target) el.removeAttribute("data-part");
  }
  if (target && target.getAttribute("data-part") !== full) target.setAttribute("data-part", full);
}

/** The row each named title was found in. A rename replaces the title button, but the row stays. */
const rowsByTitle = new Map<string, HTMLElement>();

/** The task row a named title belongs to: the row it was found in while it is still on screen, else the row now titled so. */
function rowOf(title: string): HTMLElement | null {
  const known = rowsByTitle.get(title);
  if (known?.isConnected) return known;
  const pane = document.querySelector('[data-part="app.mainwindow.taskPane"]');
  const rows = Array.from(pane?.querySelectorAll<HTMLElement>("[data-task-id]") ?? []);
  const row = rows.find((r) => buttonsIn(r).some((el) => el.getAttribute("aria-label") === title)) ?? null;
  if (row) rowsByTitle.set(title, row);
  return row;
}

/** The title button of a row: the row's button that is neither its check, its disclosure nor its More button. */
function titleOf(title: string): HTMLElement | null {
  const row = rowOf(title);
  if (!row) return null;
  return buttonsIn(row).find((el) => !el.matches('[data-slot="check"], [data-row-control]') && el.getAttribute("aria-label") !== "More") ?? null;
}

/** The More button of the row titled `title`. */
function moreOf(title: string): HTMLElement | null {
  return rowOf(title)?.querySelector<HTMLElement>('[aria-label="More"]') ?? null;
}

/** Renames every element whose name is a source name. */
function renameAll(): void {
  for (const el of Array.from(document.querySelectorAll<HTMLElement>("[data-part]"))) {
    const name = el.getAttribute("data-part") ?? "";
    const part = SOURCES[name];
    if (part !== undefined) el.setAttribute("data-part", P + part);
  }
}

/** The parts that no name marks: found by their rows, their submenu, the sheet and the project list. */
function claimStructure(): void {
  claim("morePrescription", moreOf("Pick up prescription"));
  claim("moreBikeService", moreOf("Book bike service"));
  claim("moreCutKey", moreOf("Cut a spare key"));
  claim("moreGroceries", moreOf("Buy groceries for the week"));
  claim("rowPrescription", titleOf("Pick up prescription"));
  claim("rowBikeService", titleOf("Book bike service"));
  claim("rowEggs", titleOf("Buy eggs"));
  claim("subtaskTitle", titleOf("Renew passport"));

  // The Move to list submenu: its panel, and its Garden item (the Reading item carries its own name).
  const submenu = document.querySelector<HTMLElement>('[data-slot="submenuPanel"]');
  claim("moveSubmenu", submenu);
  const garden = Array.from(submenu?.querySelectorAll<HTMLElement>('[role="menuitem"]') ?? []).find((el) => text(el) === "Garden");
  claim("moveGarden", garden);

  // The project dropdown's last project row, when there are three or more (the first two carry their names).
  const menu = document.querySelector<HTMLElement>(`[data-part="${P}projectMenu"]`);
  const items = Array.from(menu?.querySelectorAll<HTMLElement>('[role="menuitem"]') ?? []).filter(
    (el) => el.getAttribute("data-part") !== P + "projectNew",
  );
  claim("projectLast", items.length >= 3 ? items[items.length - 1] : null);
}

/** Gives every context-menu part its name. Safe to run after any change: it only writes what differs. */
export function tagContextMenuParts(): void {
  renameAll();
  claimStructure();
}

/** Tags the page now and after every change to it. Returns the function that stops it. */
export function observeContextMenuParts(): () => void {
  tagContextMenuParts();
  const observer = new MutationObserver(() => tagContextMenuParts());
  observer.observe(document.body, {
    subtree: true,
    childList: true,
    characterData: true,
    attributes: true,
    attributeFilter: ["data-part", "aria-label", "aria-selected", "role"],
  });
  return () => observer.disconnect();
}
