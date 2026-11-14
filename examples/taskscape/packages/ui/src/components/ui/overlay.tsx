import * as React from "react";
import * as DialogPrimitive from "@radix-ui/react-dialog";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/utils";

/**
 * The scrim behind a layer. Dim is content ink at 40%; dark mode uses the dark surface.
 * Strong is denser and adds the blur the attachment lightbox shell uses.
 */
export const overlayScrimVariants = cva(
  "fixed inset-0 z-(--z-index-modal) animate-[fade-in_var(--duration-fast)_ease-out_both]",
  {
    variants: {
      scrim: {
        dim: "bg-content-1/40 dark:bg-surface-0/70",
        strong: "bg-content-1/60 backdrop-blur-sm dark:bg-surface-0/85",
      },
    },
    defaultVariants: { scrim: "dim" },
  },
);

/** Where the layer content sits: a card near the top, a centred card, or the whole window. */
export const overlayLayerVariants = cva(
  "fixed z-(--z-index-modal) flex flex-col gap-[0.714rem] outline-none data-[state=open]:animate-rise",
  {
    variants: {
      placement: {
        top: "top-[5.714rem] left-1/2 -translate-x-1/2 w-[min(35.714rem,calc(100vw_-_1.5rem))] rounded-(--radius-panel) border border-edge-1 bg-surface-2 p-[1.143rem] shadow-menu",
        center: "top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[min(25rem,calc(100vw_-_2rem))] rounded-(--radius-panel) border border-edge-1 bg-surface-2 p-[1.143rem] shadow-menu",
        fill: "inset-0",
      },
    },
    defaultVariants: { placement: "center" },
  },
);

type ScrimVariants = VariantProps<typeof overlayScrimVariants>;
type LayerVariants = VariantProps<typeof overlayLayerVariants>;

export interface OverlayProps
  extends Omit<React.HTMLAttributes<HTMLDivElement>, "title" | "children"> {
  /** Controlled visibility; closed means not mounted. */
  open?: boolean;
  /** Called with false on dismissal. */
  onOpenChange?: (open: boolean) => void;
  /** Called on Escape (when captured) and on outside press (when dismissOn is outside). */
  onDismiss?: () => void;
  placement?: LayerVariants["placement"];
  scrim?: ScrimVariants["scrim"];
  /** "outside" lets a press outside the layer dismiss it; "none" ignores it. */
  dismissOn?: "outside" | "none";
  /** true routes Escape to onDismiss; false leaves Escape to the content. */
  captureEscape?: boolean;
  /** Visible heading of the layer, and its accessible name. */
  title?: React.ReactNode;
  children?: React.ReactNode;
}

/**
 * Modal layer shell: a Radix Dialog root with a Portal, a scrim, a focus trap and Escape handling.
 * App content (palette panel, lightbox stage, dialog card) goes in as children.
 */
export const Overlay = React.forwardRef<HTMLDivElement, OverlayProps>(function Overlay(
  {
    open = false,
    onOpenChange,
    onDismiss,
    placement = "center",
    scrim = "dim",
    dismissOn = "outside",
    captureEscape = true,
    title,
    className,
    children,
    ...props
  },
  ref,
) {
  const handleOpenChange = (next: boolean) => {
    if (next) return;
    onOpenChange?.(false);
    onDismiss?.();
  };

  return (
    <DialogPrimitive.Root open={open} onOpenChange={handleOpenChange}>
      <DialogPrimitive.Portal>
        <DialogPrimitive.Overlay data-slot="scrim" className={overlayScrimVariants({ scrim })} />
        <DialogPrimitive.Content
          ref={ref}
          data-slot="layer"
          aria-describedby={undefined}
          onInteractOutside={(event) => {
            if (dismissOn === "none") event.preventDefault();
          }}
          onEscapeKeyDown={(event) => {
            if (!captureEscape) event.preventDefault();
          }}
          className={cn(overlayLayerVariants({ placement }), className)}
          {...props}
        >
          {title ? (
            <DialogPrimitive.Title data-slot="title" className="text-[1.071rem] leading-[1.4rem] font-semibold text-content-1">
              {title}
            </DialogPrimitive.Title>
          ) : (
            <DialogPrimitive.Title className="sr-only">Dialog</DialogPrimitive.Title>
          )}
          {children}
        </DialogPrimitive.Content>
      </DialogPrimitive.Portal>
    </DialogPrimitive.Root>
  );
});
