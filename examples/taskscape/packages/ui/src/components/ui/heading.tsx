import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/utils";

/** Display-family semibold title in the primary tone. Sizes: 1rem is 14px in this app. */
export const headingVariants = cva("block min-w-0 font-display font-semibold leading-tight text-content-1", {
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
});

export type HeadingTag = "h1" | "h2" | "h3" | "p" | "span";

export interface HeadingProps
  extends Omit<React.HTMLAttributes<HTMLElement>, "children">,
    VariantProps<typeof headingVariants> {
  /** The element: h1 to h3 give a heading role; p and span give none. Default h2. */
  as?: HeadingTag;
  children?: React.ReactNode;
}

export const Heading = React.forwardRef<HTMLElement, HeadingProps>(function Heading(
  { as = "h2", size = "sm", overflow = "truncate", className, children, ...props },
  ref,
) {
  const Tag: React.ElementType = as;
  return (
    <Tag
      ref={ref as React.Ref<HTMLHeadingElement>}
      // A paragraph is reported as "paragraph"; the contract expects it to read as generic.
      role={as === "p" ? "generic" : undefined}
      {...props}
      data-slot="title"
      className={cn(headingVariants({ size, overflow }), className)}
    >
      {children}
    </Tag>
  );
});
