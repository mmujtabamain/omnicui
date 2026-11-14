import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/utils";

/**
 * Layout only: a region of a PaneGroup. Task panes take a percent share of the group (flex basis),
 * the aside host and fixed side columns take a pixel width. Hairlines and surfaces come from the
 * tokens. The pane emits nothing and has no keys of its own.
 */
const paneVariants = cva("flex min-h-0 min-w-0", {
  variants: {
    slot: {
      // Position only: the widths are computed in the component.
      primary: "",
      split: "",
      aside: "",
    },
    edge: {
      none: "",
      leading: "border-l border-edge-2",
      trailing: "border-r border-edge-2",
    },
    fill: {
      none: "",
      surface: "h-full flex-col overflow-hidden bg-surface-1",
    },
    shrink: {
      true: "shrink-0",
      false: "",
    },
  },
  defaultVariants: { slot: "primary", edge: "none", fill: "none", shrink: false },
});

type PaneVariants = VariantProps<typeof paneVariants>;

const SLOT_MIN_PERCENT = 25;
const SLOT_MAX_PERCENT = 75;
const ASIDE_MIN_PX = 280;
const ASIDE_MAX_PX = 420;

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}

export interface PaneProps extends Omit<React.HTMLAttributes<HTMLDivElement>, "slot"> {
  /** primary: left task pane; split: second task pane; aside: right aside host. */
  slot?: NonNullable<PaneVariants["slot"]>;
  /** none, leading (border-l) or trailing (border-r) hairline in edge-2. */
  edge?: NonNullable<PaneVariants["edge"]>;
  /** none is transparent; surface paints bg-surface-1 as a full-height column. */
  fill?: NonNullable<PaneVariants["fill"]>;
  /** div is generic; aside is complementary; nav is navigation. */
  as?: "div" | "aside" | "nav";
  /** Share of the task area when split, clamped to 0.25 to 0.75. */
  ratio?: number;
  /** Aside width in px, clamped to minSize and maxSize (280 to 420 by default). */
  previewWidth?: number;
  /** Percent of the group: 100 when unsplit, ratio times 100 when split. */
  defaultSize?: number;
  /** Lower clamp. 0 means the slot default: 25 percent, or 280 px for aside. */
  minSize?: number;
  /** Upper clamp. 0 means the slot default: 75 percent, or 420 px for aside. */
  maxSize?: number;
  /** true makes the pane shrink-0. */
  shrink?: boolean;
  /** false removes the pane. */
  open?: boolean;
  /** Fixed px width (a column such as the Settings sidebar). 0 means the group decides. */
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

  const Comp: React.ElementType = as;

  let geometry: React.CSSProperties;
  if (slot === "aside") {
    geometry = { width: clamp(previewWidth, minSize || ASIDE_MIN_PX, maxSize || ASIDE_MAX_PX) };
  } else if (width > 0) {
    geometry = { width };
  } else {
    // An unsplit primary sits at 100 and is not clamped; a split pane is clamped to its slot bounds.
    const split = slot === "split" || defaultSize < 100;
    const share = defaultSize < 100 ? defaultSize : clamp(ratio, 0.25, 0.75) * 100;
    const percent = split ? clamp(share, minSize || SLOT_MIN_PERCENT, maxSize || SLOT_MAX_PERCENT) : 100;
    geometry = { flex: `0 1 ${percent}%` };
  }

  return (
    <Comp
      ref={ref}
      data-slot="pane"
      className={cn(paneVariants({ slot, edge, fill, shrink: shrink || width > 0 }), className)}
      style={{ ...geometry, ...style }}
      {...props}
    >
      {children}
    </Comp>
  );
});

export { paneVariants };
