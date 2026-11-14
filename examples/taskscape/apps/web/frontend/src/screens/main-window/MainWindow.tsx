// The main window (contract app.mainwindow): the title bar, the task pane of the active list (or the
// no-lists state), the right aside, and the overlays (sheets, toast, command palette, Settings). The
// shell owns the app-wide shortcuts, the project and list it opens on, and the pane layout.
import { useEffect, useRef } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { Resizer } from "@taskscape/ui/resizer";
import { Text } from "@taskscape/ui/text";
import * as ep from "../../api/endpoints";
import { keys, useLists, useProjects, useSettings } from "../../api/queries";
import type { TaskList } from "../../api/types";
import { PREVIEW_MAX, PREVIEW_MIN, useUi } from "../../state/ui";
import { SheetHost, useSheets } from "../sheets";
import { ToastHost } from "../toast";
import { SettingsDialog } from "../settings/SettingsDialog";
import { AppPalette } from "../palette/AppPalette";
import { TitleBar } from "../title-bar/TitleBar";
import { TaskList as TaskPane } from "../task-list/TaskList";
import { Aside } from "./Aside";
import { tagParts } from "./parts";

const IS_MAC = typeof navigator !== "undefined" && /Mac|iPhone|iPad|iPod/.test(navigator.platform);

const byOrder = <T extends { sort_order: number; id: number }>(a: T, b: T) => a.sort_order - b.sort_order || a.id - b.id;

const isTyping = (target: EventTarget | null) => {
  const el = target as HTMLElement | null;
  return !!el && (el.isContentEditable || /^(INPUT|TEXTAREA|SELECT)$/.test(el.tagName));
};

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
  const paletteOpen = useUi((s) => s.paletteOpen);
  const settingsOpen = useUi((s) => s.settingsOpen);
  const paneRef = useRef<HTMLDivElement>(null);

  const projects = [...(projectsQuery.data ?? [])].sort(byOrder);
  const activeProject = projects.find((p) => p.id === projectId) ?? null;
  const listsQuery = useLists(activeProject?.id);
  const lists: TaskList[] = [...(listsQuery.data ?? [])].sort(byOrder);
  const noLists = listsQuery.isSuccess && lists.length === 0;
  const splitOn = panes.length > 1;
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

  // A preview of a task ends the draft: one of the two shows in the aside.
  useEffect(() => {
    if (previewId != null && useUi.getState().draftListId != null) useUi.getState().setDraft(null);
  }, [previewId]);

  // Keeps the data-part names of the parts on screen (the child screens render them, and portals too).
  useEffect(() => {
    // Once per frame at most: the tagger writes only what differs, so its own writes do not loop.
    let frame = 0;
    const run = () => {
      frame = 0;
      tagParts();
    };
    const schedule = () => {
      if (frame === 0) frame = window.requestAnimationFrame(run);
    };
    schedule();
    const observer = new MutationObserver(schedule);
    observer.observe(document.body, {
      subtree: true,
      childList: true,
      characterData: true,
      attributes: true,
      attributeFilter: ["data-part", "aria-label", "role"],
    });
    return () => {
      observer.disconnect();
      window.cancelAnimationFrame(frame);
    };
  }, []);

  // App-wide shortcuts. The web chords come from decision 18; Mod is Ctrl on Windows and Linux, Cmd on macOS.
  useEffect(() => {
    const startDraft = () => {
      const listId = useUi.getState().panes[0];
      if (listId != null) useUi.getState().startDraft(listId);
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
        else return;
        event.preventDefault();
        return;
      }
      // Slash, and a letter typed on a focused row, start a search in the focused list.
      const plain = !event.ctrlKey && !event.metaKey && !event.altKey;
      if (plain && !typing && (event.code === "Slash" || /^Key[A-Z]$/.test(event.code))) {
        const input = paneRef.current?.querySelector<HTMLInputElement>("input");
        if (input) {
          input.focus();
          if (event.code === "Slash") event.preventDefault();
        }
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  const signOut = () => {
    ep.auth.signout().catch(() => undefined).finally(() => {
      void qc.invalidateQueries({ queryKey: keys.me });
    });
  };
  // Events from the title bar and the task list: a new-task draft and sign out are the shell's to run.
  const barEmit = (name: string) => {
    if (name === "signOutRequested") signOut();
  };
  const listEmit = (name: string) => {
    if (name === "draftStarted") {
      const listId = useUi.getState().panes[0];
      if (listId != null) useUi.getState().startDraft(listId);
    }
    if (name === "splitClosed") useUi.getState().split(false);
  };

  return (
    <div data-part="app.mainwindow.window" className="flex h-screen w-screen flex-col overflow-hidden bg-surface-0 text-content-1">
      <div data-part="app.mainwindow.titleBar" className="flex shrink-0 flex-col">
        <TitleBar emit={barEmit} />
      </div>

      <div className="flex min-h-0 flex-1">
        <div ref={paneRef} className="flex min-h-0 min-w-0 flex-1 flex-col">
          {!noLists && panes[0] != null && (
            <div data-part="app.mainwindow.taskPane" className="flex min-h-0 min-w-0 flex-1 flex-col">
              <TaskPane key={splitOn ? "split" : "single"} split={splitOn} emit={listEmit} />
            </div>
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
      {settingsOpen && <SettingsDialog />}
    </div>
  );
}
