// The task list pane (contract app.tasklist): the search bar, the task tree of the active list (or the matches
// grouped by list), the empty states, and the footer with the done stats or the bulk bar.
import { Fragment, useEffect, useRef, useState, type KeyboardEvent, type SyntheticEvent } from "react";
import { useQueries } from "@tanstack/react-query";
import { Button } from "@taskscape/ui/button";
import { EmptyState } from "@taskscape/ui/empty-state";
import { Heading } from "@taskscape/ui/heading";
import { Icon } from "@taskscape/ui/icon";
import { Input } from "@taskscape/ui/input";
import { Menu, MenuSubmenuPanel } from "@taskscape/ui/menu";
import { MenuItem } from "@taskscape/ui/menu-item";
import { ProgressBar } from "@taskscape/ui/progress-bar";
import { Separator } from "@taskscape/ui/separator";
import { Text } from "@taskscape/ui/text";
import * as ep from "../../api/endpoints";
import { keys, useLists, useProjects, useSettings, useTasks, useWrite } from "../../api/queries";
import type { Id, Task, TaskList as ListRecord } from "../../api/types";
import { useUi } from "../../state/ui";
import { openSheet } from "../sheets/store";
import {
  cx,
  descendantIds,
  matchRows,
  percentOf,
  siblingsOf,
  treeRows,
  type Fields,
  type GroupModel,
  type RowModel,
  type Scope,
  type Where,
} from "./model";
import { SubtaskComposer, TaskRow, type RowParts } from "./TaskRow";

export interface TaskListProps {
  /** Left pane of a split view: Close split shows. */
  split?: boolean;
  /** The harness hook: emit(name, fields) with the contract's event names. */
  emit?: (name: string, fields: Record<string, string | number>) => void;
}

const P = (part: string) => `app.tasklist.${part}`;
const IS_MAC = typeof navigator !== "undefined" && /Mac|iPhone|iPad|iPod/.test(navigator.platform);
const FOOTER =
  "flex h-[2.143rem] shrink-0 items-center gap-2 border-t border-edge-1 bg-surface-2 pl-[0.857rem] pr-[0.857rem]";

/**
 * A pointer over a submenu row opens its submenu, as a hover does in the reference. The row takes the ArrowRight
 * that opens a submenu row (MenuItem's own keyboard path), so the open does not depend on which pointer event
 * the pointer delivers.
 */
const openSubmenuOnHover = (e: SyntheticEvent<HTMLElement>) => {
  e.currentTarget.dispatchEvent(new window.KeyboardEvent("keydown", { key: "ArrowRight", bubbles: true, cancelable: true }));
};

/** Contract parts on rows, by title (the rows the tests name). */
const TITLE_PARTS: Record<string, RowParts> = {
  "Post the parcel to Mum": { row: P("rowPostParcel"), check: P("checkPost") },
  "Book bike service": { row: P("rowBookBike"), check: P("checkBike"), badge: P("badgeBike") },
  "Cut a spare key": { check: P("checkCut") },
  "Buy groceries for the week": {
    row: P("rowGroceries"),
    check: P("checkGroceries"),
    badge: P("badgeGroceries"),
    disclosure: P("disclosure"),
  },
  "Sourdough loaf": { check: P("checkSourdough"), more: P("moreSourdough") },
  "Oat milk": { row: P("rowOatMilk") },
  "Buy stamps": { row: P("rowNewSubtask") },
  "Errand number 40": { row: P("rowLastErrand") },
  "Rewrite the hero section": { attach: P("attachmentBadge") },
  "Return library books": { row: P("rowReturnBooks") },
  "Trim the hedge": { row: P("matchTrimHedge") },
};

/** The Reading item of a move menu carries the contract part. */
const targetPart = (listName: string) => (listName === "Reading" ? P("moveTargetReading") : undefined);

interface Source {
  projectId: Id;
  projectName: string;
  lists: ListRecord[];
  tasks: Task[];
}

export function TaskList({ split = false, emit }: TaskListProps) {
  const projectId = useUi((s) => s.projectId);
  const listId = useUi((s) => s.panes[0] ?? null);
  const selection = useUi((s) => s.selection);
  const previewId = useUi((s) => s.previewId);
  const aside = useUi((s) => s.aside);
  const select = useUi((s) => s.select);
  const preview = useUi((s) => s.preview);
  const openList = useUi((s) => s.openList);
  const setProject = useUi((s) => s.setProject);
  const setAside = useUi((s) => s.setAside);

  const projects = useProjects();
  const settings = useSettings();
  const lists = useLists(projectId);
  const tasks = useTasks({ project: projectId });
  const showCompleted = settings.data?.show_completed ?? true;

  const [query, setQuery] = useState("");
  const [scope, setScope] = useState<Scope>("list");
  const [fields, setFields] = useState<Fields>("both");
  const [optionsOpen, setOptionsOpen] = useState(false);
  const [collapsed, setCollapsed] = useState<ReadonlySet<Id>>(() => new Set<Id>());
  const [renaming, setRenaming] = useState<Id | null>(null);
  const [composing, setComposing] = useState<Id | null>(null);
  const [menuFor, setMenuFor] = useState<Id | null>(null);
  const [bulkMoveOpen, setBulkMoveOpen] = useState(false);
  // Where the bulk Move menu opens: at its trigger, so the menu lifts into view above the footer.
  const [bulkPoint, setBulkPoint] = useState({ x: 0, y: 0 });
  const [splitOpen, setSplitOpen] = useState(split);
  const [matchAt, setMatchAt] = useState(-1);
  // Modifiers held: Option (Alt) for the done check, Control or Command for the picking click.
  const [held, setHeld] = useState({ option: false, platform: false });
  const optionHeld = held.option;
  const platformHeld = held.platform;
  const anchor = useRef<Id | null>(null);
  const bodies = useRef(new Map<Id, HTMLElement>());
  const searchRef = useRef<HTMLInputElement | HTMLTextAreaElement | null>(null);

  // The modifier keys: a key press sets its state and its release clears it; an event that carries a modifier
  // flag sets that state; leaving the window clears both.
  useEffect(() => {
    const setOption = (v: boolean) => setHeld((h) => (h.option === v ? h : { ...h, option: v }));
    const setPlatform = (v: boolean) => setHeld((h) => (h.platform === v ? h : { ...h, platform: v }));
    const track = (e: Event) => {
      const f = e as unknown as { key?: string; altKey?: boolean; ctrlKey?: boolean; metaKey?: boolean };
      const down = e.type === "keydown";
      if (f.key === "Alt" || f.key === "Option") setOption(down);
      else if (f.key === "Control" || f.key === "Meta") setPlatform(down);
      else {
        if (f.altKey) setOption(true);
        if (f.ctrlKey || f.metaKey) setPlatform(true);
      }
    };
    const blur = () => setHeld((h) => (h.option || h.platform ? { option: false, platform: false } : h));
    const names = ["keydown", "keyup", "pointerdown", "pointermove", "mousedown", "mousemove"];
    for (const name of names) window.addEventListener(name, track, true);
    window.addEventListener("blur", blur);
    return () => {
      for (const name of names) window.removeEventListener(name, track, true);
      window.removeEventListener("blur", blur);
    };
  }, []);

  // Other projects load only when the search covers all projects.
  const others = (projects.data ?? []).filter((p) => p.id !== projectId);
  const crossScope = scope === "all";
  const otherLists = useQueries({
    queries: others.map((p) => ({
      queryKey: keys.lists(p.id),
      queryFn: () => ep.lists.list(p.id),
      enabled: crossScope,
    })),
  });
  const otherTasks = useQueries({
    queries: others.map((p) => ({
      queryKey: keys.tasks({ project: p.id }),
      queryFn: () => ep.tasks.list({ project: p.id }),
      enabled: crossScope,
    })),
  });

  const activeLists = lists.data ?? [];
  const activeTasks = tasks.data ?? [];
  const activeProject = projects.data?.find((p) => p.id === projectId);
  const listRecord = activeLists.find((l) => l.id === listId);
  const listName = listRecord?.name ?? "";
  const projectName = activeProject?.name ?? "";
  const listTasks = activeTasks.filter((t) => t.list === listId);
  const everyTask = [...activeTasks, ...otherTasks.flatMap((r) => r.data ?? [])];
  const taskOf = (id: Id) => everyTask.find((t) => t.id === id);

  const sources: Source[] = [
    { projectId: projectId ?? 0, projectName, lists: activeLists, tasks: activeTasks },
    ...others.map((p, i) => ({
      projectId: p.id,
      projectName: p.name,
      lists: otherLists[i]?.data ?? [],
      tasks: otherTasks[i]?.data ?? [],
    })),
  ];

  // The rows on screen: the tree, or the matches of a search (grouped by list for the wider scopes).
  const q = query.trim().toLowerCase();
  const searching = q !== "";
  const here: Where = { projectId: projectId ?? 0, projectName, listId: listId ?? 0, listName };
  let groups: GroupModel[];
  if (!searching) {
    groups = [{ key: "list", where: here, rows: treeRows(listTasks, here, collapsed, showCompleted) }];
  } else if (scope === "list") {
    groups = [{ key: "list", where: here, rows: matchRows(listTasks, here, q, fields, showCompleted) }];
  } else {
    const shown = scope === "project" ? sources.slice(0, 1) : sources;
    groups = shown
      .flatMap((src) =>
        [...src.lists]
          .sort((a, b) => a.sort_order - b.sort_order || a.id - b.id)
          .map((l) => {
            const where: Where = { projectId: src.projectId, projectName: src.projectName, listId: l.id, listName: l.name };
            return {
              key: `${src.projectId}:${l.id}`,
              where,
              rows: matchRows(src.tasks.filter((t) => t.list === l.id), where, q, fields, showCompleted),
            };
          }),
      )
      .filter((g) => g.rows.length > 0);
  }
  const visible = groups.flatMap((g) => g.rows);
  const showHeadings = searching && scope !== "list";

  const total = listTasks.length;
  const done = listTasks.filter((t) => t.done).length;
  const pct = percentOf(done, total);

  const firstRoot = visible.find((r) => r.depth === 0);
  const firstSub = visible.find((r) => r.depth > 0);
  const firstMatch = searching ? visible[0] : undefined;
  // Parts of a row. The first root carries row, check and more. With no search typed, those win over a title's own
  // parts (the first root is whichever task is first now, after a move too); while searching the title's parts win.
  // Then the first subtask and the first match add theirs.
  const partsOf = (row: RowModel): RowParts => {
    const firstRootParts: RowParts = row.task.id === firstRoot?.task.id ? { row: P("row"), check: P("check"), more: P("more") } : {};
    const titleParts: RowParts = TITLE_PARTS[row.task.title] ?? {};
    return {
      ...(searching ? { ...firstRootParts, ...titleParts } : { ...titleParts, ...firstRootParts }),
      ...(row.task.id === firstSub?.task.id ? { subtask: P("subtask") } : {}),
      ...(row.task.id === firstMatch?.task.id ? { mark: P("matchMark") } : {}),
    };
  };

  const update = useWrite(ep.tasks.update);
  const bulk = useWrite(ep.tasks.bulk);
  const create = useWrite(ep.tasks.create);
  const move = useWrite(ep.tasks.move);

  const setExpanded = (id: Id, open: boolean) =>
    setCollapsed((prev) => {
      const next = new Set<Id>(prev);
      if (open) next.delete(id);
      else next.add(id);
      return next;
    });
  const focusRow = (id: Id) => bodies.current.get(id)?.focus();
  const focusSearch = () => searchRef.current?.focus();

  const copyTasks = (ids: Id[]) => {
    const lines = ids
      .map((id) => taskOf(id))
      .filter((t): t is Task => t !== undefined)
      .map((t) => `- [${t.done ? "x" : " "}] ${t.title}`);
    void navigator.clipboard?.writeText(lines.join("\n")).catch(() => undefined);
    emit?.("copied", { count: ids.length });
  };

  const toggleDone = (id: Id) => {
    const t = taskOf(id);
    if (!t) return;
    const verb = t.done ? "undone" : "done";
    bulk.mutate([[id, ...descendantIds(everyTask, id)], verb]);
  };

  const togglePick = (id: Id) => {
    anchor.current = id;
    select(selection.includes(id) ? selection.filter((x) => x !== id) : [...selection, id]);
  };

  const moveTo = (ids: Id[], target: Id) => {
    bulk.mutate([ids, "move", target]);
    select(selection.filter((x) => !ids.includes(x)));
  };

  const rename = (id: Id, text: string | null) => {
    setRenaming(null);
    const t = taskOf(id);
    if (text && t && text !== t.title) update.mutate([id, { title: text }]);
  };

  const addSubtask = (parent: Task, title: string) => {
    create.mutate([parent.list, title, { parent: parent.id }]);
    setExpanded(parent.id, true);
  };

  const remove = (t: Task) => {
    void openSheet({ kind: "deleteConfirm", task: t });
  };

  // A cross-list match opens its list with the task selected and previewed.
  const openMatch = (row: RowModel) => {
    if (row.projectId !== projectId) setProject(row.projectId);
    openList(row.listId);
    setQuery("");
    setScope("list");
    setMatchAt(-1);
    preview(row.task.id);
    emit?.("listOpened", { list: row.listName });
    emit?.("taskPreviewed", { title: row.task.title });
  };

  const activate = (row: RowModel, modifier: boolean) => {
    const id = row.task.id;
    if (modifier) {
      togglePick(id);
      return;
    }
    if (searching && row.listId !== listId) {
      openMatch(row);
      return;
    }
    anchor.current = id;
    select([]);
    preview(id);
  };

  // Enter and Shift+Enter in the search field step through the matches, in tree order, and wrap.
  const step = (dir: 1 | -1) => {
    const n = visible.length;
    if (n === 0) return;
    const at = dir === 1 ? (matchAt + 1) % n : matchAt <= 0 ? n - 1 : matchAt - 1;
    setMatchAt(at);
    const row = visible[at];
    if (!row) return;
    focusRow(row.task.id);
    preview(row.task.id);
    emit?.("taskPreviewed", { title: row.task.title });
  };

  // A shift-press on a row selects the visible rows between the anchor and that row.
  const rangeTo = (row: RowModel) => {
    const idx = visible.findIndex((r) => r.task.id === row.task.id);
    if (idx < 0) return;
    const a = anchor.current == null ? -1 : visible.findIndex((r) => r.task.id === anchor.current);
    const from = a >= 0 ? a : idx;
    select(visible.slice(Math.min(from, idx), Math.max(from, idx) + 1).map((r) => r.task.id));
  };

  const extend = (idx: number, dir: 1 | -1) => {
    if (idx < 0) return;
    const next = Math.min(Math.max(idx + dir, 0), visible.length - 1);
    const a = anchor.current == null ? -1 : visible.findIndex((r) => r.task.id === anchor.current);
    const from = a >= 0 ? a : idx;
    const lo = Math.min(from, next);
    const hi = Math.max(from, next);
    select(visible.slice(lo, hi + 1).map((r) => r.task.id));
    const target = visible[next];
    if (target) focusRow(target.task.id);
  };

  const onRowKey = (e: KeyboardEvent<HTMLDivElement>, row: RowModel) => {
    const target = e.target as HTMLElement;
    if (!e.currentTarget.contains(target) || target.closest("[data-row-control]")) return;
    const id = row.task.id;
    const idx = visible.findIndex((r) => r.task.id === id);
    const modifier = IS_MAC ? e.metaKey : e.ctrlKey;

    if (e.altKey && e.code === "KeyN") {
      e.preventDefault();
      emit?.("draftStarted", { list: listName });
      return;
    }
    if (e.altKey && (e.code === "Space" || e.key === " ")) {
      e.preventDefault();
      if (selection.length > 0 && selection.includes(id)) {
        const allDone = selection.every((x) => taskOf(x)?.done === true);
        bulk.mutate([selection, allDone ? "undone" : "done"]);
      } else {
        toggleDone(id);
      }
      return;
    }
    if (modifier && e.code === "KeyA") {
      e.preventDefault();
      select(visible.map((r) => r.task.id));
      return;
    }
    if (e.altKey || e.ctrlKey || e.metaKey) return;

    switch (e.key) {
      case "ArrowDown":
      case "ArrowUp": {
        e.preventDefault();
        const dir = e.key === "ArrowDown" ? 1 : -1;
        if (e.shiftKey) {
          extend(idx, dir);
          return;
        }
        const next = visible[idx + dir];
        if (next) focusRow(next.task.id);
        return;
      }
      case "ArrowRight": {
        if (row.kids > 0 && !row.expanded) {
          e.preventDefault();
          setExpanded(id, true);
        } else if (row.kids > 0) {
          const next = visible[idx + 1];
          if (next && next.depth > row.depth) {
            e.preventDefault();
            focusRow(next.task.id);
          }
        }
        return;
      }
      case "ArrowLeft": {
        if (row.kids > 0 && row.expanded) {
          e.preventDefault();
          setExpanded(id, false);
          return;
        }
        if (row.depth > 0) {
          for (let i = idx - 1; i >= 0; i--) {
            const r = visible[i];
            if (r && r.depth < row.depth) {
              e.preventDefault();
              focusRow(r.task.id);
              break;
            }
          }
        }
        return;
      }
      case "Tab": {
        const t = row.task;
        if (e.shiftKey) {
          if (t.parent == null) return;
          e.preventDefault();
          move.mutate([id, { parent: null }]);
          return;
        }
        const sibs = siblingsOf(everyTask, t);
        const at = sibs.findIndex((s) => s.id === id);
        const prev = at > 0 ? sibs[at - 1] : undefined;
        if (prev) {
          e.preventDefault();
          move.mutate([id, { parent: prev.id }]);
        }
        return;
      }
      case " ":
      case "Spacebar":
      case "Enter":
        // On the check these are the press of the check; on the row they are the press of the row.
        e.preventDefault();
        if (target.dataset.slot === "check") togglePick(id);
        else activate(row, false);
        return;
      case "Escape":
        if (selection.length > 0) select([]);
        return;
      case "F2":
        e.preventDefault();
        setRenaming(id);
        return;
      case "/":
        e.preventDefault();
        focusSearch();
        return;
      default:
        // A typed letter starts a search in the list, seeded with that letter.
        if (e.key.length === 1) {
          e.preventDefault();
          setQuery(e.key);
          setMatchAt(-1);
          focusSearch();
        }
    }
  };

  // Only the field steps through matches; the buttons in the well keep their own keys.
  const onSearchKey = (e: KeyboardEvent<HTMLDivElement>) => {
    if (e.target !== searchRef.current) return;
    e.stopPropagation();
    if (e.key === "Enter") {
      e.preventDefault();
      step(e.shiftKey ? -1 : 1);
    }
  };

  // The More menu of a row: the single-task actions.
  const rowMenu = (row: RowModel, parts: RowParts) => {
    const t = row.task;
    const targets = activeLists.filter((l) => l.id !== row.listId);
    return (
      <Menu
        open={menuFor === t.id}
        onOpenChange={(open) => setMenuFor(open ? t.id : null)}
        placement="control"
        width="task"
        ariaLabel="Task menu"
        trigger={
          <Button
            data-part={parts.more}
            data-row-control
            variant="ghost"
            size="sm"
            shape="icon"
            tone="muted"
            icon="more_horiz"
            iconSize={16}
            ariaLabel="More"
          />
        }
      >
        <div data-part={P("rowMenu")} className="flex flex-col">
          <MenuItem data-part={P("menuCopy")} label="Copy" icon="content_copy" onSelect={() => copyTasks([t.id])} />
          <MenuItem
            data-part={P("menuAddSubtask")}
            label="Add subtask"
            icon="subdirectory_arrow_right"
            onSelect={() => setComposing(t.id)}
          />
          <MenuItem data-part={P("menuRename")} label="Rename" icon="edit" onSelect={() => setRenaming(t.id)} />
          <MenuItem
            data-part={P("menuMoveToList")}
            label="Move to list"
            icon="arrow_forward"
            trailing="submenu"
            disabled={targets.length === 0}
            onPointerEnter={openSubmenuOnHover}
            onPointerMove={openSubmenuOnHover}
            onMouseEnter={openSubmenuOnHover}
            onMouseMove={openSubmenuOnHover}
            submenu={
              <MenuSubmenuPanel ariaLabel="Move to list">
                {targets.map((l) => (
                  <MenuItem
                    key={l.id}
                    data-part={targetPart(l.name)}
                    label={l.name}
                    onSelect={() => moveTo([t.id], l.id)}
                  />
                ))}
              </MenuSubmenuPanel>
            }
          />
          {t.parent != null && (
            <MenuItem
              data-part={P("menuPromote")}
              label="Promote to top level"
              icon="north_west"
              onSelect={() => move.mutate([t.id, { parent: null }])}
            />
          )}
          <Separator inset="menu" />
          <MenuItem
            data-part={P("menuDelete")}
            label="Delete task…"
            icon="delete"
            tone="danger"
            onSelect={() => remove(t)}
          />
        </div>
      </Menu>
    );
  };

  const rowFor = (row: RowModel) => (
    <Fragment key={`${row.listId}:${row.task.id}`}>
      <TaskRow
        row={row}
        query={query}
        searching={searching}
        picked={selection.includes(row.task.id)}
        previewed={previewId === row.task.id}
        optionHeld={optionHeld}
        parts={partsOf(row)}
        renaming={renaming === row.task.id}
        menu={rowMenu(row, partsOf(row))}
        onActivate={(r, modifier, shift) => {
          if (shift) rangeTo(r);
          else activate(r, modifier || platformHeld);
        }}
        onCheck={(r, alt) => {
          if (alt || optionHeld) toggleDone(r.task.id);
          else togglePick(r.task.id);
        }}
        onDoubleToggle={(r) => toggleDone(r.task.id)}
        onContext={(r) => setMenuFor(r.task.id)}
        onToggleExpanded={(r) => setExpanded(r.task.id, !r.expanded)}
        onRenamed={(id, text) => rename(id, text)}
        onKey={onRowKey}
        registerBody={(id, el) => {
          if (el) bodies.current.set(id, el);
          else bodies.current.delete(id);
        }}
      />
      {composing === row.task.id && (
        <SubtaskComposer
          depth={row.depth + 1}
          onSubmit={(title) => addSubtask(row.task, title)}
          onDismiss={() => setComposing(null)}
        />
      )}
    </Fragment>
  );

  const headingPart = (g: GroupModel, i: number) =>
    i === 0 ? P("groupHeading") : g.where.listName === "Reading" ? P("groupHeadingReading") : undefined;

  const emptyBlock = searching ? (
    <EmptyState
      data-part={P("emptyState")}
      scope="list"
      icon="search_off"
      title={`No matches in ${listName}`}
      subtitle="Try a different search or scope"
    />
  ) : listRecord && total === 0 ? (
    <EmptyState
      data-part={P("emptyState")}
      scope="list"
      icon="landscape"
      title="Nothing here yet"
      subtitle="Press + to add a task"
    />
  ) : null;

  const moveTargets = activeLists.filter((l) => l.id !== listId);

  const searchMenu = (
    <Menu
      open={optionsOpen}
      onOpenChange={setOptionsOpen}
      placement="control"
      width="content"
      ariaLabel="Search options"
      trigger={
        <Button
          data-part={P("searchOptions")}
          variant="ghost"
          size="sm"
          shape="icon"
          tone="muted"
          icon="tune"
          ariaLabel="Search options"
        />
      }
    >
      <div data-part={P("searchMenu")} className="flex flex-col">
        <MenuItem
          selection="radio"
          value="list"
          checked={scope === "list"}
          data-part={P("scopeThisList")}
          label="This list"
          onSelect={() => setScope("list")}
        />
        <MenuItem
          selection="radio"
          value="project"
          checked={scope === "project"}
          data-part={P("scopeWholeProject")}
          label="Whole project"
          onSelect={() => setScope("project")}
        />
        <MenuItem
          selection="radio"
          value="all"
          checked={scope === "all"}
          data-part={P("scopeAllProjects")}
          label="All projects"
          onSelect={() => setScope("all")}
        />
        <Separator inset="menu" />
        <MenuItem
          selection="radio"
          value="both"
          checked={fields === "both"}
          data-part={P("fieldsTitleAndNotes")}
          label="Title & notes"
          onSelect={() => setFields("both")}
        />
        <MenuItem
          selection="radio"
          value="title"
          checked={fields === "title"}
          data-part={P("fieldsTitleOnly")}
          label="Title only"
          onSelect={() => setFields("title")}
        />
        <MenuItem
          selection="radio"
          value="notes"
          checked={fields === "notes"}
          data-part={P("fieldsNotesOnly")}
          label="Notes only"
          onSelect={() => setFields("notes")}
        />
      </div>
    </Menu>
  );

  return (
    <div className={cx("flex h-full min-h-0 min-w-0 flex-1 flex-col", split && "border-r border-edge-1")}>
      <div className="flex shrink-0 items-center gap-2 px-[1rem] pb-[0.5rem] pt-[0.714rem]">
        <div
          className="flex h-[2.5rem] min-w-0 flex-1 items-center gap-[0.357rem] rounded-full bg-wash-2 pl-[0.643rem] pr-[0.357rem] focus-within:ring-2 focus-within:ring-focus-1"
          onKeyDown={onSearchKey}
        >
          <Icon name="search" size={18} tone="muted" />
          <Input
            data-part={P("search")}
            ref={searchRef}
            mode="bare"
            size="md"
            value={query}
            placeholder={`Search ${listName}`}
            ariaLabel={`Search ${listName}`}
            spellCheck={false}
            className="min-w-0 flex-1 focus:ring-0"
            onValueChange={(value) => {
              setQuery(value);
              setMatchAt(-1);
            }}
            onCancel={() => {
              setQuery("");
              setMatchAt(-1);
              searchRef.current?.blur();
            }}
          />
          {searching && (
            <Text
              data-part={P("matchCount")}
              role="status"
              as="span"
              size="xs"
              weight="semibold"
              tone="muted"
              numeric="tabular"
            >
              {String(visible.length)}
            </Text>
          )}
          {searching && (
            <Button
              data-part={P("clearSearch")}
              variant="ghost"
              size="sm"
              shape="icon"
              tone="muted"
              icon="close"
              iconSize={14}
              ariaLabel="Clear search"
              onClick={() => {
                setQuery("");
                setMatchAt(-1);
                focusSearch();
              }}
            />
          )}
          {searchMenu}
          <span aria-hidden="true" className="block h-4 w-px shrink-0 bg-edge-2" />
          <Button
            data-part={P("newTask")}
            variant="ghost"
            size="sm"
            shape="icon"
            tone="muted"
            icon="add"
            iconSize={18}
            ariaLabel="New task"
            onClick={() => emit?.("draftStarted", { list: listName })}
          />
          {splitOpen && (
            <Button
              data-part={P("closeSplit")}
              variant="ghost"
              size="sm"
              shape="icon"
              tone="muted"
              icon="close"
              iconSize={16}
              ariaLabel="Close split"
              onClick={() => {
                setSplitOpen(false);
                emit?.("splitClosed", { list: listName });
              }}
            />
          )}
        </div>
      </div>

      <div
        data-part={P("scrollArea")}
        // A scrolling region takes focus, so the keyboard (Page Down, Space) scrolls it.
        tabIndex={0}
        className="min-h-0 flex-1 overflow-y-auto overscroll-contain pb-2 outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-focus-1"
        // A click on blank space (not a row, a control, a field or a menu) clears the selection and the preview.
        onClick={(e) => {
          const target = e.target as HTMLElement;
          if (!e.currentTarget.contains(target) || target.closest("[data-task-id], button, input, textarea, [role='menu']")) return;
          select([]);
          preview(null);
        }}
      >
        {groups.map((g, gi) => (
          <Fragment key={g.key}>
            {showHeadings && (
              <Heading
                as="h3"
                size="sm"
                data-part={headingPart(g, gi)}
                className="px-[1rem] pb-[0.286rem] pt-[0.643rem] text-[0.857rem] text-content-2"
              >
                {scope === "all" ? `${g.where.projectName} · ${g.where.listName}` : g.where.listName}
              </Heading>
            )}
            {g.rows.map(rowFor)}
          </Fragment>
        ))}
        {visible.length === 0 && emptyBlock}
      </div>

      {/* Both footers stay in the page; the one that does not apply is hidden. */}
      <div data-part={P("bulkBar")} className={cx(FOOTER, selection.length === 0 && "hidden")}>
          <Text
            data-part={P("selectionCount")}
            role="status"
            as="span"
            size="xs"
            weight="semibold"
            tone="primary"
            numeric="tabular"
            className="shrink-0 whitespace-nowrap"
          >
            {`${selection.length} selected`}
          </Text>
          <span aria-hidden="true" className="block h-3 w-px bg-edge-2" />
          <Button
            data-part={P("bulkDone")}
            variant="ghost"
            size="footer"
            tone="muted"
            icon="task_alt"
            ariaLabel="Mark done"
            label="Mark done"
            onClick={() => bulk.mutate([selection, "done"])}
          />
          <Button
            data-part={P("bulkUndone")}
            variant="ghost"
            size="footer"
            tone="muted"
            icon="radio_button_unchecked"
            ariaLabel="Mark not done"
            label="Mark not done"
            onClick={() => bulk.mutate([selection, "undone"])}
          />
          <Menu
            open={bulkMoveOpen}
            onOpenChange={(open) => setBulkMoveOpen(open)}
            placement="anchored"
            pointX={bulkPoint.x}
            pointY={bulkPoint.y}
            width="task"
            ariaLabel="Move to list"
            trigger={
              <Button
                data-part={P("bulkMove")}
                variant="ghost"
                size="footer"
                tone="muted"
                icon="arrow_forward"
                ariaLabel="Move to list"
                label="Move to list"
                disabled={moveTargets.length === 0}
                onClick={(e) => {
                  const r = e.currentTarget.getBoundingClientRect();
                  setBulkPoint({ x: r.left, y: r.top });
                  setBulkMoveOpen(true);
                }}
              />
            }
          >
            {moveTargets.map((l) => (
              <MenuItem
                key={l.id}
                data-part={targetPart(l.name)}
                label={l.name}
                onSelect={() => moveTo(selection, l.id)}
              />
            ))}
          </Menu>
          <Button
            data-part={P("bulkCopy")}
            variant="ghost"
            size="footer"
            tone="muted"
            icon="content_copy"
            ariaLabel="Copy"
            label="Copy"
            onClick={() => copyTasks(selection)}
          />
          <Button
            data-part={P("bulkDelete")}
            variant="ghost"
            size="footer"
            tone="danger"
            icon="delete"
            ariaLabel="Delete"
            label="Delete"
            disabled={selection.length !== 1}
            onClick={() => {
              const t = taskOf(selection[0] ?? -1);
              if (t) remove(t);
            }}
          />
          <Button
            data-part={P("bulkClear")}
            variant="ghost"
            size="footer"
            tone="muted"
            icon="close"
            ariaLabel="Clear"
            label="Clear"
            className="ml-auto"
            onClick={() => select([])}
          />
      </div>
      <div className={cx(FOOTER, selection.length > 0 && "hidden")}>
          <Button
            data-part={P("sortFilter")}
            variant="ghost"
            size="footer"
            tone="muted"
            icon="tune"
            ariaLabel="Sort & filter"
            label={aside === "filter" ? "Filtered" : ""}
            onClick={() => setAside(aside === "filter" ? "preview" : "filter")}
          />
          <span aria-hidden="true" className="block h-4 w-px bg-edge-2" />
          <Text
            data-part={P("doneCount")}
            role="status"
            as="span"
            size="xs"
            weight="semibold"
            tone="muted"
            case="uppercase"
            numeric="tabular"
          >
            {`${done}/${total} done`}
          </Text>
          <ProgressBar className="ml-auto" value={done} max={Math.max(total, 1)} ariaLabel="Done progress" />
          <Text data-part={P("percent")} as="span" size="xs" weight="semibold" tone="secondary" numeric="tabular">
            {`${pct}%`}
          </Text>
      </div>
    </div>
  );
}
