// The web's window (replaces tauri/main/src/App.tsx). The same surfaces in the
// same places, built from the same stores: the header, the task pane (two in
// split view), the details drawer, and the palette, sheets and toast over
// them. What a page does differently:
//
// - the address bar follows the window (url.ts) and the tab's title follows
//   the selection;
// - Settings and quick capture are drawn in the page (SettingsDialog,
//   CaptureBar), never in a window of their own (WHY.md);
// - no boot veil: the page paints its shell at once and the panes fill in
//   when the data lands;
// - below 960 px the drawer is an overlay at the right edge, over the pane and
//   a scrim that closes it, instead of a column beside it, so the pane keeps a
//   usable width;
// - quick capture is kept alive between uses (hidden), as the tray keeps its
//   window, so it shows at once with its draft;
// - there is no menu bar or tray to route events from.
import { cn } from '@taskscape/common-ui/cn';
import { EmptyState } from '@taskscape/common-ui/components';
import { buildCommands } from '@shared/commands/palette';
import { CommandPalette } from '@shared/components/CommandPalette';
import { ContextMenuProvider } from '@shared/components/ContextMenu';
import { FilterPanel } from '@shared/components/preview/FilterPanel';
import { PreviewPanel } from '@shared/components/preview/PreviewPanel';
import { SheetHost } from '@shared/components/SheetHost';
import { TaskPane } from '@shared/components/TaskPane';
import { Toast } from '@shared/components/Toast';
import { TrashPane } from '@shared/components/TrashPane';
import { useAppKeyboard } from '@shared/hooks/useAppKeyboard';
import { initialLoad, startBootstrap } from '@shared/stores/bootstrap';
import { useLayoutStore } from '@shared/stores/layoutStore';
import { useListStore } from '@shared/stores/listStore';
import { useProjectStore } from '@shared/stores/projectStore';
import { useSelectionStore } from '@shared/stores/selectionStore';
import { useTaskStore } from '@shared/stores/taskStore';
import { useUiStore } from '@shared/stores/uiStore';
import { useEffect, useMemo, useState, useSyncExternalStore } from 'react';
import { CaptureBar } from './CaptureBar';
import { SettingsDialog } from './SettingsDialog';
import { TitleBar } from './TitleBar';
import { startUrlSync } from './url';
import { useWebUi } from './webUi';

/** Below this width the drawer goes over the pane instead of beside it. */
export const NARROW = '(max-width: 959px)';

function useMedia(query: string): boolean {
  return useSyncExternalStore(
    (cb) => {
      const mq = matchMedia(query);
      mq.addEventListener('change', cb);
      return () => mq.removeEventListener('change', cb);
    },
    () => matchMedia(query).matches
  );
}

/** "<task> — <list> · Taskscape", or the list and its project, as the
 *  selection changes, so the browser's tab and history say where it is. */
function useDocumentTitle() {
  const taskId = useSelectionStore((s) => s.selectedTaskId);
  const task = useTaskStore((s) => (taskId ? s.taskById[taskId] : undefined));
  const activeListId = useLayoutStore((s) => s.activeListId);
  const lists = useListStore((s) => s.lists);
  const projects = useProjectStore((s) => s.projects);
  const projectId = useProjectStore((s) => s.activeId);
  useEffect(() => {
    const list = lists.find((l) => l.id === (task?.list_id ?? activeListId));
    const project = projects.find((p) => p.id === projectId);
    document.title = task
      ? `${task.title} — ${list?.name ?? ''} · Taskscape`
      : list
        ? `${list.name} — ${project?.name ?? ''} · Taskscape`
        : project
          ? `${project.name} · Taskscape`
          : 'Taskscape';
  }, [task, activeListId, lists, projects, projectId]);
}

function App() {
  const activeProjectId = useProjectStore((s) => s.activeId);
  const lists = useListStore((s) => s.lists);
  const activeListId = useLayoutStore((s) => s.activeListId);
  const splitListId = useLayoutStore((s) => s.splitListId);
  const previewOpen = useLayoutStore((s) => s.previewOpen);
  const previewW = useLayoutStore((s) => s.previewW);
  const splitRatio = useLayoutStore((s) => s.splitRatio);
  const trashOpen = useUiStore((s) => s.trashOpen);
  const filterOpen = useUiStore((s) => s.filterOpen);
  const paletteOpen = useUiStore((s) => s.paletteOpen);
  const draftListId = useUiStore((s) => s.draftListId);
  const selectedTaskId = useSelectionStore((s) => s.selectedTaskId);
  const settingsOpen = useWebUi((s) => s.settingsOpen);
  const narrow = useMedia(NARROW);
  const [loaded, setLoaded] = useState(false);

  const listsInProject = useMemo(
    () => lists.filter((l) => l.project_id === activeProjectId),
    [lists, activeProjectId]
  );
  const activeList = listsInProject.find((l) => l.id === activeListId) ?? null;
  const splitList = listsInProject.find((l) => l.id === splitListId) ?? null;

  useEffect(() => {
    const cleanup = startBootstrap();
    let stopUrl = () => {};
    let live = true;
    void initialLoad().finally(() => {
      if (!live) return;
      stopUrl = startUrlSync();
      setLoaded(true);
    });
    return () => {
      live = false;
      cleanup();
      stopUrl();
    };
  }, []);

  useAppKeyboard();
  useDocumentTitle();

  const railOpen = previewOpen || trashOpen || filterOpen;
  // Narrow, the drawer covers part of the pane, so it comes up only when it
  // has something to show; the empty "No task selected" stays out of the way.
  const multi = useSelectionStore((s) =>
    Object.values(s.byPane).some((p) => p.ids.size > 1)
  );
  const overlay = narrow && (trashOpen || filterOpen || (previewOpen && (!!selectedTaskId || !!draftListId || multi)));
  const rail = (
    <aside
      aria-label="Details"
      style={{ width: previewW }}
      className={cn(
        'border-edge-2l dark:border-edge-2d shrink-0 border-l',
        overlay &&
          'z-dropdown bg-surface-1l dark:bg-surface-1d shadow-menu animate-rise absolute inset-y-0 right-0 max-w-[calc(100%-48px)]'
      )}
    >
      {trashOpen ? (
        <TrashPane onClose={() => useUiStore.getState().setTrashOpen(false)} />
      ) : filterOpen ? (
        <FilterPanel onClose={() => useUiStore.getState().setFilterOpen(false)} />
      ) : (
        <PreviewPanel />
      )}
    </aside>
  );

  const closeOverlay = () => {
    const ui = useUiStore.getState();
    if (ui.trashOpen) ui.setTrashOpen(false);
    else if (ui.filterOpen) ui.setFilterOpen(false);
    else useLayoutStore.getState().togglePreview();
  };

  return (
    <ContextMenuProvider>
      <div className="bg-surface-1l dark:bg-surface-1d text-content-1l dark:text-content-1d flex h-screen w-screen flex-col overflow-hidden">
        <TitleBar />

        <div className="relative flex min-h-0 flex-1">
          <main aria-label="Tasks" className="flex min-w-0 flex-1">
            {!loaded ? (
              <div className="bg-surface-2l dark:bg-surface-2d flex-1" aria-busy="true" />
            ) : activeList ? (
              <div className="flex min-w-0 flex-1">
                <div
                  className="flex min-w-0"
                  style={{ flexBasis: splitList ? `${splitRatio * 100}%` : '100%' }}
                >
                  <TaskPane list={activeList} isSplit={false} />
                </div>
                {splitList && (
                  <>
                    <Resizer
                      onResize={(x, rect) =>
                        useLayoutStore
                          .getState()
                          .setSplitRatio(Math.min(0.75, Math.max(0.25, (x - rect.left) / rect.width)))
                      }
                      onReset={() => useLayoutStore.getState().resetSplitRatio()}
                    />
                    <div className="border-edge-2l dark:border-edge-2d flex min-w-0 flex-1 border-l">
                      <TaskPane list={splitList} isSplit />
                    </div>
                  </>
                )}
              </div>
            ) : (
              <EmptyState
                part="windowEmpty"
                className="bg-surface-2l dark:bg-surface-2d"
                title={listsInProject.length === 0 ? 'No lists yet' : 'No list selected'}
                subtitle={
                  listsInProject.length === 0
                    ? 'Create a list to start adding tasks'
                    : 'Pick a list tab to view its tasks'
                }
              />
            )}
          </main>

          {narrow ? (
            overlay && (
              <>
                <button
                  type="button"
                  data-part="drawerScrim"
                  aria-label="Close the details"
                  tabIndex={-1}
                  onMouseDown={closeOverlay}
                  className="z-dropdown animate-scrim-in absolute inset-0 cursor-default bg-black/20"
                />
                {rail}
              </>
            )
          ) : (
            railOpen && (
              <>
                <Resizer
                  onResize={(x, rect) =>
                    useLayoutStore.getState().setPreviewW(Math.min(420, Math.max(280, rect.right - x)))
                  }
                />
                {rail}
              </>
            )
          )}
        </div>
      </div>

      {paletteOpen && (
        <CommandPalette
          getCommands={buildCommands}
          onClose={() => useUiStore.getState().setPaletteOpen(false)}
        />
      )}
      {settingsOpen && <SettingsDialog onClose={() => useWebUi.getState().setSettingsOpen(false)} />}
      <CaptureBar />
      <SheetHost />
      <Toast />
    </ContextMenuProvider>
  );
}

/** Invisible grab strip over a hairline that brightens while resizing. */
function Resizer({
  onResize,
  onReset,
}: {
  onResize: (clientX: number, rect: DOMRect) => void;
  onReset?: () => void;
}) {
  const [active, setActive] = useState(false);
  return (
    <div
      role="separator"
      aria-orientation="vertical"
      onDoubleClick={onReset}
      onMouseDown={(e) => {
        e.preventDefault();
        setActive(true);
        const rect = (e.currentTarget.parentElement as HTMLElement).getBoundingClientRect();
        const move = (ev: MouseEvent) => onResize(ev.clientX, rect);
        const up = () => {
          setActive(false);
          window.removeEventListener('mousemove', move);
          window.removeEventListener('mouseup', up);
        };
        window.addEventListener('mousemove', move);
        window.addEventListener('mouseup', up);
      }}
      className="z-raised relative -mr-1.25 w-1.25 shrink-0 cursor-col-resize"
    >
      <span
        className={cn(
          'absolute inset-y-0 left-0 w-px',
          active ? 'bg-edge-3l dark:bg-edge-3d' : 'hover:bg-edge-2l dark:hover:bg-edge-2d bg-transparent'
        )}
      />
    </div>
  );
}

export default App;
