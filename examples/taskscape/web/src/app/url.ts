// The address bar is the window's state, as a web app has it:
//
//   /projects/<project>              a project (one with no lists yet)
//   /lists/<list>                    the list in the (left) pane
//   /lists/<list>/tasks/<task>       …and the task in the details drawer
//
// Any of them can be opened, bookmarked or shared, and reloads where it was.
// Moving to another list or project is a new history entry, so the browser's
// Back and Forward go between them; focusing another task in the same list
// only replaces the entry (walking the rows with the arrows doesn't fill the
// history). The server answers every such path with the app (static_files.rs).
import { selectProject } from '@shared/commands/projects';
import { useLayoutStore } from '@shared/stores/layoutStore';
import { useListStore } from '@shared/stores/listStore';
import { useProjectStore } from '@shared/stores/projectStore';
import { useSelectionStore } from '@shared/stores/selectionStore';
import { useTaskStore } from '@shared/stores/taskStore';

interface Place {
  project: string | null;
  list: string | null;
  task: string | null;
}

export function parse(pathname: string): Place {
  const seg = pathname.split('/').filter(Boolean).map(decodeURIComponent);
  if (seg[0] === 'lists' && seg[1])
    return { project: null, list: seg[1], task: seg[2] === 'tasks' ? (seg[3] ?? null) : null };
  if (seg[0] === 'projects' && seg[1]) return { project: seg[1], list: null, task: null };
  return { project: null, list: null, task: null };
}

/** Where the window is now. */
function current(): Place {
  const project = useProjectStore.getState().activeId;
  const task = useSelectionStore.getState().selectedTaskId;
  const owner = task ? useTaskStore.getState().taskById[task] : undefined;
  const list = owner?.list_id ?? useLayoutStore.getState().activeListId;
  return { project, list, task: owner ? task : null };
}

export function pathOf(p: Place): string {
  const e = encodeURIComponent;
  if (p.list) return `/lists/${e(p.list)}${p.task ? `/tasks/${e(p.task)}` : ''}`;
  if (p.project) return `/projects/${e(p.project)}`;
  return '/';
}

/** Show `place`, as far as it still exists: an unknown list or task is
 *  ignored, and the window stays where it was. */
export function go(place: Place) {
  const lists = useListStore.getState().lists;
  const list = place.list ? lists.find((l) => l.id === place.list) : undefined;
  const project = list?.project_id ?? place.project;
  if (project && useProjectStore.getState().projects.some((p) => p.id === project)) {
    if (useProjectStore.getState().activeId !== project) selectProject(project);
  }
  if (list) {
    const layout = useLayoutStore.getState();
    if (layout.activeListId !== list.id && layout.splitListId !== list.id)
      useLayoutStore.setState({ activeListId: list.id });
    layout.selectList(list.id);
  }
  const task = place.task ? useTaskStore.getState().taskById[place.task] : undefined;
  if (task && (!list || task.list_id === list.id)) useSelectionStore.getState().focus(task.id);
  else if (list && place.task === null) useSelectionStore.getState().focus(null);
}

/** Opens the address the page was loaded at, then keeps the address and the
 *  history in step with the window. Returns the cleanup. */
export function startUrlSync(): () => void {
  const initial = parse(location.pathname);
  if (initial.list || initial.project) go(initial);
  let shown = current();
  if (pathOf(shown) !== location.pathname) history.replaceState(null, '', pathOf(shown));

  let applying = false;
  const sync = () => {
    if (applying) return;
    const next = current();
    const path = pathOf(next);
    if (path === location.pathname) {
      shown = next;
      return;
    }
    if (next.list !== shown.list || next.project !== shown.project)
      history.pushState(null, '', path);
    else history.replaceState(null, '', path);
    shown = next;
  };
  const onPop = () => {
    applying = true;
    try {
      const place = parse(location.pathname);
      go(place.list || place.project ? place : { ...place, task: null });
    } finally {
      applying = false;
    }
    shown = current();
  };

  const subs = [
    useLayoutStore.subscribe((s, p) => {
      if (s.activeListId !== p.activeListId) sync();
    }),
    useProjectStore.subscribe((s, p) => {
      if (s.activeId !== p.activeId) sync();
    }),
    useSelectionStore.subscribe((s, p) => {
      if (s.selectedTaskId !== p.selectedTaskId) sync();
    }),
  ];
  window.addEventListener('popstate', onPop);
  return () => {
    subs.forEach((u) => u());
    window.removeEventListener('popstate', onPop);
  };
}
