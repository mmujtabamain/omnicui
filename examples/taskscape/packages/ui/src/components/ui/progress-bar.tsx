import * as React from "react";
import * as ProgressPrimitive from "@radix-ui/react-progress";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/utils";

/** The track: a thin, fixed-size strip (84px wide, about 4px tall, on the 14px root). */
const progressBarVariants = cva("relative block h-[0.286rem] w-[6rem] shrink-0 overflow-hidden rounded-full bg-wash-2");

/** The fill: the done-lamp colour is the only tone. Its width comes from Radix's translateX. */
const progressBarFillVariants = cva(
  "block h-full w-full rounded-full transition-transform duration-(--duration-base) ease-standard",
  {
    variants: {
      tone: {
        done: "bg-done-lamp-1",
      },
    },
    defaultVariants: { tone: "done" },
  },
);

type FillVariants = VariantProps<typeof progressBarFillVariants>;

export interface ProgressBarProps
  extends Omit<React.ComponentPropsWithoutRef<typeof ProgressPrimitive.Root>, "value" | "max"> {
  /** Done ratio between 0 and max; values outside are clamped. */
  value?: number;
  /** Upper bound; a max that is not above 0 falls back to 1. */
  max?: number;
  /** Accessible name. The bar has no text, so this is its only label. */
  ariaLabel?: string;
  tone?: FillVariants["tone"];
}

export const ProgressBar = React.forwardRef<HTMLDivElement, ProgressBarProps>(function ProgressBar(
  { value = 0, max = 1, ariaLabel = "", tone = "done", className, ...props },
  ref,
) {
  const safeMax = max > 0 ? max : 1;
  const safeValue = Math.min(Math.max(value, 0), safeMax);
  return (
    <ProgressPrimitive.Root
      ref={ref}
      data-slot="bar"
      aria-label={ariaLabel || undefined}
      value={safeValue}
      max={safeMax}
      className={cn(progressBarVariants(), className)}
      {...props}
    >
      {safeValue > 0 && (
        <ProgressPrimitive.Indicator
          data-slot="fill"
          className={progressBarFillVariants({ tone })}
          style={{ transform: `translateX(-${100 - (safeValue / safeMax) * 100}%)` }}
        />
      )}
    </ProgressPrimitive.Root>
  );
});

export { progressBarVariants, progressBarFillVariants };
