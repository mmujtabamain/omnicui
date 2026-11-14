// The command palette (contract app.palette): a search input over the commands and the jump rows,
// grouped under View, Task, Manage, Go to list and Go to project. It is open while the store's
// paletteOpen is set. A run closes it, then reports the row's visible label through onRan.
import { Fragment, useEffect, useId, useLayoutEffect, useRef, useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { CommandInput } from "@taskscape/ui/command-input";
import { commandPaletteVariants } from "@taskscape/ui/command-palette";
import { ComboboxItem } from "@taskscape/ui/combobox-item";
import { Overlay, OverlayTitle } from "@taskscape/ui/overlay";
import { Scrim } from "@taskscape/ui/scrim";
import { Text } from "@taskscape/ui/text";
import * as ep from "../../api/endpoints";
import { keys, useLists, useProjects, useSettings, useTasks, useWrite } from "../../api/queries";
import { useUi } from "../../state/ui";

type Group = "View" | "Task" | "Manage" | "Go to list" | "Go to project";

interface PaletteRow {
  id: string;
  group: Group;
  /** The visible text: the filter matches it and a run reports it. */
  label: string;
  icon: string;
  /** A web accelerator (Mod is Ctrl on Windows and Linux, Cmd on macOS); empty for none. */
  accel: string;
  /** The contract part of this row, when it has one (rowSwapPanes, rowDelete, rowSpringLaunch). */
  part: string;
  run: () => void;
}

const GROUP_PART: Record<Group, string> = {
  View: "app.palette.groupView",
  Task: "app.palette.groupTask",
  Manage: "app.palette.groupManage",
  "Go to list": "app.palette.groupGoList",
  "Go to project": "app.palette.groupGoProject",
};

const ui = () => useUi.getState();
const noop = () => undefined;

function row(r: Pick<PaletteRow, "id" | "group" | "label" | "icon" | "run"> & Partial<Pick<PaletteRow, "accel" | "part">>): PaletteRow {
  return { accel: "", part: "", ...r };
}

/** Mounted by the app shell; it renders only while the palette is open. */
export function AppPalette({ onRan }: { onRan?: (label: string) => void }) {
  const open = useUi((s) => s.paletteOpen);
  return open ? <PaletteDialog onRan={onRan} /> : null;
}

interface PaletteSearchProps {
  value: string;
  activeDescendantId: string;
  onValueChange: (value: string) => void;
  onMoveActive: (direction: "up" | "down") => void;
  onSubmit: () => void;
}

/**
 * The command input. It sits in a component of its own so its layout effect runs once the
 * dialog's portal has mounted; the inner field then carries the search part name.
 */
function PaletteSearch({ value, activeDescendantId, onValueChange, onMoveActive, onSubmit }: PaletteSearchProps) {
  const root = useRef<HTMLDivElement>(null);
  useLayoutEffect(() => {
    root.current?.querySelector('[data-slot="input"]')?.setAttribute("data-part", "app.palette.search");
  }, []);
  return (
    <CommandInput
      ref={root}
      data-slot="palette-search"
      className="shrink-0"
      placeholder="Run a command or jump to…"
      ariaLabel="Search commands"
      value={value}
      onValueChange={onValueChange}
      onMoveActive={onMoveActive}
      onSubmit={() => onSubmit()}
      activeDescendantId={activeDescendantId}
    />
  );
}

function PaletteDialog({ onRan }: { onRan?: (label: string) => void }) {
  const base = useId();
  const [query, setQuery] = useState("");
  const [active, setActive] = useState(0);

  const projectId = useUi((s) => s.projectId);
  const focusedList = useUi((s) => s.panes[s.activePane] ?? null);
  const selection = useUi((s) => s.selection);

  const projects = useProjects();
  const lists = useLists(projectId);
  const tasks = useTasks({ list: focusedList });
  const settings = useSettings();
  const qc = useQueryClient();
  const saveSettings = useWrite(ep.settings.update);
  const createTask = useWrite(ep.tasks.create);
  const trash = useWrite(ep.tasks.trash);

  // The one selected task is the Delete target; the Delete row is listed only for it.
  const selected = selection.length === 1 ? tasks.data?.find((t) => t.id === selection[0]) : undefined;

  const close = () => ui().setPalette(false);
  const optionId = (index: number) => `${base}-option-${index}`;

  const toggleShowCompleted = () => {
    const shown = settings.data?.show_completed ?? false;
    void saveSettings
      .mutateAsync([{ show_completed: !shown }])
      .then(() => qc.invalidateQueries({ queryKey: keys.settings }));
  };

  const newTask = () => {
    const list = ui().panes[ui().activePane];
    if (list == null) return;
    void createTask.mutateAsync([list, "Untitled"]).then((task) => ui().preview(task.id));
  };

  const deleteTask = (taskId: number) => {
    void trash.mutateAsync([taskId]);
    ui().select([]);
    ui().preview(null);
  };

  const commands: PaletteRow[] = [
    row({ id: "toggle-preview", group: "View", label: "Toggle preview", icon: "right_panel_open", accel: "Mod+\\",
      run: () => ui().setAside(ui().aside === "preview" ? "none" : "preview") }),
    row({ id: "toggle-split", group: "View", label: "Toggle split view", icon: "vertical_split", accel: "Mod+Alt+S",
      run: () => ui().split(ui().panes.length < 2) }),
    row({ id: "swap-panes", group: "View", label: "Swap panes", icon: "swap_horiz", part: "rowSwapPanes",
      run: () => useUi.setState((s) => ({ panes: [...s.panes].reverse() })) }),
    row({ id: "show-completed", group: "View", label: "Show / hide completed", icon: "visibility", accel: "Mod+Shift+H",
      run: toggleShowCompleted }),
    row({ id: "collapse-all", group: "View", label: "Collapse all", icon: "unfold_less", accel: "Mod+Alt+-", run: noop }),
    row({ id: "expand-all", group: "View", label: "Expand all", icon: "unfold_more", accel: "Mod+Alt+=", run: noop }),
    row({ id: "new-task", group: "Task", label: "New task", icon: "add", accel: "Alt+N", run: newTask }),
    row({ id: "search-list", group: "Task", label: "Search this list", icon: "search", accel: "/", run: noop }),
    row({ id: "undo", group: "Task", label: "Undo", icon: "undo", accel: "Mod+Z", run: noop }),
    row({ id: "redo", group: "Task", label: "Redo", icon: "redo", accel: "Mod+Shift+Z", run: noop }),
    row({ id: "new-list", group: "Manage", label: "New list", icon: "playlist_add", accel: "Alt+Shift+N", run: noop }),
    row({ id: "new-project", group: "Manage", label: "New project", icon: "create_new_folder", run: noop }),
    row({ id: "open-trash", group: "Manage", label: "Open Trash", icon: "delete", run: () => ui().setAside("trash") }),
    row({ id: "settings", group: "Manage", label: "Settings", icon: "settings", accel: "Mod+Comma",
      run: () => ui().setSettings(true) }),
  ];

  // Order: the commands, then the Delete row under a repeated Task heading, then the jump rows.
  const all: PaletteRow[] = [
    ...commands,
    ...(selected
      ? [row({ id: "delete-task", group: "Task", label: `Delete “${selected.title}”`, icon: "delete", part: "rowDelete",
          run: () => deleteTask(selected.id) })]
      : []),
    ...(lists.data ?? []).map((l) =>
      row({ id: `list-${l.id}`, group: "Go to list", label: l.name, icon: "list", run: () => ui().openList(l.id) }),
    ),
    ...(projects.data ?? []).map((p) =>
      row({ id: `project-${p.id}`, group: "Go to project", label: p.name, icon: "folder",
        part: p.name === "Spring launch" ? "rowSpringLaunch" : "", run: () => ui().setProject(p.id) }),
    ),
  ];

  const needle = query.trim().toLowerCase();
  const rows = needle === "" ? all : all.filter((r) => r.label.toLowerCase().includes(needle));
  const last = Math.max(rows.length - 1, 0);
  const current = rows.length > 0 ? Math.min(active, last) : 0;

  const step = (direction: "up" | "down") =>
    setActive((a) => Math.min(Math.max(Math.min(a, last) + (direction === "down" ? 1 : -1), 0), last));

  const run = (index: number) => {
    const target = rows[index];
    if (!target) return;
    close();
    target.run();
    onRan?.(target.label);
  };

  // The active row is kept in view.
  useEffect(() => {
    if (rows.length === 0) return;
    document.getElementById(optionId(current))?.scrollIntoView({ block: "nearest" });
  }, [current, rows.length]);

  return (
    <>
      <Scrim data-part="app.palette.scrim" dim="soft" />
      <Overlay
        open
        placement="top"
        onDismiss={close}
        data-part="app.palette.panel"
        className={commandPaletteVariants({ presentation: "overlay" })}
      >
        <OverlayTitle className="sr-only">Command palette</OverlayTitle>
        <PaletteSearch
          value={query}
          activeDescendantId={rows.length > 0 ? optionId(current) : ""}
          onValueChange={(value) => {
            setQuery(value);
            setActive(0);
          }}
          onMoveActive={step}
          onSubmit={() => run(current)}
        />
        {rows.length === 0 ? (
          <Text as="p" data-part="app.palette.emptyMessage" size="md" tone="muted" align="center" className="px-space-7 py-6">
            No matching commands
          </Text>
        ) : (
          <div data-part="app.palette.results" className="min-h-0 flex-1 overflow-y-auto pb-space-3">
            {rows.map((r, index) => {
              const heading = index === 0 || rows[index - 1].group !== r.group;
              const firstOfGroup = rows.findIndex((x) => x.group === r.group) === index;
              // A row's own part wins; otherwise the first row is the contract's "row".
              const part = r.part ? `app.palette.${r.part}` : index === 0 ? "app.palette.row" : undefined;
              return (
                <Fragment key={r.id}>
                  {heading && (
                    <Text
                      as="div"
                      size="xs"
                      weight="semibold"
                      tone="muted"
                      case="uppercase"
                      data-part={firstOfGroup ? GROUP_PART[r.group] : undefined}
                      className="mt-space-3 px-space-6"
                    >
                      {r.group}
                    </Text>
                  )}
                  <ComboboxItem
                    id={optionId(index)}
                    role="button"
                    aria-selected={undefined}
                    data-part={part}
                    icon={r.icon}
                    label={r.label}
                    shortcut={r.accel}
                    active={index === current}
                    className="h-[2.286rem]"
                    onSelect={() => run(index)}
                    onHighlight={() => setActive(index)}
                  />
                </Fragment>
              );
            })}
          </div>
        )}
      </Overlay>
    </>
  );
}
