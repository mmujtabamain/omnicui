// The right pane of a split view: the second list, with its own search row, task rows and footer. The left
// pane is the task list screen. The list model and the row are the task list's own; the pane is composed
// here because the task list draws one pane, from the first list of the store.
import { useRef, useState, type KeyboardEvent } from "react";
import { Button } from "@taskscape/ui/button";
import { Icon } from "@taskscape/ui/icon";
import { Input } from "@taskscape/ui/input";
import { ProgressBar } from "@taskscape/ui/progress-bar";
import { Text } from "@taskscape/ui/text";
import * as ep from "../../api/endpoints";
import { useLists, useProjects, useSettings, useTasks, useWrite } from "../../api/queries";
import type { Id } from "../../api/types";
import { useUi } from "../../state/ui";
import { matchRows, percentOf, treeRows, type RowModel, type Where } from "../task-list/model";
import { TaskRow } from "../task-list/TaskRow";

export function SplitPane({ listId }: { listId: Id }) {
  const projectId = useUi((s) => s.projectId);
  const selection = useUi((s) => s.selection);
  const previewId = useUi((s) => s.previewId);
  const projects = useProjects();
  const settings = useSettings();
  const lists = useLists(projectId);
  const tasksQuery = useTasks({ list: listId });
  const update = useWrite(ep.tasks.update);
  const [query, setQuery] = useState("");
  const [collapsed, setCollapsed] = useState<ReadonlySet<Id>>(() => new Set<Id>());
  const bodies = useRef(new Map<Id, HTMLElement>());

  const list = lists.data?.find((l) => l.id === listId) ?? null;
  if (!list) return null;

  const tasks = tasksQuery.data ?? [];
  const showCompleted = settings.data?.show_completed ?? true;
  const where: Where = {
    projectId: list.project,
    projectName: projects.data?.find((p) => p.id === list.project)?.name ?? "",
    listId: list.id,
    listName: list.name,
  };
  const q = query.trim();
  const searching = q !== "";
  const rows: RowModel[] = searching
    ? matchRows(tasks, where, q, "both", showCompleted)
    : treeRows(tasks, where, collapsed, showCompleted);
  const done = tasks.filter((t) => t.done).length;
  const total = tasks.length;
  const pct = percentOf(done, total);

  const toggleExpanded = (id: Id) =>
    setCollapsed((c) => {
      const next = new Set(c);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });

  const onKey = (e: KeyboardEvent<HTMLDivElement>, row: RowModel) => {
    if (e.altKey || e.ctrlKey || e.metaKey || e.shiftKey) return;
    if (e.key !== "ArrowDown" && e.key !== "ArrowUp") return;
    e.preventDefault();
    const at = rows.findIndex((r) => r.task.id === row.task.id);
    const next = rows[at + (e.key === "ArrowDown" ? 1 : -1)];
    if (next) {
      useUi.getState().preview(next.task.id);
      bodies.current.get(next.task.id)?.focus();
    }
  };

  return (
    <div
      data-part="app.mainwindow.splitPane"
      className="flex min-h-0 min-w-0 flex-1 flex-col"
      onPointerDownCapture={() => useUi.setState({ activePane: 1 })}
    >
      <div className="flex shrink-0 items-center gap-2 px-[1rem] pt-[0.714rem] pb-[0.5rem]">
        <div className="flex h-[2.5rem] min-w-0 flex-1 items-center gap-[0.357rem] rounded-full bg-wash-2 pr-[0.357rem] pl-[0.643rem] focus-within:ring-2 focus-within:ring-focus-1">
          <Icon name="search" size={18} tone="muted" />
          <Input
            data-part="app.mainwindow.splitSearch"
            mode="bare"
            size="md"
            value={query}
            placeholder={`Search ${list.name}`}
            ariaLabel={`Search ${list.name}`}
            spellCheck={false}
            className="min-w-0 flex-1 focus:ring-0"
            onValueChange={setQuery}
            onCancel={() => setQuery("")}
          />
          <span aria-hidden="true" className="block h-4 w-px shrink-0 bg-edge-2" />
          <Button
            data-part="app.mainwindow.splitClose"
            variant="ghost"
            size="sm"
            shape="icon"
            tone="muted"
            icon="close"
            iconSize={16}
            ariaLabel="Close split"
            onClick={() => useUi.setState((s) => ({ panes: [s.panes[0] ?? null], activePane: 0 }))}
          />
        </div>
      </div>

      <div className="min-h-0 flex-1 overflow-y-auto">
        {rows.map((row, index) => (
          <TaskRow
            key={row.task.id}
            row={row}
            query={q}
            searching={searching}
            picked={selection.includes(row.task.id)}
            previewed={previewId === row.task.id}
            optionHeld={false}
            parts={index === 0 ? { subtask: "app.mainwindow.splitRow" } : {}}
            renaming={false}
            menu={null}
            onActivate={(r) => {
              useUi.setState({ activePane: 1 });
              useUi.getState().preview(r.task.id);
            }}
            onCheck={(r) => {
              const id = r.task.id;
              const current = useUi.getState().selection;
              useUi.getState().select(current.includes(id) ? current.filter((x) => x !== id) : [...current, id]);
            }}
            onDoubleToggle={(r) => update.mutate([r.task.id, { done: !r.task.done }])}
            onContext={() => undefined}
            onToggleExpanded={(r) => toggleExpanded(r.task.id)}
            onRenamed={() => undefined}
            onKey={onKey}
            registerBody={(id, el) => {
              if (el) bodies.current.set(id, el);
              else bodies.current.delete(id);
            }}
          />
        ))}
      </div>

      <div className="flex h-[2.143rem] shrink-0 items-center gap-2 border-t border-edge-1 bg-surface-2 pr-[0.857rem] pl-[0.857rem]">
        <Text
          data-part="app.mainwindow.splitFooterDoneCount"
          as="span"
          size="xs"
          weight="semibold"
          tone="muted"
          case="uppercase"
          numeric="tabular"
        >
          {`${done} / ${total} done`}
        </Text>
        <ProgressBar className="ml-auto" value={done} max={Math.max(total, 1)} ariaLabel="Done progress" />
        <Text as="span" size="xs" weight="semibold" tone="secondary" numeric="tabular">
          {`${pct}%`}
        </Text>
      </div>
    </div>
  );
}
