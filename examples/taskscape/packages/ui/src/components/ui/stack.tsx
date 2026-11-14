import * as React from "react";
import { Slot } from "@radix-ui/react-slot";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/utils";

/**
 * A plain flex wrapper: no fill, no colour, no role (the generic role). The entrance
 * animation is the only motion; gap values are the space tokens.
 */
const stackVariants = cva("flex", {
  variants: {
    direction: {
      row: "flex-row",
      column: "flex-col",
    },
    justify: {
      start: "justify-start",
      center: "justify-center",
      end: "justify-end",
      between: "justify-between",
    },
    align: {
      start: "items-start",
      center: "items-center",
      end: "items-end",
      stretch: "items-stretch",
    },
    grow: {
      none: "",
      fill: "flex-1 min-w-0",
    },
    gap: {
      none: "",
      "space-3": "gap-space-3",
      "space-4": "gap-space-4",
      "space-5": "gap-space-5",
      "space-6": "gap-space-6",
    },
    motion: {
      none: "",
      rise: "animate-rise",
    },
  },
  compoundVariants: [
    // A filling column also lets its height shrink below its content.
    { grow: "fill", direction: "column", className: "min-h-0" },
  ],
  defaultVariants: {
    direction: "row",
    justify: "start",
    align: "stretch",
    grow: "none",
    gap: "none",
    motion: "none",
  },
});

type Variants = VariantProps<typeof stackVariants>;

export interface StackProps extends Omit<React.HTMLAttributes<HTMLDivElement>, "align"> {
  direction?: Variants["direction"];
  justify?: Variants["justify"];
  align?: Variants["align"];
  grow?: Variants["grow"];
  gap?: Variants["gap"];
  motion?: Variants["motion"];
  /** Desktop: a window drag region (data-tauri-drag-region). On web it is an empty spacer. */
  dragRegion?: boolean;
  /** true: the layout classes go onto the single child element instead of a wrapper. */
  asChild?: boolean;
}

export const Stack = React.forwardRef<HTMLDivElement, StackProps>(function Stack(
  {
    direction = "row",
    justify = "start",
    align = "stretch",
    grow = "none",
    gap = "none",
    motion = "none",
    dragRegion = false,
    asChild = false,
    className,
    ...props
  },
  ref,
) {
  const Comp = asChild ? Slot : "div";

  return (
    <Comp
      ref={ref}
      data-slot="root"
      data-tauri-drag-region={dragRegion || undefined}
      className={cn(stackVariants({ direction, justify, align, grow, gap, motion }), className)}
      {...props}
    />
  );
});

export { stackVariants };
