// Dragging tasks, as in the reference: a row drags itself, or the whole pick when it is picked. It drops on another
// row (before it, after it, or into it as a subtask, by where the pointer is on the row), on a pane's empty space
// (the end of that list), or on a list tab (TitleBar). The ids ride on the drag as "1,2,3" under TASK_MIME, which
// is also what the tabs read.
import type { DragEvent } from "react";
import type { Id, Task } from "../../api/types";

export const TASK_MIME = "application/x-taskscape-task";
export type DropZone = "before" | "after" | "nest";
export interface MoveTo {
  list: Id;
  parent: Id | null;
  sort_order: number;
}

export function startTaskDrag(e: DragEvent, ids: Id[]) {
  e.dataTransfer.setData(TASK_MIME, ids.join(","));
  e.dataTransfer.effectAllowed = "move";
}

/** A task drag is over this element (the ids themselves are only readable on drop). */
export const isTaskDrag = (e: DragEvent) => e.dataTransfer.types.includes(TASK_MIME);

export function droppedIds(e: DragEvent): Id[] {
  return e.dataTransfer
    .getData(TASK_MIME)
    .split(",")
    .map(Number)
    .filter((id) => Number.isInteger(id) && id > 0);
}

/** The top and bottom 30% of a row drop before and after it; the middle nests into it. */
export function zoneOf(e: DragEvent): DropZone {
  const r = e.currentTarget.getBoundingClientRect();
  const y = (e.clientY - r.top) / r.height;
  return y < 0.3 ? "before" : y > 0.7 ? "after" : "nest";
}

const bySort = (a: Task, b: Task) => a.sort_order - b.sort_order || a.id - b.id;

/** The task and everything under it. */
function subtree(tasks: Task[], id: Id): Set<Id> {
  const out = new Set<Id>([id]);
  let grew = true;
  while (grew) {
    grew = false;
    for (const t of tasks) if (t.parent != null && out.has(t.parent) && !out.has(t.id)) { out.add(t.id); grew = true; }
  }
  return out;
}

/**
 * The moves a drop makes. `tasks` holds every task the drop can see (both lists in split view); `ids` are the
 * dragged tasks; `target` is the row dropped on (null: the empty space of `list`). Dragged tasks whose parent is
 * dragged too ride along with it. Empty when the drop would put a task inside itself.
 */
export function planDrop(tasks: Task[], ids: Id[], target: Task | null, zone: DropZone, list: Id): [Id, MoveTo][] {
  const dragged = new Set(ids);
  const roots = ids.filter((id) => {
    const t = tasks.find((x) => x.id === id);
    return t && (t.parent == null || !dragged.has(t.parent));
  });
  if (roots.length === 0) return [];
  if (target) {
    for (const id of roots) if (subtree(tasks, id).has(target.id)) return [];
  }

  const siblingsOf = (parent: Id | null, inList: Id) =>
    tasks.filter((t) => t.list === inList && t.parent === parent && !dragged.has(t.id)).sort(bySort);

  if (!target || zone === "nest") {
    const parent = target ? target.id : null;
    const inList = target ? target.list : list;
    const last = siblingsOf(parent, inList).at(-1);
    const base = (last?.sort_order ?? 0) + 1;
    return roots.map((id, i) => [id, { list: inList, parent, sort_order: base + i }]);
  }

  // Before or after a row: the row's siblings are renumbered with the dragged tasks in their new place.
  const siblings = siblingsOf(target.parent, target.list);
  const at = siblings.findIndex((t) => t.id === target.id) + (zone === "after" ? 1 : 0);
  const order: Id[] = [...siblings.slice(0, at).map((t) => t.id), ...roots, ...siblings.slice(at).map((t) => t.id)];
  const moves: [Id, MoveTo][] = [];
  order.forEach((id, i) => {
    const t = tasks.find((x) => x.id === id);
    const to = { list: target.list, parent: target.parent, sort_order: i + 1 };
    if (!t || dragged.has(id) || t.sort_order !== to.sort_order) moves.push([id, to]);
  });
  return moves;
}
