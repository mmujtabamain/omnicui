import * as React from "react";
import * as PopoverPrimitive from "@radix-ui/react-popover";
import { cn } from "../../lib/utils";

export interface PopoverItemProps extends Omit<React.HTMLAttributes<HTMLDivElement>, "value"> {
  /** The text picked when this row is chosen. */
  value: string;
}

interface OptionContext {
  index: number;
  id: string;
  active: boolean;
  onPick: (value: string) => void;
}

const OptionContext = React.createContext<OptionContext | null>(null);

/** One mention row. Rendered by Popover, which numbers it (row1 to row6) and links it to the editor. */
export function PopoverItem({ value, children, className, ...props }: PopoverItemProps) {
  const option = React.useContext(OptionContext);
  return (
    <div
      {...props}
      data-slot={option ? `row${option.index + 1}` : undefined}
      role="option"
      id={option?.id}
      aria-selected={option?.active ?? false}
      data-active={option?.active || undefined}
      onMouseDown={(e) => {
        // Pointer picks never take focus from the editor.
        e.preventDefault();
        option?.onPick(value);
      }}
      className={cn(
        "flex h-[1.857rem] min-w-0 cursor-default items-center truncate rounded-(--radius-field) px-2 text-[0.929rem] text-content-1 select-none",
        "hover:bg-wash-1 data-[active=true]:bg-selection-1",
        className,
      )}
    >
      {children ?? value}
    </div>
  );
}

export interface PopoverProps {
  /** Text in the @ input that owns the list. */
  value?: string;
  onValueChange?: (value: string) => void;
  /** Shown when there are matches for the @ query. */
  open?: boolean;
  /** Caret point, in viewport pixels. */
  anchor?: { x: number; y: number };
  /** Most rows shown. */
  maxItems?: number;
  placeholder?: string;
  ariaLabel?: string;
  /** Escape closed the list; the typed text stays in the editor. */
  onEscape?: () => void;
  /** A row was picked, by Enter, Tab or a pointer press. */
  onPick?: (value: string) => void;
  /** PopoverItem rows. */
  children?: React.ReactNode;
}

const ORIGIN = { x: 0, y: 0 };

/**
 * The @-mention popover: a combobox editor whose listbox opens at the caret. Focus stays in the
 * editor (APG combobox pattern); the highlighted row is linked by aria-activedescendant.
 */
export function Popover({
  value = "",
  onValueChange,
  open = false,
  anchor = ORIGIN,
  maxItems = 6,
  placeholder = "",
  ariaLabel = "Mention",
  onEscape,
  onPick,
  children,
}: PopoverProps) {
  const listId = React.useId();
  const [active, setActive] = React.useState(-1);
  const items = React.Children.toArray(children)
    .filter(React.isValidElement)
    .slice(0, Math.max(0, maxItems)) as React.ReactElement<PopoverItemProps>[];
  const highlighted = active < items.length ? active : -1;
  const isOpen = open && items.length > 0;
  const optionId = (i: number) => `${listId}-option-${i}`;

  const virtualRef = React.useMemo(
    () => ({ current: { getBoundingClientRect: () => new DOMRect(anchor.x, anchor.y, 0, 0) } }),
    [anchor.x, anchor.y],
  );

  const pick = (picked: string) => {
    setActive(-1);
    onPick?.(picked);
  };

  const onKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (!isOpen) return;
    const n = items.length;
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setActive(highlighted < 0 ? 0 : (highlighted + 1) % n);
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setActive(highlighted <= 0 ? n - 1 : highlighted - 1);
    } else if ((e.key === "Enter" || e.key === "Tab") && highlighted >= 0) {
      e.preventDefault();
      pick(items[highlighted].props.value);
    }
  };

  return (
    <PopoverPrimitive.Root open={isOpen}>
      <PopoverPrimitive.Anchor virtualRef={virtualRef} />
      <input
        data-slot="editor"
        type="text"
        role="combobox"
        aria-label={ariaLabel}
        aria-autocomplete="list"
        aria-expanded={isOpen}
        aria-controls={isOpen ? listId : undefined}
        aria-activedescendant={highlighted >= 0 ? optionId(highlighted) : undefined}
        placeholder={placeholder}
        value={value}
        onChange={(e) => {
          setActive(-1);
          onValueChange?.(e.target.value);
        }}
        onKeyDown={onKeyDown}
        className={cn(
          "h-[2rem] w-full min-w-0 rounded-(--radius-field) border border-edge-2 bg-surface-2 px-2",
          "text-[0.929rem] text-content-1 placeholder:text-content-3 outline-none",
          "transition-[border-color] duration-(--duration-fast) ease-standard",
          "focus-visible:border-accent-500 focus-visible:ring-2 focus-visible:ring-focus-1",
        )}
      />
      <PopoverPrimitive.Portal>
        <PopoverPrimitive.Content
          side="bottom"
          align="start"
          sideOffset={6}
          collisionPadding={8}
          onOpenAutoFocus={(e) => e.preventDefault()}
          onCloseAutoFocus={(e) => e.preventDefault()}
          onEscapeKeyDown={(e) => {
            e.preventDefault();
            onEscape?.();
          }}
          className={cn(
            "z-(--z-index-dropdown) w-60 rounded-(--radius-control) border border-edge-2 bg-surface-3 p-1 shadow-menu outline-none animate-rise",
          )}
        >
          <div data-slot="list" role="listbox" id={listId} aria-label={ariaLabel} className="max-h-56 overflow-y-auto">
            {items.map((item, i) => (
              <OptionContext.Provider
                key={i}
                value={{ index: i, id: optionId(i), active: i === highlighted, onPick: pick }}
              >
                {item}
              </OptionContext.Provider>
            ))}
          </div>
        </PopoverPrimitive.Content>
      </PopoverPrimitive.Portal>
    </PopoverPrimitive.Root>
  );
}
