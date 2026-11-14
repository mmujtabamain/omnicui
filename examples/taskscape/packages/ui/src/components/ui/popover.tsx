import * as React from "react";
import * as PopoverPrimitive from "@radix-ui/react-popover";
import { cn } from "../../lib/utils";

export interface PopoverPoint {
  x: number;
  y: number;
}

interface PopoverContextValue {
  open: boolean;
  /** Rows shown: the matches, capped at maxItems. */
  count: number;
  /** Highlighted row index, -1 when there are no rows. */
  active: number;
  listId: string;
  optionId: (index: number) => string;
  onActiveChange?: (index: number) => void;
  onPick?: (index: number) => void;
  onEscape?: () => void;
}

const PopoverContext = React.createContext<PopoverContextValue | null>(null);

function usePopover(part: string): PopoverContextValue {
  const ctx = React.useContext(PopoverContext);
  if (!ctx) throw new Error(`${part} must be rendered inside <Popover>`);
  return ctx;
}

export interface PopoverProps {
  /** Shown when there are matches for the @ query. */
  open?: boolean;
  /** Caret point in viewport px; the list is placed against it. */
  anchor?: PopoverPoint;
  /** Most rows shown. */
  maxItems?: number;
  /** Number of matches the caller has for the query (before the cap). */
  matchCount?: number;
  /** Highlighted row index. */
  active?: number;
  onActiveChange?: (index: number) => void;
  /** A row was picked, by Enter, Tab or a mousedown on it. */
  onPick?: (index: number) => void;
  /** Escape closed the list; focus and typed text stay in the editor. */
  onEscape?: () => void;
  /** Radix dismissal (pointer down outside the list). */
  onOpenChange?: (open: boolean) => void;
  children?: React.ReactNode;
}

/** Anchored floating list for the mention autocomplete. Renders no DOM of its own. */
export function Popover({
  open = false,
  anchor = { x: 0, y: 0 },
  maxItems = 6,
  matchCount = 0,
  active = 0,
  onActiveChange,
  onPick,
  onEscape,
  onOpenChange,
  children,
}: PopoverProps) {
  const listId = React.useId();
  const count = Math.max(0, Math.min(matchCount, maxItems));
  const isOpen = open && count > 0;
  const highlight = count > 0 ? Math.min(Math.max(active, 0), count - 1) : -1;
  const { x, y } = anchor;
  // A zero-size virtual element at the caret; Radix places the list against it.
  const virtualRef = React.useMemo(
    () => ({ current: { getBoundingClientRect: () => new DOMRect(x, y, 0, 0) } }),
    [x, y],
  );

  return (
    <PopoverContext.Provider
      value={{
        open: isOpen,
        count,
        active: highlight,
        listId,
        optionId: (index: number) => `${listId}-option-${index}`,
        onActiveChange,
        onPick,
        onEscape,
      }}
    >
      <PopoverPrimitive.Root open={isOpen} onOpenChange={onOpenChange}>
        <PopoverPrimitive.Anchor virtualRef={virtualRef} />
        {children}
      </PopoverPrimitive.Root>
    </PopoverContext.Provider>
  );
}

/** The @ input that owns the list (APG combobox). Focus stays here throughout. */
export const PopoverEditor = React.forwardRef<HTMLInputElement, React.InputHTMLAttributes<HTMLInputElement>>(
  function PopoverEditor({ className, onKeyDown, ...props }, ref) {
    const ctx = usePopover("PopoverEditor");

    const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
      onKeyDown?.(e);
      if (e.defaultPrevented || !ctx.open) return;
      const last = ctx.count - 1;
      switch (e.key) {
        case "ArrowDown":
          e.preventDefault();
          ctx.onActiveChange?.(ctx.active >= last ? 0 : ctx.active + 1);
          break;
        case "ArrowUp":
          e.preventDefault();
          ctx.onActiveChange?.(ctx.active <= 0 ? last : ctx.active - 1);
          break;
        case "Enter":
        case "Tab":
          if (ctx.active < 0) break;
          e.preventDefault();
          ctx.onPick?.(ctx.active);
          break;
        default:
          break;
      }
    };

    return (
      <input
        ref={ref}
        data-slot="editor"
        type="text"
        role="combobox"
        autoComplete="off"
        aria-autocomplete="list"
        aria-expanded={ctx.open}
        aria-controls={ctx.listId}
        aria-activedescendant={ctx.open && ctx.active >= 0 ? ctx.optionId(ctx.active) : undefined}
        className={cn(
          "h-[1.75rem] w-full min-w-0 rounded-(--radius-field) border border-edge-2 bg-surface-3 px-2 text-[0.929rem] text-content-1 outline-none placeholder:text-content-3 focus-visible:ring-2 focus-visible:ring-focus-1",
          className,
        )}
        onKeyDown={handleKeyDown}
        {...props}
      />
    );
  },
);

/** The listbox. Closed, it is not rendered; Escape asks the caller to close it. */
export const PopoverList = React.forwardRef<HTMLDivElement, React.HTMLAttributes<HTMLDivElement>>(
  function PopoverList({ className, children, ...props }, ref) {
    const ctx = usePopover("PopoverList");
    return (
      <PopoverPrimitive.Portal>
        <PopoverPrimitive.Content
          ref={ref}
          id={ctx.listId}
          role="listbox"
          data-slot="list"
          side="bottom"
          align="start"
          sideOffset={4}
          collisionPadding={8}
          onOpenAutoFocus={(e) => e.preventDefault()}
          onCloseAutoFocus={(e) => e.preventDefault()}
          onEscapeKeyDown={(e) => {
            e.preventDefault();
            ctx.onEscape?.();
          }}
          className={cn(
            "z-(--z-index-dropdown) max-h-56 w-60 overflow-y-auto rounded-(--radius-panel) border border-edge-2 bg-surface-3 p-1 shadow-menu outline-none",
            className,
          )}
          {...props}
        >
          {children}
        </PopoverPrimitive.Content>
      </PopoverPrimitive.Portal>
    );
  },
);

export interface PopoverItemProps extends React.HTMLAttributes<HTMLDivElement> {
  /** Position in the list, from 0. Rows past maxItems are not rendered. */
  index: number;
}

/** A mention row (role option). Mousedown picks it and keeps focus in the editor. */
export const PopoverItem = React.forwardRef<HTMLDivElement, PopoverItemProps>(
  function PopoverItem({ index, className, children, onMouseDown, onMouseEnter, ...props }, ref) {
    const ctx = usePopover("PopoverItem");
    if (index >= ctx.count) return null;
    const highlighted = index === ctx.active;
    return (
      <div
        ref={ref}
        role="option"
        id={ctx.optionId(index)}
        data-slot="row"
        aria-selected={highlighted}
        data-highlighted={highlighted || undefined}
        onMouseEnter={(e) => {
          onMouseEnter?.(e);
          ctx.onActiveChange?.(index);
        }}
        onMouseDown={(e) => {
          e.preventDefault();
          onMouseDown?.(e);
          ctx.onPick?.(index);
        }}
        className={cn(
          "flex h-[1.75rem] cursor-default select-none items-center rounded-(--radius-field) px-2 text-[0.929rem] text-content-1",
          highlighted && "bg-accent-500 text-on-accent",
          className,
        )}
        {...props}
      >
        <span className="min-w-0 truncate">{children}</span>
      </div>
    );
  },
);
