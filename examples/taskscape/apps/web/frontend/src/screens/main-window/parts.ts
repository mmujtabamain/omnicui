// data-part names for the main window (contract app.mainwindow). The child screens tag their own elements
// (app.appbar.*, app.tasklist.*, ...): RENAMES gives each of those a main-window name as well. The parts no
// child tags are claimed here, by position, accessible name or text.
//
// data-part is a list of names, and the test matches any one of them. This tagger adds its own names
// (app.mainwindow.*) next to the child screen's and the context-menu tagger's, and removes only its own names
// that are no longer due. Every write is conditional, so the tagger is idempotent and the MutationObserver that
// runs it settles after one extra pass.
const P = "app.mainwindow.";

/** Child-screen data-part -> the main window's part. */
const RENAMES: Record<string, string> = {
  "app.appbar.projectSwitcher": "projectSwitcher",
  "app.appbar.projectNew": "projectNew",
  "app.appbar.projectRow": "projectMenuHome",
  "app.appbar.projectRowSpring": "projectMenuSpring",
  "app.appbar.listTabs": "listTabs",
  "app.appbar.newList": "newList",
  "app.appbar.splitView": "splitView",
  "app.appbar.swapPanes": "swapPanes",
  "app.appbar.previewToggle": "previewToggle",
  "app.appbar.trashButton": "trash",
  "app.appbar.paletteButton": "commandButton",
  "app.appbar.settingsButton": "settingsButton",
  "app.appbar.accountButton": "accountButton",
  "app.appbar.accountMenu": "accountMenu",
  "app.appbar.accountEmail": "accountEmail",
  "app.appbar.accountSettings": "accountSettings",
  "app.appbar.accountSignOut": "accountSignOut",
  "app.tasklist.search": "searchField",
  "app.tasklist.searchOptions": "searchOptions",
  "app.tasklist.searchMenu": "scopeMenu",
  "app.tasklist.scopeWholeProject": "scopeWholeProject",
  "app.tasklist.newTask": "newTask",
  "app.tasklist.groupHeading": "groupHeading",
  "app.tasklist.sortFilter": "sortFilter",
  "app.tasklist.doneCount": "doneCount",
  "app.tasklist.percent": "footerPercent",
  "app.tasklist.selectionCount": "selectionCount",
  "app.tasklist.bulkDone": "bulkMarkDone",
  "app.tasklist.more": "rowMore",
  "app.tasklist.rowMenu": "rowMenu",
  "app.tasklist.renameField": "rowTitleEdit",
  "app.tasklist.badgeGroceries": "subtaskCount",
  "app.tasklist.disclosure": "rowGroceriesExpand",
  "app.tasklist.checkBike": "checkBike",
  "app.tasklist.checkCut": "rowKeysSelect",
  "app.tasklist.attachmentBadge": "attachmentBadge",
  "app.multiinspector.clearSelection": "clearSelection",
  "app.multiinspector.rowOpen": "openSelection",
  "app.taskinspector.title": "previewTitle",
  "app.trash.title": "trashHeading",
  "app.trash.closeControl": "trashClose",
  "app.filterpanel.panelTitle": "filterHeading",
  "app.sheets.card": "dialog",
  "app.sheets.headline": "dialogHeading",
  "app.sheets.nameField": "dialogName",
  "app.sheets.cancel": "dialogCancel",
  "app.sheets.detail": "dialogDetail",
  "app.sheets.toast": "toast",
  "app.sheets.toastUndo": "toastUndo",
  "app.palette.panel": "commandPalette",
  "app.settingsgeneral.heading": "settingsHeading",
};

/** Task titles the tests name: the title button whose accessible name starts with the title. */
const TITLES: [string, string][] = [
  ["rowBike", "Book bike service"],
  ["rowKeysTitle", "Cut a spare key"],
  ["rowGroceries", "Buy groceries for the week"],
  ["rowHeroRewrite", "Rewrite the hero section"],
  ["subtaskOatMilk", "Oat milk"],
];

/** Round checks: role checkbox, checked while the row is picked (aria-selected on the check). */
const CHECKS = ["rowSelect", "checkBike", "rowKeysSelect"];

/** Labels of a row's controls that are not its title. */
const NOT_TITLE = /^(Select|Deselect|Mark done|Mark not done|More)$|expand|collapse|disclosure/i;

/** The number of lists in the active project; the split button is disabled below two. */
let listCount = 0;

export function setListCount(count: number): void {
  listCount = count;
}

/** The names an element carries, in order. */
const namesOf = (el: Element): string[] => (el.getAttribute("data-part") ?? "").split(/\s+/).filter(Boolean);

/** Writes the element's names only when the list differs from the one it carries. */
function setNames(el: Element, next: string[]): void {
  const value = next.join(" ");
  if (value === namesOf(el).join(" ")) return;
  if (value) el.setAttribute("data-part", value);
  else el.removeAttribute("data-part");
}

/**
 * Every main-window name that a child name gives: the values of RENAMES, prefixed. rowMore is left out: the
 * structure claims it too (the first row's More, which may lack the child's name), so dropping it here and
 * claiming it again would write on every pass.
 */
const RENAMED = new Set(Object.values(RENAMES).map((part) => P + part).filter((name) => name !== P + "rowMore"));

/** Adds the main-window name of each child name an element carries; drops its own rename once the child no longer gives it. */
function renameAll(): void {
  for (const el of Array.from(document.querySelectorAll("[data-part]"))) {
    const names = namesOf(el);
    const due = names.flatMap((name) => (RENAMES[name] !== undefined ? [P + RENAMES[name]] : []));
    const next = names.filter((name) => !RENAMED.has(name) || due.includes(name));
    for (const name of due) if (!next.includes(name)) next.push(name);
    setNames(el, next);
  }
}

const text = (el: Element | null | undefined) => (el?.textContent ?? "").replace(/\s+/g, " ").trim();

/** Gives `part` to the first element of `found` and takes it from every other element; the other names stay. */
function claim(part: string, found: (Element | null | undefined)[]): void {
  const full = P + part;
  const target = found.find((el): el is Element => !!el) ?? null;
  for (const el of Array.from(document.querySelectorAll(`[data-part~="${full}"]`))) {
    if (el !== target) setNames(el, namesOf(el).filter((name) => name !== full));
  }
  if (target && !namesOf(target).includes(full)) setNames(target, [...namesOf(target), full]);
}

const buttonsIn = (root: ParentNode | null | undefined) =>
  Array.from(root?.querySelectorAll<HTMLElement>('[role="button"][aria-label], button[aria-label]') ?? []);

/**
 * A title's own text: the title button also holds the note under it, so the part is the button's first
 * span (the title alone). It takes focus from code so a key step on the title reaches the row's keys.
 */
function titleSpan(button: HTMLElement | null | undefined): HTMLElement | null {
  const span = button?.querySelector<HTMLElement>(":scope > span") ?? null;
  if (span && !span.hasAttribute("tabindex")) span.setAttribute("tabindex", "-1");
  return span ?? button ?? null;
}

function tagStructure(): void {
  // Named task titles (rows the tests name, wherever they are in the list).
  const all = buttonsIn(document);
  for (const [part, title] of TITLES) {
    claim(part, [titleSpan(all.find((el) => (el.getAttribute("aria-label") ?? "").startsWith(title)))]);
  }

  // The active list's first row (the left pane): its container, title, round check and More button.
  const pane = document.querySelector(`[data-part~="${P}taskPane"]`);
  const rows = Array.from(pane?.querySelectorAll<HTMLElement>("[data-task-id]") ?? []);
  const firstRow = rows[0] ?? null;
  // The row the tests name is "Pick up prescription", wherever a search has moved it (or dropped it from the list).
  const namedRow = rows.find((row) => buttonsIn(row).some((el) => el.getAttribute("aria-label") === "Pick up prescription")) ?? null;
  claim("row", [namedRow]);
  claim("rowTitle", [titleSpan(buttonsIn(firstRow).find((el) => !NOT_TITLE.test(el.getAttribute("aria-label") ?? "")))]);
  claim("rowSelect", [firstRow?.querySelector('[data-slot="check"]')]);
  claim("rowMore", [firstRow?.querySelector('[aria-label="More"]')]);

  // Whole-project search: the match row that holds the query's title.
  const options = Array.from(document.querySelectorAll<HTMLElement>('[role="button"], [role="option"]'));
  claim("matchRow", [options.find((el) => text(el).includes("soil health"))]);

  // List tabs: the tabs in order, and their Delete buttons in the same order.
  const listTabs = document.querySelector(`[data-part~="${P}listTabs"]`);
  const tabs = Array.from(listTabs?.querySelectorAll<HTMLElement>('[role="tab"]') ?? []);
  const tab = (name: string) => tabs.find((el) => text(el).includes(name));
  const deletes = Array.from(listTabs?.querySelectorAll<HTMLElement>('[aria-label="Delete list"]') ?? []);
  claim("listTab", [tabs[0]]);
  claim("listTabReading", [tab("Reading")]);
  claim("listTabGarden", [tab("Garden")]);
  claim("listTabNew", [tab("Hardware")]);
  claim("deleteList", [deletes[0]]);
  const readingAt = tabs.findIndex((el) => text(el).includes("Reading"));
  claim("deleteListReading", [readingAt >= 0 ? deletes[readingAt] : undefined]);

  // The split button is disabled with fewer than two lists (F09-032); the title bar only checks for none.
  const split = document.querySelector<HTMLButtonElement>(`[data-part~="${P}splitView"]`);
  if (split) {
    const want = listCount < 2;
    if (want !== split.hasAttribute("disabled")) {
      if (want) split.setAttribute("disabled", "");
      else split.removeAttribute("disabled");
    }
  }

  // The command palette: its input and the Open Trash command.
  const palette = document.querySelector(`[data-part~="${P}commandPalette"]`);
  claim("commandInput", [palette?.querySelector('input, textarea, [role="combobox"], [contenteditable="true"]')]);
  const items = Array.from(palette?.querySelectorAll<HTMLElement>('[role="option"], [role="button"], [role="menuitem"]') ?? []);
  claim("paletteOpenTrash", [items.find((el) => text(el).includes("Open Trash"))]);

  // Sheets: the card's buttons, found by their label.
  const card = document.querySelector(`[data-part~="${P}dialog"]`);
  const sheetButtons = Array.from(card?.querySelectorAll<HTMLElement>('button, [role="button"]') ?? []);
  const named = (label: string) => sheetButtons.find((el) => text(el) === label) ?? sheetButtons.find((el) => text(el).includes(label));
  claim("dialogCreate", [named("Create")]);
  claim("dialogDelete", [named("Delete list")]);
  claim("dialogMoveToTrash", [named("Move to Trash")]);
}

/** The round checks are checkboxes: the check itself carries the role and its picked state. */
function tagChecks(): void {
  for (const part of CHECKS) {
    const el = document.querySelector(`[data-part~="${P + part}"]`);
    if (!el) continue;
    if (el.getAttribute("role") !== "checkbox") el.setAttribute("role", "checkbox");
    const checked = el.getAttribute("aria-selected") === "true" ? "true" : "false";
    if (el.getAttribute("aria-checked") !== checked) el.setAttribute("aria-checked", checked);
  }
}

/** Gives every main-window part its name. Safe to run after any change: it only writes what differs. */
export function tagParts(): void {
  renameAll();
  tagStructure();
  tagChecks();
}

/** Tags the page now and after every change to it, synchronously, so a read right after a step sees the parts. */
export function observeParts(): () => void {
  tagParts();
  const observer = new MutationObserver(() => tagParts());
  observer.observe(document.body, {
    subtree: true,
    childList: true,
    characterData: true,
    attributes: true,
    attributeFilter: ["data-part", "aria-label", "aria-selected", "role"],
  });
  return () => observer.disconnect();
}
