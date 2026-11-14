import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/utils";

/**
 * Insertion marker drawn while a pointer drag is over a valid drop target. Presentational: the
 * drop result is announced by the target that accepts it. Drawn in a neutral ink, not accent
 * (accent marks primary action and selection). Appears and disappears at once, with no motion.
 */
const dropIndicatorVariants = cva("relative rounded-full bg-content-1", {
  variants: {
    target: {
      row: "h-0.5 w-full",
      root: "h-0.5 w-full",
      tab: "h-6 w-1 shrink-0",
    },
    edge: {
      before: "",
      after: "",
    },
    visible: {
      true: "",
      false: "hidden",
    },
  },
  defaultVariants: { target: "row", edge: "before", visible: false },
});

/** The dot at the left end of a row or root line; absent on a tab bar. */
const dropIndicatorDotClass = "absolute left-0 top-1/2 size-2 -translate-y-1/2 rounded-full bg-content-1";

type DropIndicatorVariants = VariantProps<typeof dropIndicatorVariants>;

export interface DropIndicatorProps extends React.HTMLAttributes<HTMLDivElement> {
  /** Where it lands. */
  target?: NonNullable<DropIndicatorVariants["target"]>;
  /** Row only: before (top) or after (bottom) the task row. */
  edge?: NonNullable<DropIndicatorVariants["edge"]>;
  /** True while a drag is over the target; false draws nothing and hides it from assistive tech. */
  visible?: boolean;
}

export const DropIndicator = React.forwardRef<HTMLDivElement, DropIndicatorProps>(function DropIndicator(
  { target = "row", edge = "before", visible = false, className, ...props },
  ref,
) {
  return (
    <div
      ref={ref}
      {...props}
      data-slot="indicator"
      data-target={target}
      data-edge={target === "row" ? edge : undefined}
      role="none"
      aria-hidden="true"
      className={cn(dropIndicatorVariants({ target, edge, visible }), className)}
    >
      {target !== "tab" && <span data-slot="dot" role="none" className={dropIndicatorDotClass} />}
    </div>
  );
});

export { dropIndicatorVariants };
