import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/utils";

/** The pane's box: a flex child that takes its width from the group or from a pixel clamp. */
const paneVariants = cva("flex h-full min-w-0", {
  variants: {
    slot: {
      primary: "",
      split: "",
      aside: "shrink-0",
    },
    edge: {
      none: "",
      leading: "border-l border-edge-2",
      trailing: "border-r border-edge-2",
    },
    fill: {
      none: "",
      surface: "flex-col overflow-hidden bg-surface-1",
    },
  },
  defaultVariants: { slot: "primary", edge: "none", fill: "none" },
});

type Variants = VariantProps<typeof paneVariants>;

/** 1rem is 14px in this app. */
const REM_PX = 14;
const ASIDE_MIN_PX = 280;
const ASIDE_MAX_PX = 420;
const SPLIT_MIN_PCT = 25;
const SPLIT_MAX_PCT = 75;

const clamp = (n: number, lo: number, hi: number) => Math.min(hi, Math.max(lo, n));

export interface PaneProps extends Omit<React.HTMLAttributes<HTMLDivElement>, "slot"> {
  /** Position and default look. */
  slot?: Variants["slot"];
  /** Hairline side. */
  edge?: Variants["edge"];
  /** Background fill. */
  fill?: Variants["fill"];
  /** aside gives the complementary role; nav gives the navigation role; div is generic. */
  as?: "div" | "aside" | "nav";
  /** Share of the task area when split, clamped to 0.25 to 0.75. */
  ratio?: number;
  /** Aside width in px, clamped to 280 to 420. */
  previewWidth?: number;
  /** Percent of the group: 100 unsplit, ratio times 100 when split. */
  defaultSize?: number;
  /** Lower clamp; 0 means the slot default (25 percent, or 280 px for aside). */
  minSize?: number;
  /** Upper clamp; 0 means the slot default (75 percent, or 420 px for aside). */
  maxSize?: number;
  /** Shrink-0 when true. */
  shrink?: boolean;
  /** false removes the pane. */
  open?: boolean;
  /** Fixed px width, shrink-0, used when above 0; 0 takes the width from the group. */
  width?: number;
}

export const Pane = React.forwardRef<HTMLDivElement, PaneProps>(function Pane(
  {
    slot = "primary",
    edge = "none",
    fill = "none",
    as = "div",
    ratio = 0.5,
    previewWidth = 320,
    defaultSize = 100,
    minSize = 0,
    maxSize = 0,
    shrink = false,
    open = true,
    width = 0,
    className,
    style,
    children,
    ...props
  },
  ref,
) {
  if (!open) return null;

  const Comp = as === "aside" ? "aside" : as === "nav" ? "nav" : "div";

  let geometry: React.CSSProperties;
  if (width > 0) {
    geometry = { width: `${width / REM_PX}rem` };
  } else if (slot === "aside") {
    const px = clamp(previewWidth, minSize || ASIDE_MIN_PX, maxSize || ASIDE_MAX_PX);
    geometry = { width: `${px / REM_PX}rem` };
  } else {
    // Unsplit primary is the whole group and is not clamped; a split pane takes its ratio share.
    const split = slot === "split" || defaultSize !== 100;
    const share = defaultSize === 100 ? clamp(ratio, 0.25, 0.75) * 100 : defaultSize;
    const pct = split ? clamp(share, minSize || SPLIT_MIN_PCT, maxSize || SPLIT_MAX_PCT) : 100;
    geometry = { width: `${pct}%` };
  }

  return (
    <Comp
      ref={ref}
      data-slot="pane"
      className={cn(paneVariants({ slot, edge, fill }), (shrink || width > 0) && "shrink-0", className)}
      style={{ ...geometry, ...style }}
      {...props}
    >
      {children}
    </Comp>
  );
});

export { paneVariants };
