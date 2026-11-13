// The web app's component registry, for the omnicui target (harness.ts): one
// entry per contract id, each surface built from primitive props and the
// stores exactly as the web App builds it. Nothing here changes how a surface
// behaves; it only chooses which one is on screen and reports what the user
// did that writes nothing to the store, in the contracts' terms (`emit`); what
// is written, the server reports. It follows the Tauri app's registry
// (tauri/main/src/omnicui/registry.tsx), surface for surface, with the web's
// own window, title bar and Settings. The capture bar's contract is desktop
// only (a menu-bar agent), so it has no entry here.
import React from 'react';
import ReactDOM from 'react-dom/client';
import { buildCommands } from '@shared/commands/palette';
import {
  createProject,
  deleteProject,
  renameProject,
} from '@shared/commands/projects';
import { requestDeleteTask } from '@shared/commands/tasks';
import { CommandPalette } from '@shared/components/CommandPalette';
import { ContextMenuProvider } from '@shared/components/ContextMenu';
import { FilterPanel } from '@shared/components/preview/FilterPanel';
import { PreviewPanel } from '@shared/components/preview/PreviewPanel';
import { RichTextEditor } from '@shared/components/preview/RichTextEditorLazy';
import { SheetHost } from '@shared/components/SheetHost';
import { TaskPane } from '@shared/components/TaskPane';
import { Toast } from '@shared/components/Toast';
import { TrashPane } from '@shared/components/TrashPane';
import { useAppKeyboard } from '@shared/hooks/useAppKeyboard';
import { initReducedMotion } from '@shared/lib/reducedMotion';
import { initTheme } from '@shared/lib/theme';
import { deleteTasks } from '@shared/stores/actions';
import { initialLoad, startBootstrap } from '@shared/stores/bootstrap';
import { useHistoryStore } from '@shared/stores/history';
import { useLayoutStore } from '@shared/stores/layoutStore';
import { useListStore } from '@shared/stores/listStore';
import { useProjectStore } from '@shared/stores/projectStore';
import { useSelectionStore } from '@shared/stores/selectionStore';
import { useSheetStore } from '@shared/stores/sheetStore';
import { useTaskStore } from '@shared/stores/taskStore';
import { useToastStore } from '@shared/stores/toastStore';
import { useUiStore } from '@shared/stores/uiStore';
import { DEFAULT_VIEW, useViewStore } from '@shared/stores/viewStore';
import { api } from '../app/api';
import App from '../app/App';
import { createList, deleteList, renameList } from '../app/lists';
import { SettingsPanel } from '../app/SettingsDialog';
import { TitleBar } from '../app/TitleBar';
import { useWebUi } from '../app/webUi';

export type Props = Record<string, string | number | boolean>;

export interface Host {
  /** Report what the user did, as the harness names it. */
  emit: (name: string, fields?: Record<string, unknown>) => void;
  /** A fixture key → the store's id ('' and unknown keys pass through). */
  id: (key: string) => string;
  /** A store id → the fixture's key (or the id, for something made since). */
  key: (id: string) => string;
}

type Surface = (props: Props, host: Host) => Promise<React.ReactElement>;

const str = (props: Props, name: string) => String(props[name] ?? '');

/** The stores loaded the way App loads them, then pointed where the props say. */
async function loadApp(props: Props, host: Host) {
  if (props.showCompleted === false)
    await api.setSetting('show_completed', '0');
  startBootstrap();
  await initialLoad();
  const list = host.id(str(props, 'list'));
  if (list) useLayoutStore.setState({ activeListId: list, paneFocus: list });
  const split = host.id(str(props, 'split'));
  if (split) useLayoutStore.setState({ splitListId: split });
  if (typeof props.preview === 'boolean')
    useLayoutStore.getState().setPreviewOpen(props.preview);
  const multi = str(props, 'multi').split(',').filter(Boolean).map(host.id);
  const focused = useLayoutStore.getState().focusedListId();
  if (multi.length > 0 && focused)
    useSelectionStore.getState().setPane(focused, new Set(multi), multi[0]);
  const selected = host.id(str(props, 'selected'));
  if (selected) useSelectionStore.getState().focus(selected);
  const draft = host.id(str(props, 'draft'));
  if (draft) useUiStore.getState().setDraftListId(draft);
  watch(host);
}

/** UI changes that never reach `invoke` but are what the user asked for. */
function watch(host: Host) {
  useSelectionStore.subscribe((s, prev) => {
    if (s.selectedTaskId && s.selectedTaskId !== prev.selectedTaskId)
      host.emit('taskFocused', { id: host.key(s.selectedTaskId) });
  });
  useSheetStore.subscribe((s, prev) => {
    if (s.current && s.current !== prev.current)
      host.emit('confirmAsked', { headline: s.current.spec.headline });
  });
  useToastStore.subscribe((s, prev) => {
    if (s.toast && s.toast !== prev.toast)
      host.emit('toastShown', { message: s.toast.message });
  });
  useUiStore.subscribe((s, prev) => {
    if (s.paletteOpen && !prev.paletteOpen) host.emit('paletteOpened');
    if (s.trashOpen !== prev.trashOpen)
      host.emit('trashToggled', { open: s.trashOpen });
    if (s.filterOpen !== prev.filterOpen)
      host.emit('filterToggled', { open: s.filterOpen });
    if (s.draftListId && s.draftListId !== prev.draftListId)
      host.emit('draftOpened', { listId: host.key(s.draftListId) });
    if (s.composeFor && s.composeFor !== prev.composeFor)
      host.emit('subtaskComposerOpened', { parentId: host.key(s.composeFor) });
  });
  useWebUi.subscribe((s, prev) => {
    // Settings opens in the page (the desktop app asks Rust for a window).
    if (s.settingsOpen && !prev.settingsOpen) host.emit('settingsOpened');
  });
  useLayoutStore.subscribe((s, prev) => {
    if (s.previewOpen !== prev.previewOpen)
      host.emit('previewToggled', { open: s.previewOpen });
    if (s.splitListId !== prev.splitListId)
      host.emit('splitChanged', {
        listId: s.splitListId ? host.key(s.splitListId) : '',
      });
    if (s.activeListId !== prev.activeListId && s.activeListId)
      host.emit('paneShows', { listId: host.key(s.activeListId) });
    if (s.collapsed !== prev.collapsed)
      for (const id of new Set([...s.collapsed, ...prev.collapsed]))
        if (s.collapsed.has(id) !== prev.collapsed.has(id))
          host.emit('taskExpanded', {
            id: host.key(id),
            expanded: !s.collapsed.has(id),
          });
  });
  useViewStore.subscribe((s, prev) => {
    // A reset view is dropped from the store: it is the default again.
    const panes = new Set([
      ...Object.keys(s.byPane),
      ...Object.keys(prev.byPane),
    ]);
    for (const pane of panes) {
      const view = s.byPane[pane] ?? DEFAULT_VIEW;
      if (view !== (prev.byPane[pane] ?? DEFAULT_VIEW))
        host.emit('viewChanged', {
          listId: host.key(pane),
          sort: view.sort,
          dir: view.dir,
          filter: view.filter,
        });
    }
  });
}

function listOf(props: Props, host: Host) {
  const id =
    host.id(str(props, 'list')) || useLayoutStore.getState().activeListId;
  const list = useListStore.getState().lists.find((l) => l.id === id);
  if (!list) throw new Error(`no list ${str(props, 'list')} in the fixture`);
  return list;
}

/** The sheets the app opens, by the flow that opens each. */
const SHEETS: Record<
  string,
  (props: Props, host: Host) => Promise<unknown> | void
> = {
  newList: () => createList(),
  renameList: (p, h) => renameList(listOf(p, h)),
  deleteList: (p, h) => deleteList(listOf(p, h)),
  deleteTask: (p, h) => {
    const task = useTaskStore.getState().taskById[h.id(str(p, 'task'))];
    if (!task) throw new Error(`no task ${str(p, 'task')} in the fixture`);
    return requestDeleteTask(task);
  },
  newProject: () => createProject(),
  renameProject: (p, h) => renameProject(projectOf(p, h)),
  deleteProject: (p, h) => deleteProject(projectOf(p, h)),
};

function projectOf(props: Props, host: Host) {
  const id =
    host.id(str(props, 'project')) || useProjectStore.getState().activeId;
  const project = useProjectStore.getState().projects.find((p) => p.id === id);
  if (!project)
    throw new Error(`no project ${str(props, 'project')} in the fixture`);
  return project;
}

const SURFACES: Record<string, Surface> = {
  'taskscape.window': async (_props, host) => {
    // App runs its own bootstrap and first load, as at launch.
    queueMicrotask(() => watch(host));
    return <App />;
  },
  'taskscape.titlebar': async (props, host) => {
    await loadApp(props, host);
    return <TitleBar />;
  },
  'taskscape.pane': async (props, host) => {
    await loadApp(props, host);
    const list = listOf(props, host);
    return (
      <div className="bg-surface-1l dark:bg-surface-1d flex h-screen w-screen">
        <TaskPane list={list} isSplit={props.isSplit === true} />
      </div>
    );
  },
  'taskscape.inspector': async (props, host) => {
    await loadApp(props, host);
    return (
      <aside className="h-screen w-screen">
        <PreviewPanel />
      </aside>
    );
  },
  'taskscape.filter': async (props, host) => {
    await loadApp(props, host);
    return (
      <aside className="h-screen w-screen">
        <FilterPanel onClose={() => host.emit('closed')} />
      </aside>
    );
  },
  'taskscape.trash': async (props, host) => {
    await loadApp(props, host);
    return (
      <aside className="h-screen w-screen">
        <TrashPane onClose={() => host.emit('closed')} />
      </aside>
    );
  },
  'taskscape.palette': async (props, host) => {
    await loadApp(props, host);
    const getCommands = () =>
      buildCommands().map((c) => ({
        ...c,
        run: () => {
          host.emit('commandRun', { command: commandName(c.id, host) });
          c.run();
        },
      }));
    return (
      <div className="bg-surface-1l dark:bg-surface-1d h-screen w-screen">
        <CommandPalette
          getCommands={getCommands}
          onClose={() => host.emit('closed')}
        />
      </div>
    );
  },
  'taskscape.sheet': async (props, host) => {
    await loadApp(props, host);
    const open = SHEETS[str(props, 'kind')];
    if (!open) throw new Error(`no sheet ${str(props, 'kind')}`);
    // The sheet's answer is the store's: wrap `answer` to report it.
    const answer = useSheetStore.getState().answer;
    useSheetStore.setState({
      answer: (id, result) => {
        if (useSheetStore.getState().current?.id === id)
          host.emit('answered', {
            action: result.action,
            text: result.action === 'accept' ? result.text : '',
          });
        answer(id, result);
      },
    });
    void open(props, host);
    return (
      <div className="bg-surface-1l dark:bg-surface-1d h-screen w-screen">
        <SheetHost />
      </div>
    );
  },
  'taskscape.toast': async (props, host) => {
    await loadApp(props, host);
    const task = host.id(str(props, 'trashed'));
    if (task) {
      // commands/tasks.ts: delete, then the undo toast.
      await deleteTasks([task]);
      useToastStore
        .getState()
        .show(
          'Moved to Trash',
          'Undo',
          () => void useHistoryStore.getState().undo()
        );
    }
    return (
      <div className="bg-surface-1l dark:bg-surface-1d h-screen w-screen">
        <Toast />
      </div>
    );
  },
  // Settings on its own, filling the viewport as the dialog's box fills its
  // 640×520; closing it is the dialog's window action.
  'taskscape.settings': async (_props, host) => (
    <div className="h-screen w-screen">
      <SettingsPanel onClose={() => host.emit('windowAction', { action: 'close' })} />
    </div>
  ),
};

/** Each surface's props when a test leaves them out (the contracts' defaults). */
const DEFAULTS: Record<string, Props> = {
  'taskscape.sheet': {
    kind: 'deleteTask',
    list: 'launch',
    task: 'brief',
    project: 'atlas',
  },
  'taskscape.toast': { trashed: 'banners' },
  'taskscape.titlebar': { list: 'launch' },
  'taskscape.pane': { list: 'launch' },
  'taskscape.filter': { list: 'launch' },
};

/** A palette command's id in the fixture's names (`go-list-<key>`). */
function commandName(id: string, host: Host) {
  const m = /^(go-list-|go-proj-)(.+)$/.exec(id);
  return m ? m[1] + host.key(m[2]) : id;
}

/** The window's keyboard map (App installs it), live under every surface but
 *  Settings, which holds the keyboard while it is open. */
function MainWindowKeys({ children }: { children: React.ReactNode }) {
  useAppKeyboard();
  return <>{children}</>;
}

const OWN_KEYBOARD = new Set(['taskscape.window', 'taskscape.settings']);

/** The note editor is a React.lazy chunk, and React reveals a suspended lazy
 *  component up to 300 ms after it resolves: a note would appear well after the
 *  surface looks settled. Rendering the lazy editor once, off screen, resolves
 *  it for good (React keeps the resolved module on the lazy object), so every
 *  editor after it renders at once, as in a running app after its first. */
async function warmNoteEditor() {
  const el = document.createElement('div');
  el.style.cssText = 'position:fixed;left:-10000px;top:0;width:200px';
  document.body.append(el);
  const warm = ReactDOM.createRoot(el);
  warm.render(<RichTextEditor attachments={[]} />);
  const until = performance.now() + 3000;
  while (!el.querySelector('[contenteditable]') && performance.now() < until)
    await new Promise((r) => setTimeout(r, 10));
  warm.unmount();
  el.remove();
}

let root: ReactDOM.Root | null = null;

/** Mount one surface into `el`; resolves once its data has loaded. */
export async function mount(
  component: string,
  props: Props,
  el: HTMLElement,
  host: Host
): Promise<void> {
  const surface = SURFACES[component];
  if (!surface) throw new Error(`no component ${component}`);
  props = { ...DEFAULTS[component], ...props };
  await Promise.all([initTheme(), initReducedMotion(), warmNoteEditor()]);
  const element = await surface(props, host);
  root?.unmount();
  root = ReactDOM.createRoot(el);
  root.render(
    <React.StrictMode>
      <ContextMenuProvider>
        {OWN_KEYBOARD.has(component) ? (
          element
        ) : (
          <MainWindowKeys>{element}</MainWindowKeys>
        )}
      </ContextMenuProvider>
    </React.StrictMode>
  );
}

export const COMPONENTS = Object.keys(SURFACES);
