import * as React from "react";
import * as TabsPrimitive from "@radix-ui/react-tabs";
import { cva } from "class-variance-authority";
import { cn } from "../../lib/utils";
import { Button } from "./button";

const TAB_VALUE = "list";

/**
 * Label-only list tab. The host owns `selected`; the widget never changes it.
 * Sizes: 13px text on a 19.5px line (0.929rem / 1.393rem), 24.5px tall (1.75rem).
 * Colours: content-1 active, content-3 inactive (the spec's oklch values match these tokens).
 */
const listTabLabelVariants = cva(
  [
    "group/tab relative inline-flex min-w-0 max-w-[11.429rem] items-center whitespace-nowrap",
    "h-[1.75rem] rounded-(--radius-field) border px-2 gap-[0.357rem]",
    "text-[0.929rem] leading-[1.393rem] font-medium select-none outline-none",
    "transition-[color,background-color,border-color,box-shadow] duration-(--duration-fast) ease-standard",
    "focus-visible:ring-2 focus-visible:ring-focus-1 focus-visible:ring-offset-0",
  ],
  {
    variants: {
      selected: {
        true: "text-content-1 border-edge-2",
        false: "border-transparent text-content-3",
      },
      surface: {
        flat: "bg-transparent",
        raised: "bg-surface-3 shadow-lift",
      },
    },
    defaultVariants: { selected: false, surface: "flat" },
  },
);

export interface ListTabLabelProps extends Omit<React.HTMLAttributes<HTMLDivElement>, "onSelect" | "defaultValue" | "dir"> {
  /** List name. Truncates at max-w-40 (160px = 11.429rem). */
  name?: string;
  /** true = active tab; false = inactive tab. Host-controlled. */
  selected?: boolean;
  /** 'raised' only applies with selected true. */
  surface?: "flat" | "raised";
  /** Split view: decorative blue marker before the name. */
  paneMarker?: boolean;
  /** Shows the close x; applies only when selected is true. */
  closable?: boolean;
  /** Press, Enter or Space on the tab. Not emitted on an already selected tab. */
  onSelect?: () => void;
  /** Press, Enter or Space on the close x. Never selects. */
  onClose?: () => void;
}

export const ListTabLabel = React.forwardRef<HTMLDivElement, ListTabLabelProps>(function ListTabLabel(
  { name = "", selected = false, surface = "flat", paneMarker = false, closable = false, onSelect, onClose, className, ...props },
  ref,
) {
  const showClose = closable && selected;
  const effectiveSurface = selected ? surface : "flat";

  return (
    <TabsPrimitive.Root
      ref={ref}
      {...props}
      value={selected ? TAB_VALUE : ""}
      activationMode="manual"
      onValueChange={() => onSelect?.()}
      className={cn("inline-flex min-w-0 max-w-full items-center", className)}
    >
      {/* The close x sits outside the tablist, so the tab's accessible name excludes it (OPEN 11). */}
      <TabsPrimitive.List className="inline-flex min-w-0 max-w-full items-center">
        <TabsPrimitive.Trigger
          data-slot="tab"
          value={TAB_VALUE}
          aria-controls={undefined}
          className={listTabLabelVariants({ selected, surface: effectiveSurface })}
        >
          {paneMarker && (
            <span data-slot="paneMarker" aria-hidden="true" className="size-[0.5rem] shrink-0 rounded-full bg-accent-500" />
          )}
          <span className="min-w-0 truncate">{name}</span>
        </TabsPrimitive.Trigger>
      </TabsPrimitive.List>
      {showClose && (
        <Button
          data-slot="close"
          variant="ghost"
          size="sm"
          shape="icon"
          tone="default"
          icon="close"
          ariaLabel={`Close ${name}`}
          onClick={() => onClose?.()}
        />
      )}
    </TabsPrimitive.Root>
  );
});

export { listTabLabelVariants };
