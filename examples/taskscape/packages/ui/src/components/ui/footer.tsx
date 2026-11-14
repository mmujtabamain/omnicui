import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/utils";

/**
 * Layout only: the caller's actions sit right-aligned in a bar or a padded sheet row.
 * The desktop drag region excludes the buttons, so a press on a button presses it.
 */
const footerVariants = cva(
  "flex min-w-0 items-center justify-end",
  {
    variants: {
      density: {
        bar: "h-[2.25rem] gap-2 px-6",
        row: "gap-3 px-6 py-5",
      },
      tone: {
        base: "",
        raised: "bg-surface-2",
      },
      edge: {
        top: "border-t border-edge-2",
        bottom: "border-b border-edge-2",
        none: "",
      },
      drag: {
        none: "",
        window: "[-webkit-app-region:drag] [&_button]:[-webkit-app-region:no-drag]",
      },
    },
    defaultVariants: { density: "bar", tone: "base", edge: "top", drag: "none" },
  },
);

export type FooterProps = React.HTMLAttributes<HTMLDivElement> & VariantProps<typeof footerVariants>;

export const Footer = React.forwardRef<HTMLDivElement, FooterProps>(function Footer(
  { density, tone, edge, drag, className, ...props },
  ref,
) {
  return (
    <div
      ref={ref}
      data-slot="footer"
      className={cn(footerVariants({ density, tone, edge, drag }), className)}
      {...props}
    />
  );
});

export { footerVariants };
