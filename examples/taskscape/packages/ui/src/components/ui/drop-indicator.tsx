import * as React from "react";
import { cva } from "class-variance-authority";
import { cn } from "../../lib/utils";

/**
 * Insertion marker drawn while a pointer drag is over a valid drop target. Presentational: the
 * drop itself is announced by the target that accepts it. Not accent (accent is reserved for
 * primary action and selection); the drop token is still open, so this uses the neutral ink.
 */
const dropIndicatorVariants = cva("shrink-0 bg-content-1", {
  variants: {
    target: {
      row: "relative h-0.5 w-full rounded-full",
      root: "relative h-0.5 w-full rounded-full",
      tab: "h-6 w-1 rounded-full",
    },
    visible: {
      true: "",
      // Kept in the layout (so the list does not jump) but not painted: nothing is drawn.
      false: "invisible",
    },
  },
  defaultVariants: { target: "row", visible: false },
});

const dropDotClass = "absolute top-1/2 left-0 size-2 -translate-y-1/2 rounded-full bg-content-1";

type DropIndicatorVariants = {
  target?: "row" | "root" | "tab";
  visible?: boolean;
};

export interface DropIndicatorProps extends React.HTMLAttributes<HTMLDivElement>, DropIndicatorVariants {
  /** Where it lands: a task row, the list root, or a tab divider or end drop zone. */
  target?: "row" | "root" | "tab";
  /** Row only: before (top) or after (bottom) the task row. Ignored by root and tab. */
  edge?: "before" | "after";
  /** true while a drag is over the target; false draws nothing and hides it from assistive tech. */
  visible?: boolean;
}

export const DropIndicator = React.forwardRef<HTMLDivElement, DropIndicatorProps>(function DropIndicator(
  { target = "row", edge = "before", visible = false, className, ...props },
  ref,
) {
  return (
    <div
      ref={ref}
      data-slot="indicator"
      data-target={target}
      data-edge={target === "row" ? edge : undefined}
      role="none"
      aria-hidden="true"
      className={cn(dropIndicatorVariants({ target, visible }), className)}
      {...props}
    >
      {target !== "tab" && <span data-slot="dot" className={dropDotClass} />}
    </div>
  );
});

export { dropIndicatorVariants };
