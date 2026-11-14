import * as React from "react";
import { cva } from "class-variance-authority";
import { cn } from "../../lib/utils";
import { Button } from "./button";
import { ListTabLabel } from "./list-tab-label";

/** Drag payloads: a task row dropped on a tab, or a tab dragged along the strip. */
const TASK_MIME = "application/x-taskscape-task";
const TAB_MIME = "application/x-taskscape-list-tab";

/** The row that holds the tab, the rename input and the delete button. */
const listTabVariants = cva("group/row relative inline-flex min-w-0 max-w-full items-center gap-px", {
  variants: {
    dragging: { true: "opacity-40", false: "" },
  },
  defaultVariants: { dragging: false },
});

/**
 * The tab's fill. ListTabLabel owns the tab element (role tab, selection, name, pane marker); these
 * classes restyle that trigger to the full list tab look: selected surface-2 fill with hairline
 * edges, wash on hover when idle, selection colour as a drop target.
 */
const listTabFillVariants = cva("", {
  variants: {
    fill: {
      selected: "[&_[data-slot=tab]]:border-edge-1 [&_[data-slot=tab]]:bg-surface-2",
      idle: "[&_[data-slot=tab]:hover]:bg-wash-1",
      drop: "[&_[data-slot=tab]]:bg-selection-1",
    },
  },
  defaultVariants: { fill: "idle" },
});

export interface ListTabProps extends Omit<React.HTMLAttributes<HTMLDivElement>, "onSelect" | "onContextMenu"> {
  /** List name shown on the tab; the rename input shows it while editing. */
  name?: string;
  /** Active list: surface-2 fill with hairline edges. */
  selected?: boolean;
  /** Inline rename input over the tab. */
  editing?: boolean;
  /** A dragged task row or tab is over this tab. */
  dropTarget?: boolean;
  /** This tab is being dragged: dimmed. */
  dragging?: boolean;
  /** Split-view pane marker before the name. */
  paneMarker?: boolean;
  /** Position in the strip, reported by onReorder. Not a contract prop; defaults to 0. */
  index?: number;
  /** Click, Enter or Space on the tab. */
  onSelect?: () => void;
  /** F2 on the tab. */
  onRenameStart?: () => void;
  /** Enter in the rename input, or blur with a changed name. */
  onRenameCommit?: (name: string) => void;
  /** Escape in the rename input; the name is restored. */
  onRenameCancel?: () => void;
  /** The delete button. */
  onDelete?: () => void;
  /** Right-click, Shift+F10 or the ContextMenu key on the tab. */
  onContextMenu?: (event: React.SyntheticEvent) => void;
  /** A task row dropped on this tab. */
  onDropTask?: (taskId: string) => void;
  /** This tab was dragged to a new place in the strip. */
  onReorder?: (fromIndex: number, toIndex: number) => void;
}

/**
 * Full list tab for the desktop title bar: ListTabLabel as the tab, with inline rename, drag
 * reorder, task drop and a delete button on its right. The delete button and the rename input are
 * siblings of the tab, never children, so the tab keeps no interactive children.
 * Sizes and colours follow ListTabLabel: 24.5px tall (1.75rem), 13px text (0.929rem).
 */
export const ListTab = React.forwardRef<HTMLDivElement, ListTabProps>(function ListTab(
  {
    name = "", selected = false, editing = false, dropTarget = false, dragging = false, paneMarker = false,
    index = 0, onSelect, onRenameStart, onRenameCommit, onRenameCancel, onDelete, onContextMenu,
    onDropTask, onReorder, className, ...props
  },
  ref,
) {
  const [draft, setDraft] = React.useState(name);
  // Set once Enter, Escape or a commit on blur has settled the edit, so a later blur does not emit again.
  const settled = React.useRef(false);
  const inputRef = React.useRef<HTMLInputElement>(null);

  React.useEffect(() => {
    if (!editing) return;
    settled.current = false;
    setDraft(name);
    inputRef.current?.focus();
  }, [editing, name]);

  const openMenu = (event: React.SyntheticEvent) => {
    event.preventDefault();
    onContextMenu?.(event);
  };

  // Middle-click on the row deletes the list (spec: aux button 1 on desktop). Not while renaming.
  const onRowAuxClick = (event: React.MouseEvent<HTMLDivElement>) => {
    if (event.button !== 1 || editing) return;
    event.preventDefault();
    onDelete?.();
  };

  // Double-click on the tab starts rename (spec: onRenameStart).
  const onTabDoubleClick = () => {
    if (!editing) onRenameStart?.();
  };

  const onTabKeyDown = (event: React.KeyboardEvent<HTMLDivElement>) => {
    if (event.key === "F2") onRenameStart?.();
    else if (event.key === "ContextMenu" || (event.key === "F10" && event.shiftKey)) openMenu(event);
  };

  const onRenameKeyDown = (event: React.KeyboardEvent<HTMLInputElement>) => {
    if (event.key === "Enter") {
      event.preventDefault();
      settled.current = true;
      onRenameCommit?.(draft);
    } else if (event.key === "Escape") {
      event.preventDefault();
      settled.current = true;
      setDraft(name);
      onRenameCancel?.();
    }
  };

  const onRenameBlur = () => {
    if (settled.current || draft === name) return;
    settled.current = true;
    onRenameCommit?.(draft);
  };

  const fill = dropTarget ? "drop" : selected ? "selected" : "idle";

  return (
    <div
      ref={ref}
      data-slot="wrapper"
      data-selected={selected ? "true" : undefined}
      className={cn(listTabVariants({ dragging }), editing && "min-w-[7.143rem]", className)}
      onAuxClick={onRowAuxClick}
      {...props}
    >
      <ListTabLabel
        name={name}
        selected={selected}
        paneMarker={paneMarker}
        draggable={!editing}
        onSelect={onSelect}
        onKeyDown={onTabKeyDown}
        onContextMenu={openMenu}
        onDoubleClick={onTabDoubleClick}
        onDragStart={(event) => {
          event.dataTransfer.effectAllowed = "move";
          event.dataTransfer.setData(TAB_MIME, String(index));
        }}
        onDragOver={(event) => {
          const types = Array.from(event.dataTransfer.types);
          if (types.includes(TASK_MIME) || types.includes(TAB_MIME)) {
            event.preventDefault();
            event.dataTransfer.dropEffect = "move";
          }
        }}
        onDrop={(event) => {
          const taskId = event.dataTransfer.getData(TASK_MIME);
          if (taskId) {
            event.preventDefault();
            onDropTask?.(taskId);
            return;
          }
          const fromIndex = Number(event.dataTransfer.getData(TAB_MIME));
          if (event.dataTransfer.types.includes(TAB_MIME) && fromIndex !== index) {
            event.preventDefault();
            onReorder?.(fromIndex, index);
          }
        }}
        className={listTabFillVariants({ fill })}
      />
      {editing && (
        <input
          ref={inputRef}
          data-slot="renameInput"
          aria-label="List name"
          value={draft}
          onChange={(event) => setDraft(event.target.value)}
          onFocus={(event) => event.currentTarget.select()}
          onKeyDown={onRenameKeyDown}
          onBlur={onRenameBlur}
          className="absolute inset-y-0 left-0 w-full min-w-0 rounded-(--radius-field) border border-edge-2 bg-surface-3 px-2 text-[0.929rem] leading-[1.393rem] font-medium text-content-1 outline-none select-text focus:border-focus-1 focus:ring-2 focus:ring-focus-1"
        />
      )}
      {!editing && (
        <Button
          data-slot="deleteButton"
          variant="ghost"
          size="sm"
          shape="icon"
          tone="default"
          reveal="row"
          icon="close"
          ariaLabel={`Delete ${name}`}
          onClick={() => onDelete?.()}
          className="hover:bg-danger-100 hover:text-danger-500 focus-visible:bg-danger-100 focus-visible:text-danger-500"
        />
      )}
    </div>
  );
});

export { listTabVariants, listTabFillVariants };
