import * as React from "react";
import * as SeparatorPrimitive from "@radix-ui/react-separator";
import { cva } from "class-variance-authority";
import { cn } from "../../lib/utils";

/**
 * Sizes are the spec's measured pixels at 1rem = 14px. Vertical length applies to vertical rules
 * only; the Separator passes it through as undefined for horizontal rules.
 */
export const separatorVariants = cva("shrink-0", {
  variants: {
    orientation: {
      horizontal: "h-px",
      vertical: "w-px",
    },
    length: {
      xs: "h-[0.786rem]",
      sm: "h-[1rem]",
      full: "h-full",
      fill: "h-auto self-stretch my-[0.571rem]",
    },
    inset: {
      none: "",
      menu: "mx-[1.429rem] my-[0.857rem]",
      row: "absolute right-0 bottom-0",
      label: "min-w-0 flex-1 self-center",
    },
    emphasis: {
      default: "bg-edge-1",
      adjacent: "bg-edge-2",
    },
  },
  defaultVariants: { orientation: "horizontal", inset: "none", emphasis: "default" },
});

export interface SeparatorProps
  extends Omit<React.ComponentPropsWithoutRef<typeof SeparatorPrimitive.Root>, "orientation" | "decorative"> {
  orientation?: "horizontal" | "vertical";
  /** Vertical length only. */
  length?: "xs" | "sm" | "full" | "fill";
  inset?: "none" | "menu" | "row" | "label";
  emphasis?: "default" | "adjacent";
  /** Whole rows of nesting; used by the row inset only. */
  depth?: number;
  /** true gives role none; false gives role separator. */
  decorative?: boolean;
  className?: string;
}

export const Separator = React.forwardRef<HTMLDivElement, SeparatorProps>(function Separator(
  {
    orientation = "horizontal", length = "sm", inset = "none", emphasis = "default",
    depth = 0, decorative = true, className, style, ...props
  },
  ref,
) {
  return (
    <SeparatorPrimitive.Root
      ref={ref}
      data-slot="rule"
      orientation={orientation}
      decorative={decorative}
      className={cn(
        separatorVariants({
          orientation,
          length: orientation === "vertical" ? length : undefined,
          inset,
          emphasis,
        }),
        className,
      )}
      style={inset === "row" ? { left: `${(depth * 24 + 44) / 14}rem`, ...style } : style}
      {...props}
    />
  );
});
