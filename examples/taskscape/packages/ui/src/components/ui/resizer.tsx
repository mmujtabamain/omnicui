import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/utils";

/**
 * A 4px grab strip (0.2857rem at 14px per rem). `reset` changes what Home, Enter and double-click
 * do, not how the strip is painted, so both reset values share the same classes.
 */
const resizerVariants = cva(
  [
    "group/resizer relative flex shrink-0 self-stretch touch-none select-none justify-center outline-none",
    "bg-transparent transition-colors duration-(--duration-fast) ease-standard",
    "hover:bg-edge-2 data-[dragging=true]:bg-edge-3",
    "focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-focus-1",
  ],
  {
    variants: {
      orientation: {
        vertical: "w-[0.2857rem] cursor-col-resize",
      },
      reset: {
        "double-click": "",
        none: "",
      },
    },
    defaultVariants: { orientation: "vertical", reset: "none" },
  },
);

/** The 1px hairline in the centre of the strip: transparent at rest, shown on hover and drag. */
const hairlineClasses =
  "pointer-events-none h-full w-px bg-transparent transition-colors duration-(--duration-fast) ease-standard group-hover/resizer:bg-edge-3 group-data-[dragging=true]/resizer:bg-edge-3";

type Variants = VariantProps<typeof resizerVariants>;

export interface ResizerProps extends Omit<React.HTMLAttributes<HTMLDivElement>, "onResize" | "onReset"> {
  orientation?: Variants["orientation"];
  /** splitRatio (0.25 to 0.75) for the split divider, or previewWidth in px (280 to 420). */
  value?: number;
  min?: number;
  max?: number;
  /** double-click: the split divider (reset on double-click, Home and Enter). none: drag and keys only. */
  reset?: Variants["reset"];
  /** false when the aside is closed: the handle is not rendered. */
  open?: boolean;
  /** Accessible name. */
  label?: string;
  /** Called with the new value, already clamped to min and max. */
  onResize?: (value: number) => void;
  /** Called when the default split is asked for; only the double-click variant calls it. */
  onReset?: () => void;
}

const clamp = (v: number, lo: number, hi: number) => Math.min(hi, Math.max(lo, v));

/**
 * WAI-ARIA window splitter. The split divider (reset double-click) moves its ratio with the pointer
 * over the parent's width; the preview edge (reset none) moves its pixel width, so dragging left widens it.
 */
export const Resizer = React.forwardRef<HTMLDivElement, ResizerProps>(function Resizer(
  {
    orientation = "vertical", value = 0.5, min = 0.25, max = 0.75, reset = "none", open = true,
    label = "Resize columns", onResize, onReset, className,
    onPointerDown, onPointerMove, onPointerUp, onPointerCancel, onDoubleClick, onKeyDown,
    ...props
  },
  ref,
) {
  const el = React.useRef<HTMLDivElement>(null);
  React.useImperativeHandle(ref, () => el.current as HTMLDivElement);
  const drag = React.useRef<{ x: number; start: number; span: number } | null>(null);
  const [dragging, setDragging] = React.useState(false);

  const split = reset === "double-click";
  const current = clamp(value, min, max);
  // Direction of a value change per unit of rightward movement: the divider grows right, the preview edge shrinks right.
  const sign = split ? 1 : -1;
  const step = split ? 0.01 : 1;

  const commit = (next: number) => onResize?.(clamp(next, min, max));

  const handlePointerDown = (e: React.PointerEvent<HTMLDivElement>) => {
    onPointerDown?.(e);
    if (e.button !== 0) return;
    const span = el.current?.parentElement?.getBoundingClientRect().width || 1;
    drag.current = { x: e.clientX, start: current, span };
    e.currentTarget.setPointerCapture(e.pointerId);
    setDragging(true);
  };

  const handlePointerMove = (e: React.PointerEvent<HTMLDivElement>) => {
    onPointerMove?.(e);
    const d = drag.current;
    if (!d) return;
    const dx = e.clientX - d.x;
    commit(d.start + sign * (split ? dx / d.span : dx));
  };

  const endDrag = (e: React.PointerEvent<HTMLDivElement>) => {
    if (e.currentTarget.hasPointerCapture(e.pointerId)) e.currentTarget.releasePointerCapture(e.pointerId);
    drag.current = null;
    setDragging(false);
  };

  const handlePointerUp = (e: React.PointerEvent<HTMLDivElement>) => {
    onPointerUp?.(e);
    endDrag(e);
  };

  const handlePointerCancel = (e: React.PointerEvent<HTMLDivElement>) => {
    onPointerCancel?.(e);
    endDrag(e);
  };

  const handleDoubleClick = (e: React.MouseEvent<HTMLDivElement>) => {
    onDoubleClick?.(e);
    if (split) onReset?.();
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLDivElement>) => {
    onKeyDown?.(e);
    switch (e.key) {
      case "ArrowRight":
        commit(current + sign * step);
        break;
      case "ArrowLeft":
        commit(current - sign * step);
        break;
      case "Home":
        if (split) onReset?.();
        else commit(min);
        break;
      case "End":
        commit(max);
        break;
      case "Enter":
        if (!split) return;
        onReset?.();
        break;
      default:
        return;
    }
    e.preventDefault();
  };

  return (
    <div
      {...props}
      ref={el}
      role="separator"
      tabIndex={open ? 0 : -1}
      aria-orientation="vertical"
      aria-valuenow={current}
      aria-valuemin={min}
      aria-valuemax={max}
      aria-label={label}
      aria-hidden={!open || undefined}
      hidden={!open}
      data-slot="resizer"
      data-dragging={dragging || undefined}
      className={cn(resizerVariants({ orientation, reset }), className)}
      onPointerDown={handlePointerDown}
      onPointerMove={handlePointerMove}
      onPointerUp={handlePointerUp}
      onPointerCancel={handlePointerCancel}
      onDoubleClick={handleDoubleClick}
      onKeyDown={handleKeyDown}
    >
      <div role="none" data-slot="hairline" className={hairlineClasses} />
    </div>
  );
});

export { resizerVariants };
