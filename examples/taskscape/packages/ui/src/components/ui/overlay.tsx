import * as React from "react";
import * as DialogPrimitive from "@radix-ui/react-dialog";
import { cva } from "class-variance-authority";
import { cn } from "../../lib/utils";

/** The scrim. Desktop draws none: the platform rule is applied at the md breakpoint. */
const overlayScrimVariants = cva(
  "pointer-events-none fixed inset-0 z-(--z-index-overlay) animate-[fade-in_var(--duration-fast)_var(--ease-standard)_both]",
  {
    variants: {
      scrim: {
        dim: "bg-content-1/40 dark:bg-surface-0/70 md:bg-transparent",
        strong: "bg-content-1/60 backdrop-blur-sm dark:bg-surface-0/85 md:bg-transparent md:backdrop-blur-none",
      },
    },
    defaultVariants: { scrim: "dim" },
  },
);

/** The layer: the Radix Dialog content that holds the app's panel, stage or card. */
const overlayLayerVariants = cva(
  "fixed z-(--z-index-modal) min-w-0 overflow-y-auto outline-none animate-rise",
  {
    variants: {
      placement: {
        top: "top-[4rem] left-1/2 w-[min(36rem,calc(100%_-_2rem))] max-h-[calc(100%_-_5rem)] -translate-x-1/2 rounded-(--radius-panel) border border-edge-1 bg-surface-2 shadow-menu",
        fill: "inset-0 h-full w-full bg-transparent",
        center: "top-1/2 left-1/2 w-[min(28rem,calc(100%_-_2rem))] max-h-[calc(100%_-_2rem)] -translate-x-1/2 -translate-y-1/2 rounded-(--radius-panel) border border-edge-1 bg-surface-2 shadow-menu",
      },
    },
    defaultVariants: { placement: "center" },
  },
);

export type OverlayPlacement = NonNullable<Parameters<typeof overlayLayerVariants>[0]>["placement"];
export type OverlayScrim = NonNullable<Parameters<typeof overlayScrimVariants>[0]>["scrim"];
export type OverlayDismissOn = "outside" | "none";

export interface OverlayProps
  extends Omit<
    React.ComponentPropsWithoutRef<typeof DialogPrimitive.Content>,
    "onEscapeKeyDown" | "onPointerDownOutside" | "onInteractOutside" | "onOpenAutoFocus" | "onCloseAutoFocus"
  > {
  /** Controlled visibility; closed means not mounted. */
  open?: boolean;
  /** Radix open change, called with false on every dismissal. */
  onOpenChange?: (open: boolean) => void;
  /** Called on Escape and, when dismissOn is outside, on an outside press. */
  onDismiss?: () => void;
  placement?: OverlayPlacement;
  scrim?: OverlayScrim;
  /** 'none' makes outside presses do nothing; Escape still dismisses. */
  dismissOn?: OverlayDismissOn;
  /** false leaves Escape to the content. */
  captureEscape?: boolean;
}

/** The single modal root: Portal, scrim, focus trap, Escape capture and dismissal. */
export const Overlay = React.forwardRef<HTMLDivElement, OverlayProps>(function Overlay(
  {
    open = false,
    onOpenChange,
    onDismiss,
    placement = "center",
    scrim = "dim",
    dismissOn = "outside",
    captureEscape = true,
    className,
    children,
    ...props
  },
  ref,
) {
  // Without a Radix Trigger, focus returns to whatever held it when the layer opened.
  const openerRef = React.useRef<HTMLElement | null>(null);
  return (
    <DialogPrimitive.Root
      open={open}
      onOpenChange={(next) => {
        onOpenChange?.(next);
        if (!next) onDismiss?.();
      }}
    >
      <DialogPrimitive.Portal>
        <DialogPrimitive.Overlay data-slot="scrim" className={overlayScrimVariants({ scrim })} />
        <DialogPrimitive.Content
          ref={ref}
          data-slot="layer"
          aria-modal="true"
          onOpenAutoFocus={() => {
            openerRef.current = document.activeElement instanceof HTMLElement ? document.activeElement : null;
          }}
          onCloseAutoFocus={(event) => {
            event.preventDefault();
            openerRef.current?.focus();
          }}
          onEscapeKeyDown={(event) => {
            if (!captureEscape) event.preventDefault();
          }}
          onPointerDownOutside={(event) => {
            if (dismissOn === "none") event.preventDefault();
          }}
          onInteractOutside={(event) => {
            if (dismissOn === "none") event.preventDefault();
          }}
          className={cn(overlayLayerVariants({ placement }), className)}
          {...props}
        >
          {children}
        </DialogPrimitive.Content>
      </DialogPrimitive.Portal>
    </DialogPrimitive.Root>
  );
});

/** The layer's accessible name: a heading the layer is labelled by. */
export const OverlayTitle = React.forwardRef<
  HTMLHeadingElement,
  React.ComponentPropsWithoutRef<typeof DialogPrimitive.Title>
>(function OverlayTitle({ className, ...props }, ref) {
  return (
    <DialogPrimitive.Title
      ref={ref}
      data-slot="title"
      className={cn("text-[1.071rem] leading-tight font-semibold text-content-1", className)}
      {...props}
    />
  );
});

export { overlayLayerVariants, overlayScrimVariants };
