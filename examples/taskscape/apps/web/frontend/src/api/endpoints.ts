// One function per endpoint of API.md.
import { api } from "./client";
import type { Attachment, Binding, ExportedTask, Id, Me, Note, Project, Settings, Task, TaskList, TrashItem } from "./types";

const qs = (o: Record<string, string | number | undefined | null>) => {
  const p = Object.entries(o).filter(([, v]) => v !== undefined && v !== null && v !== "");
  return p.length ? "?" + new URLSearchParams(p.map(([k, v]) => [k, String(v)])).toString() : "";
};

export const auth = {
  me: () => api.get<Me>("auth/me"),
  signin: (email: string, password: string) => api.post<Me>("auth/signin", { email, password }),
  signup: (email: string, password: string, name?: string) => api.post<Me>("auth/signup", { email, password, name }),
  signout: () => api.post<null>("auth/signout"),
};
export const settings = {
  get: () => api.get<Settings>("settings"),
  update: (patch: Partial<Settings>) => api.patch<Settings>("settings", patch),
  shortcuts: () => api.get<Binding[]>("shortcuts"),
  /** accel "" unbinds, null resets to the default; 409 when the chord is taken. */
  bind: (command: string, accel: string | null) => api.put<Binding[]>("shortcuts", { command, accel }),
};
export const projects = {
  list: () => api.get<Project[]>("projects"),
  create: (name: string) => api.post<Project>("projects", { name }),
  update: (id: Id, patch: Partial<Pick<Project, "name" | "sort_order">>) => api.patch<Project>(`projects/${id}`, patch),
  remove: (id: Id) => api.del<null>(`projects/${id}`),
};
export const lists = {
  list: (project?: Id) => api.get<TaskList[]>(`lists${qs({ project })}`),
  create: (project: Id, name: string) => api.post<TaskList>("lists", { project, name }),
  update: (id: Id, patch: Partial<Pick<TaskList, "name" | "sort_order" | "project">>) => api.patch<TaskList>(`lists/${id}`, patch),
  remove: (id: Id) => api.del<null>(`lists/${id}`),
  import: (id: Id, tasks: ExportedTask[]) => api.post<Task[]>(`lists/${id}/import`, { tasks }),
  export: (id: Id) => api.get<{ name: string; tasks: ExportedTask[] }>(`lists/${id}/export`),
};
export const tasks = {
  list: (scope: { list?: Id; project?: Id; q?: string }) => api.get<Task[]>(`tasks${qs(scope)}`),
  create: (list: Id, title: string, extra: { notes?: string; parent?: Id | null } = {}) => api.post<Task>("tasks", { list, title, ...extra }),
  update: (id: Id, patch: Partial<Pick<Task, "title" | "notes" | "done">>) => api.patch<Task>(`tasks/${id}`, patch),
  move: (id: Id, to: { list?: Id; parent?: Id | null; sort_order?: number }) => api.post<Task>(`tasks/${id}/move`, to),
  reorder: (id: Id, sort_order: number) => api.post<Task>(`tasks/${id}/reorder`, { sort_order }),
  bulk: (ids: Id[], action: "done" | "undone" | "trash" | "move", list?: Id) => api.post<{ trashed?: Id[] }>("tasks/bulk", { ids, action, list }),
  trash: (id: Id) => api.del<{ trashed: Id[] }>(`tasks/${id}`),
  copy: (id: Id) => api.get<{ text: string }>(`tasks/${id}/copy`),
};
export const notes = {
  list: (task: Id) => api.get<Note[]>(`notes${qs({ task })}`),
  create: (task: Id, content: string) => api.post<Note>("notes", { task, content }),
  update: (id: Id, patch: Partial<Pick<Note, "content" | "sort_order">>) => api.patch<Note>(`notes/${id}`, patch),
  remove: (id: Id) => api.del<null>(`notes/${id}`),
};
export const attachments = {
  list: (task: Id) => api.get<Attachment[]>(`attachments${qs({ task })}`),
  upload: (task: Id, file: File, name?: string) => {
    const f = new FormData();
    f.append("task", String(task));
    f.append("file", file);
    if (name) f.append("name", name);
    return api.post<Attachment>("attachments", f);
  },
  link: (task: Id, url: string, name?: string) => api.post<Attachment>("attachments", { task, url, name }),
  rename: (id: Id, name: string) => api.patch<Attachment>(`attachments/${id}`, { name }),
  remove: (id: Id) => api.del<null>(`attachments/${id}`),
};
export const trash = {
  list: () => api.get<TrashItem[]>("trash"),
  restore: (ids: Id[]) => api.post<{ restored: Id[] }>("trash/restore", { ids }),
  purge: (ids: Id[]) => api.post<null>("trash/purge", { ids }),
  empty: () => api.post<null>("trash/empty"),
};
