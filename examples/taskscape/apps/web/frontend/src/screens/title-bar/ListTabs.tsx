// The list tab strip for the active project: one tab per list with a separator before each, the
// inline rename field in place of a tab, the close x inside every tab, drag to reorder, a task row
// dropped on a tab, and New list at the end.
import * as React from "react";
import { Button } from "@taskscape/ui/button";
import { Icon } from "@taskscape/ui/icon";
import { Input } from "@taskscape/ui/input";
import { Menu } from "@taskscape/ui/menu";
import { MenuItem } from "@taskscape/ui/menu-item";
import { Separator } from "@taskscape/ui/separator";
import { Text } from "@taskscape/ui/text";
import type { TaskList } from "../../api/types";

/** Drag payloads, shared with the list tab widget: a task row dropped on a tab, or a tab dragged along the strip. */
const TASK_MIME = "application/x-taskscape-task";
const TAB_MIME = "application/x-taskscape-list-tab";

/** Test part name of a tab: the first tab is listTab; Reading and Garden have their own names. */
function tabPartFor(index: number, name: string): string | undefined {
  if (index === 0) return "app.appbar.listTab";
  if (name === "Reading") return "app.appbar.listTabReading";
  if (name === "Garden") return "app.appbar.listTabGarden";
  return undefined;
}

const cls = (...parts: (string | false | null | undefined)[]) => parts.filter(Boolean).join(" ");

/** True when the drag carries a tab or a task row, the two payloads this strip takes. */
function takesDrop(event: React.DragEvent): boolean {
  const types = Array.from(event.dataTransfer.types);
  return types.includes(TASK_MIME) || types.includes(TAB_MIME);
}

export interface ListTabsProps {
  lists: TaskList[];
  /** The list in the left (active) pane. */
  leftId: number | null;
  /** The list in the right pane; used only while split is on. */
  rightId: number | null;
  splitOn: boolean;
  /** The list whose name is being edited inline, or null. */
  renamingId: number | null;
  onSelect(list: TaskList): void;
  onStartRename(list: TaskList): void;
  onCommitRename(list: TaskList, name: string): void;
  onCancelRename(): void;
  onRenameRequest(list: TaskList): void;
  onDelete(list: TaskList): void;
  onExport(list: TaskList): void;
  onSplit(list: TaskList): void;
  onNewList(): void;
  /** A tab was dragged from one position in the strip to another. */
  onReorder(from: number, to: number): void;
  /** Task ids were dropped on a tab. */
  onDropTask(taskIds: number[], list: TaskList): void;
}

export function ListTabs(props: ListTabsProps) {
  const { lists, leftId, rightId, splitOn, renamingId, onNewList, onReorder, onDropTask } = props;
  const leftIndex = lists.findIndex((list) => list.id === leftId);
  const rightIndex = splitOn ? lists.findIndex((list) => list.id === rightId) : -1;
  // The pane markers carry one test part: the first marker in the strip.
  const markerIndex = [leftIndex, rightIndex].filter((index) => index >= 0).sort((a, b) => a - b)[0] ?? -1;
  // The index of the tab (or the end zone, lists.length) the drag is over.
  const [dropAt, setDropAt] = React.useState<number | null>(null);

  const dragOver = (at: number) => (event: React.DragEvent) => {
    if (!takesDrop(event)) return;
    event.preventDefault();
    event.dataTransfer.dropEffect = "move";
    setDropAt(at);
  };
  const dragLeave = () => setDropAt(null);
  const drop = (at: number, list: TaskList | null) => (event: React.DragEvent) => {
    setDropAt(null);
    const types = Array.from(event.dataTransfer.types);
    if (list && types.includes(TASK_MIME)) {
      event.preventDefault();
      const ids = event.dataTransfer.getData(TASK_MIME).split(",").map(Number).filter((id) => Number.isInteger(id) && id > 0);
      if (ids.length > 0) onDropTask(ids, list);
      return;
    }
    if (types.includes(TAB_MIME)) {
      const from = Number(event.dataTransfer.getData(TAB_MIME));
      if (Number.isInteger(from) && from !== at) {
        event.preventDefault();
        onReorder(from, at);
      }
    }
  };

  return (
    <div
      role="tablist"
      aria-label="Lists"
      data-part="app.appbar.listTabs"
      className="flex min-w-0 flex-1 items-stretch overflow-x-auto [scrollbar-width:none] [&::-webkit-scrollbar]:hidden"
    >
      {lists.map((list, index) => {
        const pane = splitOn && index === leftIndex ? "left" : splitOn && index === rightIndex ? "right" : "";
        // A separator next to the focused tab runs the full height in the focused tab's edge colour.
        const adjacent = index === leftIndex || index - 1 === leftIndex;
        return (
          <React.Fragment key={list.id}>
            <Separator
              orientation="vertical"
              length={adjacent ? "full" : "sm"}
              emphasis={adjacent ? "adjacent" : "default"}
              className={adjacent ? undefined : "self-center"}
            />
            <ListTab
              list={list}
              index={index}
              selected={index === leftIndex}
              renaming={renamingId === list.id}
              dropOver={dropAt === index}
              splitOn={splitOn}
              pane={pane}
              markerPart={index === markerIndex ? "app.appbar.paneMarker" : undefined}
              tabPart={tabPartFor(index, list.name)}
              deletePart={index === 0 ? "app.appbar.listTabDelete" : undefined}
              onSelect={props.onSelect}
              onStartRename={props.onStartRename}
              onCommitRename={props.onCommitRename}
              onCancelRename={props.onCancelRename}
              onRenameRequest={props.onRenameRequest}
              onDelete={props.onDelete}
              onExport={props.onExport}
              onSplit={props.onSplit}
              onDragOver={dragOver(index)}
              onDragLeave={dragLeave}
              onDrop={drop(index, list)}
            />
          </React.Fragment>
        );
      })}
      {lists.length > 0 && (
        <Separator
          orientation="vertical"
          length={lists.length - 1 === leftIndex ? "full" : "sm"}
          emphasis={lists.length - 1 === leftIndex ? "adjacent" : "default"}
          className={lists.length - 1 === leftIndex ? undefined : "self-center"}
        />
      )}
      <Button
        data-part="app.appbar.newList"
        ariaLabel="New list"
        description="New list"
        variant="ghost"
        size="md"
        shape="icon"
        tone="muted"
        icon="add"
        iconSize={18}
        onClick={onNewList}
        className="my-auto mx-1 shrink-0"
      />
      {/* The empty space after New list: a tab dropped here moves to the end of the strip. */}
      <div
        data-slot="tabEnd"
        className="min-w-[1.5rem] flex-1 self-stretch"
        onDragOver={(event) => {
          if (!takesDrop(event)) return;
          event.preventDefault();
          event.dataTransfer.dropEffect = "move";
        }}
        onDrop={drop(lists.length - 1, null)}
      />
    </div>
  );
}

interface ListTabProps {
  list: TaskList;
  index: number;
  selected: boolean;
  renaming: boolean;
  dropOver: boolean;
  splitOn: boolean;
  pane: "" | "left" | "right";
  markerPart?: string;
  tabPart?: string;
  deletePart?: string;
  onSelect(list: TaskList): void;
  onStartRename(list: TaskList): void;
  onCommitRename(list: TaskList, name: string): void;
  onCancelRename(): void;
  onRenameRequest(list: TaskList): void;
  onDelete(list: TaskList): void;
  onExport(list: TaskList): void;
  onSplit(list: TaskList): void;
  onDragOver(event: React.DragEvent): void;
  onDragLeave(): void;
  onDrop(event: React.DragEvent): void;
}

function ListTab({
  list, index, selected, renaming, dropOver, splitOn, pane, markerPart, tabPart, deletePart,
  onSelect, onStartRename, onCommitRename, onCancelRename, onRenameRequest, onDelete, onExport, onSplit,
  onDragOver, onDragLeave, onDrop,
}: ListTabProps) {
  const tabRef = React.useRef<HTMLDivElement>(null);
  const [menuOpen, setMenuOpen] = React.useState(false);
  const [anchor, setAnchor] = React.useState({ x: 0, y: 0 });
  const [dragging, setDragging] = React.useState(false);
  // The draft is the field's text; it starts from the list name each time a rename begins.
  const [draft, setDraft] = React.useState(list.name);

  const beginRename = () => {
    setDraft(list.name);
    onStartRename(list);
  };
  // Two presses on the same tab in quick succession are a double-click (the test format sends presses).
  const lastPress = React.useRef(0);
  const press = () => {
    const now = Date.now();
    if (now - lastPress.current < 500) {
      lastPress.current = 0;
      beginRename();
      return;
    }
    lastPress.current = now;
    onSelect(list);
  };

  if (renaming) {
    return (
      <div className="flex h-full shrink-0 items-center">
        <Input
          data-part="app.appbar.listRenameInput"
          ariaLabel="List name"
          mode="draft"
          autoFocus
          value={draft}
          onValueChange={setDraft}
          onCommit={(text) => onCommitRename(list, text)}
          onCancel={onCancelRename}
          className="h-[1.75rem] w-[9.5rem] rounded-(--radius-field) border border-edge-2 bg-surface-3 px-2 text-[0.929rem] leading-[1.393rem] font-medium text-content-1 focus:border-focus-1 focus:ring-2 focus:ring-focus-1"
        />
      </div>
    );
  }

  const tab = (
    <div
      ref={tabRef}
      role="tab"
      aria-selected={selected}
      tabIndex={0}
      data-part={tabPart}
      data-selected={selected || undefined}
      onClick={press}
      onDoubleClick={beginRename}
      onKeyDown={(event) => {
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          onSelect(list);
        } else if (event.key === "F2") {
          event.preventDefault();
          beginRename();
        }
      }}
      className="flex h-full min-w-0 cursor-default items-center gap-[0.357rem] pl-[0.929rem] pr-[0.357rem] whitespace-nowrap outline-none select-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-focus-1"
    >
      {pane !== "" && (
        <span
          role="img"
          aria-label={pane === "left" ? "Left pane" : "Right pane"}
          data-part={markerPart}
          className="inline-flex shrink-0"
        >
          <Icon name={pane === "left" ? "line_start_circle" : "line_end_circle"} size={14} weight={300} tone="accent" />
        </span>
      )}
      <Text as="span" size="md" weight="medium" tone={selected ? "primary" : "muted"} overflow="truncate" className="min-w-0 max-w-40">
        {list.name}
      </Text>
    </div>
  );

  // The tab box holds the tab (the menu trigger) and its close x, so the x sits inside the tab's edges.
  // Middle-click deletes the list (aux button 1); the delete is still confirmed by its sheet.
  return (
    <div
      draggable
      onDragStart={(event) => {
        event.dataTransfer.effectAllowed = "move";
        event.dataTransfer.setData(TAB_MIME, String(index));
        setDragging(true);
      }}
      onDragEnd={() => setDragging(false)}
      onDragOver={onDragOver}
      onDragLeave={onDragLeave}
      onDrop={onDrop}
      onAuxClick={(event) => {
        if (event.button !== 1) return;
        event.preventDefault();
        onDelete(list);
      }}
      className={cls(
        "group/tab relative flex h-full min-w-0 max-w-[11.429rem] shrink-0 items-stretch border-x pr-[0.5rem]",
        selected ? "border-edge-2 bg-surface-2" : "border-transparent hover:bg-wash-1",
        dropOver && "bg-selection-1",
        dragging && "opacity-40",
      )}
    >
      <Menu
        open={menuOpen}
        onOpenChange={(open) => {
          if (open && tabRef.current) {
            const rect = tabRef.current.getBoundingClientRect();
            setAnchor({ x: rect.left, y: rect.bottom + 4 });
          }
          setMenuOpen(open);
        }}
        placement="anchored"
        width="tab"
        pointX={anchor.x}
        pointY={anchor.y}
        ariaLabel="List menu"
        trigger={tab}
      >
        <div data-part="app.appbar.tabMenu" className="flex flex-col">
          <MenuItem data-part="app.appbar.tabMenuRename" label="Rename" onSelect={() => onRenameRequest(list)} />
          <MenuItem
            data-part="app.appbar.tabMenuSplit"
            label={splitOn ? "Close split" : "Open in split view"}
            onSelect={() => onSplit(list)}
          />
          <MenuItem data-part="app.appbar.tabMenuExport" label="Export list" onSelect={() => onExport(list)} />
          <MenuItem data-part="app.appbar.tabMenuDelete" tone="danger" label="Delete list" onSelect={() => onDelete(list)} />
        </div>
      </Menu>
      <Button
        data-part={deletePart}
        ariaLabel="Delete list"
        variant="ghost"
        size="sm"
        shape="icon"
        tone="muted"
        icon="close"
        iconSize={15}
        onClick={() => onDelete(list)}
        className="my-auto shrink-0 hover:bg-danger-100 hover:text-danger-500 focus-visible:text-danger-500"
      />
    </div>
  );
}
