import * as React from "react";
import * as DropdownMenuPrimitive from "@radix-ui/react-dropdown-menu";
import { Slot } from "@radix-ui/react-slot";
import { cva } from "class-variance-authority";
import { cn } from "../../lib/utils";

/**
 * The one role=menu floating panel. Radix DropdownMenu serves every placement. Control and submenu
 * menus anchor to the trigger (Radix Popper, below it by `offset`). Cursor and anchored menus anchor to
 * a zero-size point at pointX/pointY, clamped by Radix collision handling with a 4px inset. Widths are
 * the measured pixels at 1rem = 14px: task 152px, tab 156px, project min 210px.
 */
export const menuPanelVariants = cva(
  "z-(--z-index-dropdown) flex flex-col max-h-(--radix-dropdown-menu-content-available-height) overflow-y-auto rounded-(--radius-panel) bg-surface-3 py-space-3 text-content-1 shadow-menu outline-none animate-panel-in",
  {
    variants: {
      width: {
        content: "w-max",
        task: "w-[10.857rem]",
        tab: "w-[11.143rem]",
        project: "min-w-[15rem]",
      },
    },
    defaultVariants: { width: "content" },
  },
);

export type MenuPlacement = "cursor" | "control" | "submenu" | "anchored";
export type MenuWidth = "content" | "task" | "tab" | "project";
export type MenuOpenReason = "keyboard" | "pointer" | "escape" | "outside" | "tab" | "select" | "blur" | "resize";

export interface MenuProps {
  /** Controlled open state. */
  open?: boolean;
  /** Fired on every open change, with the reason. */
  onOpenChange?: (open: boolean, reason: MenuOpenReason) => void;
  placement?: MenuPlacement;
  width?: MenuWidth;
  /** Gap below the trigger for control placement, in px. */
  offset?: number;
  /** Pointer position in px for cursor and anchored placement. */
  pointX?: number;
  pointY?: number;
  /** Whether activating a row closes the menu. */
  closeOnSelect?: boolean;
  ariaLabel?: string;
  /** The element that opens the menu. */
  trigger?: React.ReactElement;
  /** MenuItem and MenuSeparator rows. */
  children?: React.ReactNode;
}

type Point = { x: number; y: number };

/**
 * A floating menu. Rows are MenuItem and Separator. Keyboard, pointer and dismiss rules follow the WAI-ARIA
 * menu pattern: arrows skip disabled rows and do not wrap, Escape and Tab close, Escape in a submenu closes
 * only the submenu, and Shift+F10 or the ContextMenu key open a cursor menu.
 */
export function Menu({
  open = false,
  onOpenChange,
  placement = "cursor",
  width = "content",
  offset = 4,
  pointX = 0,
  pointY = 0,
  closeOnSelect = true,
  ariaLabel = "Menu",
  trigger,
  children,
}: MenuProps) {
  const control = placement === "control" || placement === "submenu";
  const triggerRef = React.useRef<HTMLElement | null>(null);
  const openRef = React.useRef(open);
  openRef.current = open;
  // The reason for the next open change. Radix reports closes without one, so the handler that causes a close sets it.
  const reasonRef = React.useRef<MenuOpenReason>("select");
  // Set by an outside press or focus; the focus then does not return to the trigger.
  const outsideRef = React.useRef(false);
  const [clickPoint, setClickPoint] = React.useState<Point | null>(null);

  const change = (next: boolean, reason: MenuOpenReason) => {
    if (openRef.current === next) return;
    openRef.current = next;
    reasonRef.current = "select";
    onOpenChange?.(next, reason);
  };

  const handleRootChange = (next: boolean) => {
    if (next) {
      change(true, reasonRef.current === "select" ? "pointer" : reasonRef.current);
      return;
    }
    if (reasonRef.current === "select" && !closeOnSelect) return;
    change(false, reasonRef.current);
  };

  // A submenu closes with ArrowLeft on its row: the row's own handler does it, and focus returns to the row.
  const closeSubmenu = (panel: Element) => {
    const row = panel.parentElement?.parentElement?.querySelector<HTMLElement>(':scope > [aria-haspopup="menu"]');
    row?.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowLeft", bubbles: true, cancelable: true }));
    row?.focus();
  };

  const onEscape = (event: KeyboardEvent) => {
    const panel = (event.target as Element | null)?.closest('[data-slot="submenuPanel"]');
    if (panel) {
      event.preventDefault();
      closeSubmenu(panel);
      return;
    }
    change(false, "escape");
  };

  const onContentKeyDown = (event: React.KeyboardEvent<HTMLDivElement>) => {
    if (event.key === "Tab") {
      change(false, "tab");
      return;
    }
    if (event.key === "ArrowLeft") {
      const panel = (event.target as Element).closest('[data-slot="submenuPanel"]');
      if (panel) {
        event.preventDefault();
        closeSubmenu(panel);
      }
    }
  };

  const triggerHandlers = control
    ? {
        onPointerDown: () => {
          reasonRef.current = openRef.current ? "outside" : "pointer";
        },
        onKeyDown: () => {
          reasonRef.current = "keyboard";
        },
      }
    : {
        onKeyDown: (event: React.KeyboardEvent) => {
          if (event.key === "ContextMenu" || (event.key === "F10" && event.shiftKey)) {
            event.preventDefault();
            setClickPoint(null);
            change(true, "keyboard");
          }
        },
        onContextMenu: (event: React.MouseEvent) => {
          event.preventDefault();
          setClickPoint({ x: event.clientX, y: event.clientY });
          change(true, "pointer");
        },
      };

  const triggerNode = trigger ? (
    <Slot
      ref={triggerRef}
      data-slot="trigger"
      aria-haspopup="menu"
      aria-expanded={open}
      {...triggerHandlers}
    >
      {trigger}
    </Slot>
  ) : null;

  const point = clickPoint ?? { x: pointX, y: pointY };

  return (
    <DropdownMenuPrimitive.Root open={open} onOpenChange={handleRootChange} modal={false}>
      {control ? (
        triggerNode && <DropdownMenuPrimitive.Trigger asChild>{triggerNode}</DropdownMenuPrimitive.Trigger>
      ) : (
        <>
          {triggerNode}
          <DropdownMenuPrimitive.Trigger asChild>
            <span aria-hidden="true" style={{ position: "fixed", left: point.x, top: point.y, width: 0, height: 0 }} />
          </DropdownMenuPrimitive.Trigger>
        </>
      )}
      <DropdownMenuPrimitive.Portal>
        <DropdownMenuPrimitive.Content
          data-slot="panel"
          aria-label={ariaLabel}
          side={placement === "submenu" ? "right" : "bottom"}
          align="start"
          sideOffset={control ? offset : 0}
          avoidCollisions={placement !== "control"}
          collisionPadding={4}
          className={menuPanelVariants({ width })}
          onKeyDown={onContentKeyDown}
          onEscapeKeyDown={onEscape}
          onPointerDownOutside={() => {
            reasonRef.current = "outside";
            outsideRef.current = true;
          }}
          onFocusOutside={() => {
            reasonRef.current = "blur";
            outsideRef.current = true;
          }}
          onCloseAutoFocus={(event) => {
            event.preventDefault();
            if (!outsideRef.current) triggerRef.current?.focus();
            outsideRef.current = false;
          }}
        >
          {children}
        </DropdownMenuPrimitive.Content>
      </DropdownMenuPrimitive.Portal>
    </DropdownMenuPrimitive.Root>
  );
}

/**
 * The panel of a nested menu, passed as MenuItem's `submenu`. Focus moves to its first enabled row when it
 * mounts, so arrow keys work at once.
 */
export function MenuSubmenuPanel({ ariaLabel = "Submenu", children }: { ariaLabel?: string; children?: React.ReactNode }) {
  const ref = React.useRef<HTMLDivElement>(null);
  React.useEffect(() => {
    ref.current?.querySelector<HTMLElement>('[role="menuitem"]:not([data-disabled])')?.focus();
  }, []);
  return (
    <div ref={ref} role="menu" aria-label={ariaLabel} data-slot="submenuPanel" className={cn("flex flex-col")}>
      {children}
    </div>
  );
}
