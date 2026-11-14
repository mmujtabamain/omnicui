import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/utils";

/**
 * Full-window layer behind a modal, a menu or the lightbox. A press on it outside the owning
 * panel dismisses the owner. The scrim takes no keys (Escape belongs to the owner) and is hidden
 * from assistive technology. It is a plain layer rather than Radix Dialog Overlay: that overlay
 * only renders under a modal Dialog root, which would add a scroll lock and a focus trap.
 */
export const scrimVariants = cva("fixed inset-0 z-(--z-index-overlay)", {
  variants: {
    dim: {
      // Sheets and the command palette. A literal black, not a token, as the spec asks.
      soft: "bg-black/30",
      // Attachment lightbox over photos: a strong dim, a backdrop blur, a flex column for its content.
      "strong-blur": "flex flex-col bg-black/50 backdrop-blur-md",
      // Invisible hit area under a menu: catches left and right presses outside the menu.
      none: "bg-transparent",
    },
  },
  defaultVariants: { dim: "soft" },
});

export interface ScrimProps
  extends React.HTMLAttributes<HTMLDivElement>,
    VariantProps<typeof scrimVariants> {
  /** false: the scrim is not rendered at all. */
  open?: boolean;
  /** A press (any button) on the scrim, outside the owning panel. */
  onDismiss?: () => void;
}

export const Scrim = React.forwardRef<HTMLDivElement, ScrimProps>(function Scrim(
  { dim = "soft", open = true, onDismiss, className, onMouseDown, onContextMenu, ...props },
  ref,
) {
  if (!open) return null;
  return (
    <div
      ref={ref}
      data-slot="scrim"
      aria-hidden="true"
      className={cn(scrimVariants({ dim }), className)}
      onMouseDown={(e) => {
        onMouseDown?.(e);
        onDismiss?.();
      }}
      onContextMenu={(e) => {
        // The click-catcher under a menu takes the right press too, so no native menu opens over it.
        onContextMenu?.(e);
        e.preventDefault();
      }}
      {...props}
    />
  );
});
