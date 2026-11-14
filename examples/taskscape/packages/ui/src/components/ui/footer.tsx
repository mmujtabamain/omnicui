import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/utils";

/**
 * Layout only: the caller's actions sit at the right, in DOM order. Density sets the padding and
 * height, tone the fill, edge the hairline. The drag region (desktop only) leaves the buttons out.
 */
const footerVariants = cva("flex items-center justify-end", {
  variants: {
    density: {
      bar: "h-[2.25rem] gap-[0.5rem] px-[1.5rem]",
      row: "gap-[0.75rem] px-[1.5rem] py-[1.25rem]",
    },
    tone: {
      base: "bg-transparent",
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
});

export type FooterProps = React.HTMLAttributes<HTMLDivElement> & VariantProps<typeof footerVariants>;

export const Footer = React.forwardRef<HTMLDivElement, FooterProps>(function Footer(
  { density, tone, edge, drag, className, children, ...props },
  ref,
) {
  return (
    <div
      ref={ref}
      data-slot="footer"
      className={cn(footerVariants({ density, tone, edge, drag }), className)}
      {...props}
    >
      {children}
    </div>
  );
});

export { footerVariants };
