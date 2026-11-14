import * as React from "react";
import * as TabsPrimitive from "@radix-ui/react-tabs";
import { cva } from "class-variance-authority";
import { cn } from "../../lib/utils";
import { Icon } from "./icon";
import { Text } from "./text";

/**
 * One pane tab in a vertical settings tablist. Sizes are measured (139x30, radius 4.375px) at
 * 1rem = 14px. The selected fill is the accent tint when the selection is scoped to this pane
 * and the neutral wash when it is not (both proposals in the spec; colours are tokens only).
 */
export const sidebarTabVariants = cva(
  [
    "relative flex h-[2.143rem] w-[9.929rem] max-w-full min-w-0 items-center gap-2 overflow-hidden px-2 text-left font-normal select-none outline-none",
    "rounded-(--radius-field) transition-[opacity,background-color] duration-(--duration-fast) ease-standard",
    "focus-visible:ring-2 focus-visible:ring-focus-1 focus-visible:ring-offset-0",
  ],
  {
    variants: {
      selected: {
        true: "",
        false: "bg-transparent",
      },
      scoped: {
        true: "",
        false: "",
      },
      dimmed: {
        true: "opacity-40",
        false: "opacity-100",
      },
    },
    compoundVariants: [
      { selected: true, scoped: true, className: "bg-accent-500/10" },
      { selected: true, scoped: false, className: "bg-wash-2" },
    ],
    defaultVariants: { selected: false, scoped: true, dimmed: false },
  },
);

export interface SidebarTabProps
  extends Omit<React.ComponentPropsWithoutRef<typeof TabsPrimitive.Trigger>, "value" | "children"> {
  /** Material Symbols name, 16x16. */
  icon?: string;
  /** Pane name; also the tab's value in the parent Tabs root. */
  label?: string;
  /** Current pane. Must match the Tabs root value; the parent sets it. */
  selected?: boolean;
  /** false: selected during a search that is not scoped to this pane. */
  scoped?: boolean;
  /** The search has no matches on this pane: opacity 40 percent, still focusable and pressable. */
  dimmed?: boolean;
  /** Trailing match count; 0 or null hides the badge. */
  badge?: number | null;
}

export const SidebarTab = React.forwardRef<HTMLButtonElement, SidebarTabProps>(function SidebarTab(
  { icon = "", label = "", selected = false, scoped = true, dimmed = false, badge = 0, className, ...props },
  ref,
) {
  const count = badge ?? 0;

  return (
    <TabsPrimitive.Trigger
      ref={ref}
      data-slot="tab"
      value={label}
      className={cn(sidebarTabVariants({ selected, scoped, dimmed }), className)}
      {...props}
    >
      <Icon data-slot="icon" name={icon} size={16} tone="inherit" aria-hidden="true" className="shrink-0 text-content-3" />
      <Text data-slot="label" size="md" tone="primary" overflow="truncate" className="min-w-0 flex-1 leading-[1.393rem]">
        {label}
      </Text>
      {count > 0 && (
        <Text data-slot="badge" size="xs" tone="secondary" numeric="tabular" className="shrink-0">
          {count}
        </Text>
      )}
    </TabsPrimitive.Trigger>
  );
});
