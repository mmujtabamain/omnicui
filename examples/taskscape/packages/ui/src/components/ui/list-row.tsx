import * as React from "react";
import * as DropdownMenuPrimitive from "@radix-ui/react-dropdown-menu";
import { cva } from "class-variance-authority";
import { cn } from "../../lib/utils";
import { Button } from "./button";
import { Icon } from "./icon";

/**
 * Row geometry at 1rem = 14px. The row is 2.25rem tall. Trash rows (host list) pad px-space-5 with
 * gap-space-5; project rows (host menu) pad px-space-6 with gap-space-4. Title 13px/16px, meta 11px/14px.
 * Hover washes wash-1; the selection uses selection-1. Actions are revealed on hover and focus-within.
 */
export const listRowVariants = cva(
  [
    "relative flex h-[2.25rem] w-full min-w-0 cursor-default select-none items-center text-left text-content-1 outline-none",
    "rounded-(--radius-field) focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-focus-1",
  ],
  {
    variants: {
      host: {
        list: "gap-space-5 px-space-5",
        menu: "gap-space-4 px-space-6",
      },
      selection: {
        none: "group-hover/row:bg-wash-1 data-[highlighted]:bg-wash-1",
        selected: "bg-selection-1",
      },
      actions: {
        none: "",
        hover: "pr-[4rem]",
      },
    },
    defaultVariants: { host: "list", selection: "none", actions: "none" },
  },
);

export type ListRowHost = "list" | "menu";
export type ListRowAction = "restore" | "edit" | "delete";

export interface ListRowProps extends Omit<React.HTMLAttributes<HTMLDivElement>, "title"> {
  /** Row title. Required. */
  title: string;
  /** Secondary line under the title. "" shows no meta line. */
  meta?: string;
  /** list: a listitem in a plain list. menu: a menuitemradio in a DropdownMenu (project switcher). */
  host?: ListRowHost;
  /** check shows the accent check when selected (host menu). */
  indicator?: "none" | "check";
  /** hover: restore or edit, and delete, revealed on hover and focus-within. */
  actions?: "none" | "hover";
  /** Selection background; with indicator check it also shows the check. */
  selected?: boolean;
  /** Row body press, Enter or Space; choosing a menu row. */
  onActivate?: () => void;
  /** A row action was pressed. */
  onAction?: (action: ListRowAction) => void;
}

const noop = () => {};

/**
 * One row for the trash list (a listitem) and the project switcher (a menuitemradio on Radix
 * DropdownMenu.RadioItem). The body is the focus stop and the activation target; the actions sit
 * beside it, never inside it. The ref goes to the root element.
 */
export const ListRow = React.forwardRef<HTMLDivElement, ListRowProps>(function ListRow(
  {
    title, meta = "", host = "list", indicator = "none", actions = "none", selected = false,
    onActivate, onAction, className, ...props
  },
  ref,
) {
  // Radix moves focus onto a row the pointer is over. That focus is not keyboard focus, so the ring
  // drops while the pointer holds it (as in MenuItem).
  const [pointerHighlight, setPointerHighlight] = React.useState(false);
  const isMenu = host === "menu";
  const selection = selected ? "selected" : "none";
  const hasActions = actions === "hover";

  const content = (
    <>
      {indicator === "check" && (
        <span className="flex w-4 shrink-0 items-center justify-center">
          {selected && (
            <span data-slot="check" aria-hidden="true" className="flex">
              <Icon name="check" size={14} weight={900} tone="accent" />
            </span>
          )}
        </span>
      )}
      <span className="flex min-w-0 flex-1 flex-col">
        <span data-slot="title" className="block truncate text-[0.929rem] leading-[1.143rem]">
          {title}
        </span>
        {meta !== "" && (
          <span data-slot="meta" className="block truncate text-[0.786rem] leading-[1rem] text-content-3">
            {meta}
          </span>
        )}
      </span>
    </>
  );

  const rowActions = hasActions ? (
    <div className="absolute inset-y-0 right-space-5 flex items-center gap-space-3">
      {isMenu ? (
        <Button
          data-slot="edit"
          ariaLabel={`Edit ${title}`}
          description="Edit"
          icon="edit"
          shape="icon"
          size="sm"
          variant="ghost"
          tone="default"
          hover="wash"
          reveal="row"
          onClick={() => onAction?.("edit")}
        />
      ) : (
        <Button
          data-slot="restore"
          ariaLabel={`Restore ${title}`}
          description="Restore"
          icon="restore_from_trash"
          shape="icon"
          size="sm"
          variant="ghost"
          tone="default"
          hover="wash"
          reveal="row"
          onClick={() => onAction?.("restore")}
        />
      )}
      <Button
        data-slot="deleteAction"
        ariaLabel={`Delete ${title}`}
        description="Delete"
        icon="delete"
        shape="icon"
        size="sm"
        variant="ghost"
        tone="danger"
        hover="wash"
        reveal="row"
        onClick={() => onAction?.("delete")}
      />
    </div>
  ) : null;

  if (!isMenu) {
    return (
      <div
        ref={ref}
        data-slot="listRow"
        role="listitem"
        className={cn("group/row relative w-full min-w-0", className)}
        {...props}
      >
        <button
          type="button"
          data-slot="body"
          className={listRowVariants({ host: "list", selection, actions })}
          onClick={() => onActivate?.()}
        >
          {content}
        </button>
        {rowActions}
      </div>
    );
  }

  return (
    <div ref={ref} className={cn("group/row relative w-full min-w-0", className)} {...props}>
      <DropdownMenuPrimitive.RadioGroup value={selected ? "on" : "off"} onValueChange={noop}>
        <DropdownMenuPrimitive.RadioItem
          data-slot="body"
          value="on"
          className={cn(
            listRowVariants({ host: "menu", selection, actions }),
            pointerHighlight && "focus-visible:ring-0",
          )}
          onSelect={() => onActivate?.()}
          onPointerMove={() => setPointerHighlight(true)}
          onBlur={() => setPointerHighlight(false)}
        >
          {content}
        </DropdownMenuPrimitive.RadioItem>
      </DropdownMenuPrimitive.RadioGroup>
      {rowActions}
    </div>
  );
});
