import * as React from "react";
import { cva } from "class-variance-authority";
import { cn } from "../../lib/utils";
import { Checkbox } from "./checkbox";

/**
 * One row of the task tree. Measured at 1rem = 14px: the round check is an 18px box (md) whose left edge sits
 * 20px in from the row's indent, and the title starts 27px after the check's left edge (root title x 47; subtask
 * indented 24px, check x 44, title x 71). Each depth adds 24px of indent. Title: 500 weight, 19px line height.
 */
export const taskTreeRowVariants = cva(
  [
    "group/row relative flex min-w-0 min-h-[2.143rem] items-center gap-[0.643rem] rounded-(--radius-field) py-[0.357rem] pr-[0.714rem] text-content-1 outline-none select-none",
    "transition-[background-color,opacity] duration-(--duration-fast) ease-standard",
    "focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-focus-1",
  ],
  {
    variants: {
      selection: {
        none: "",
        selected: "bg-selection-1",
      },
      drop: {
        none: "",
        before: "before:absolute before:inset-x-0 before:top-0 before:h-[0.143rem] before:bg-accent-500 before:content-['']",
        after: "after:absolute after:inset-x-0 after:bottom-0 after:h-[0.143rem] after:bg-accent-500 after:content-['']",
        nest: "bg-selection-1 ring-2 ring-inset ring-accent-500",
      },
      dragging: {
        true: "opacity-40",
        false: "",
      },
    },
    compoundVariants: [
      // Hover washes only a row that is neither selected nor a drop target.
      { selection: "none", drop: "none", className: "hover:bg-wash-1" },
    ],
    defaultVariants: { selection: "none", drop: "none", dragging: false },
  },
);

export const taskTreeRowTitleVariants = cva("min-w-0 flex-1 truncate text-[1rem] leading-[1.357rem] font-medium", {
  variants: {
    done: {
      true: "text-content-3 line-through",
      false: "text-content-1",
    },
  },
  defaultVariants: { done: false },
});

export type TaskTreeDropPosition = "before" | "after" | "nest";
export type TaskTreeCheckAction = "select" | "done";

export interface TaskTreeRowProps extends Omit<React.HTMLAttributes<HTMLDivElement>, "onDrop" | "onContextMenu"> {
  /** Task title. */
  title?: string;
  /** Nesting level: 0 for a root task, 1 for a subtask; each level adds 24px of indent. */
  depth?: number;
  /** Muted title with strikethrough; the check shows the done-lamp token. */
  done?: boolean;
  /** Previewed or picked: selection background, reported as aria-selected. */
  selected?: boolean;
  /** Part of the multi selection: blue tint. */
  picked?: boolean;
  /** Drawn at 40 percent opacity. */
  dragging?: boolean;
  /** Drop feedback: a line on the top or bottom edge, or a ring for a nest. */
  dropIndicator?: "none" | TaskTreeDropPosition;
  /** Action of the round check: "done" while Option (macOS) or Alt is held. */
  checkAction?: TaskTreeCheckAction;
  /** Whether the row can be dragged. */
  draggable?: boolean;
  /** A parent row carries aria-expanded. */
  hasChildren?: boolean;
  /** Open state of a parent row. */
  expanded?: boolean;
  onActivate?: (event: { platformModifier: boolean; shift: boolean }) => void;
  onCheck?: (action: TaskTreeCheckAction) => void;
  onToggleDone?: () => void;
  onContextMenu?: (event: React.SyntheticEvent<HTMLDivElement>) => void;
  onDrop?: (position: TaskTreeDropPosition) => void;
  onToggleExpanded?: (expanded: boolean) => void;
}

const isMacPlatform = () =>
  typeof navigator !== "undefined" && /Mac|iPhone|iPad|iPod/.test(navigator.platform || navigator.userAgent);

/** Command on macOS, Control elsewhere. */
const platformModifierOf = (event: { metaKey: boolean; ctrlKey: boolean }) =>
  isMacPlatform() ? event.metaKey : event.ctrlKey;

export const TaskTreeRow = React.forwardRef<HTMLDivElement, TaskTreeRowProps>(function TaskTreeRow(
  {
    title = "", depth = 0, done = false, selected = false, picked = false, dragging = false,
    dropIndicator = "none", checkAction = "select", draggable = true, hasChildren = false, expanded = false,
    onActivate, onCheck, onToggleDone, onContextMenu, onDrop, onToggleExpanded,
    className, style, onKeyDown, onClick, onDoubleClick, onDragOver, ...props
  },
  ref,
) {
  const highlighted = selected || picked;
  // The check is checked when the task is done, or, in select mode, when the row is in the selection.
  const checked = done || (checkAction === "select" && highlighted);
  // Done-lamp colour for a done task or a done action; the selection accent otherwise.
  const intent = done || checkAction === "done" ? "done" : "select";
  const checkLabel = checkAction === "done" ? `Mark ${title} done` : `Select ${title}`;

  const handleKeyDown = (event: React.KeyboardEvent<HTMLDivElement>) => {
    onKeyDown?.(event);
    // Keys pressed on the round check belong to the check.
    if (event.defaultPrevented || event.target !== event.currentTarget) return;
    const modifier = platformModifierOf(event);
    switch (event.key) {
      case "Enter":
        event.preventDefault();
        if (modifier) onToggleDone?.();
        else onActivate?.({ platformModifier: false, shift: false });
        return;
      case " ":
      case "Spacebar":
        event.preventDefault();
        onActivate?.({ platformModifier: modifier, shift: false });
        return;
      case "ArrowDown":
      case "ArrowUp":
        if (event.shiftKey) {
          event.preventDefault();
          onActivate?.({ platformModifier: false, shift: true });
        }
        return;
      case "ArrowRight":
        if (hasChildren && !expanded) {
          event.preventDefault();
          onToggleExpanded?.(true);
        }
        return;
      case "ArrowLeft":
        if (hasChildren && expanded) {
          event.preventDefault();
          onToggleExpanded?.(false);
        }
        return;
      case "ContextMenu":
        event.preventDefault();
        onContextMenu?.(event);
        return;
      case "F10":
        if (event.shiftKey) {
          event.preventDefault();
          onContextMenu?.(event);
        }
        return;
    }
  };

  // A treeitem is owned by a tree or group; the group is the row's wrapper, so the row keeps its treeitem role.
  return (
    <div role="group" className="min-w-0">
      <div
        ref={ref}
        role="treeitem"
        data-slot="row"
        tabIndex={0}
        aria-level={depth + 1}
        aria-selected={highlighted}
        aria-expanded={hasChildren ? expanded : undefined}
        aria-label={title}
        draggable={draggable}
        style={{ paddingLeft: `${(20 + Math.max(0, depth) * 24) / 14}rem`, ...style }}
        className={cn(
          taskTreeRowVariants({
            selection: highlighted ? "selected" : "none",
            drop: dropIndicator,
            dragging,
          }),
          className,
        )}
        {...props}
        onKeyDown={handleKeyDown}
        onClick={(event) => {
          onClick?.(event);
          onActivate?.({ platformModifier: platformModifierOf(event), shift: event.shiftKey });
        }}
        onDoubleClick={(event) => {
          onDoubleClick?.(event);
          onToggleDone?.();
        }}
        onContextMenu={(event) => {
          event.preventDefault();
          onContextMenu?.(event);
        }}
        onDragOver={(event) => {
          onDragOver?.(event);
          if (onDrop) event.preventDefault();
        }}
        onDrop={(event) => {
          event.preventDefault();
          if (!onDrop) return;
          const rect = event.currentTarget.getBoundingClientRect();
          const ratio = rect.height > 0 ? (event.clientY - rect.top) / rect.height : 0.5;
          onDrop(ratio < 0.3 ? "before" : ratio > 0.7 ? "after" : "nest");
        }}
      >
        <Checkbox
          data-slot="check"
          size="md"
          shape="round"
          intent={intent}
          checked={checked}
          ariaLabel={checkLabel}
          onCheckedChange={() => onCheck?.(checkAction)}
          // The check's own presses and double-clicks must not also activate or toggle the row.
          onClick={(event) => event.stopPropagation()}
          onDoubleClick={(event) => event.stopPropagation()}
        />
        <span data-slot="title" className={taskTreeRowTitleVariants({ done })}>
          {title}
        </span>
      </div>
    </div>
  );
});
