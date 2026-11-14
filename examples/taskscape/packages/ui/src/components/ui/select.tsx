import * as React from "react";
import * as SelectPrimitive from "@radix-ui/react-select";
import { cva } from "class-variance-authority";
import { cn } from "../../lib/utils";
import { Icon } from "./icon";

export type SelectOptionSet = "sort" | "dateRange";

export interface SelectOption {
  value: string;
  label: string;
}

/** The fixed presets: sort (five modes) and dateRange (five ranges). */
export const SELECT_OPTION_SETS: Record<SelectOptionSet, readonly SelectOption[]> = {
  sort: [
    { value: "manual", label: "Manual" },
    { value: "dateCreated", label: "Date created" },
    { value: "dateUpdated", label: "Date updated" },
    { value: "alphabetical", label: "Alphabetical" },
    { value: "doneLast", label: "Done last" },
  ],
  dateRange: [
    { value: "anyTime", label: "Any time" },
    { value: "last24Hours", label: "Last 24 hours" },
    { value: "last7Days", label: "Last 7 days" },
    { value: "last30Days", label: "Last 30 days" },
    { value: "custom", label: "Custom..." },
  ],
};

/** A neutral Button-style field: 26px tall, 12.5px text at weight 500, wash on hover. */
export const selectTriggerVariants = cva(
  [
    "group/select relative inline-flex min-w-0 items-center gap-1.5 select-none whitespace-nowrap pointer-events-auto",
    "h-[1.857rem] px-2 rounded-(--radius-field) bg-surface-0 text-content-1",
    "text-[0.893rem] leading-[1.339rem] font-medium text-left",
    "outline-none transition-[background-color,opacity] duration-(--duration-fast) ease-standard",
    "enabled:hover:bg-wash-1 focus-visible:ring-2 focus-visible:ring-focus-1 focus-visible:ring-offset-0",
    "disabled:opacity-45",
  ],
  {
    variants: {
      width: {
        full: "w-full",
        content: "w-auto",
      },
    },
    defaultVariants: { width: "full" },
  },
);

const selectItemClass = cn(
  "relative flex w-full cursor-default select-none items-center rounded-(--radius-field)",
  "py-[0.286rem] pr-2 pl-7 text-[0.893rem] leading-[1.339rem] font-medium text-content-1 outline-none",
  "data-[highlighted]:bg-wash-2 data-[disabled]:pointer-events-none data-[disabled]:opacity-45",
);

/** Only a printable character opens or searches: Space keeps its Select meaning. */
function isTypeKey(event: React.KeyboardEvent): boolean {
  return event.key.length === 1 && event.key !== " " && !event.ctrlKey && !event.metaKey && !event.altKey;
}

/**
 * The option index that a typed prefix selects. A repeated letter ("dd") moves on to the
 * next option that starts with it; a longer prefix keeps the current option when it matches.
 */
function matchIndex(labels: readonly string[], text: string, from: number): number {
  const n = labels.length;
  const repeated = text.length > 1 && text.split("").every((c) => c === text[0]);
  const start = repeated ? from + 1 : from;
  const prefix = repeated ? text[0] : text;
  for (let step = 0; step < n; step++) {
    const i = (((start + step) % n) + n) % n;
    if (labels[i].toLowerCase().startsWith(prefix)) return i;
  }
  return -1;
}

const SEARCH_RESET_MS = 1000;

export interface SelectProps
  extends Omit<React.ComponentPropsWithoutRef<typeof SelectPrimitive.Trigger>, "value" | "defaultValue" | "children"> {
  /** Current item value (controlled). The trigger keeps it until the caller changes it. */
  value: string;
  /** Called once per user pick with the item value. */
  onValueChange?: (value: string) => void;
  optionSet?: SelectOptionSet;
  /** Trigger glyph name; "" shows no glyph. */
  leadingIcon?: string;
  fullWidth?: boolean;
  disabled?: boolean;
  /** Accessible name of the field, for example "Sort". */
  ariaLabel?: string;
}

export const Select = React.forwardRef<HTMLButtonElement, SelectProps>(function Select(
  {
    value, onValueChange, optionSet = "sort", leadingIcon = "", fullWidth = true, disabled = false,
    ariaLabel = "", className, onKeyDown, ...props
  },
  ref,
) {
  const [open, setOpen] = React.useState(false);
  const options = SELECT_OPTION_SETS[optionSet] ?? SELECT_OPTION_SETS.sort;
  const labels = options.map((option) => option.label);
  const current = options.find((option) => option.value === value)?.label ?? "";
  const selectedIndex = Math.max(0, options.findIndex((option) => option.value === value));

  const listRef = React.useRef<HTMLDivElement>(null);
  const search = React.useRef({ text: "", timer: 0 });

  React.useEffect(() => () => window.clearTimeout(search.current.timer), []);

  /** Adds a key to the typed prefix and returns the prefix. */
  const typeAhead = (key: string): string => {
    window.clearTimeout(search.current.timer);
    search.current.text += key.toLowerCase();
    search.current.timer = window.setTimeout(() => {
      search.current.text = "";
    }, SEARCH_RESET_MS);
    return search.current.text;
  };

  const optionElements = (): HTMLElement[] =>
    Array.from(listRef.current?.querySelectorAll<HTMLElement>('[role="option"]') ?? []);

  // Closed trigger: a typed letter opens the list on the first match, without a pick.
  const handleTriggerKeyDown = (event: React.KeyboardEvent<HTMLButtonElement>) => {
    onKeyDown?.(event);
    if (event.defaultPrevented || open || !isTypeKey(event)) return;
    event.preventDefault();
    const text = typeAhead(event.key);
    setOpen(true);
    // The list mounts and focuses the current item first; the match takes focus after that.
    window.setTimeout(() => optionElements()[matchIndex(labels, text, selectedIndex)]?.focus(), 0);
  };

  // Open list: a typed letter moves the highlight; the prefix builds up across keys.
  const handleListKeyDown = (event: React.KeyboardEvent<HTMLDivElement>) => {
    if (!isTypeKey(event)) return;
    // Stops Radix's own typeahead on the content, so only this search moves the highlight.
    event.preventDefault();
    event.stopPropagation();
    const text = typeAhead(event.key);
    const items = optionElements();
    const from = Math.max(0, items.indexOf(document.activeElement as HTMLElement));
    const index = matchIndex(labels, text, from);
    if (index >= 0) items[index]?.focus();
  };

  return (
    <SelectPrimitive.Root
      value={value}
      open={open}
      onOpenChange={setOpen}
      onValueChange={(next: string) => onValueChange?.(next)}
      disabled={disabled}
    >
      <SelectPrimitive.Trigger
        ref={ref}
        data-slot="trigger"
        aria-label={ariaLabel || undefined}
        onKeyDown={handleTriggerKeyDown}
        className={cn(selectTriggerVariants({ width: fullWidth ? "full" : "content" }), className)}
        {...props}
      >
        {leadingIcon && <Icon data-slot="icon" name={leadingIcon} size={16} tone="inherit" className="-ml-0.5" />}
        <span data-slot="current" className="min-w-0 flex-1 truncate">
          {current}
        </span>
        <Icon name="expand_more" size={16} className="-mr-0.5" />
      </SelectPrimitive.Trigger>
      <SelectPrimitive.Portal>
        <SelectPrimitive.Content
          ref={listRef}
          data-slot="listbox"
          position="popper"
          sideOffset={4}
          collisionPadding={0}
          className={cn(
            "z-(--z-index-popover) min-w-(--radix-select-trigger-width) max-h-(--radix-select-content-available-height)",
            "overflow-hidden rounded-(--radius-panel) border border-edge-2 bg-surface-3 text-content-1 shadow-menu",
            "data-[state=open]:animate-[scale-in_var(--duration-fast)_var(--ease-standard)_both]",
          )}
        >
          <SelectPrimitive.Viewport className="p-1" onKeyDown={handleListKeyDown}>
            {options.map((option, index) => (
              <SelectPrimitive.Item key={option.value} data-slot={`option${index + 1}`} value={option.value} className={selectItemClass}>
                {/* The check is drawn from its glyph code, so the option text holds only its label. */}
                <SelectPrimitive.ItemIndicator
                  aria-hidden="true"
                  className="absolute left-2 flex size-[1.143rem] items-center justify-center text-[1.143rem] leading-none [font-family:'Material_Symbols_Outlined'] [font-variation-settings:'wght'_300] before:content-['\e5ca']"
                />
                <SelectPrimitive.ItemText>{option.label}</SelectPrimitive.ItemText>
              </SelectPrimitive.Item>
            ))}
          </SelectPrimitive.Viewport>
        </SelectPrimitive.Content>
      </SelectPrimitive.Portal>
    </SelectPrimitive.Root>
  );
});
