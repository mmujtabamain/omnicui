import * as React from "react";
import { cva } from "class-variance-authority";
import { cn } from "../../lib/utils";
import { Button } from "./button";
import { Checkbox } from "./checkbox";
import { Row } from "./row";

/** One nesting level in px (1.7143rem at 1rem = 14px). */
const INDENT_PX = 24;
/** A touch drag to the left of this many px removes the row. */
const SWIPE_PX = 48;

/** Lines: one title, or a title plus a subtitle. The heights are minimums. */
export const taskRowVariants = cva("h-auto", {
  variants: {
    lines: {
      one: "min-h-[2.25rem]",
      two: "min-h-[3.25rem]",
    },
  },
  defaultVariants: { lines: "one" },
});

export type TaskRowCheck = "none" | "done" | "select";
export type TaskRowDisclosure = "none" | "chevron";
export type TaskRowSelection = "single" | "multi";

export interface TaskRowProps
  extends Omit<React.HTMLAttributes<HTMLDivElement>, "title" | "onSelect" | "color"> {
  title?: string;
  /** Secondary line. Its presence sets lines to two. */
  subtitle?: string;
  /** Nesting level, 24px per level. */
  depth?: number;
  /** Muted text and strikethrough; the check shows done. */
  done?: boolean;
  /** The previewed task: selection background. */
  selected?: boolean;
  /** Part of a multi selection: blue tint. Only with selection multi. */
  picked?: boolean;
  /** none: no round check. done: toggles done. select: selects into the multi selection. */
  check?: TaskRowCheck;
  selection?: TaskRowSelection;
  /** chevron: a subtask group toggles at the row start. */
  disclosure?: TaskRowDisclosure;
  expanded?: boolean;
  onExpandedChange?: (expanded: boolean) => void;
  subtaskCount?: number;
  onCheckedChange?: (next: boolean) => void;
  /** Title click. Carries the modifier state: platform modifier toggles, Shift extends. */
  onSelect?: (event: { platformModifier: boolean; shift: boolean }) => void;
  onRemove?: () => void;
}

const isApplePlatform = () =>
  typeof navigator !== "undefined" && /Mac|iPhone|iPad|iPod/.test(navigator.platform);

export const TaskRow = React.forwardRef<HTMLDivElement, TaskRowProps>(function TaskRow(
  {
    title = "", subtitle = "", depth = 0, done = false, selected = false, picked = false,
    check = "none", selection = "single", disclosure = "none", expanded = false,
    onExpandedChange, subtaskCount, onCheckedChange, onSelect, onRemove, className,
    onPointerDown, onPointerUp, onPointerCancel, ...props
  },
  ref,
) {
  const swipe = React.useRef<{ x: number; touch: boolean } | null>(null);
  const picking = picked && selection === "multi";
  const tinted = selected || picking;
  // The select check shows picked; the done check shows done.
  const checked = check === "select" ? picked : done;
  // The round check is shown only when the caller handles its change.
  const showCheck = check !== "none" && onCheckedChange !== undefined;
  const hasLead = showCheck || disclosure === "chevron";

  return (
    <Row
      ref={ref}
      role="listitem"
      selected={tinted}
      indent={depth * INDENT_PX}
      data-picked={picking || undefined}
      className={cn(taskRowVariants({ lines: subtitle ? "two" : "one" }), className)}
      onPointerDown={(event) => {
        swipe.current = { x: event.clientX, touch: event.pointerType !== "mouse" };
        onPointerDown?.(event);
      }}
      onPointerUp={(event) => {
        const start = swipe.current;
        swipe.current = null;
        if (start?.touch && event.clientX - start.x <= -SWIPE_PX) onRemove?.();
        onPointerUp?.(event);
      }}
      onPointerCancel={(event) => {
        swipe.current = null;
        onPointerCancel?.(event);
      }}
      {...props}
    >
      {hasLead && (
        <span data-slot="lead" className="flex shrink-0 items-center gap-1">
          {disclosure === "chevron" && (
            <Button
              data-slot="disclosure"
              variant="ghost"
              tone="muted"
              size="sm"
              shape="icon"
              icon="chevron_right"
              ariaLabel={`Subtasks of ${title}`}
              aria-expanded={expanded}
              className={cn(
                "transition-[color,background-color,border-color,opacity,transform]",
                expanded && "rotate-90",
              )}
              onClick={() => onExpandedChange?.(!expanded)}
              onKeyDown={(event) => {
                if (!onExpandedChange) return;
                if (event.key === "ArrowLeft" && expanded) {
                  event.preventDefault();
                  onExpandedChange(false);
                } else if (event.key === "ArrowRight" && !expanded) {
                  event.preventDefault();
                  onExpandedChange(true);
                }
              }}
            />
          )}
          {showCheck && (
            <Checkbox
              data-slot="done"
              size="sm"
              shape="round"
              intent={check}
              checked={checked}
              ariaLabel={check === "select" ? `Select ${title}` : `Mark ${title} done`}
              onCheckedChange={onCheckedChange}
            />
          )}
        </span>
      )}

      <div className="flex min-w-0 flex-1 flex-col">
        <h3 data-slot="title" className="m-0 min-w-0">
          <Button
            variant="ghost"
            tone="strong"
            size="default"
            width="full"
            label={title}
            className={cn(
              "h-auto min-h-0 justify-start px-0 font-medium text-[1rem] leading-[1.1875rem]",
              done ? "text-content-3 line-through" : "text-content-1",
            )}
            onClick={(event) =>
              onSelect?.({
                platformModifier: isApplePlatform() ? event.metaKey : event.ctrlKey,
                shift: event.shiftKey,
              })
            }
          />
        </h3>
        {subtitle && (
          <span data-slot="subtitle" className="truncate text-[0.893rem] leading-[1.1rem] text-content-2">
            {subtitle}
          </span>
        )}
      </div>

      {subtaskCount !== undefined && (
        <span
          data-slot="count"
          className="shrink-0 rounded-(--radius-field) bg-wash-2 px-1.5 text-[0.786rem] leading-[1.1rem] text-content-2 tabular-nums"
        >
          {subtaskCount}
        </span>
      )}
      {done && (
        <span data-slot="status" role="status" className="sr-only">
          Done
        </span>
      )}
      <Button
        data-slot="remove"
        variant="ghost"
        tone="muted"
        size="sm"
        shape="icon"
        icon="close"
        ariaLabel={`Remove ${title}`}
        className="ml-auto shrink-0 opacity-60 hover:opacity-100"
        onClick={() => onRemove?.()}
      />
    </Row>
  );
});
