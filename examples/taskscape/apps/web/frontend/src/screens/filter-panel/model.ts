// Pure view logic of the Filter & Sort screen: the predicates a task has to pass, the tree of rows
// that comes out of them, and the counts the footer, the summary and the search count show.
import type { Id, Task, TaskList } from "../../api/types";

export type SortMode = "manual" | "created" | "updated" | "alpha" | "done";
export type Dir = "asc" | "desc";
export type ShowMode = "all" | "active" | "done";
export type Tri = "any" | "has" | "none";
export type DateField = "created" | "updated";
export type DateRange = "any" | "24h" | "7d" | "30d" | "custom";
export type Fields = "both" | "title" | "notes";
export type Scope = "list" | "project" | "all";

/** The view of one list: sort, show, refine and date. The panel writes it to every target list at once. */
export interface View {
  sort: SortMode;
  dir: Dir;
  show: ShowMode;
  notes: Tri;
  attachments: Tri;
  subtasks: Tri;
  dateField: DateField;
  dateRange: DateRange;
  days: number;
}

export const DEFAULT_VIEW: View = {
  sort: "manual",
  dir: "asc",
  show: "all",
  notes: "any",
  attachments: "any",
  subtasks: "any",
  dateField: "created",
  dateRange: "any",
  days: 7,
};

export const MIN_DAYS = 1;
export const MAX_DAYS = 365;

export function clampDays(n: number): number {
  if (!Number.isFinite(n)) return MIN_DAYS;
  return Math.min(MAX_DAYS, Math.max(MIN_DAYS, Math.trunc(n)));
}

export function viewOf(views: Record<number, View>, id: Id | null | undefined): View {
  if (id == null) return DEFAULT_VIEW;
  return views[id] ?? DEFAULT_VIEW;
}

/** Sort is non-default: a sort mode, or the descending direction (which reverses the stored order too). */
export function sortActive(v: View): boolean {
  return v.sort !== "manual" || v.dir !== "asc";
}

export function refineActive(v: View): boolean {
  return v.notes !== "any" || v.attachments !== "any" || v.subtasks !== "any";
}

/** Reset is enabled while sort, show, refine or the date range is non-default (review reply 12). */
export function viewActive(v: View): boolean {
  return sortActive(v) || v.show !== "all" || refineActive(v) || v.dateRange !== "any";
}

export interface Ctx {
  query: string;
  fields: Fields;
  showCompleted: boolean;
  now: number;
}

export interface RowModel {
  task: Task;
  depth: number;
  hasKids: boolean;
  expanded: boolean;
  kidsVisible: number;
  kidsDone: number;
}

export interface PaneModel {
  rows: RowModel[];
  visible: number;
  total: number;
  done: number;
  matches: number;
}

export interface PaneSection {
  key: string;
  heading: string | null;
  listName: string;
  rows: RowModel[];
}

export interface PaneView {
  index: number;
  list: TaskList | undefined;
  view: View;
  query: string;
  groupMode: boolean;
  sections: PaneSection[];
  visible: number;
  total: number;
  done: number;
  matches: number;
}

const DAY = 86_400_000;

function hasNotes(t: Task): boolean {
  return t.notes.trim() !== "" || t.note_items.length > 0;
}

/** q is lower case. Title & notes, Title only or Notes only, per the search options. */
export function matchesQuery(t: Task, q: string, fields: Fields): boolean {
  if (fields !== "notes" && t.title.toLowerCase().includes(q)) return true;
  if (fields === "title") return false;
  if (t.notes.toLowerCase().includes(q)) return true;
  return t.note_items.some((n) => n.content.toLowerCase().includes(q));
}

function inWindow(t: Task, v: View, now: number): boolean {
  const span =
    v.dateRange === "24h" ? DAY : v.dateRange === "7d" ? 7 * DAY : v.dateRange === "30d" ? 30 * DAY : v.days * DAY;
  const stamp = Date.parse(v.dateField === "created" ? t.created_at : t.updated_at);
  return Number.isFinite(stamp) && now - stamp <= span;
}

function predicates(v: View, ctx: Ctx, kids: Map<Id, Task[]>): Array<(t: Task) => boolean> {
  const q = ctx.query.trim().toLowerCase();
  const out: Array<(t: Task) => boolean> = [];
  if (q !== "") out.push((t) => matchesQuery(t, q, ctx.fields));
  out.push((t) => (v.show === "active" ? !t.done : v.show === "done" ? t.done : ctx.showCompleted || !t.done));
  if (v.notes !== "any") out.push((t) => (hasNotes(t) ? "has" : "none") === v.notes);
  if (v.attachments !== "any") out.push((t) => (t.attachments.length > 0 ? "has" : "none") === v.attachments);
  if (v.subtasks !== "any") out.push((t) => ((kids.get(t.id)?.length ?? 0) > 0 ? "has" : "none") === v.subtasks);
  if (v.dateRange !== "any") out.push((t) => inWindow(t, v, ctx.now));
  return out;
}

function compareTasks(sort: SortMode, a: Task, b: Task): number {
  switch (sort) {
    case "created":
      return a.created_at.localeCompare(b.created_at) || a.id - b.id;
    case "updated":
      return a.updated_at.localeCompare(b.updated_at) || a.id - b.id;
    case "alpha":
      return a.title.localeCompare(b.title, undefined, { sensitivity: "base" });
    case "done":
      return Number(a.done) - Number(b.done) || a.sort_order - b.sort_order;
    default:
      return a.sort_order - b.sort_order || a.id - b.id;
  }
}

/**
 * The rows of one list. A task is shown when every predicate keeps it: it passes the predicate itself
 * or one of its subtasks does (ancestors of passing tasks stay, F05-030, F05-039). The counts come from
 * the same pass. Search matches are counted on their own, ignoring the show and refine filters.
 */
export function computePane(tasks: Task[], v: View, ctx: Ctx, collapsed: Record<number, boolean>): PaneModel {
  const ids = new Set(tasks.map((t) => t.id));
  const kids = new Map<Id, Task[]>();
  for (const t of tasks) {
    if (t.parent != null && ids.has(t.parent)) kids.set(t.parent, [...(kids.get(t.parent) ?? []), t]);
  }
  const preds = predicates(v, ctx, kids);
  const memo = new Map<string, boolean>();
  const kept = (pi: number, t: Task): boolean => {
    const key = `${pi}:${t.id}`;
    const hit = memo.get(key);
    if (hit !== undefined) return hit;
    const ok = preds[pi](t) || (kids.get(t.id) ?? []).some((c) => kept(pi, c));
    memo.set(key, ok);
    return ok;
  };
  const visible = new Set<Id>();
  for (const t of tasks) {
    if (preds.every((_, pi) => kept(pi, t))) visible.add(t.id);
  }

  const searching = ctx.query.trim() !== "";
  const order = (xs: Task[]): Task[] => {
    const sorted = [...xs].sort((a, b) => compareTasks(v.sort, a, b));
    return v.dir === "desc" ? sorted.reverse() : sorted;
  };
  const rows: RowModel[] = [];
  const walk = (t: Task, depth: number) => {
    const all = kids.get(t.id) ?? [];
    const shown = order(all.filter((c) => visible.has(c.id)));
    const expanded = searching || !collapsed[t.id];
    rows.push({
      task: t,
      depth,
      hasKids: all.length > 0,
      expanded,
      kidsVisible: shown.length,
      kidsDone: shown.filter((c) => c.done).length,
    });
    if (expanded) for (const c of shown) walk(c, depth + 1);
  };
  const roots = order(tasks.filter((t) => visible.has(t.id) && (t.parent == null || !ids.has(t.parent))));
  for (const r of roots) walk(r, 0);

  const q = ctx.query.trim().toLowerCase();
  const matches = q === "" ? 0 : tasks.filter((t) => matchesQuery(t, q, ctx.fields)).length;
  return {
    rows,
    visible: visible.size,
    total: tasks.length,
    done: tasks.filter((t) => t.done).length,
    matches,
  };
}
