import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/utils";
import { Icon } from "./icon";

/**
 * One size: a 12px square (0.857rem on the 14px root). The glyph inherits the colour of the
 * surrounding text. The spin is the global animation, which the reduced-motion rule in
 * tokens.css already slows to a stop.
 */
const spinnerVariants = cva(
  "relative inline-flex shrink-0 items-center justify-center align-middle leading-none",
  {
    variants: {
      size: {
        sm: "h-[0.857rem] w-[0.857rem]",
      },
    },
    defaultVariants: { size: "sm" },
  },
);

type Variants = VariantProps<typeof spinnerVariants>;

export interface SpinnerProps extends Omit<React.HTMLAttributes<HTMLSpanElement>, "role" | "children"> {
  size?: Variants["size"];
  /** Accessible name, announced politely; also rendered as visually hidden text. */
  label?: string;
  className?: string;
}

export const Spinner = React.forwardRef<HTMLSpanElement, SpinnerProps>(function Spinner(
  { size = "sm", label = "Loading", className, ...props },
  ref,
) {
  return (
    <span
      ref={ref}
      role="status"
      aria-label={label}
      data-slot="indicator"
      className={cn(spinnerVariants({ size }), className)}
      {...props}
    >
      <Icon name="progress_activity" size="xs" tone="inherit" className="animate-spin" />
      <span className="sr-only">{label}</span>
    </span>
  );
});

export { spinnerVariants };
