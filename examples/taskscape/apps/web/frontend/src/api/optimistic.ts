// Instant feedback for writes: the cached data changes the moment a write starts, as the server will change it,
// so the screen does not wait for the round trip. useWrite snapshots the caches first and puts them back if the
// write fails. A write with no entry here waits for the server as before.
import type { QueryClient } from "@tanstack/react-query";
import * as ep from "./endpoints";
import type { Id, Note, Project, Settings, Task, TaskList, TrashItem } from "./types";

/** A write's expected effect, given the write's own arguments. */
type Patch = (qc: QueryClient, args: never) => void;

const now = () => new Date().toISOString();

function editTasks(qc: QueryClient, edit: (tasks: Task[]) => Task[]) {
  qc.setQueriesData<Task[]>({ queryKey: ["tasks"] }, (tasks) => (tasks ? edit(tasks) : tasks));
}

/** The ids and every descendant of them, in any cached task list. */
function withDescendants(qc: QueryClient, ids: Id[]): Set<Id> {
  const all = new Set<Id>(ids);
  for (const [, tasks] of qc.getQueriesData<Task[]>({ queryKey: ["tasks"] })) {
    let grew = true;
    while (grew) {
      grew = false;
      for (const t of tasks ?? []) if (t.parent != null && all.has(t.parent) && !all.has(t.id)) { all.add(t.id); grew = true; }
    }
  }
  return all;
}

/** Done counts on the list tabs follow the tasks. */
function recount(qc: QueryClient) {
  const byList = new Map<Id, Task[]>();
  for (const [, tasks] of qc.getQueriesData<Task[]>({ queryKey: ["tasks"] })) {
    for (const t of tasks ?? []) {
      const list = byList.get(t.list) ?? [];
      if (!list.some((x) => x.id === t.id)) list.push(t);
      byList.set(t.list, list);
    }
  }
  qc.setQueriesData<TaskList[]>({ queryKey: ["lists"] }, (lists) =>
    lists?.map((l) => {
      const tasks = byList.get(l.id);
      return tasks ? { ...l, task_count: tasks.length, done_count: tasks.filter((t) => t.done).length } : l;
    }),
  );
}

function setDone(qc: QueryClient, ids: Id[], done: boolean) {
  const set = new Set(ids);
  editTasks(qc, (tasks) => tasks.map((t) => (set.has(t.id) ? { ...t, done, done_at: done ? now() : null } : t)));
  recount(qc);
}

function removeTasks(qc: QueryClient, ids: Id[]) {
  const gone = withDescendants(qc, ids);
  editTasks(qc, (tasks) => tasks.filter((t) => !gone.has(t.id)));
  recount(qc);
}

function moveTasks(qc: QueryClient, ids: Id[], list: Id) {
  const moved = withDescendants(qc, ids);
  editTasks(qc, (tasks) => tasks.map((t) => (moved.has(t.id) ? { ...t, list, parent: ids.includes(t.id) ? null : t.parent } : t)));
  recount(qc);
}

let tempId = -1;

const PATCHES = new Map<unknown, Patch>([
  [ep.tasks.update, (qc, [id, patch]: [Id, Partial<Task>]) => {
    editTasks(qc, (tasks) =>
      tasks.map((t) => (t.id === id ? { ...t, ...patch, ...("done" in patch ? { done_at: patch.done ? now() : null } : {}) } : t)),
    );
    if ("done" in patch) recount(qc);
  }],
  [ep.tasks.bulk, (qc, [ids, action, list]: [Id[], "done" | "undone" | "trash" | "move", Id?]) => {
    if (action === "done" || action === "undone") setDone(qc, ids, action === "done");
    else if (action === "trash") removeTasks(qc, ids);
    else if (list != null) moveTasks(qc, ids, list);
  }],
  [ep.tasks.trash, (qc, [id]: [Id]) => removeTasks(qc, [id])],
  [ep.tasks.move, (qc, [id, to]: [Id, { list?: Id; parent?: Id | null }]) => {
    if (to.list != null && to.parent === undefined) moveTasks(qc, [id], to.list);
    else editTasks(qc, (tasks) => tasks.map((t) => (t.id === id ? { ...t, ...(to.list != null ? { list: to.list } : {}), ...(to.parent !== undefined ? { parent: to.parent } : {}) } : t)));
    recount(qc);
  }],
  // A new task shows at once under a temporary id; the refetch after the write swaps in the real one.
  [ep.tasks.create, (qc, [list, title, extra = {}]: [Id, string, { notes?: string; parent?: Id | null }]) => {
    const at = now();
    const task: Task = {
      id: tempId--, list, parent: extra.parent ?? null, title, notes: extra.notes ?? "", done: false, done_at: null,
      sort_order: Number.MAX_SAFE_INTEGER, created_at: at, updated_at: at, attachments: [], note_items: [],
    };
    for (const [key, tasks] of qc.getQueriesData<Task[]>({ queryKey: ["tasks"] })) {
      const scope = (key[1] ?? {}) as { list?: Id; project?: Id; q?: string };
      if (!tasks || scope.q) continue;
      const inScope = scope.list === list || (scope.project != null && qc.getQueryData<TaskList[]>(["lists", scope.project])?.some((l) => l.id === list));
      if (inScope) qc.setQueryData<Task[]>(key, [...tasks, task]);
    }
    recount(qc);
  }],
  [ep.lists.update, (qc, [id, patch]: [Id, Partial<TaskList>]) => {
    qc.setQueriesData<TaskList[]>({ queryKey: ["lists"] }, (lists) => lists?.map((l) => (l.id === id ? { ...l, ...patch } : l)));
  }],
  [ep.lists.remove, (qc, [id]: [Id]) => {
    qc.setQueriesData<TaskList[]>({ queryKey: ["lists"] }, (lists) => lists?.filter((l) => l.id !== id));
    editTasks(qc, (tasks) => tasks.filter((t) => t.list !== id));
  }],
  [ep.projects.update, (qc, [id, patch]: [Id, Partial<Project>]) => {
    qc.setQueryData<Project[]>(["projects"], (projects) => projects?.map((p) => (p.id === id ? { ...p, ...patch } : p)));
  }],
  [ep.projects.remove, (qc, [id]: [Id]) => {
    qc.setQueryData<Project[]>(["projects"], (projects) => projects?.filter((p) => p.id !== id));
  }],
  [ep.settings.update, (qc, [patch]: [Partial<Settings>]) => {
    qc.setQueryData<Settings>(["settings"], (settings) => (settings ? { ...settings, ...patch } : settings));
  }],
  [ep.notes.update, (qc, [id, patch]: [Id, Partial<Note>]) => {
    qc.setQueriesData<Note[]>({ queryKey: ["notes"] }, (notes) => notes?.map((n) => (n.id === id ? { ...n, ...patch } : n)));
    editTasks(qc, (tasks) => tasks.map((t) => (t.note_items.some((n) => n.id === id) ? { ...t, note_items: t.note_items.map((n) => (n.id === id ? { ...n, ...patch } : n)) } : t)));
  }],
  [ep.notes.remove, (qc, [id]: [Id]) => {
    qc.setQueriesData<Note[]>({ queryKey: ["notes"] }, (notes) => notes?.filter((n) => n.id !== id));
    editTasks(qc, (tasks) => tasks.map((t) => (t.note_items.some((n) => n.id === id) ? { ...t, note_items: t.note_items.filter((n) => n.id !== id) } : t)));
  }],
  [ep.trash.restore, (qc, [ids]: [Id[]]) => {
    qc.setQueryData<TrashItem[]>(["trash"], (items) => items?.filter((i) => !ids.includes(i.id)));
  }],
  [ep.trash.purge, (qc, [ids]: [Id[]]) => {
    qc.setQueryData<TrashItem[]>(["trash"], (items) => items?.filter((i) => !ids.includes(i.id)));
  }],
  [ep.trash.empty, (qc) => qc.setQueryData<TrashItem[]>(["trash"], [])],
]);

/** The caches a write may touch, as they were, to put back if it fails. */
export type Snapshot = [readonly unknown[], unknown][];
const TOUCHED = ["lists", "tasks", "notes", "trash", "projects", "settings"];

/** Applies the write's expected effect to the caches and returns what they held before (null: no patch). */
export async function applyOptimistic(qc: QueryClient, fn: unknown, args: unknown[]): Promise<Snapshot | null> {
  const patch = PATCHES.get(fn);
  if (!patch) return null;
  // A refetch in flight would land on top of the patch with the old data.
  await Promise.all(TOUCHED.map((k) => qc.cancelQueries({ queryKey: [k] })));
  const snapshot: Snapshot = TOUCHED.flatMap((k) => qc.getQueriesData({ queryKey: [k] }));
  patch(qc, args as never);
  return snapshot;
}

export function restore(qc: QueryClient, snapshot: Snapshot) {
  for (const [key, data] of snapshot) qc.setQueryData(key, data);
}
