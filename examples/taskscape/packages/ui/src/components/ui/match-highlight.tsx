import * as React from "react";
import { cva } from "class-variance-authority";
import { cn } from "../../lib/utils";

/**
 * The search-match highlight. The fill is accent-500 at 25% (30% in dark) until the
 * non-accent search-highlight token is chosen (open product decision). Radius 3px.
 * Text colour and font inherit from the surrounding text. No keyboard or hover behaviour.
 */
const matchHighlightVariants = cva(
  "[color:inherit] [font:inherit] rounded-[0.2143rem]",
  {
    variants: {
      active: {
        true: "bg-accent-500/25 dark:bg-accent-500/30",
        false: "bg-transparent",
      },
    },
    defaultVariants: { active: false },
  },
);

export interface MatchHighlightProps extends React.HTMLAttributes<HTMLElement> {
  /** The search query; the highlight shows only when query.trim() is non-empty. */
  query?: string;
  /** The matched text (the React children take precedence when given). */
  text?: string;
}

export const MatchHighlight = React.forwardRef<HTMLElement, MatchHighlightProps>(function MatchHighlight(
  { query = "", text = "", children, className, ...props },
  ref,
) {
  const active = query.trim() !== "";
  return (
    <mark
      ref={ref}
      data-slot="highlight"
      className={cn(matchHighlightVariants({ active }), className)}
      {...props}
    >
      {children ?? text}
    </mark>
  );
});

export { matchHighlightVariants };
