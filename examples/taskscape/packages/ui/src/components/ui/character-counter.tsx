import * as React from "react";
import { cva } from "class-variance-authority";
import { cn } from "../../lib/utils";

/** Tone is a text colour; the digits carry the value, so colour is not the only cue. */
const characterCounterVariants = cva("inline-block whitespace-nowrap text-xs font-normal tabular-nums", {
  variants: {
    tone: {
      normal: "text-content-2",
      near: "text-danger-500",
    },
  },
  defaultVariants: { tone: "normal" },
});

/** Read once on crossing into the near state; the live region is empty while normal. */
const NEAR_NOTICE = "Approaching the character limit.";

export interface CharacterCounterProps extends Omit<React.HTMLAttributes<HTMLSpanElement>, "children"> {
  /** Current character count, a whole number from 0 up. */
  count?: number;
  /** Maximum length in characters. */
  limit?: number;
  /** Share of limit above which the danger tone starts (strictly greater). */
  nearRatio?: number;
}

export const CharacterCounter = React.forwardRef<HTMLSpanElement, CharacterCounterProps>(function CharacterCounter(
  { count = 0, limit = 2000, nearRatio = 0.9, className, ...props },
  ref,
) {
  // The epsilon keeps a product that rounds a hair low (0.29 * 100) from flipping the edge.
  const near = count > nearRatio * limit + 1e-9;

  return (
    <span ref={ref} className={cn("inline-block", className)} {...props}>
      <span data-slot="readout" className={characterCounterVariants({ tone: near ? "near" : "normal" })}>
        {`${count}/${limit}`}
      </span>
      <span data-slot="announcement" role="status" className="sr-only">
        {near ? NEAR_NOTICE : ""}
      </span>
    </span>
  );
});

export { characterCounterVariants };
