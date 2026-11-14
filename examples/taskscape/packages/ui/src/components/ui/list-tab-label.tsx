import * as React from "react";
import * as TabsPrimitive from "@radix-ui/react-tabs";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/utils";
import { Button } from "./button";

const TAB = "tab";

/**
 * Label-only list tab. Sizes at 1rem = 14px: text 13px/19.5px at weight 500 is
 * 0.929rem / 1.393rem; the label truncates at 160px (max-w-40 in the spec, 11.429rem here).
 * Colours: active content-1, inactive content-3 (the spec's oklch values match the tokens).
 * The hairline frame around the active flat label is styling of selected (OPEN 5).
 */
const listTabLabelVariants = cva(
  [
    "relative inline-flex min-w-0 items-center gap-[0.357rem] whitespace-nowrap select-none",
    "h-[1.75rem] px-2 rounded-(--radius-field) font-medium text-[0.929rem] leading-[1.393rem]",
    "outline-none transition-[color,background-color,box-shadow] duration-(--duration-fast) ease-standard",
    "focus-visible:ring-2 focus-visible:ring-focus-1 focus-visible:ring-offset-0",
  ],
  {
    variants: {
      selected: {
        true: "text-content-1",
        false: "text-content-3",
      },
      surface: {
        flat: "",
        raised: "bg-surface-3 shadow-lift",
      },
    },
    compoundVariants: [
      { selected: true, surface: "flat", className: "ring-1 ring-inset ring-edge-2" },
    ],
    defaultVariants: { selected: false, surface: "flat" },
  },
);

type Variants = VariantProps<typeof listTabLabelVariants>;

export interface ListTabLabelProps extends Omit<React.HTMLAttributes<HTMLDivElement>, "onSelect" | "defaultValue" | "dir"> {
  /** List name. Truncates at 160px. */
  name?: string;
  /** Active tab. Host-controlled: the widget never changes it. */
  selected?: boolean;
  /** Raised applies only with selected true (draft-inspector). */
  surface?: Variants["surface"];
  /** Split view: decorative blue marker before the name, outside the accessible name. */
  paneMarker?: boolean;
  /** Shows the close x; rendered only when closable and selected. */
  closable?: boolean;
  /** Press, Enter or Space on the tab. Never fires on an already selected tab. */
  onSelect?: () => void;
  /** Press, Enter or Space on the close x. Never selects. */
  onClose?: () => void;
}

export const ListTabLabel = React.forwardRef<HTMLDivElement, ListTabLabelProps>(function ListTabLabel(
  {
    name = "",
    selected = false,
    surface = "flat",
    paneMarker = false,
    closable = false,
    onSelect,
    onClose,
    className,
    ...props
  },
  ref,
) {
  const showClose = closable && selected;

  return (
    <TabsPrimitive.Root
      ref={ref}
      value={selected ? TAB : ""}
      activationMode="manual"
      onValueChange={() => onSelect?.()}
      className={cn("inline-flex min-w-0 max-w-full items-center", className)}
      {...props}
    >
      <TabsPrimitive.List className="inline-flex min-w-0">
        <TabsPrimitive.Trigger
          value={TAB}
          role="button"
          data-slot="tab"
          className={cn(listTabLabelVariants({ selected, surface }), "max-w-[11.429rem]")}
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
          size="sm"
          shape="icon"
          variant="ghost"
          tone="muted"
          icon="close"
          ariaLabel={`Close ${name}`}
          className="shrink-0"
          onClick={() => onClose?.()}
        />
      )}
    </TabsPrimitive.Root>
  );
});

export { listTabLabelVariants };
