// The main window (contract app.mainwindow): the title bar, the task pane of the active list (or the
// no-lists state), the split view's second pane, the right aside, and the overlays (sheets, toast, command
// palette, Settings). The shell owns the app-wide shortcuts, the project and list it opens on, and the pane
// layout.
import { useEffect, useRef } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { Button } from "@taskscape/ui/button";
import { Overlay, OverlayTitle } from "@taskscape/ui/overlay";
import { Resizer } from "@taskscape/ui/resizer";
import { Scrim } from "@taskscape/ui/scrim";
import { Text } from "@taskscape/ui/text";
import * as ep from "../../api/endpoints";
import { keys, useLists, useProjects, useSettings } from "../../api/queries";
import type { Task, TaskList } from "../../api/types";
import { PREVIEW_MAX, PREVIEW_MIN, SPLIT_MAX, SPLIT_MIN, useUi } from "../../state/ui";
import { SheetHost, useSheets } from "../sheets";
import { openSheet } from "../sheets/store";
import { ToastHost } from "../toast";
import { SettingsDialog } from "../settings/SettingsDialog";
import { AppPalette } from "../palette/AppPalette";
import { TitleBar } from "../title-bar/TitleBar";
import { TaskList as TaskPane } from "../task-list/TaskList";
import { Aside } from "./Aside";
import { SplitPane } from "./SplitPane";
import { observeParts, setListCount, tagParts } from "./parts";

const IS_MAC = typeof navigator !== "undefined" && /Mac|iPhone|iPad|iPod/.test(navigator.platform);

const byOrder = <T extends { sort_order: number; id: number }>(a: T, b: T) => a.sort_order - b.sort_order || a.id - b.id;

/** The title button of a task row: the row's first button that is not its check or its disclosure. */
const rowTitleOf = (row: Element): HTMLElement | null =>
  Array.from(row.querySelectorAll<HTMLElement>('[role="button"][aria-label]')).find((el) => !el.matches('[data-slot="check"], [data-row-control]')) ?? null;

/** ArrowUp on the first row of the task pane moves to the last visible row: the list wraps (F09-020 style keys). */
function wrapToLastRow(event: KeyboardEvent): void {
  if (event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return;
  const st = useUi.getState();
  if (useSheets.getState().pending != null || st.paletteOpen || st.settingsOpen) return;
  const target = event.target instanceof Element ? event.target : null;
  if (!target || target.closest('[data-slot="check"], [data-row-control]')) return;
  const pane = document.querySelector('[data-part="app.mainwindow.taskPane"]');
  const rows = Array.from(pane?.querySelectorAll<HTMLElement>("[data-task-id]") ?? []);
  const first = rows[0];
  const last = rows[rows.length - 1];
  if (!first || !last || rows.length < 2 || target.closest("[data-task-id]") !== first) return;
  const title = rowTitleOf(last);
  if (!title) return;
  event.preventDefault();
  event.stopPropagation();
  st.preview(Number(last.getAttribute("data-task-id")));
  title.focus();
}

const isTyping = (target: EventTarget | null) => {
  const el = target as HTMLElement | null;
  return !!el && (el.isContentEditable || /^(INPUT|TEXTAREA|SELECT)$/.test(el.tagName));
};

/** The left pane's width for a split ratio, one class per whole percent (the ratio is 0.25 to 0.75). */
const LEFT_WIDTHS = [
  "w-[25%]", "w-[26%]", "w-[27%]", "w-[28%]", "w-[29%]", "w-[30%]", "w-[31%]", "w-[32%]", "w-[33%]", "w-[34%]",
  "w-[35%]", "w-[36%]", "w-[37%]", "w-[38%]", "w-[39%]", "w-[40%]", "w-[41%]", "w-[42%]", "w-[43%]", "w-[44%]",
  "w-[45%]", "w-[46%]", "w-[47%]", "w-[48%]", "w-[49%]", "w-[50%]", "w-[51%]", "w-[52%]", "w-[53%]", "w-[54%]",
  "w-[55%]", "w-[56%]", "w-[57%]", "w-[58%]", "w-[59%]", "w-[60%]", "w-[61%]", "w-[62%]", "w-[63%]", "w-[64%]",
  "w-[65%]", "w-[66%]", "w-[67%]", "w-[68%]", "w-[69%]", "w-[70%]", "w-[71%]", "w-[72%]", "w-[73%]", "w-[74%]",
  "w-[75%]",
];
const leftWidth = (ratio: number) => LEFT_WIDTHS[Math.round(Math.min(SPLIT_MAX, Math.max(SPLIT_MIN, ratio)) * 100) - 25] ?? "w-[50%]";

export function MainWindow() {
  const qc = useQueryClient();
  const projectsQuery = useProjects();
  const settings = useSettings();
  const projectId = useUi((s) => s.projectId);
  const panes = useUi((s) => s.panes);
  const aside = useUi((s) => s.aside);
  const previewId = useUi((s) => s.previewId);
  const selection = useUi((s) => s.selection);
  const draftListId = useUi((s) => s.draftListId);
  const previewWidth = useUi((s) => s.previewWidth);
  const splitRatio = useUi((s) => s.splitRatio);
  const paletteOpen = useUi((s) => s.paletteOpen);
  const settingsOpen = useUi((s) => s.settingsOpen);
  const paneRef = useRef<HTMLDivElement>(null);

  const projects = [...(projectsQuery.data ?? [])].sort(byOrder);
  const activeProject = projects.find((p) => p.id === projectId) ?? null;
  const listsQuery = useLists(activeProject?.id);
  const lists: TaskList[] = [...(listsQuery.data ?? [])].sort(byOrder);
  const noLists = listsQuery.isSuccess && lists.length === 0;
  const splitOn = panes.length > 1 && !noLists;
  const rightId = splitOn ? (panes[1] ?? null) : null;
  const draftList = lists.find((l) => l.id === draftListId) ?? null;

  // Opens on the first project (Home in the demo data) until a project is picked.
  useEffect(() => {
    if (!projectsQuery.data || settings.isLoading) return;
    const current = useUi.getState().projectId;
    if (current != null && projectsQuery.data.some((p) => p.id === current)) return;
    const first = [...projectsQuery.data].sort(byOrder)[0];
    if (first) useUi.getState().setProject(first.id);
  }, [projectsQuery.data, settings.isLoading]);

  // The left pane shows the first list of the project unless its list is still there.
  useEffect(() => {
    if (!listsQuery.data) return;
    const st = useUi.getState();
    if (st.panes[0] != null && listsQuery.data.some((l) => l.id === st.panes[0])) return;
    const first = [...listsQuery.data].sort(byOrder)[0];
    useUi.setState({ panes: [first?.id ?? null], activePane: 0 });
  }, [listsQuery.data]);

  // Split view: the right pane is the list after the left one, unless the right pane holds another list.
  useEffect(() => {
    if (!splitOn || lists.length < 2) return;
    const [left, right] = useUi.getState().panes;
    if (right != null && right !== left && lists.some((l) => l.id === right)) return;
    const at = lists.findIndex((l) => l.id === left);
    const next = lists[(at + 1) % lists.length];
    if (next && next.id !== left) useUi.setState({ panes: [left ?? null, next.id] });
  }, [splitOn, panes, listsQuery.data]);

  // A preview of a task ends the draft: one of the two shows in the aside.
  useEffect(() => {
    if (previewId != null && useUi.getState().draftListId != null) useUi.getState().setDraft(null);
  }, [previewId]);

  // The data-part names of the parts on screen. The tagger runs after every change to the page.
  useEffect(() => observeParts(), []);
  useEffect(() => {
    setListCount(lists.length);
    tagParts();
  });

  // App-wide shortcuts (decision 18). Mod is Ctrl on Windows and Linux, Cmd on macOS.
  useEffect(() => {
    const startDraft = () => {
      const listId = useUi.getState().panes[0];
      if (listId != null) useUi.getState().startDraft(listId);
    };
    const onKeyCapture = (event: KeyboardEvent) => {
      if (event.code === "ArrowUp") {
        wrapToLastRow(event);
        return;
      }
      // The disclosure of a parent row: ArrowLeft collapses an expanded parent, ArrowRight expands a collapsed one.
      // It is handled before the row sees the key, which leaves a disclosure's keys to the shell.
      if (event.code !== "ArrowLeft" && event.code !== "ArrowRight") return;
      if (event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return;
      if (useSheets.getState().pending != null || useUi.getState().paletteOpen || useUi.getState().settingsOpen) return;
      const control = event.target instanceof Element ? event.target.closest<HTMLElement>('[aria-label="Collapse"], [aria-label="Expand"]') : null;
      if (!control || !control.closest("[data-task-id]")) return;
      const label = control.getAttribute("aria-label");
      if (!((event.code === "ArrowLeft" && label === "Collapse") || (event.code === "ArrowRight" && label === "Expand"))) return;
      event.preventDefault();
      event.stopPropagation();
      control.click();
    };
    const onKey = (event: KeyboardEvent) => {
      if (event.defaultPrevented) return;
      const st = useUi.getState();
      const overlay = useSheets.getState().pending != null || st.paletteOpen || st.settingsOpen;
      const mod = IS_MAC ? event.metaKey && !event.ctrlKey : event.ctrlKey && !event.metaKey;
      const typing = isTyping(event.target);

      if (event.altKey && !mod && !event.ctrlKey && !event.metaKey && !event.shiftKey && event.code === "KeyN") {
        if (!overlay) startDraft();
        event.preventDefault();
        return;
      }
      if (overlay) return;
      if (mod && !event.altKey && !event.shiftKey) {
        if (event.code === "KeyK") st.setPalette(true);
        else if (event.code === "Comma") st.setSettings(true);
        else if (event.code === "Backslash") st.setAside(st.aside === "preview" ? "none" : "preview");
        else if (event.code === "KeyZ") {
          // Undo of the last Move to Trash, while the toast is up.
          const toast = useSheets.getState().toast;
          if (!toast?.retry || typing) return;
          useSheets.getState().setToast(null);
          toast.retry().catch(() => undefined);
        } else return;
        event.preventDefault();
        return;
      }
      if (typing) return;
      // Escape clears the picks when no overlay is open (the task list also does this from a row).
      if (event.code === "Escape" && !mod && !event.altKey && !event.shiftKey) {
        if (st.selection.length > 0) st.select([]);
        return;
      }
      // Delete asks first: the active task goes to the Trash after the confirmation (decision 10).
      if (event.code === "Delete" && !mod && !event.altKey && !event.shiftKey) {
        // The task list is cached under the scope it was loaded for (a list, or a project): look in every one.
        let task: Task | undefined;
        for (const [, data] of qc.getQueriesData<Task[]>({ queryKey: ["tasks"] })) {
          task ??= data?.find((t) => t.id === st.previewId);
        }
        if (task) {
          event.preventDefault();
          void openSheet({ kind: "deleteConfirm", task });
        }
        return;
      }
      // Slash, and a letter typed on a focused row, start a search in the focused list.
      const plain = !event.ctrlKey && !event.metaKey && !event.altKey;
      if (plain && (event.code === "Slash" || /^Key[A-Z]$/.test(event.code))) {
        const input = paneRef.current?.querySelector<HTMLInputElement>("input");
        if (input) {
          input.focus();
          if (event.code === "Slash") event.preventDefault();
        }
      }
    };
    window.addEventListener("keydown", onKeyCapture, true);
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("keydown", onKeyCapture, true);
      window.removeEventListener("keydown", onKey);
    };
  }, [qc]);

  // A task row that takes keyboard focus is the active task, so the arrows move the preview with the focus.
  // The check is not a title: focusing it picks nothing and previews nothing.
  useEffect(() => {
    const onFocusIn = (event: FocusEvent) => {
      const target = event.target instanceof Element ? event.target : null;
      const row = target?.closest("[data-task-id]");
      if (!target || !row || target.closest('[data-slot="check"], [data-row-control]')) return;
      const id = Number(row.getAttribute("data-task-id"));
      if (useUi.getState().previewId !== id) useUi.getState().preview(id);
    };
    window.addEventListener("focusin", onFocusIn);
    return () => window.removeEventListener("focusin", onFocusIn);
  }, []);

  const signOut = () => {
    ep.auth.signout().catch(() => undefined).finally(() => {
      void qc.invalidateQueries({ queryKey: keys.me });
    });
  };
  // Events from the title bar: sign out is the shell's to run.
  const barEmit = (name: string) => {
    if (name === "signOutRequested") signOut();
  };
  const listEmit = (name: string) => {
    if (name === "draftStarted") {
      const listId = useUi.getState().panes[0];
      if (listId != null) useUi.getState().startDraft(listId);
    }
  };
  const activate = (pane: number) => {
    if (useUi.getState().activePane !== pane) useUi.setState({ activePane: pane });
  };

  return (
    <div data-part="app.mainwindow.window" className="flex h-screen w-screen flex-col overflow-hidden bg-surface-1 text-content-1">
      <div data-part="app.mainwindow.titleBar" className="flex shrink-0 flex-col">
        <TitleBar emit={barEmit} />
      </div>

      <div className="flex min-h-0 flex-1">
        <div ref={paneRef} className="flex min-h-0 min-w-0 flex-1">
          {!noLists && panes[0] != null && (
            <div
              data-part="app.mainwindow.taskPane"
              onPointerDownCapture={() => activate(0)}
              className={splitOn ? `flex min-h-0 min-w-0 shrink-0 flex-col ${leftWidth(splitRatio)}` : "flex min-h-0 min-w-0 flex-1 flex-col"}
            >
              <TaskPane key={splitOn ? "split" : "single"} emit={listEmit} />
            </div>
          )}
          {!noLists && splitOn && rightId != null && (
            <>
              <Resizer
                data-part="app.mainwindow.splitDivider"
                label="Resize the panes"
                value={splitRatio}
                min={SPLIT_MIN}
                max={SPLIT_MAX}
                reset="double-click"
                onResize={(ratio) => useUi.getState().setSplitRatio(ratio)}
                onReset={() => useUi.getState().setSplitRatio(0.5)}
              />
              <SplitPane listId={rightId} />
            </>
          )}
          {noLists && (
            <div className="flex min-h-0 flex-1 flex-col items-center justify-center gap-1 bg-surface-0 px-4 text-center">
              <Text data-part="app.mainwindow.noListsTitle" as="div" tone="secondary" className="font-display text-[1.071rem] font-medium">
                No lists yet
              </Text>
              <Text data-part="app.mainwindow.noListsSubtitle" as="div" tone="muted" className="text-[0.893rem]">
                Create a list to start adding tasks
              </Text>
            </div>
          )}
        </div>

        {aside !== "none" && (
          <>
            <Resizer
              data-part="app.mainwindow.previewDivider"
              label="Resize the preview panel"
              value={previewWidth}
              min={PREVIEW_MIN}
              max={PREVIEW_MAX}
              reset="none"
              open
              onResize={(px) => useUi.getState().setPreviewWidth(px)}
            />
            <Aside
              mode={aside}
              width={previewWidth}
              picked={selection.length}
              previewId={previewId}
              draftListId={draftListId}
              draftListName={draftList?.name ?? null}
              projectName={activeProject?.name ?? null}
            />
          </>
        )}
      </div>

      <SheetHost />
      <ToastHost />
      {paletteOpen && <AppPalette />}
      {settingsOpen && (
        <>
          <Scrim data-part="app.mainwindow.settingsScrim" dim="soft" />
          <Overlay
            data-part="app.mainwindow.settingsDialog"
            open
            onOpenChange={(open) => {
              if (!open) useUi.getState().setSettings(false);
            }}
            className="w-[min(45.714rem,calc(100%-2rem))] overflow-hidden"
          >
            {/* The dialog's name for assistive technology; the page draws its own visible heading. */}
            <OverlayTitle className="sr-only">Settings</OverlayTitle>
            <div className="relative flex h-[min(37.143rem,calc(100dvh-2rem))] min-h-0 flex-col overflow-hidden">
              <SettingsDialog />
              <div className="absolute top-[0.571rem] right-[0.571rem] z-(--z-index-raised)">
                <Button
                  data-part="app.mainwindow.settingsClose"
                  variant="ghost"
                  size="sm"
                  shape="icon"
                  tone="muted"
                  icon="close"
                  iconSize={16}
                  ariaLabel="Close settings"
                  onClick={() => useUi.getState().setSettings(false)}
                />
              </div>
            </div>
          </Overlay>
        </>
      )}
    </div>
  );
}
