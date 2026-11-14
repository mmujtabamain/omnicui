import * as React from "react";
import * as ToolbarPrimitive from "@radix-ui/react-toolbar";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/utils";

/** Spacing and rule per placement. Sizes are the spec's: 1rem is 14px. */
const toolbarVariants = cva("flex items-center", {
  variants: {
    placement: {
      inline: "gap-4 px-6",
      strip: "shrink-0 gap-3 border-b border-edge-2 px-6 py-5",
      footer: "h-9 border-t border-edge-2 px-6",
    },
  },
  defaultVariants: { placement: "inline" },
});

type Variants = VariantProps<typeof toolbarVariants>;

export interface ToolbarProps extends React.ComponentPropsWithoutRef<typeof ToolbarPrimitive.Root> {
  placement?: Variants["placement"];
  /** Names the group. */
  ariaLabel?: string;
  /** Items the strip or footer acts on. 0 means the strip and footer are not rendered. Inline ignores it. */
  count?: number;
}

/** A button in the group. Pass a Button with asChild; arrow keys move focus between items. */
export const ToolbarItem = ToolbarPrimitive.Button;

export const Toolbar = React.forwardRef<HTMLDivElement, ToolbarProps>(function Toolbar(
  { placement = "inline", ariaLabel = "Toolbar", count = 1, className, children, ...props },
  ref,
) {
  if (placement !== "inline" && count < 1) return null;
  return (
    <ToolbarPrimitive.Root
      ref={ref}
      data-slot="toolbar"
      aria-label={ariaLabel}
      className={cn(toolbarVariants({ placement }), className)}
      {...props}
    >
      {children}
    </ToolbarPrimitive.Root>
  );
});

export { toolbarVariants };
