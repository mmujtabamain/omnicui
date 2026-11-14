// The task list's data shaping: the tree of one list, the flat matches of a search, and the done stats.
import type { Id, Task } from "../../api/types";

export type Scope = "list" | "project" | "all";
export type Fields = "both" | "title" | "notes";

/** Where a row sits: its project and list, by id and name. */
export interface Where {
  projectId: Id;
  projectName: string;
  listId: Id;
  listName: string;
}

export interface RowModel extends Where {
  task: Task;
  /** 0 for a root task, 1 for a subtask (in a search every match is 0). */
  depth: number;
  /** Subtasks of this task, done or not. */
  kids: number;
  kidsDone: number;
  expanded: boolean;
}

export interface GroupModel {
  key: string;
  where: Where;
  rows: RowModel[];
}

export function cx(...parts: (string | false | null | undefined)[]): string {
  return parts.filter(Boolean).join(" ");
}

/** Notes may carry markup; the row shows the words only. */
export function plainNotes(notes: string | null | undefined): string {
  return (notes ?? "").replace(/<[^>]*>/g, "").trim();
}

export function byOrder(a: Task, b: Task): number {
  return a.sort_order - b.sort_order || a.id - b.id;
}

export function percentOf(done: number, total: number): number {
  return total > 0 ? Math.round((done / total) * 100) : 0;
}

/** A task matches when the query (trimmed, lower case) is in the fields in scope. */
export function matchesQuery(task: Task, q: string, fields: Fields): boolean {
  if (q === "") return true;
  const inTitle = task.title.toLowerCase().includes(q);
  const inNotes = plainNotes(task.notes).toLowerCase().includes(q);
  if (fields === "title") return inTitle;
  if (fields === "notes") return inNotes;
  return inTitle || inNotes;
}

/** Roots and children of one list. A task whose parent is not in the list is a root. */
function tree(tasks: Task[]): { roots: Task[]; kids: Map<Id, Task[]> } {
  const ids = new Set(tasks.map((t) => t.id));
  const kids = new Map<Id, Task[]>();
  const roots: Task[] = [];
  for (const t of [...tasks].sort(byOrder)) {
    if (t.parent != null && ids.has(t.parent)) {
      const siblings = kids.get(t.parent) ?? [];
      siblings.push(t);
      kids.set(t.parent, siblings);
    } else {
      roots.push(t);
    }
  }
  return { roots, kids };
}

/** The tree of one list as rows, in order; collapsed parents hide their subtasks. */
export function treeRows(tasks: Task[], where: Where, collapsed: ReadonlySet<Id>, showCompleted: boolean): RowModel[] {
  const { roots, kids } = tree(tasks);
  const out: RowModel[] = [];
  const walk = (task: Task, depth: number) => {
    if (!showCompleted && task.done) return;
    const children = kids.get(task.id) ?? [];
    const expanded = !collapsed.has(task.id);
    out.push({
      ...where,
      task,
      depth,
      kids: children.length,
      kidsDone: children.filter((c) => c.done).length,
      expanded,
    });
    if (expanded) for (const c of children) walk(c, depth + 1);
  };
  for (const r of roots) walk(r, 0);
  return out;
}

/** The matches of a search in one list, flattened in tree order. */
export function matchRows(tasks: Task[], where: Where, q: string, fields: Fields, showCompleted: boolean): RowModel[] {
  const { roots, kids } = tree(tasks);
  const out: RowModel[] = [];
  const walk = (task: Task) => {
    if ((showCompleted || !task.done) && matchesQuery(task, q, fields)) {
      out.push({ ...where, task, depth: 0, kids: 0, kidsDone: 0, expanded: false });
    }
    for (const c of kids.get(task.id) ?? []) walk(c);
  };
  for (const r of roots) walk(r);
  return out;
}

/** Every subtask below a task, at any depth. */
export function descendantIds(tasks: Task[], id: Id): Id[] {
  const out: Id[] = [];
  let frontier: Id[] = [id];
  while (frontier.length > 0) {
    const next: Id[] = [];
    for (const t of tasks) {
      if (t.parent != null && frontier.includes(t.parent)) {
        out.push(t.id);
        next.push(t.id);
      }
    }
    frontier = next;
  }
  return out;
}

/** The tasks that share a task's parent in its list, in order. */
export function siblingsOf(tasks: Task[], task: Task): Task[] {
  return tasks.filter((t) => t.list === task.list && t.parent === task.parent).sort(byOrder);
}
