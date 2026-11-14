// data-part names for the context menus (contract app.contextmenus), in the main window. The child screens
// tag their own elements (app.appbar.*, app.tasklist.*, app.sheets.*, app.toasthost.*), and the main window's
// own tagger (parts.ts) adds app.mainwindow.* names to some of them. SOURCES gives every name a part can carry
// the contract's name. The parts with no name of their own (the More buttons, the task rows, the submenu,
// the project row that is last) are claimed by structure.
//
// data-part is a list of names, and the test matches any one of them. This tagger adds its own names
// (app.contextmenus.*) next to the other names an element carries, and removes only its own names that are no
// longer due. Every write is conditional, so the tagger is idempotent and the MutationObserver that runs it
// settles after one extra pass.
const P = "app.contextmenus.";

/** Names a part carries before this tagger renames it: the child screen's, and the main window's name for it. */
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

const buttonsIn = (root: ParentNode | null | undefined) =>
  Array.from(root?.querySelectorAll<HTMLElement>('[role="button"][aria-label], button[aria-label]') ?? []);

const text = (el: Element | null | undefined) => (el?.textContent ?? "").replace(/\s+/g, " ").trim();

/** The names an element carries, in order. */
const namesOf = (el: Element): string[] => (el.getAttribute("data-part") ?? "").split(/\s+/).filter(Boolean);

/** Writes the element's names only when the list differs from the one it carries. */
function setNames(el: Element, next: string[]): void {
  const value = next.join(" ");
  if (value === namesOf(el).join(" ")) return;
  if (value) el.setAttribute("data-part", value);
  else el.removeAttribute("data-part");
}

/** Every context-menu name that a source name gives: the values of SOURCES, prefixed. */
const RENAMED = new Set(Object.values(SOURCES).map((part) => P + part));

/** Gives `part` to `target` and takes it from every other element; the other names an element carries stay. */
function claim(part: string, target: Element | null | undefined): void {
  const full = P + part;
  for (const el of Array.from(document.querySelectorAll(`[data-part~="${full}"]`))) {
    if (el !== target) setNames(el, namesOf(el).filter((name) => name !== full));
  }
  if (target && !namesOf(target).includes(full)) setNames(target, [...namesOf(target), full]);
}

/** The row each named title was found in. A rename replaces the title button, but the row stays. */
const rowsByTitle = new Map<string, HTMLElement>();

/** The task row a named title belongs to: the row it was found in while it is still on screen, else the row now titled so. */
function rowOf(title: string): HTMLElement | null {
  const known = rowsByTitle.get(title);
  if (known?.isConnected) return known;
  const pane = document.querySelector('[data-part~="app.mainwindow.taskPane"]');
  const rows = Array.from(pane?.querySelectorAll<HTMLElement>("[data-task-id]") ?? []);
  const row = rows.find((r) => buttonsIn(r).some((el) => (el.getAttribute("aria-label") ?? "").startsWith(title))) ?? null;
  if (row) rowsByTitle.set(title, row);
  return row;
}

/**
 * The title of a row: the title button's first span (the title alone; the button also holds the note under it).
 * The title button is the row's button that is neither its check, its disclosure nor its More button.
 */
function titleOf(title: string): HTMLElement | null {
  const row = rowOf(title);
  if (!row) return null;
  const button = buttonsIn(row).find((el) => !el.matches('[data-slot="check"], [data-row-control]') && el.getAttribute("aria-label") !== "More") ?? null;
  return button?.querySelector<HTMLElement>(":scope > span") ?? button;
}

/** The More button of the row titled `title`. */
function moreOf(title: string): HTMLElement | null {
  return rowOf(title)?.querySelector<HTMLElement>('[aria-label="More"]') ?? null;
}

/** Adds the context-menu name of each source name an element carries; drops its own once the source no longer gives it. */
function renameAll(): void {
  for (const el of Array.from(document.querySelectorAll<HTMLElement>("[data-part]"))) {
    const names = namesOf(el);
    const due = names.flatMap((name) => (SOURCES[name] !== undefined ? [P + SOURCES[name]] : []));
    const next = names.filter((name) => !RENAMED.has(name) || due.includes(name));
    for (const name of due) if (!next.includes(name)) next.push(name);
    setNames(el, next);
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
  const menu = document.querySelector<HTMLElement>(`[data-part~="${P}projectMenu"]`);
  const items = Array.from(menu?.querySelectorAll<HTMLElement>('[role="menuitem"]') ?? []).filter(
    (el) => !namesOf(el).includes(P + "projectNew"),
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
