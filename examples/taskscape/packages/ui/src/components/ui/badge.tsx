import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/utils";
import { Icon } from "./icon";

/**
 * A read-only inline mark: a pill or circle with a count or label, or a 6px dot or 2px bar with no text.
 * Never interactive: no focus, hover or press state, and no events.
 * Sizes are measured px converted at 1rem = 14px.
 */
export const badgeVariants = cva(
  "inline-flex shrink-0 items-center justify-center gap-[0.286rem] whitespace-nowrap font-semibold tabular-nums select-none",
  {
    variants: {
      tone: {
        outline: "border border-edge-2 text-content-2",
        recessed: "bg-wash-2 text-content-2",
        accent: "bg-accent-500 text-on-accent",
        neutral: "bg-edge-3 text-content-3",
      },
      shape: {
        pill: "rounded-full",
        round: "rounded-full",
        dot: "rounded-full h-[0.429rem] w-[0.429rem]",
        bar: "rounded-full h-[1.143rem] w-[0.143rem]",
      },
      size: {
        md: "text-[0.786rem] leading-[1.179rem]",
        lg: "text-[0.929rem] leading-[1.393rem]",
      },
    },
    compoundVariants: [
      { shape: "pill", size: "md", className: "h-[1.286rem] px-space-3" },
      { shape: "pill", size: "lg", className: "h-[2rem] px-space-5" },
      { shape: "round", size: "lg", className: "h-[2rem] min-w-[2rem] px-0" },
    ],
    defaultVariants: { tone: "outline", shape: "pill", size: "md" },
  },
);

export interface BadgeProps extends React.HTMLAttributes<HTMLSpanElement>, VariantProps<typeof badgeVariants> {
  /** Leading paperclip before the count (attachments). */
  icon?: boolean;
  /** Visible count or label, e.g. "1 / 3", "4", "Errands". Empty for dot and bar. */
  children?: React.ReactNode;
  /** Native hover tooltip only; keyboard and screen reader users get srLabel. */
  title?: string;
  /** Visually hidden text read by screen readers; the visible text is then aria-hidden. */
  srLabel?: string;
  /** false takes a section dot out of view. */
  visible?: boolean;
}

/**
 * Pill and round: the root holds the surface and the optional icon; the text part (data-slot badge)
 * is the inner span, so the icon is not part of the count text. Dot and bar: the root is the mark.
 */
export const Badge = React.forwardRef<HTMLSpanElement, BadgeProps>(function Badge(
  {
    tone = "outline", shape = "pill", size = "md", icon = false, children = "", title = "", srLabel = "",
    visible = true, className, ...props
  },
  ref,
) {
  // A round badge is valid only at size lg; round with md renders as a pill.
  const effectiveShape = shape === "round" && size !== "lg" ? "pill" : shape;
  const decorative = effectiveShape === "dot" || effectiveShape === "bar";
  const glyphSize = size === "lg" ? 13 : 11;
  const classes = cn(badgeVariants({ tone, shape: effectiveShape, size }), !visible && "hidden", className);

  if (decorative)
    return <span ref={ref} data-slot="badge" aria-hidden="true" title={title || undefined} className={classes} {...props} />;

  return (
    <span ref={ref} title={title || undefined} className={classes} {...props}>
      {icon && <Icon data-slot="icon" data-part="ui.badge.icon" name="attach_file" size={glyphSize} tone="inherit" />}
      <span data-slot="badge">
        {srLabel ? (
          <>
            <span aria-hidden="true">{children}</span>
            <span className="sr-only">{srLabel}</span>
          </>
        ) : (
          children
        )}
      </span>
    </span>
  );
});
