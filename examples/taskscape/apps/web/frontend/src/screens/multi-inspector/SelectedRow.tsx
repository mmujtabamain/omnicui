// One selected task in the multi-select inspector: a done check, the title (which peeks the task),
// the list name, and an Open button that shows on hover and narrows the selection to this task.
import { Badge } from "@taskscape/ui/badge";
import { Button } from "@taskscape/ui/button";
import { Checkbox } from "@taskscape/ui/checkbox";
import { Tooltip } from "@taskscape/ui/tooltip";
import type { Task } from "../../api/types";

/** Which contract part a row carries: the first row, the parcel row, the last row, or none. */
export type RowSlot = "first" | "parcel" | "last" | "other";

export interface SelectedRowProps {
  task: Task;
  /** The name of the task's list; undefined until the lists load. */
  listName: string | undefined;
  /** The row the user peeked last: highlighted. */
  active: boolean;
  slot: RowSlot;
  onCheck: (done: boolean) => void;
  onPeek: () => void;
  onOpen: () => void;
}

export function SelectedRow({ task, listName, active, slot, onCheck, onPeek, onOpen }: SelectedRowProps) {
  const first = slot === "first";
  const parcel = slot === "parcel";
  const last = slot === "last";

  const titlePart = first
    ? "app.multiinspector.rowTitle"
    : parcel
      ? "app.multiinspector.parcelTitle"
      : last
        ? "app.multiinspector.lastRowTitle"
        : undefined;

  return (
    <div
      data-part={first ? "app.multiinspector.row" : undefined}
      className={`group/row flex h-9 shrink-0 items-center gap-space-5 rounded-(--radius-field) px-space-4 ${
        active ? "bg-selection-1" : "hover:bg-wash-1"
      }`}
    >
      {/* The tooltip sits on a span, so the checkbox's own data-state (checked or unchecked) is not overwritten. */}
      <Tooltip content={task.done ? "Mark not done" : "Mark done"}>
        <span className="inline-flex shrink-0">
          <Checkbox
            data-part={first ? "app.multiinspector.rowCheck" : parcel ? "app.multiinspector.parcelCheck" : undefined}
            size="sm"
            intent="done"
            checked={task.done}
            ariaLabel={task.done ? "Mark not done" : "Mark done"}
            onCheckedChange={onCheck}
          />
        </span>
      </Tooltip>
      <Button
        data-part={titlePart}
        variant="ghost"
        tone={task.done ? "muted" : "strong"}
        size="md"
        width="full"
        label={task.title}
        description={task.title}
        active={active}
        onClick={onPeek}
        className={`h-auto min-w-0 flex-1 justify-start bg-transparent px-0 text-left text-[0.964rem] font-normal ${
          task.done ? "line-through" : ""
        }`}
      />
      {listName !== undefined && (
        <Badge data-part={first ? "app.multiinspector.rowListBadge" : undefined} tone="outline" className="shrink-0">
          {listName}
        </Badge>
      )}
      <Button
        data-part={first ? "app.multiinspector.rowOpen" : undefined}
        variant="ghost"
        tone="muted"
        size="sm"
        shape="icon"
        width="content"
        reveal="row"
        icon="open_in_full"
        iconSize={13}
        ariaLabel="Open"
        description="Open"
        onClick={onOpen}
      />
    </div>
  );
}
