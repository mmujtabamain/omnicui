import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/utils";
import { Footer, type FooterProps } from "./footer";

/**
 * The stats footer of a task pane: a Footer raised one step (surface-2) and 32px tall.
 * Tone is the surface; the stats sit left and the caller's controls sit right.
 */
const paneFooterVariants = cva("h-[2.286rem] text-[0.857rem]", {
  variants: {
    tone: {
      "surface-2": "bg-surface-2 border-t border-edge-2",
    },
  },
  defaultVariants: { tone: "surface-2" },
});

/** Done percentage, rounded, or 0 when there is nothing to do. */
export function percentDone(done: number, total: number): number {
  return total > 0 ? Math.round((done / total) * 100) : 0;
}

export interface PaneFooterProps
  extends Omit<FooterProps, "tone">,
    VariantProps<typeof paneFooterVariants> {
  /** Done count. */
  done?: number;
  /** Total count. */
  total?: number;
  /** Done percentage (derived from done and total when omitted). */
  percent?: number;
}

export const PaneFooter = React.forwardRef<HTMLDivElement, PaneFooterProps>(function PaneFooter(
  { tone, done = 0, total = 0, percent, className, children, ...props },
  ref,
) {
  const pct = percent ?? percentDone(done, total);
  return (
    <Footer
      ref={ref}
      tone="raised"
      data-part="ui.panefooter.footer"
      className={cn(paneFooterVariants({ tone }), className)}
      {...props}
    >
      <span className="mr-auto flex min-w-0 items-center gap-1.5 whitespace-nowrap text-content-2 tabular-nums">
        <span className="truncate text-content-1 font-medium">
          {done} of {total} done
        </span>
        <span aria-hidden="true">·</span>
        <span className="shrink-0">{pct}%</span>
      </span>
      {children}
    </Footer>
  );
});

export { paneFooterVariants };
