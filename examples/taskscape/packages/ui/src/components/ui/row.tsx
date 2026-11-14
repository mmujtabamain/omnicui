import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/utils";

/**
 * The surface of one list row: fill, radius, height and hover. Content comes in as children
 * and is drawn in the "content" slot. The parent list owns the role, the selection and the gesture.
 */
export const rowVariants = cva(
  "group/row relative flex w-full min-w-0 items-center px-4 transition-colors duration-(--duration-fast) ease-standard",
  {
    variants: {
      shape: {
        inset: "rounded-(--radius-field)",
        "full-bleed": "rounded-none",
      },
      tone: {
        selection: "",
        wash: "",
      },
      density: {
        md: "h-9",
        sm: "h-8",
      },
      selected: {
        true: "",
        false: "hover:bg-wash-1",
      },
    },
    compoundVariants: [
      // A selected row keeps its fill while hovered: the hover wash only applies when not selected.
      { selected: true, tone: "selection", className: "bg-selection-1" },
      { selected: true, tone: "wash", className: "bg-wash-1" },
    ],
    defaultVariants: { shape: "inset", tone: "selection", density: "md", selected: false },
  },
);

export const rowContentVariants = cva("flex min-w-0 flex-1 items-center", {
  variants: {
    shape: {
      inset: "gap-5",
      "full-bleed": "",
    },
  },
  defaultVariants: { shape: "inset" },
});

export interface RowProps
  extends Omit<React.HTMLAttributes<HTMLDivElement>, "color">,
    VariantProps<typeof rowVariants> {
  /** Depth times 24 for task rows, in pixels. Shifts the content slot. */
  indent?: number;
}

export const Row = React.forwardRef<HTMLDivElement, RowProps>(function Row(
  { shape = "inset", tone = "selection", density = "md", selected = false, indent = 0, className, style, children, ...props },
  ref,
) {
  return (
    <div
      ref={ref}
      className={cn(rowVariants({ shape, tone, density, selected }), className)}
      style={style}
      data-selected={selected || undefined}
      {...props}
      data-slot="row"
    >
      <div
        data-slot="content"
        className={rowContentVariants({ shape })}
        style={indent ? { marginLeft: indent } : undefined}
      >
        {children}
      </div>
    </div>
  );
});
