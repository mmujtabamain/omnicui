// data-part names for the main window (contract app.mainwindow). Most of its parts belong to the child
// screens, which already tag their own elements (app.appbar.*, app.tasklist.*, ...): those are renamed
// here. The rest (the first row's title, list tabs, sheet buttons, palette items) have no part of their
// own, so they are found by accessible name or text after each change to the page.
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
  "app.tasklist.closeSplit": "splitClose",
  "app.tasklist.row": "row",
  "app.tasklist.more": "rowMore",
  "app.tasklist.check": "rowSelect",
  "app.tasklist.rowMenu": "rowMenu",
  "app.tasklist.renameField": "rowTitleEdit",
  "app.tasklist.badgeGroceries": "subtaskCount",
  "app.tasklist.disclosure": "rowGroceriesExpand",
  "app.tasklist.checkBike": "checkBike",
  "app.tasklist.checkCut": "rowKeysSelect",
  "app.tasklist.attachmentBadge": "attachmentBadge",
  "app.multiinspector.clearSelection": "clearSelection",
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
  "app.toasthost.notice": "toast",
  "app.toasthost.undo": "toastUndo",
  "app.palette.panel": "commandPalette",
  "app.settingsgeneral.heading": "settingsHeading",
};

/** The name each renamed element had from its child screen, and the name it was given here. */
const origin = new WeakMap<Element, { orig: string; applied: string }>();

function renameAll(): void {
  for (const el of Array.from(document.querySelectorAll("[data-part]"))) {
    const current = el.getAttribute("data-part") ?? "";
    const known = origin.get(el);
    // React writes a prop only when it changes: a name other than the one applied is the child's new name.
    const orig = known && known.applied === current ? known.orig : current;
    const part = RENAMES[orig];
    if (part === undefined) continue;
    const full = P + part;
    if (current !== full) el.setAttribute("data-part", full);
    origin.set(el, { orig, applied: full });
  }
}

const text = (el: Element | null | undefined) => (el?.textContent ?? "").replace(/\s+/g, " ").trim();

/** Gives `part` to the first element of `found`, and takes it from any other element. */
function claim(part: string, found: (Element | null | undefined)[]): void {
  const full = P + part;
  const target = found.find((el): el is Element => !!el);
  for (const el of Array.from(document.querySelectorAll(`[data-part="${full}"]`))) {
    if (el !== target) el.removeAttribute("data-part");
  }
  if (target && target.getAttribute("data-part") !== full) target.setAttribute("data-part", full);
}

const buttonsIn = (root: ParentNode | null | undefined) =>
  Array.from(root?.querySelectorAll<HTMLElement>('[role="button"][aria-label], button[aria-label]') ?? []);

/** Task titles are buttons named by their title (the row's aria-label). The first match in the page wins. */
const TITLES: [string, string][] = [
  ["rowBike", "Book bike service"],
  ["rowKeysTitle", "Cut a spare key"],
  ["rowGroceries", "Buy groceries for the week"],
  ["rowHeroRewrite", "Rewrite the hero section"],
  ["subtaskOatMilk", "Oat milk"],
];

/** Labels of a row's controls that are not its title. */
const NOT_TITLE = /^(Select|More)$|expand|collapse|disclosure/i;

function tagText(): void {
  const all = buttonsIn(document);
  for (const [part, title] of TITLES) {
    claim(part, [all.find((el) => (el.getAttribute("aria-label") ?? "").startsWith(title))]);
  }

  // The active list's first row: its title button is the row's first button that is not a control.
  const firstRow = document.querySelector(`[data-part="${P}row"]`);
  claim("rowTitle", [buttonsIn(firstRow).find((el) => !NOT_TITLE.test(el.getAttribute("aria-label") ?? ""))]);

  // Whole-project search: the match row that holds the query's title.
  const options = Array.from(document.querySelectorAll<HTMLElement>('[role="button"], [role="option"]'));
  claim("matchRow", [options.find((el) => text(el).includes("soil health"))]);

  const listTabs = document.querySelector(`[data-part="${P}listTabs"]`);
  const tabs = Array.from(listTabs?.querySelectorAll('[role="tab"]') ?? []);
  const tab = (name: string) => tabs.find((el) => text(el).startsWith(name));
  claim("listTab", [tabs[0]]);
  claim("listTabReading", [tab("Reading")]);
  claim("listTabGarden", [tab("Garden")]);
  claim("listTabNew", [tab("Hardware")]);
  const deletes = Array.from(listTabs?.querySelectorAll('[aria-label="Delete list"]') ?? []);
  claim("deleteList", [deletes[0]]);
  const reading = tab("Reading");
  const readingAt = reading ? tabs.indexOf(reading) : -1;
  claim("deleteListReading", [readingAt >= 0 ? deletes[readingAt] : undefined]);

  const palette = document.querySelector(`[data-part="${P}commandPalette"]`);
  claim("commandInput", [palette?.querySelector("input")]);
  const items = Array.from(palette?.querySelectorAll<HTMLElement>('[role="option"], [role="button"], [role="menuitem"]') ?? []);
  claim("paletteOpenTrash", [items.find((el) => text(el).includes("Open Trash"))]);

  // The sheet's accept button is named by what the sheet does; the sheet's own card is the scope.
  const card = document.querySelector(`[data-part="${P}dialog"]`);
  const sheetButtons = buttonsIn(card).length > 0 ? buttonsIn(card) : Array.from(card?.querySelectorAll<HTMLElement>("button") ?? []);
  const named = (label: string) => sheetButtons.find((el) => text(el).includes(label));
  claim("dialogCreate", [named("Create")]);
  claim("dialogDelete", [named("Delete list")]);
  claim("dialogMoveToTrash", [named("Move to Trash")]);

  // The Settings dialog: the open dialog that is not a sheet, with its heading and close control.
  const dialogs = Array.from(document.querySelectorAll<HTMLElement>('[role="dialog"]'));
  const settings = dialogs.find((d) => !d.contains(card) && !card?.contains(d) && text(d).includes("Settings"));
  claim("settingsDialog", [settings]);
  const close = settings ? Array.from(settings.querySelectorAll<HTMLElement>("[aria-label]")).find((el) => /close/i.test(el.getAttribute("aria-label") ?? "")) : undefined;
  claim("settingsClose", [close]);
}

/** Gives every main-window part its name. Safe to run after any change: it only writes what differs. */
export function tagParts(): void {
  renameAll();
  tagText();
}
