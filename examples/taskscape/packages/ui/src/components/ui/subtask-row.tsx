import * as React from "react";
import { cn } from "../../lib/utils";
import { Button } from "./button";
import { Checkbox } from "./checkbox";

/**
 * A subtask row in the task inspector. The row is 28px tall (h-8 at 1rem = 14px). Its left padding
 * is 6px at depth 0 and 18px more per nesting level. The checkbox is controlled: done is the prop,
 * and a press reports the next value through onToggleDone. The title button selects the subtask.
 */
export interface SubtaskRowProps extends Omit<React.HTMLAttributes<HTMLDivElement>, "title" | "children"> {
  title?: string;
  done?: boolean;
  /** Nesting level: 6px left padding at 0, plus 18px per level. */
  depth?: number;
  /** Nested SubtaskRow items for the next level. */
  children?: React.ReactNode;
  onToggleDone?: (done: boolean) => void;
  onSelectTask?: () => void;
}

export const SubtaskRow = React.forwardRef<HTMLDivElement, SubtaskRowProps>(function SubtaskRow(
  { title = "", done = false, depth = 0, children, onToggleDone, onSelectTask, className, ...props },
  ref,
) {
  // Empty arrays and booleans are not children: the nested list renders only when there is a child.
  const hasChildren = React.Children.toArray(children).length > 0;

  return (
    <div ref={ref} data-slot="subtaskrow" className={cn("flex flex-col", className)} {...props}>
      <div
        data-slot="row"
        className="group/row flex h-[2rem] items-center gap-2 rounded-(--radius-field) pr-2 hover:bg-wash-1"
        style={{ paddingLeft: `${0.4286 + depth * 1.2857}rem` }}
      >
        <Checkbox
          data-slot="checkbox"
          ariaLabel={title}
          intent="done"
          checked={done}
          onCheckedChange={(next) => onToggleDone?.(next)}
        />
        <Button
          data-slot="title"
          variant="ghost"
          size="md"
          width="content"
          tone={done ? "muted" : "strong"}
          label={title}
          className={cn("h-[2rem] min-w-0 flex-1 justify-start px-0 text-left", done && "line-through")}
          onClick={() => onSelectTask?.()}
        />
      </div>
      {hasChildren && (
        <div data-slot="children" className="flex flex-col">
          {children}
        </div>
      )}
    </div>
  );
});
