// The web's list commands (tauri/main/src/commands/lists.ts, imports pointed
// back at the shared modules). Import and export go through the browser:
// export downloads the list's JSON (the desktop app's format), import reads a
// file the user picks; the rest is the shared code as it is.
// List-level commands: CRUD, JSON import/export, and tab navigation. The
// "sheet → mutation" orchestration that used to live inline in App.
import { api, type List } from './api';
import { askConfirm, askText, openSheet } from '@shared/lib/sheet';
import { useLayoutStore } from '@shared/stores/layoutStore';
import { useListStore } from '@shared/stores/listStore';
import { useProjectStore } from '@shared/stores/projectStore';
import { useTaskStore } from '@shared/stores/taskStore';

/** The browser's file picker; null when the user cancels it. */
export function pickFile(accept?: string): Promise<File | null> {
  return new Promise((resolve) => {
    const input = document.createElement('input');
    input.type = 'file';
    if (accept) input.accept = accept;
    input.onchange = () => resolve(input.files?.[0] ?? null);
    input.addEventListener('cancel', () => resolve(null));
    input.click();
  });
}

/** The picker for several files; empty when cancelled. */
export function pickFiles(): Promise<File[]> {
  return new Promise((resolve) => {
    const input = document.createElement('input');
    input.type = 'file';
    input.multiple = true;
    input.onchange = () => resolve([...(input.files ?? [])]);
    input.addEventListener('cancel', () => resolve([]));
    input.click();
  });
}

export async function importList(name?: string): Promise<void> {
  const projectId = useProjectStore.getState().activeId;
  if (!projectId) return;
  const file = await pickFile('application/json,.json');
  if (!file) return;
  let data: unknown;
  try {
    data = JSON.parse(await file.text());
  } catch {
    await askConfirm({
      glyph: 'error',
      headline: 'Not a list export',
      detail: `“${file.name}” isn't JSON.`,
      accept: 'OK',
    });
    return;
  }
  await api.importList(projectId, data, name);
  await Promise.all([
    useListStore.getState().load(),
    useTaskStore.getState().load(),
  ]);
}

export async function exportList(list: List): Promise<void> {
  const link = document.createElement('a');
  link.href = api.exportListUrl(list.id);
  link.download = `${list.name}.json`;
  link.click();
}

export async function createList(): Promise<void> {
  const projectId = useProjectStore.getState().activeId;
  if (!projectId) return;
  const res = await openSheet({
    glyph: 'list_alt_add',
    headline: 'New list',
    ask: { dice: 'list', placeholder: 'Name this list' },
    accept: 'Create',
    instead: { id: 'import', label: 'Import JSON…' },
  });
  if (res.action === 'instead') return importList();
  if (res.action !== 'accept') return;
  const list = await useListStore.getState().create(projectId, res.text);
  useLayoutStore.getState().selectList(list.id);
}

export async function renameList(list: List): Promise<void> {
  const name = await askText({
    glyph: 'edit',
    headline: 'Rename list',
    value: list.name,
    dice: 'list',
    accept: 'Rename',
  });
  if (!name || name === list.name) return;
  await useListStore.getState().rename(list.id, name);
}

export async function deleteList(list: List): Promise<void> {
  const taskCount = useTaskStore
    .getState()
    .tasks.filter((t) => t.list_id === list.id).length;
  const ok = await askConfirm({
    glyph: 'delete',
    tone: 'danger',
    headline: `Delete “${list.name}”?`,
    detail:
      taskCount > 0
        ? `Its ${taskCount} task${taskCount === 1 ? '' : 's'} go with it. No undo.`
        : 'The list is empty. No undo.',
    accept: 'Delete list',
  });
  if (!ok) return;
  await useListStore.getState().remove(list.id);
  // Deleting the active/split list leaves the panes pointing at a list that no
  // longer exists; fall back to a sibling so the view never lands on "No lists yet".
  const inProject = useListStore
    .getState()
    .listsInProject(useProjectStore.getState().activeId);
  useLayoutStore
    .getState()
    .reconcileLists(
      new Set(inProject.map((l) => l.id)),
      inProject[0]?.id ?? null
    );
}

/** Move focus to the next/previous tab in the active project. */
export function cycleTab(dir: -1 | 1): void {
  const listsInProject = useListStore
    .getState()
    .listsInProject(useProjectStore.getState().activeId);
  const n = listsInProject.length;
  if (n === 0) return;
  const focusedListId = useLayoutStore.getState().focusedListId();
  const idx = listsInProject.findIndex((l) => l.id === focusedListId);
  const next = listsInProject[((idx < 0 ? 0 : idx) + dir + n) % n];
  if (next) useLayoutStore.getState().selectList(next.id);
}
