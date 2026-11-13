// The web's `api.ts` (replaces tauri/main/src/api.ts). Same names and types,
// so every shared store and component calls it unchanged; each Tauri command
// becomes an HTTP request to server/ (the common crate's data_router and the
// server's own routes, all under /api). A failed request rejects with the
// server's message as a string, as a Tauri command rejects.
//
// What a browser does itself instead of a command: windows (Settings is drawn
// in the page), the system's file dialogs (uploads and downloads), opening a
// link or a file (a new tab), and Low Power Mode (not visible to a page).
// Screenshots aren't available in a browser at all.
import type { Accel, HotkeyBinding } from '@taskscape/common-ui/hotkeys';
import type { Attachment, LinkType } from '@taskscape/common-ui/types';
import { emitToOtherTabs } from '../platform/events';
import { request } from '../platform/http';
import { useWebUi } from './webUi';

export type { Attachment, HotkeyBinding, LinkType };

export interface Project {
  id: string;
  name: string;
  created_at: number;
  updated_at: number;
}

export interface List {
  id: string;
  project_id: string;
  name: string;
  sort_order: number;
  created_at: number;
  updated_at: number;
}

export interface Note {
  id: string;
  task_id: string;
  /** Sanitized rich-text HTML. */
  content: string;
  sort_order: number;
  created_at: number;
  updated_at: number;
}

export interface Task {
  id: string;
  list_id: string;
  parent_id: string | null;
  title: string;
  notes: string | null;
  done: boolean;
  sort_order: number;
  created_at: number;
  updated_at: number;
  deleted_at: number | null;
  attachments: Attachment[];
  note_items: Note[];
}

export interface DataPaths {
  data_dir: string;
  screenshots_dir: string;
}

export interface TaskPatch {
  title?: string;
  notes?: string;
  done?: boolean;
}

/** A write: when it lands, the app's other tabs reload (events.ts). */
async function write<T>(method: string, path: string, body?: unknown): Promise<T> {
  const out = await request<T>(method, path, body);
  refreshOtherTabs();
  return out;
}

let refreshTimer: ReturnType<typeof setTimeout> | undefined;
function refreshOtherTabs() {
  clearTimeout(refreshTimer);
  refreshTimer = setTimeout(() => emitToOtherTabs('refresh'), 150);
}

const get = <T>(path: string) => request<T>('GET', path);
const enc = encodeURIComponent;

// The desktop app queues note autosaves in order on its Rust side; here the
// queue is the page's, so edits typed quickly still land in the order typed.
let noteQueue: Promise<unknown> = Promise.resolve();

function openInNewTab(url: string) {
  window.open(url, '_blank', 'noopener');
}

const fileUrl = (a: Attachment) => `/api/attachments/${enc(a.id)}/file`;

const unavailable = <T,>(what: string): Promise<T> =>
  Promise.reject(`${what} isn't available in a browser`);

export const api = {
  // projects
  listProjects: () => get<Project[]>('/projects'),
  createProject: (name: string) => write<Project>('POST', '/projects', { name }),
  renameProject: (id: string, name: string) =>
    write<void>('PATCH', `/projects/${enc(id)}`, { name }),
  deleteProject: (id: string) => write<void>('DELETE', `/projects/${enc(id)}`),
  defaultProject: () => get<Project>('/projects/default'),

  // lists (data_router: GET/POST /lists)
  listLists: () => get<List[]>('/lists'),
  createList: (projectId: string, name: string) =>
    write<List>('POST', '/lists', { name, project_id: projectId }),
  renameList: (id: string, name: string) =>
    write<void>('PATCH', `/lists/${enc(id)}`, { name }),
  deleteList: (id: string) => write<void>('DELETE', `/lists/${enc(id)}`),
  reorderList: (id: string, sortOrder: number) =>
    write<void>('PATCH', `/lists/${enc(id)}`, { sort_order: sortOrder }),

  // tasks (data_router: GET /lists/{id}/tasks, GET/POST /tasks, PATCH /tasks/{id})
  listTasks: (listId: string) => get<Task[]>(`/lists/${enc(listId)}/tasks`),
  allTasks: () => get<Task[]>('/tasks'),
  createTask: (listId: string, title: string, notes?: string, parentId?: string) =>
    write<Task>('POST', '/tasks', {
      list_id: listId,
      title,
      notes: notes ?? null,
      parent_id: parentId ?? null,
    }),
  updateTask: (id: string, patch: TaskPatch) =>
    write<Task>('PATCH', `/tasks/${enc(id)}`, patch),
  deleteTask: (id: string) => write<string[]>('POST', '/tasks/trash', { ids: [id] }),
  moveTask: (id: string, parentId: string | null, listId: string | null, sortOrder?: number) =>
    write<Task>('POST', `/tasks/${enc(id)}/move`, {
      parent_id: parentId,
      list_id: listId,
      sort_order: sortOrder ?? null,
    }),
  reorderTask: (id: string, sortOrder: number) =>
    write<Task>('POST', `/tasks/${enc(id)}/reorder`, { sort_order: sortOrder }),
  deleteTasks: (ids: string[]) => write<string[]>('POST', '/tasks/trash', { ids }),
  restoreTasks: (ids: string[]) => write<void>('POST', '/trash/restore', { ids }),
  purgeTasks: (ids: string[]) => write<void>('POST', '/trash/purge', { ids }),
  listTrashed: () => get<Task[]>('/trash'),
  emptyTrash: () => write<void>('DELETE', '/trash'),
  setTasksDone: (ids: string[], done: boolean) =>
    write<void>('POST', '/tasks/done', { ids, done }),
  copyTasks: (ids: string[]) => request<string>('POST', '/tasks/copy', { ids }),

  // import / export: the document goes through the browser, not a path.
  /** The URL that downloads the list as the desktop app's JSON export. */
  exportListUrl: (listId: string) => `/api/lists/${enc(listId)}/export`,
  importList: (projectId: string, data: unknown, name?: string) =>
    write<List>('POST', `/projects/${enc(projectId)}/import`, { data, name: name ?? null }),

  // notes
  listNotes: (taskId: string) => get<Note[]>(`/tasks/${enc(taskId)}/notes`),
  createNote: (taskId: string, content: string) =>
    write<Note>('POST', `/tasks/${enc(taskId)}/notes`, { content }),
  updateNote: (id: string, content: string) => {
    const next = noteQueue.then(() => write<void>('PATCH', `/notes/${enc(id)}`, { content }));
    noteQueue = next.catch(() => {});
    return next;
  },
  deleteNote: (id: string) => write<void>('DELETE', `/notes/${enc(id)}`),

  // app state
  setActiveList: (id: string) =>
    write<void>('PUT', '/settings/last_active_list', { value: id }),
  setActiveProject: (id: string) =>
    write<void>('PUT', '/settings/last_active_project', { value: id }),
  getSetting: (key: string) => get<string | null>(`/settings/${enc(key)}`),
  setSetting: (key: string, value: string) =>
    write<void>('PUT', `/settings/${enc(key)}`, { value }),

  // shortcuts: the catalog with the web's defaults (server/src/hotkeys.rs)
  listHotkeys: () => get<HotkeyBinding[]>('/hotkeys'),
  setHotkey: (id: string, accel: Accel) =>
    write<void>('PUT', `/hotkeys/${enc(id)}`, { accel }),
  resetHotkey: (id: string) => write<void>('DELETE', `/hotkeys/${enc(id)}`),

  // windows: there are none. Settings is a dialog in the page.
  presentWindow: (_width: number, _height: number) => Promise.resolve(),
  openSettings: () => {
    useWebUi.getState().setSettingsOpen(true);
    return Promise.resolve();
  },
  revealMain: () => Promise.resolve(),
  bootDone: () => Promise.resolve(),
  setWindowTheme: (_dark: boolean) => Promise.resolve(),
  /** A page can't see Low Power Mode; the system's reduced-motion setting
   *  still applies through CSS. */
  isLowPowerMode: () => Promise.resolve(false),

  // attachments (data_router: GET/POST /tasks/{id}/attachments, by reference)
  listAttachments: (taskId: string) =>
    get<Attachment[]>(`/tasks/${enc(taskId)}/attachments`),
  addReference: (taskId: string, name: string, location: string) =>
    write<Attachment>('POST', `/tasks/${enc(taskId)}/attachments`, { name, location }),
  /** A file the user picked or dropped, copied into the data folder. */
  uploadAttachment: (taskId: string, file: File) =>
    write<Attachment>(
      'POST',
      `/tasks/${enc(taskId)}/attachments/upload?name=${enc(file.name)}`,
      file
    ),
  addCopy: (_taskId: string, _sourcePath: string, _name?: string) =>
    unavailable<Attachment>('Copying a file by its path'),
  attachScreenshot: (_taskId: string) => unavailable<Attachment | null>('Screen capture'),
  captureScreenshot: () => unavailable<string | null>('Screen capture'),
  deleteAttachment: (id: string) => write<void>('DELETE', `/attachments/${enc(id)}`),
  renameAttachment: (id: string, name: string) =>
    write<Attachment>('PATCH', `/attachments/${enc(id)}`, { name }),
  /** A link opens in a new tab; a stored file opens there too, as the
   *  browser shows it (or downloads it). */
  openAttachment: (a: Attachment) => {
    openInNewTab(a.link_type === 'copy' ? fileUrl(a) : a.location);
    return Promise.resolve();
  },
  /** "Reveal in Finder" has no browser twin: the file is downloaded. */
  revealAttachment: (a: Attachment) => {
    if (a.link_type === 'copy') {
      const link = document.createElement('a');
      link.href = fileUrl(a);
      link.download = a.name;
      link.click();
    }
    return Promise.resolve();
  },
  attachmentUrl: fileUrl,
  dataPaths: () => get<DataPaths>('/paths'),
  /** A page can't open a folder; Settings shows the path to copy instead. */
  openPath: (_path: string) => Promise.resolve(),
};
