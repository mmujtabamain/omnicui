import * as React from "react";
import * as DropdownMenuPrimitive from "@radix-ui/react-dropdown-menu";
import { cva } from "class-variance-authority";
import { cn } from "../../lib/utils";
import { Button } from "./button";
import { Icon } from "./icon";

/**
 * Geometry in rem (1rem = 14px). Default rows are 30px with a 13px/19.5px label; large rows and the
 * action row are 32px with a 13.5px/20.25px label. The label inherits the row's text colour.
 */
export const menuItemVariants = cva(
  [
    "relative flex w-full min-w-0 cursor-default select-none items-center gap-space-5 px-space-6 py-space-4 font-normal outline-none",
    "rounded-(--radius-field) transition-colors duration-(--duration-fast) ease-standard",
    "data-[disabled]:pointer-events-none data-[disabled]:opacity-40",
    "focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-focus-1",
  ],
  {
    variants: {
      variant: { item: "", action: "" },
      tone: {
        default: "text-content-1 data-[highlighted]:bg-wash-2",
        danger: "text-danger-500 data-[highlighted]:bg-danger-100",
      },
      size: {
        default: "h-[2.143rem] text-[0.929rem] leading-[1.393rem]",
        large: "h-[2.286rem] text-[0.964rem] leading-[1.446rem]",
      },
    },
    defaultVariants: { variant: "item", tone: "default", size: "default" },
  },
);

export interface MenuItemProps
  extends Omit<React.ComponentPropsWithoutRef<typeof DropdownMenuPrimitive.Item>, "children" | "onSelect" | "value" | "size" | "disabled"> {
  /** item: ordinary row. action: separated footer row, full width, 32px, leading icon. */
  variant?: "item" | "action";
  tone?: "default" | "danger";
  /** radio: a 14px check slot is always reserved; the row sits in a RadioGroup and shows the check when its value is the group's. */
  selection?: "action" | "radio";
  /** submenu: a trailing chevron; press, hover, ArrowRight, Enter or Space open the submenu. */
  trailing?: "none" | "submenu";
  size?: "default" | "large";
  /** Leading Material Symbols name, 15px. Empty for no icon. */
  icon?: string;
  label?: string;
  truncate?: boolean;
  disabled?: boolean;
  /** Radio rows only: the value this row sets in its group. */
  value?: string;
  /** Project rows: hover- and focus-revealed rename and delete buttons, siblings of the row. */
  rowActions?: boolean;
  /** The submenu content, shown while a submenu row is open. */
  submenu?: React.ReactNode;
  onSelect?: (event: Event) => void;
  onSubmenuToggle?: (open: boolean) => void;
  onRenameAction?: () => void;
  onDeleteAction?: () => void;
}

export const MenuItem = React.forwardRef<HTMLDivElement, MenuItemProps>(function MenuItem(
  {
    variant = "item", tone = "default", selection = "action", trailing = "none", size = "default",
    icon = "", label = "Item", truncate = false, disabled = false, value = "",
    rowActions = false, submenu, onSelect, onSubmenuToggle, onRenameAction, onDeleteAction,
    onKeyDown, onPointerMove, className, ...props
  },
  ref,
) {
  const [open, setOpen] = React.useState(false);
  const openRef = React.useRef(false);
  const radio = selection === "radio";
  const hasSubmenu = trailing === "submenu";
  const geometry = variant === "action" || size === "large" ? "large" : "default";

  // Opening only: a press, hover, ArrowRight, Enter or Space opens the submenu and never runs the row.
  const setSubmenu = (next: boolean) => {
    if (next === openRef.current) return;
    openRef.current = next;
    setOpen(next);
    onSubmenuToggle?.(next);
  };

  const handleSelect = (event: Event) => {
    if (hasSubmenu) {
      event.preventDefault();
      setSubmenu(true);
      return;
    }
    onSelect?.(event);
  };

  const handleKeyDown = (event: React.KeyboardEvent<HTMLDivElement>) => {
    onKeyDown?.(event);
    if (!hasSubmenu || event.defaultPrevented) return;
    if (event.key === "ArrowRight") {
      event.preventDefault();
      setSubmenu(true);
    } else if (event.key === "ArrowLeft") {
      event.preventDefault();
      setSubmenu(false);
    }
  };

  const handlePointerMove = (event: React.PointerEvent<HTMLDivElement>) => {
    onPointerMove?.(event);
    if (hasSubmenu && !disabled) setSubmenu(true);
  };

  const content = (
    <>
      {radio && (
        <span aria-hidden="true" className="inline-flex size-[1rem] shrink-0 items-center justify-center">
          <DropdownMenuPrimitive.ItemIndicator className="flex items-center justify-center">
            <Icon data-slot="check" name="check" size={15} weight={900} tone="accent" className="shrink-0" />
          </DropdownMenuPrimitive.ItemIndicator>
        </span>
      )}
      {icon !== "" && (
        <Icon data-slot="icon" name={icon} size={15} weight={300} tone={tone === "danger" ? "danger" : "muted"} className="shrink-0" />
      )}
      <span data-slot="label" className={cn("min-w-0 flex-1", truncate && "truncate")}>
        {label}
      </span>
      {hasSubmenu && <Icon data-slot="chevron" name="chevron_right" size={15} weight={300} tone="muted" className="shrink-0" />}
    </>
  );

  const rowClass = cn(
    menuItemVariants({ variant, tone, size: geometry }),
    rowActions && "flex-1",
    className,
  );
  const rowCommon = {
    "data-slot": "row",
    disabled,
    textValue: label,
    "aria-haspopup": hasSubmenu ? ("menu" as const) : undefined,
    "aria-expanded": hasSubmenu ? open : undefined,
    className: rowClass,
    onSelect: handleSelect,
    onKeyDown: handleKeyDown,
    onPointerMove: handlePointerMove,
    ...props,
  };

  const item = radio ? (
    <DropdownMenuPrimitive.RadioItem ref={ref} {...rowCommon} value={value}>
      {content}
    </DropdownMenuPrimitive.RadioItem>
  ) : (
    <DropdownMenuPrimitive.Item ref={ref} {...rowCommon}>
      {content}
    </DropdownMenuPrimitive.Item>
  );

  const body = rowActions ? (
    <div className="group/row flex min-w-0 items-center">
      {item}
      <Button
        data-slot="renameAction"
        icon="edit"
        shape="icon"
        size="sm"
        reveal="row"
        ariaLabel={`Rename ${label}`}
        description="Rename"
        className="shrink-0"
        onClick={() => onRenameAction?.()}
      />
      <Button
        data-slot="deleteAction"
        icon="delete"
        shape="icon"
        size="sm"
        reveal="row"
        ariaLabel={`Delete ${label}`}
        description="Delete"
        className="shrink-0"
        onClick={() => onDeleteAction?.()}
      />
    </div>
  ) : (
    item
  );

  if (!hasSubmenu || !open) return body;

  return (
    <>
      {body}
      <div data-slot="submenu" role="menu" aria-label={label} className="flex flex-col pl-[1.25rem]">
        {submenu}
      </div>
    </>
  );
});
