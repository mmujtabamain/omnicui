import * as React from "react";
import { cn } from "../../lib/utils";

export interface MatchHighlightProps extends React.HTMLAttributes<HTMLElement> {
  /** The search query; the fill shows only when query.trim() is non-empty. */
  query?: string;
  /** The matched text. */
  text?: string;
}

/**
 * Inline highlight behind the text that matches the search query. The caller decides what
 * matches; this widget only decides whether its text is filled. The fill is accent-500 at 25%
 * (30% dark) until the non-accent search token is chosen. Text colour and font inherit.
 */
export const MatchHighlight = React.forwardRef<HTMLElement, MatchHighlightProps>(function MatchHighlight(
  { query = "", text = "", children, className, ...props },
  ref,
) {
  const highlighted = query.trim() !== "";
  return (
    <mark
      ref={ref}
      data-slot="highlight"
      className={cn(
        "rounded-[0.214rem] [color:inherit]",
        highlighted ? "bg-accent-500/25 dark:bg-accent-500/30" : "bg-transparent",
        className,
      )}
      {...props}
    >
      {text !== "" ? text : children}
    </mark>
  );
});
