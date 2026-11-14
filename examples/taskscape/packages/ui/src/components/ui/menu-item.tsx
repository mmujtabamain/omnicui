import * as React from "react";
import * as DropdownMenuPrimitive from "@radix-ui/react-dropdown-menu";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/utils";
import { Button } from "./button";
import { Icon } from "./icon";

/**
 * Row geometry at 1rem = 14px. The default row is 30px with a 13px/19.5px label; the large row and the
 * action row are 32px with a 13.5px/20.25px label. Padding is the task menu's (px-space-6, py-space-4).
 */
const ignore = () => {};

const menuItemVariants = cva(
  [
    "relative flex w-full min-w-0 cursor-default select-none items-center gap-space-5 px-space-6 py-space-4 text-left font-normal outline-none",
    "rounded-(--radius-field) focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-focus-1",
    "data-[disabled]:pointer-events-none data-[disabled]:opacity-40",
  ],
  {
    variants: {
      variant: {
        item: "",
        action: "rounded-none border-t border-edge-1",
      },
      tone: {
        default: "text-content-1 data-[highlighted]:bg-wash-2",
        danger: "text-danger-500 data-[highlighted]:bg-danger-100",
      },
      size: {
        default: "h-[2.143rem] text-[0.929rem] leading-[1.393rem]",
        large: "h-[2.286rem] text-[0.964rem] leading-[1.446rem]",
      },
    },
    compoundVariants: [{ variant: "action", className: "h-[2.286rem] text-[0.964rem] leading-[1.446rem]" }],
    defaultVariants: { variant: "item", tone: "default", size: "default" },
  },
);

export { menuItemVariants };

type MenuItemVariants = VariantProps<typeof menuItemVariants>;

export interface MenuItemProps extends Omit<React.ComponentPropsWithoutRef<typeof DropdownMenuPrimitive.Item>, "children" | "onSelect"> {
  variant?: MenuItemVariants["variant"];
  tone?: MenuItemVariants["tone"];
  size?: MenuItemVariants["size"];
  /** radio rows sit in a labelled group and show a check when checked. */
  selection?: "action" | "radio";
  /** submenu rows open a submenu instead of running. */
  trailing?: "none" | "submenu";
  /** Leading icon name, 15px. "" for none. */
  icon?: string;
  label?: React.ReactNode;
  truncate?: boolean;
  checked?: boolean;
  /** The value a radio row sets in its group. */
  value?: string;
  /** Project rows: rename and delete buttons revealed on hover or focus, as siblings of the row. */
  rowActions?: boolean;
  /** The submenu's content, shown while a submenu row is open. */
  submenu?: React.ReactNode;
  /** The row ran its action (not for submenu rows). */
  onSelect?: (event: Event) => void;
  /** A submenu row opened (true) or closed (false). */
  onSubmenuToggle?: (open: boolean) => void;
  onRename?: () => void;
  onDelete?: () => void;
}

/**
 * One row of a Menu, on Radix DropdownMenu.Item (RadioItem for radio rows). Must sit inside a
 * DropdownMenu.Content. The ref goes to the row (data-slot="row"); the wrapper holds the row's
 * submenu and rename and delete siblings.
 */
export const MenuItem = React.forwardRef<HTMLDivElement, MenuItemProps>(function MenuItem(
  {
    variant = "item", tone = "default", size = "default", selection = "action", trailing = "none",
    icon = "", label = "Item", truncate = false, checked = false, value = "", rowActions = false,
    disabled = false, submenu, onSelect, onSubmenuToggle, onRename, onDelete,
    className, onKeyDown, onPointerEnter, onPointerMove, onBlur, ...props
  },
  ref,
) {
  const [open, setOpen] = React.useState(false);
  const openRef = React.useRef(false);
  // Radix moves focus onto a row the pointer is over. That focus is not keyboard focus: the spec's hover is a
  // wash only, so the row drops its focus ring while the pointer holds it, and the ring returns when the row
  // loses focus and keyboard focus reaches it.
  const [pointerHighlight, setPointerHighlight] = React.useState(false);
  const isRadio = selection === "radio";
  const isSubmenu = trailing === "submenu";
  const iconTone = tone === "danger" ? "danger" : "muted";

  // The callback fires only on a change, so a hover followed by a press is one opening.
  const setSubmenu = (next: boolean) => {
    if (openRef.current === next || (next && disabled)) return;
    openRef.current = next;
    setOpen(next);
    onSubmenuToggle?.(next);
  };

  // A submenu row never runs its action: press, Enter and Space open the submenu (APG parent item).
  const handleSelect = (event: Event) => {
    if (isSubmenu) {
      event.preventDefault();
      setSubmenu(true);
      return;
    }
    onSelect?.(event);
  };

  const handleKeyDown = (event: React.KeyboardEvent<HTMLDivElement>) => {
    onKeyDown?.(event);
    if (!isSubmenu || event.defaultPrevented) return;
    if (event.key === "ArrowRight") {
      event.preventDefault();
      setSubmenu(true);
    } else if (event.key === "ArrowLeft") {
      event.preventDefault();
      setSubmenu(false);
    }
  };

  const handlePointerEnter = (event: React.PointerEvent<HTMLDivElement>) => {
    onPointerEnter?.(event);
    if (isSubmenu && event.pointerType === "mouse") setSubmenu(true);
  };

  const handlePointerMove = (event: React.PointerEvent<HTMLDivElement>) => {
    onPointerMove?.(event);
    setPointerHighlight(true);
  };

  const handleBlur = (event: React.FocusEvent<HTMLDivElement>) => {
    onBlur?.(event);
    setPointerHighlight(false);
  };

  const content = (
    <>
      {isRadio && (
        <span className="flex w-4 shrink-0 items-center justify-center">
          {checked && <Icon data-slot="check" name="check" size={15} weight={900} tone="accent" />}
        </span>
      )}
      {icon !== "" && <Icon data-slot="icon" name={icon} size={15} weight={300} tone={iconTone} />}
      <span data-slot="label" className={cn("min-w-0 flex-1", truncate && "truncate")}>
        {label}
      </span>
      {isSubmenu && <Icon data-slot="chevron" name="chevron_right" size={15} weight={300} tone={iconTone} />}
    </>
  );

  const rowProps = {
    "data-slot": "row",
    "data-value": isRadio ? value : undefined,
    className: cn(
      menuItemVariants({ variant, tone, size }),
      pointerHighlight && "focus-visible:ring-0",
      rowActions && "pr-[4rem]",
      className,
    ),
    disabled,
    onSelect: handleSelect,
    onKeyDown: handleKeyDown,
    onPointerEnter: handlePointerEnter,
    onPointerMove: handlePointerMove,
    onBlur: handleBlur,
    ...(isSubmenu ? { "aria-haspopup": "menu" as const, "aria-expanded": open } : {}),
    ...props,
  };

  // Radix RadioItem reads its checked state from the group's value; "on" and "off" make that the checked prop.
  const row = isRadio ? (
    <DropdownMenuPrimitive.RadioGroup value={checked ? "on" : "off"} onValueChange={ignore}>
      <DropdownMenuPrimitive.RadioItem ref={ref} value="on" {...rowProps}>
        {content}
      </DropdownMenuPrimitive.RadioItem>
    </DropdownMenuPrimitive.RadioGroup>
  ) : (
    <DropdownMenuPrimitive.Item ref={ref} {...rowProps}>
      {content}
    </DropdownMenuPrimitive.Item>
  );

  return (
    <div className="group/row relative">
      {row}
      {isSubmenu && open && (
        <div role="menu" data-slot="submenu" className="ml-space-7 mt-space-3 flex flex-col gap-space-3 rounded-(--radius-field) bg-surface-3 p-space-4 shadow-menu">
          {submenu}
        </div>
      )}
      {rowActions && (
        <div className="absolute inset-y-0 right-space-5 flex items-center gap-space-3">
          <Button data-slot="renameAction" ariaLabel="Rename" description="Rename" icon="edit" shape="icon" size="sm" variant="ghost" tone="muted" reveal="row" onClick={onRename} />
          <Button data-slot="deleteAction" ariaLabel="Delete" description="Delete" icon="delete" shape="icon" size="sm" variant="ghost" tone="muted" reveal="row" onClick={onDelete} />
        </div>
      )}
    </div>
  );
});
