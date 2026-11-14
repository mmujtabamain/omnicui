// The web title bar (contract app.appbar): the project switcher pill, the list tabs for the active
// project with New list, and the toolbar (reduced-motion indicator, Swap panes, Split view, preview,
// Trash, command palette, Settings, account). The bar owns the view state (project, lists in the
// panes, preview, palette, settings, trash) and emits a request event for each project and list
// action. The sheets own the commits, so create, rename and delete open a sheet.
import * as React from "react";
import { Button } from "@taskscape/ui/button";
import { Icon } from "@taskscape/ui/icon";
import { Menu } from "@taskscape/ui/menu";
import { MenuItem } from "@taskscape/ui/menu-item";
import { Separator } from "@taskscape/ui/separator";
import { StatusIndicator } from "@taskscape/ui/status-indicator";
import { Text } from "@taskscape/ui/text";
import * as ep from "../../api/endpoints";
import { useLists, useMe, useProjects, useSettings, useWrite } from "../../api/queries";
import type { Project, TaskList } from "../../api/types";
import { useUi } from "../../state/ui";
import { openSheet } from "../sheets";
import { ProjectRow } from "./ProjectSwitcher";
import { ListTabs } from "./ListTabs";

/** Sends one event to the page's runner: the name and its declared fields. */
export type BarEmit = (name: string, payload: Record<string, unknown>) => void;

export interface TitleBarProps {
  emit?: BarEmit;
}

const byOrder = <T extends { sort_order: number; id: number }>(a: T, b: T) => a.sort_order - b.sort_order || a.id - b.id;

interface ChordHandlers {
  nextProject(): void;
  prevProject(): void;
  nextList(): void;
  prevList(): void;
  nthList(index: number): void;
  newList(): void;
}

export function TitleBar({ emit }: TitleBarProps) {
  const emitRef = React.useRef(emit);
  emitRef.current = emit;
  const send = (name: string, payload: Record<string, unknown>) => emitRef.current?.(name, payload);

  const me = useMe();
  const settings = useSettings();
  const projectsQuery = useProjects();
  const projects = React.useMemo(() => [...(projectsQuery.data ?? [])].sort(byOrder), [projectsQuery.data]);

  const projectId = useUi((s) => s.projectId);
  const panes = useUi((s) => s.panes);
  const aside = useUi((s) => s.aside);

  const activeProject: Project | null = projects.find((p) => p.id === projectId) ?? projects[0] ?? null;
  const listsQuery = useLists(activeProject?.id);
  const lists = React.useMemo(() => [...(listsQuery.data ?? [])].sort(byOrder), [listsQuery.data]);

  // The left pane is the active list; the right pane exists only while split is on.
  const leftList = lists.find((list) => list.id === panes[0]) ?? lists[0] ?? null;
  const splitOn = panes.length > 1;
  const rightList = splitOn ? (lists.find((list) => list.id === panes[1]) ?? leftList) : null;
  const previewOpen = aside === "preview";

  const [projectOpen, setProjectOpen] = React.useState(false);
  const [accountOpen, setAccountOpen] = React.useState(false);
  const [accountAnchor, setAccountAnchor] = React.useState({ x: 0, y: 0 });
  const [renamingId, setRenamingId] = React.useState<number | null>(null);

  const renameList = useWrite((id: number, name: string) => ep.lists.update(id, { name }));
  // Drag reorder writes each list's new position; a task dropped on a tab moves into that list.
  const reorderLists = useWrite((moves: [number, number][]) =>
    Promise.all(moves.map(([id, sortOrder]) => ep.lists.update(id, { sort_order: sortOrder }))),
  );
  const moveTasks = useWrite((ids: number[], listId: number) => Promise.all(ids.map((id) => ep.tasks.move(id, { list: listId }))));

  // The project menu closes on Escape or an outside press, but a press on the pill itself keeps it open.
  // The window's capture listener records whether the last press landed on the pill.
  const pillRef = React.useRef<HTMLButtonElement>(null);
  const pressedPill = React.useRef(false);
  const [, bump] = React.useState(0);
  React.useEffect(() => {
    const onDown = (event: PointerEvent) => {
      pressedPill.current = pillRef.current?.contains(event.target as Node) ?? false;
    };
    window.addEventListener("pointerdown", onDown, true);
    return () => window.removeEventListener("pointerdown", onDown, true);
  }, []);

  // Project actions.
  const pickProject = (project: Project) => {
    setProjectOpen(false);
    if (project.id === activeProject?.id) return;
    useUi.getState().setProject(project.id);
    send("projectSelected", { name: project.name });
  };
  const requestNewProject = () => {
    setProjectOpen(false);
    send("projectCreateRequested", { activeProject: activeProject?.name ?? "" });
    void openSheet({ kind: "newProject" });
  };
  const requestRenameProject = (project: Project) => {
    setProjectOpen(false);
    send("projectRenameRequested", { name: project.name });
    void openSheet({ kind: "renameProject", project });
  };
  const requestDeleteProject = (project: Project) => {
    setProjectOpen(false);
    send("projectDeleteRequested", { name: project.name });
    void openSheet({ kind: "deleteProject", project });
  };

  // List actions.
  const pickList = (list: TaskList) => {
    if (list.id === leftList?.id) return;
    useUi.getState().openList(list.id, 0);
    send("listSelected", { name: list.name });
  };
  const requestNewList = () => {
    if (!activeProject) return;
    send("listCreateRequested", { project: activeProject.name });
    void openSheet({ kind: "newList", projectId: activeProject.id });
  };
  const requestRenameList = (list: TaskList) => {
    send("listRenameRequested", { name: list.name });
    void openSheet({ kind: "renameList", list });
  };
  const requestDeleteList = (list: TaskList) => {
    send("listDeleteRequested", { name: list.name });
    void openSheet({ kind: "deleteList", list });
  };
  const requestExportList = (list: TaskList) => {
    send("listExportRequested", { name: list.name });
  };
  const commitRename = (list: TaskList, text: string) => {
    setRenamingId(null);
    const name = text.trim();
    if (name === "" || name === list.name) return;
    renameList.mutateAsync([list.id, name]).then(
      () => send("listRenamed", { name }),
      () => undefined,
    );
  };

  // Drag reorder: the list moves to index `to`; every list whose position changed is written.
  const moveList = (from: number, to: number) => {
    if (from === to || from < 0 || from >= lists.length || to < 0 || to >= lists.length) return;
    const next = [...lists];
    const [moved] = next.splice(from, 1);
    next.splice(to, 0, moved);
    const moves = next
      .map((list, index) => [list.id, index] as [number, number])
      .filter(([id, index]) => lists.find((list) => list.id === id)?.sort_order !== index);
    if (moves.length > 0) reorderLists.mutateAsync([moves]).catch(() => undefined);
  };
  const dropTasksOnList = (taskIds: number[], list: TaskList) => {
    moveTasks.mutateAsync([taskIds, list.id]).catch(() => undefined);
  };

  // Split view and panes. The left pane stays the active one, so the store's activePane is 0.
  const splitWith = (list: TaskList | null) => {
    if (!leftList) return;
    if (splitOn) {
      useUi.setState({ panes: [leftList.id], activePane: 0 });
      send("splitToggled", { active: false });
      return;
    }
    useUi.setState({ panes: [leftList.id, (list ?? leftList).id], activePane: 0 });
    send("splitToggled", { active: true });
  };
  const swapPanes = () => {
    if (!leftList || !rightList) return;
    useUi.setState({ panes: [rightList.id, leftList.id], activePane: 0 });
    send("panesSwapped", { left: rightList.name });
  };

  // Toolbar.
  const togglePreview = () => {
    const open = !previewOpen;
    useUi.getState().setAside(open ? "preview" : "none");
    send("previewToggled", { open });
  };
  const openTrash = () => {
    useUi.getState().setAside("trash");
    send("trashOpened", { open: true });
  };
  const openPalette = () => {
    useUi.getState().setPalette(true);
    send("paletteOpened", { open: true });
  };
  const openSettings = () => {
    setAccountOpen(false);
    useUi.getState().setSettings(true);
    send("settingsOpened", { open: true });
  };
  const signOut = () => {
    setAccountOpen(false);
    send("signOutRequested", { signedIn: true });
  };

  // Keyboard chords: project and list cycling, and the list number and new list chords. Ignored
  // while a text field has focus.
  const chords = React.useRef<ChordHandlers | null>(null);
  React.useEffect(() => {
    chords.current = {
      nextProject: () => {
        if (projects.length === 0) return;
        const index = projects.findIndex((p) => p.id === activeProject?.id);
        pickProject(projects[(index + 1) % projects.length]);
      },
      prevProject: () => {
        if (projects.length === 0) return;
        const index = projects.findIndex((p) => p.id === activeProject?.id);
        pickProject(projects[(index - 1 + projects.length) % projects.length]);
      },
      nextList: () => {
        if (lists.length === 0) return;
        const index = lists.findIndex((l) => l.id === leftList?.id);
        pickList(lists[(index + 1) % lists.length]);
      },
      prevList: () => {
        if (lists.length === 0) return;
        const index = lists.findIndex((l) => l.id === leftList?.id);
        pickList(lists[(index - 1 + lists.length) % lists.length]);
      },
      nthList: (index) => {
        if (lists[index]) pickList(lists[index]);
      },
      newList: requestNewList,
    };
  });
  React.useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (!event.altKey || event.ctrlKey || event.metaKey) return;
      const target = event.target as HTMLElement | null;
      if (target && (target.isContentEditable || /^(INPUT|TEXTAREA|SELECT)$/.test(target.tagName))) return;
      const handlers = chords.current;
      if (!handlers) return;
      let handled = true;
      if (event.shiftKey) {
        switch (event.code) {
          case "BracketRight": handlers.nextProject(); break;
          case "BracketLeft": handlers.prevProject(); break;
          case "ArrowRight": handlers.nextList(); break;
          case "ArrowLeft": handlers.prevList(); break;
          case "KeyN": handlers.newList(); break;
          default: handled = false;
        }
      } else {
        const digit = /^Digit([1-9])$/.exec(event.code);
        if (digit) handlers.nthList(Number(digit[1]) - 1);
        else handled = false;
      }
      if (handled) event.preventDefault();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  if (!me.data || !projectsQuery.data) return null;

  const reducedMotion = settings.data?.reduced_motion === "on";
  const splitLabel = splitOn ? "Close split view" : "Split view";
  const previewLabel = previewOpen ? "Hide preview panel" : "Show preview panel";
  const initial = (me.data.name || me.data.email || "?").trim().charAt(0).toUpperCase();

  return (
    <div className="relative z-(--z-index-raised) flex h-[3.286rem] min-w-0 shrink-0 items-stretch border-b border-edge-2 bg-surface-1 text-content-1 select-none">
      <div className="flex shrink-0 items-center pr-[0.929rem] pl-[0.857rem]">
        <Menu
          open={projectOpen}
          onOpenChange={(open, reason) => {
            if (!open && reason === "outside" && pressedPill.current) {
              // A re-press of the pill keeps the menu open: re-render the menu with its open state kept.
              // The flag stays set until the next press anywhere, so a second dismiss from the same press is ignored too.
              bump((n) => n + 1);
              return;
            }
            setProjectOpen(open);
          }}
          placement="control"
          offset={8}
          width="project"
          ariaLabel="Projects"
          trigger={
            <Button
              ref={pillRef}
              data-part="app.appbar.projectSwitcher"
              ariaLabel="Switch project"
              description="Switch project"
              variant="neutral"
              size="xl"
              tone="strong"
              width="content"
              label={activeProject?.name ?? ""}
              trailingIcon="unfold_more"
              iconSize={15}
              className="pr-[0.643rem] pl-[0.857rem] font-semibold"
            />
          }
        >
          <div data-part="app.appbar.projectMenu" className="flex flex-col">
            {projects.map((project, index) => (
              <ProjectRow
                key={project.id}
                project={project}
                active={project.id === activeProject?.id}
                part={index === 0 ? "app.appbar.projectRow" : index === 1 ? "app.appbar.projectRowSpring" : undefined}
                morePart={index === 0 ? "app.appbar.projectMore" : index === 1 ? "app.appbar.projectMoreSpring" : undefined}
                menuRenamePart={index === 1 ? "app.appbar.menuProjectRename" : undefined}
                menuDeletePart={index === 1 ? "app.appbar.menuProjectDelete" : undefined}
                renamePart={index === 0 ? "app.appbar.projectRename" : undefined}
                deletePart={index === 0 ? "app.appbar.projectDelete" : index === 1 ? "app.appbar.projectDeleteSpring" : undefined}
                onPick={() => pickProject(project)}
                onRename={() => requestRenameProject(project)}
                onDelete={() => requestDeleteProject(project)}
              />
            ))}
            <Separator className="mx-2 my-[0.214rem]" />
            <div
              role="menuitem"
              tabIndex={0}
              data-part="app.appbar.projectNew"
              onClick={requestNewProject}
              onKeyDown={(event) => {
                if (event.key === "Enter" || event.key === " ") {
                  event.preventDefault();
                  requestNewProject();
                }
              }}
              className="flex h-[2.286rem] cursor-default items-center gap-[0.357rem] rounded-(--radius-field) pr-2 pl-[0.643rem] outline-none hover:bg-wash-2 focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-focus-1"
            >
              <Icon name="add" size={15} weight={300} tone="muted" />
              <Text as="span" size="md" className="min-w-0 text-[0.964rem]">
                New project…
              </Text>
            </div>
          </div>
        </Menu>
      </div>

      <ListTabs
        lists={lists}
        leftId={leftList?.id ?? null}
        rightId={rightList?.id ?? null}
        splitOn={splitOn}
        renamingId={renamingId}
        onSelect={pickList}
        onStartRename={(list) => setRenamingId(list.id)}
        onCommitRename={commitRename}
        onCancelRename={() => setRenamingId(null)}
        onRenameRequest={requestRenameList}
        onDelete={requestDeleteList}
        onExport={requestExportList}
        onSplit={(list) => splitWith(list)}
        onNewList={requestNewList}
        onReorder={moveList}
        onDropTask={dropTasksOnList}
      />

      <div className="flex shrink-0 items-center gap-[0.357rem] pr-[0.643rem] pl-[0.429rem]">
        {reducedMotion && (
          <StatusIndicator
            data-part="app.appbar.reducedMotionStatus"
            role="status"
            label="Reduced motion is on"
            icon="motion_photos_off"
          />
        )}
        {splitOn && (
          <Button
            data-part="app.appbar.swapPanes"
            ariaLabel="Swap panes"
            description="Swap panes"
            icon="swap_horiz"
            iconSize={19}
            size="xl"
            shape="icon"
            variant="ghost"
            tone="muted"
            onClick={swapPanes}
          />
        )}
        <Button
          data-part="app.appbar.splitView"
          ariaLabel={splitLabel}
          description={splitLabel}
          icon="vertical_split"
          iconSize={19}
          size="xl"
          shape="icon"
          variant="ghost"
          tone="muted"
          active={splitOn}
          aria-pressed={splitOn}
          disabled={lists.length === 0}
          onClick={() => splitWith(null)}
        />
        <Button
          data-part="app.appbar.previewToggle"
          ariaLabel={previewLabel}
          description={previewLabel}
          shortcut={"Mod+\\"}
          icon={previewOpen ? "right_panel_close" : "right_panel_open"}
          iconSize={19}
          size="xl"
          shape="icon"
          variant="ghost"
          tone="muted"
          active={previewOpen}
          aria-pressed={previewOpen}
          onClick={togglePreview}
        />
        <Button
          data-part="app.appbar.trashButton"
          ariaLabel="Trash"
          description="Trash"
          icon="delete"
          iconSize={19}
          size="xl"
          shape="icon"
          variant="ghost"
          tone="muted"
          onClick={openTrash}
        />
        <Button
          data-part="app.appbar.paletteButton"
          ariaLabel="Search and commands"
          description="Search and commands"
          shortcut="Mod+K"
          icon="search"
          iconSize={19}
          size="xl"
          shape="icon"
          variant="ghost"
          tone="muted"
          onClick={openPalette}
        />
        <Button
          data-part="app.appbar.settingsButton"
          ariaLabel="Settings"
          description="Settings"
          shortcut="Mod+,"
          icon="settings"
          iconSize={19}
          size="xl"
          shape="icon"
          variant="ghost"
          tone="muted"
          onClick={openSettings}
        />
        <Separator orientation="vertical" length="sm" className="mx-1" />
        <Menu
          open={accountOpen}
          onOpenChange={setAccountOpen}
          placement="anchored"
          width="project"
          pointX={accountAnchor.x}
          pointY={accountAnchor.y}
          ariaLabel="Account"
          trigger={
            <Button
              data-part="app.appbar.accountButton"
              ariaLabel="Account"
              description="Account"
              label={initial}
              variant="filled"
              tone="strong"
              size="xl"
              width="content"
              className="w-[2rem] justify-center rounded-full px-0 font-semibold"
              onClick={(event) => {
                const rect = event.currentTarget.getBoundingClientRect();
                setAccountAnchor({ x: rect.right, y: rect.bottom + 4 });
                setAccountOpen(true);
              }}
            />
          }
        >
          <div data-part="app.appbar.accountMenu" className="flex flex-col">
            <div
              role="none"
              data-part="app.appbar.accountEmail"
              className="truncate px-[0.857rem] pt-[0.357rem] pb-[0.429rem] text-[0.857rem] text-content-3"
            >
              {me.data.email}
            </div>
            <Separator inset="menu" />
            <MenuItem data-part="app.appbar.accountSettings" icon="settings" label="Settings" onSelect={openSettings} />
            <MenuItem data-part="app.appbar.accountSignOut" icon="logout" label="Sign out" onSelect={signOut} />
          </div>
        </Menu>
      </div>
    </div>
  );
}
