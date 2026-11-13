// An in-memory stand-in for the reference's Rust commands, so its frontend runs unchanged
// in a browser. Behaviour follows the commands' contracts in main/src/api.ts; it is a
// viewer, not a port: nothing here is meant to be reused.
import catalog from './catalog.json';
import { fixtures } from './fixtures.js';

const clone = (v) => JSON.parse(JSON.stringify(v));
let seq = 0;
const newId = (p) => `${p}-new-${++seq}`;

export function createBackend(name, now) {
  const sets = fixtures(now());
  const f = clone(sets[name] ?? sets.standard);
  const db = {
    projects: f.projects,
    lists: f.lists,
    tasks: f.tasks.map((t) => ({ notes: null, done: false, deleted_at: null, ...t })),
    notes: f.notes ?? [],
    attachments: f.attachments ?? [],
    settings: { ...(f.settings ?? {}) },
    hotkeys: catalog.map((h) => ({ ...h })),
  };
  const log = [];
  const stamp = (o) => Object.assign(o, { updated_at: now() });

  const withChildren = (t) => ({
    ...t,
    attachments: db.attachments.filter((a) => a.task_id === t.id),
    note_items: db.notes.filter((n) => n.task_id === t.id).sort((a, b) => a.sort_order - b.sort_order),
  });
  const live = () => db.tasks.filter((t) => t.deleted_at == null);
  const subtree = (id) => {
    const out = [id];
    for (const t of db.tasks) if (t.parent_id === id) out.push(...subtree(t.id));
    return out;
  };
  const task = (id) => db.tasks.find((t) => t.id === id);
  const nextOrder = (rows) => (rows.length ? Math.max(...rows.map((r) => r.sort_order)) + 1 : 0);
  const setting = (k) => db.settings[k] ?? null;

  const commands = {
    list_projects: () => db.projects,
    default_project: () => db.projects[0],
    create_project: ({ name }) => {
      const p = { id: newId('project'), name, created_at: now(), updated_at: now() };
      db.projects.push(p);
      return p;
    },
    rename_project: ({ id, name }) => void stamp(Object.assign(db.projects.find((p) => p.id === id), { name })),
    delete_project: ({ id }) => {
      const lists = db.lists.filter((l) => l.project_id === id).map((l) => l.id);
      db.projects = db.projects.filter((p) => p.id !== id);
      db.lists = db.lists.filter((l) => l.project_id !== id);
      db.tasks = db.tasks.filter((t) => !lists.includes(t.list_id));
    },

    list_lists: () => [...db.lists].sort((a, b) => a.sort_order - b.sort_order),
    create_list: ({ projectId, name }) => {
      const l = {
        id: newId('list'),
        project_id: projectId,
        name,
        sort_order: nextOrder(db.lists.filter((x) => x.project_id === projectId)),
        created_at: now(),
        updated_at: now(),
      };
      db.lists.push(l);
      return l;
    },
    rename_list: ({ id, name }) => void stamp(Object.assign(db.lists.find((l) => l.id === id), { name })),
    delete_list: ({ id }) => {
      db.lists = db.lists.filter((l) => l.id !== id);
      db.tasks = db.tasks.filter((t) => t.list_id !== id);
    },
    reorder_list: ({ id, sortOrder }) => void stamp(Object.assign(db.lists.find((l) => l.id === id), { sort_order: sortOrder })),

    list_tasks: ({ listId }) => live().filter((t) => t.list_id === listId).map(withChildren),
    all_tasks: () => live().map(withChildren),
    create_task: ({ listId, title, notes, parentId }) => {
      const t = {
        id: newId('task'),
        list_id: listId,
        parent_id: parentId ?? null,
        title,
        notes: notes ?? null,
        done: false,
        sort_order: nextOrder(live().filter((x) => x.list_id === listId && x.parent_id === (parentId ?? null))),
        created_at: now(),
        updated_at: now(),
        deleted_at: null,
      };
      db.tasks.push(t);
      return withChildren(t);
    },
    update_task: ({ id, ...patch }) => {
      const t = task(id);
      for (const k of ['title', 'notes', 'done']) if (patch[k] !== undefined && patch[k] !== null) t[k] = patch[k];
      return withChildren(stamp(t));
    },
    delete_task: ({ id }) => commands.delete_tasks({ ids: [id] }),
    delete_tasks: ({ ids }) => {
      const all = [...new Set(ids.flatMap(subtree))];
      const at = now();
      for (const i of all) task(i).deleted_at = at;
      return all;
    },
    restore_tasks: ({ ids }) => ids.forEach((i) => (task(i).deleted_at = null)),
    purge_tasks: ({ ids }) => {
      const all = new Set(ids.flatMap(subtree));
      db.tasks = db.tasks.filter((t) => !all.has(t.id));
    },
    list_trashed: () =>
      db.tasks.filter((t) => t.deleted_at != null).sort((a, b) => b.deleted_at - a.deleted_at).map(withChildren),
    empty_trash: () => void (db.tasks = live()),
    set_tasks_done: ({ ids, done }) => ids.forEach((i) => stamp(Object.assign(task(i), { done }))),
    move_task: ({ id, parentId, listId, sortOrder }) => {
      const t = task(id);
      t.parent_id = parentId;
      if (listId) for (const i of subtree(id)) task(i).list_id = listId;
      if (sortOrder != null) t.sort_order = sortOrder;
      return withChildren(stamp(t));
    },
    reorder_task: ({ id, sortOrder }) => withChildren(stamp(Object.assign(task(id), { sort_order: sortOrder }))),
    copy_tasks: ({ ids }) => ids.map((i) => `- [${task(i).done ? 'x' : ' '}] ${task(i).title}`).join('\n'),
    export_list: () => null,
    import_list: ({ projectId, name }) => commands.create_list({ projectId, name: name ?? 'Imported list' }),

    list_notes: ({ taskId }) => db.notes.filter((n) => n.task_id === taskId),
    create_note: ({ taskId, content }) => {
      const n = {
        id: newId('note'),
        task_id: taskId,
        content,
        sort_order: nextOrder(db.notes.filter((x) => x.task_id === taskId)),
        created_at: now(),
        updated_at: now(),
      };
      db.notes.push(n);
      return n;
    },
    update_note: ({ id, content }) => void stamp(Object.assign(db.notes.find((n) => n.id === id), { content })),
    delete_note: ({ id }) => void (db.notes = db.notes.filter((n) => n.id !== id)),

    set_active_list: ({ id }) => void (db.settings.last_active_list = id),
    set_active_project: ({ id }) => void (db.settings.last_active_project = id),
    get_setting: ({ key }) => setting(key),
    set_setting: ({ key, value }) => void (db.settings[key] = value),

    list_hotkeys: () => db.hotkeys,
    set_hotkey: ({ id, accel }) => {
      const clash = accel && db.hotkeys.find((h) => h.id !== id && h.accel === accel);
      if (clash) throw `Already used by “${clash.label}”`;
      db.hotkeys.find((h) => h.id === id).accel = accel;
    },
    reset_hotkey: ({ id }) => {
      const h = db.hotkeys.find((x) => x.id === id);
      h.accel = h.default;
    },

    list_attachments: ({ taskId }) => db.attachments.filter((a) => a.task_id === taskId),
    add_reference: ({ taskId, name, location }) => {
      const a = { id: newId('att'), task_id: taskId, name, link_type: 'reference', location, created_at: now() };
      db.attachments.push(a);
      return a;
    },
    add_copy: ({ taskId, sourcePath, name }) => {
      const a = {
        id: newId('att'),
        task_id: taskId,
        name: name ?? sourcePath.split('/').pop(),
        link_type: 'copy',
        location: sourcePath,
        created_at: now(),
      };
      db.attachments.push(a);
      return a;
    },
    delete_attachment: ({ id }) => void (db.attachments = db.attachments.filter((a) => a.id !== id)),
    rename_attachment: ({ id, name }) => Object.assign(db.attachments.find((a) => a.id === id), { name }),
    // Screen capture needs macOS; off it the reference's own command fails.
    attach_screenshot: () => {
      throw 'Screen capture is only available on macOS';
    },
    capture_screenshot: () => {
      throw 'Screen capture is only available on macOS';
    },
    data_paths: () => ({
      data_dir: '/Users/you/.taskscape',
      screenshots_dir: '/Users/you/.taskscape/screenshots',
    }),
    is_low_power_mode: () => false,

    // tray
    get_dark: () => matchMedia('(prefers-color-scheme: dark)').matches,
    capture_target: () => {
      const listId = setting('last_active_list') ?? db.lists[0]?.id;
      const list = db.lists.find((l) => l.id === listId) ?? db.lists[0];
      const project = db.projects.find((p) => p.id === list?.project_id) ?? db.projects[0];
      return { project: project?.name ?? '', list: list?.name ?? '' };
    },
    submit_capture: ({ title, notes }) => {
      const listId = setting('last_active_list') ?? db.lists[0]?.id;
      return commands.create_task({ listId, title, notes });
    },
  };

  // Window and shell commands only change the native window; the viewer records them.
  const recorded = new Set([
    'present_window', 'open_settings', 'reveal_main', 'boot_done', 'set_window_theme',
    'open_attachment', 'reveal_attachment', 'open_path', 'hide_mini', 'open_main',
    'set_bar_height', 'capture_and_attach',
  ]);

  return {
    db,
    log,
    handle(cmd, args) {
      log.push({ cmd, args });
      if (commands[cmd]) return commands[cmd](args ?? {});
      if (recorded.has(cmd)) return null;
      if (cmd.startsWith('plugin:')) return plugin(cmd, args);
      console.warn('[viewer] unhandled command', cmd, args);
      return null;
    },
  };
}

function plugin(cmd, args) {
  const [, rest] = cmd.split(':');
  const [name, method] = rest.split('|');
  if (name === 'app' && method === 'version') return '0.1.0';
  if (name === 'app' && method === 'name') return 'Taskscape';
  if (name === 'window' || name === 'webview') {
    if (method.startsWith('is_')) return method === 'is_visible' || method === 'is_focused';
    if (method === 'scale_factor') return 1;
    if (method === 'inner_size' || method === 'outer_size')
      return { width: window.innerWidth, height: window.innerHeight };
    if (method === 'inner_position' || method === 'outer_position') return { x: 0, y: 0 };
    if (method === 'theme') return matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
    return null;
  }
  if (name === 'dialog') return null; // the user cancelled
  if (name === 'path' && method === 'join') return args.paths.join('/');
  if (name === 'path') return '/Users/you';
  return null;
}
