import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/utils";

/** Display-family semibold title, always tone primary. Sizes: sm 14px, md 16px (1rem = 14px). */
export const headingVariants = cva(
  "block min-w-0 max-w-full font-display font-semibold leading-[1.35] text-content-1",
  {
    variants: {
      size: {
        sm: "text-[1rem]",
        md: "text-[1.143rem]",
      },
      overflow: {
        truncate: "truncate",
        wrap: "text-pretty",
      },
    },
    defaultVariants: { size: "sm", overflow: "truncate" },
  },
);

export interface HeadingProps
  extends React.HTMLAttributes<HTMLElement>,
    VariantProps<typeof headingVariants> {
  /** h1, h2 and h3 keep the heading role; p and span report generic. */
  as?: "h1" | "h2" | "h3" | "p" | "span";
  children?: React.ReactNode;
}

export const Heading = React.forwardRef<HTMLElement, HeadingProps>(function Heading(
  { as = "h2", size, overflow, className, children, ...props },
  ref,
) {
  const Comp = as as React.ElementType;
  return (
    <Comp
      ref={ref}
      data-slot="title"
      className={cn(headingVariants({ size, overflow }), className)}
      {...props}
    >
      {children}
    </Comp>
  );
});
