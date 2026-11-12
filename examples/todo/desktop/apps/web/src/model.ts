// The app's state: the tasks, the list's filter and the settings. Every change goes through the
// store, and each one the user made is reported to `onEvent`, by its contract name and fields
// (desktop/SPEC-APP.md, Events). Components read it with useStore().
import { useSyncExternalStore } from "react";
import seeds from "./seeds.json" with { type: "json" };

export type Task = { id: string; title: string; notes: string; done: boolean };
export type Filter = "all" | "active" | "done";
export type Theme = "system" | "light" | "dark";
export type Seed = "empty" | "three" | "many";
export type TodoEvent = { name: string; fields: Record<string, unknown> };

/** The fixture tasks of a seed: seeds.json, byte for byte the file every version of the app uses. */
export function seedTasks(name: Seed): Task[] {
  const all = seeds as Record<string, Task[]>;
  if (!(name in all)) throw new Error(`no seed "${name}"; the seeds are ${Object.keys(all).join(", ")}`);
  return all[name].map((t) => ({ ...t }));
}

export type State = { tasks: Task[]; filter: Filter; theme: Theme; showCompleted: boolean };

/** The tasks the list shows: the filter's, and under All only the active ones when completed tasks are hidden. */
export function visible(s: State): Task[] {
  switch (s.filter) {
    case "all": return s.showCompleted ? s.tasks : s.tasks.filter((t) => !t.done);
    case "active": return s.tasks.filter((t) => !t.done);
    case "done": return s.tasks.filter((t) => t.done);
  }
}

/** How many tasks are not done, whatever the filter. */
export const left = (s: State) => s.tasks.filter((t) => !t.done).length;

/** What the list says when it shows no task. */
export function emptyMessage(s: State): string {
  if (s.filter === "done") return "Nothing done yet";
  return s.tasks.length === 0 ? "No tasks yet" : "Nothing left to do";
}

export class TodoStore {
  private state: State;
  private highest: number;
  private listeners = new Set<() => void>();
  private onEvent: (e: TodoEvent) => void;

  constructor(tasks: Task[], options: Partial<Omit<State, "tasks">> = {}, onEvent: (e: TodoEvent) => void = () => {}) {
    this.onEvent = onEvent;
    this.state = { tasks, filter: options.filter ?? "all", theme: options.theme ?? "system", showCompleted: options.showCompleted ?? true };
    // The highest task number ever used, so that a deleted task's id is never given again.
    this.highest = Math.max(0, ...tasks.map((t) => Number(t.id.replace(/^t/, "")) || 0));
  }

  subscribe = (listener: () => void) => {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  };
  get = () => this.state;
  task = (id: string) => this.state.tasks.find((t) => t.id === id);

  /** Reports something the store doesn't hold itself (navigation), through the same channel. */
  report(name: string, fields: Record<string, unknown> = {}) {
    this.onEvent({ name, fields });
  }

  /** Adds an active task at the end, with the title trimmed; a blank title adds nothing. */
  add(title: string): Task | undefined {
    const trimmed = title.trim();
    if (!trimmed) return undefined;
    const task = { id: `t${++this.highest}`, title: trimmed, notes: "", done: false };
    this.set({ tasks: [...this.state.tasks, task] });
    this.report("added", { id: task.id, title: task.title });
    return task;
  }

  setDone(id: string, done: boolean) {
    if (this.update(id, (t) => (t.done === done ? t : { ...t, done }))) this.report("toggled", { id, done });
  }
  rename(id: string, title: string) {
    if (this.update(id, (t) => (t.title === title ? t : { ...t, title }))) this.report("renamed", { id, title });
  }
  editNotes(id: string, notes: string) {
    if (this.update(id, (t) => (t.notes === notes ? t : { ...t, notes }))) this.report("notesChanged", { id, notes });
  }
  delete(id: string) {
    if (!this.task(id)) return;
    this.set({ tasks: this.state.tasks.filter((t) => t.id !== id) });
    this.report("deleted", { id });
  }
  select(filter: Filter) {
    if (filter === this.state.filter) return;
    this.set({ filter });
    this.report("filterChanged", { filter });
  }
  chooseTheme(theme: Theme) {
    if (theme === this.state.theme) return;
    this.set({ theme });
    this.report("themeChanged", { theme });
  }
  showCompleted(show: boolean) {
    if (show === this.state.showCompleted) return;
    this.set({ showCompleted: show });
    this.report("showCompletedChanged", { show });
  }

  private update(id: string, change: (t: Task) => Task): boolean {
    const i = this.state.tasks.findIndex((t) => t.id === id);
    if (i < 0) return false;
    const next = change(this.state.tasks[i]);
    if (next === this.state.tasks[i]) return false;
    const tasks = [...this.state.tasks];
    tasks[i] = next;
    this.set({ tasks });
    return true;
  }

  private set(change: Partial<State>) {
    this.state = { ...this.state, ...change };
    this.listeners.forEach((l) => l());
  }
}

/** The store's state, re-rendering when it changes. */
export const useStore = (store: TodoStore) => useSyncExternalStore(store.subscribe, store.get);
