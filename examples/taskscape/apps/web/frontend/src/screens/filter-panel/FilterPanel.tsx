// The main window with the Filter & Sort panel open in its preview column: the title bar with the list
// tabs, the task list pane(s), and the preview column that holds the panel or its empty state.
import { useEffect, useRef, useState } from "react";
import { useQueries } from "@tanstack/react-query";
import { Button } from "@taskscape/ui/button";
import { Pane } from "@taskscape/ui/pane";
import { TabList, TabListItem } from "@taskscape/ui/tab-list";
import { TitleBar } from "@taskscape/ui/title-bar";
import { Toggle } from "@taskscape/ui/toggle";
import { WindowFrame, WindowFrameBody, WindowFrameTitleBar } from "@taskscape/ui/window-frame";
import * as ep from "../../api/endpoints";
import { keys, useLists, useMe, useProjects, useSettings, useTasks, useWrite } from "../../api/queries";
import type { Task, TaskList } from "../../api/types";
import { useUi } from "../../state/ui";
import {
  DEFAULT_VIEW,
  computePane,
  viewActive,
  viewOf,
  type Fields,
  type PaneSection,
  type PaneView,
  type Scope,
  type View,
} from "./model";
import { Panel, PreviewEmpty } from "./Panel";
import { TaskPane } from "./TaskPane";

interface PoolEntry {
  list: TaskList;
  project: string;
  tasks: Task[];
}

const TAB_PARTS: Record<string, string> = {
  Errands: "listTabErrands",
  Reading: "listTabReading",
  Garden: "listTabGarden",
};

export function FilterPanel() {
  const ui = useUi();
  const { projectId, panes, activePane, aside } = ui;
  const projects = useProjects();
  const allProjects = projects.data ?? [];
  const project = allProjects.find((p) => p.id === projectId);
  const lists = useLists(projectId);
  const listItems = lists.data ?? [];
  const settings = useSettings();
  const me = useMe();
  const paneA = useTasks({ list: panes[0] ?? null });
  const paneB = useTasks({ list: panes[1] ?? null });
  const wholeProject = useTasks({ project: projectId ?? null });
  const write = useWrite(ep.tasks.update);

  const [scope, setScope] = useState<Scope>("list");
  const [fields, setFields] = useState<Fields>("both");
  const [queries, setQueries] = useState<Record<number, string>>({});
  const [views, setViews] = useState<Record<number, View>>({});
  const [collapsed, setCollapsed] = useState<Record<number, boolean>>({});
  const searchRef = useRef<HTMLInputElement | null>(null);

  // Every project's lists and tasks, fetched only for the all-projects search.
  const everyLists = useQueries({
    queries: allProjects.map((p) => ({
      queryKey: keys.lists(p.id),
      queryFn: () => ep.lists.list(p.id),
      enabled: scope === "all",
    })),
  });
  const everyTasks = useQueries({
    queries: allProjects.map((p) => ({
      queryKey: keys.tasks({ project: p.id }),
      queryFn: () => ep.tasks.list({ project: p.id }),
      enabled: scope === "all",
    })),
  });

  const pool: PoolEntry[] = [];
  if (scope === "project") {
    for (const l of listItems) {
      pool.push({ list: l, project: project?.name ?? "", tasks: (wholeProject.data ?? []).filter((t) => t.list === l.id) });
    }
  }
  if (scope === "all") {
    allProjects.forEach((p, i) => {
      const ts = everyTasks[i]?.data ?? [];
      for (const l of everyLists[i]?.data ?? []) {
        pool.push({ list: l, project: p.name, tasks: ts.filter((t) => t.list === l.id) });
      }
    });
  }

  const showCompleted = settings.data?.show_completed ?? true;
  const now = Date.now();
  const paneTasks = [paneA.data ?? [], paneB.data ?? []];

  const paneViews: PaneView[] = panes.map((id, index) => {
    const list = listItems.find((l) => l.id === id);
    const view = viewOf(views, id);
    const query = queries[index] ?? "";
    const ctx = { query, fields, showCompleted, now };
    const own = computePane(paneTasks[index] ?? [], view, ctx, collapsed);
    const groupMode = query.trim() !== "" && scope !== "list";
    if (!groupMode) {
      const section: PaneSection = { key: "own", heading: null, listName: "", rows: own.rows };
      return {
        index,
        list,
        view,
        query,
        groupMode,
        sections: [section],
        visible: own.visible,
        total: own.total,
        done: own.done,
        matches: own.matches,
      };
    }
    const sections: PaneSection[] = [];
    let matches = 0;
    for (const e of pool) {
      const m = computePane(e.tasks, view, ctx, collapsed);
      matches += m.matches;
      if (m.rows.length > 0) {
        sections.push({
          key: String(e.list.id),
          heading: scope === "all" ? `${e.project} · ${e.list.name}` : e.list.name,
          listName: e.list.name,
          rows: m.rows,
        });
      }
    }
    return {
      index,
      list,
      view,
      query,
      groupMode,
      sections,
      visible: own.visible,
      total: own.total,
      done: own.done,
      matches,
    };
  });

  const named = paneViews.filter((p) => p.list !== undefined);
  const first = named[0];
  const view = first?.view ?? DEFAULT_VIEW;
  const resetEnabled = named.some((p) => viewActive(p.view));
  const subtitle = named.map((p) => p.list?.name ?? "").join(", ") || "No list open";
  let summary = "Nothing to filter";
  if (named.length > 1) {
    summary = named.map((p) => `${p.list?.name ?? ""} ${p.visible}/${p.total}`).join(" · ");
  } else if (first) {
    summary = `Showing ${first.visible} of ${first.total} tasks`;
  }
  const split = panes.length > 1;

  // The panel edits every target list as one.
  const patch = (p: Partial<View>) =>
    setViews((prev) => {
      const next = { ...prev };
      for (const pv of named) {
        if (pv.list) next[pv.list.id] = { ...viewOf(prev, pv.list.id), ...p };
      }
      return next;
    });
  const resetAll = () =>
    setViews((prev) => {
      const next = { ...prev };
      for (const pv of named) {
        if (pv.list) delete next[pv.list.id];
      }
      return next;
    });

  const toggleSplit = () => {
    const s = useUi.getState();
    if (s.panes.length > 1) {
      s.split(false);
      return;
    }
    const at = listItems.findIndex((l) => l.id === s.panes[0]);
    const next = listItems[at + 1] ?? listItems.find((l) => l.id !== s.panes[0]);
    if (!next) return;
    s.split(true);
    useUi.getState().openList(next.id, 1);
  };

  // "/" focuses the search field of the first pane when no text field has the focus (decision 18).
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key !== "/" || e.ctrlKey || e.metaKey || e.altKey) return;
      const el = document.activeElement as HTMLElement | null;
      if (el && (el.isContentEditable || /^(INPUT|TEXTAREA|SELECT)$/.test(el.tagName))) return;
      if (!searchRef.current) return;
      e.preventDefault();
      searchRef.current.focus();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  const initial = (me.data?.name || me.data?.email || "?").trim().charAt(0).toUpperCase();
  const activeId = panes[activePane];

  const trailing = (
    <>
      <Button
        data-part="app.filterpanel.trashButton"
        variant="ghost"
        tone="muted"
        size="md"
        shape="icon"
        icon="delete"
        ariaLabel="Trash"
      />
      <Button
        data-part="app.filterpanel.paletteButton"
        variant="ghost"
        tone="muted"
        size="md"
        shape="icon"
        icon="keyboard_command_key"
        ariaLabel="Command palette"
      />
      <Toggle
        data-part="app.filterpanel.splitView"
        size="md"
        shape="icon"
        icon="vertical_split"
        ariaLabel="Split view"
        pressed={split}
        onPressedChange={toggleSplit}
      />
      <Button
        data-part="app.filterpanel.hidePreview"
        variant="ghost"
        tone="muted"
        size="md"
        shape="icon"
        icon="right_panel_close"
        ariaLabel={aside === "none" ? "Show preview panel" : "Hide preview panel"}
        active={aside !== "none"}
        onClick={() => ui.setAside(aside === "none" ? "preview" : "none")}
      />
      <Button
        data-part="app.filterpanel.settings"
        variant="ghost"
        tone="muted"
        size="md"
        shape="icon"
        icon="settings"
        ariaLabel="Settings"
        onClick={() => ui.setSettings(true)}
      />
      <Button
        data-part="app.filterpanel.accountButton"
        variant="ghost"
        tone="strong"
        size="md"
        label={initial}
        ariaLabel="Account"
      />
    </>
  );

  return (
    <WindowFrame chrome="titled">
      <WindowFrameTitleBar>
        <TitleBar scope="pane" className="w-full" trailing={trailing}>
          <Button
            data-part="app.filterpanel.projectSwitch"
            variant="ghost"
            tone="strong"
            size="md"
            width="content"
            label={project?.name ?? "Project"}
            trailingIcon="unfold_more"
            ariaLabel="Switch project"
            className="shrink-0 self-center"
          />
          <div className="flex min-w-0 flex-1 items-stretch">
            <TabList
              label="Lists"
              value={activeId != null ? String(activeId) : ""}
              onValueChange={(v) => ui.openList(Number(v))}
            >
              {listItems.map((l) => (
                <TabListItem
                  key={l.id}
                  value={String(l.id)}
                  {...(TAB_PARTS[l.name] ? { "data-part": `app.filterpanel.${TAB_PARTS[l.name]}` } : {})}
                >
                  {l.name}
                </TabListItem>
              ))}
            </TabList>
          </div>
          <Button
            data-part="app.filterpanel.newList"
            variant="ghost"
            tone="muted"
            size="sm"
            shape="icon"
            icon="add"
            ariaLabel="New list"
            className="self-center"
          />
        </TitleBar>
      </WindowFrameTitleBar>
      <WindowFrameBody>
        <div className="flex min-h-0 min-w-0 flex-1">
          {paneViews.map((pv) => (
            <TaskPane
              key={pv.index}
              pane={pv}
              primary={pv.index === 0}
              split={split}
              inputRef={pv.index === 0 ? searchRef : undefined}
              panelActive={viewActive(pv.view)}
              scope={scope}
              fields={fields}
              onQuery={(q) => setQueries((prev) => ({ ...prev, [pv.index]: q }))}
              onScope={setScope}
              onFields={setFields}
              onToggle={(id) => setCollapsed((prev) => ({ ...prev, [id]: !prev[id] }))}
              onDone={(id, done) => write.mutate([id, { done }])}
              onOpenPanel={() => ui.setAside(aside === "filter" ? "preview" : "filter")}
            />
          ))}
        </div>
        {aside !== "none" && (
          <Pane slot="aside" previewWidth={320} edge="leading">
            {aside === "filter" ? (
              <Panel
                subtitle={subtitle}
                view={view}
                resetEnabled={resetEnabled}
                summary={summary}
                onPatch={patch}
                onReset={resetAll}
                onClose={() => ui.setAside("preview")}
                onDone={() => ui.setAside("preview")}
              />
            ) : (
              <PreviewEmpty />
            )}
          </Pane>
        )}
      </WindowFrameBody>
    </WindowFrame>
  );
}
