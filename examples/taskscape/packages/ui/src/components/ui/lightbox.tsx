import * as React from "react";
import * as DialogPrimitive from "@radix-ui/react-dialog";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/utils";
import { OverlayTitle } from "./overlay";
import { Scrim } from "./scrim";

/** Why the lightbox asked to close: a click on empty stage, or Escape. */
export type LightboxCloseReason = "stage" | "escape";

/**
 * The layer: a full-window Radix Dialog content (overlay) or a centred dialog window (window).
 * Closing fades it out: the exit runs as a reversed fade, so Radix keeps it mounted until it ends.
 */
export const lightboxVariants = cva(
  "fixed z-(--z-index-modal) flex min-w-0 flex-col outline-none animate-rise data-[state=closed]:animate-[fade-in_var(--duration-fast)_var(--ease-exit)_reverse_both]",
  {
    variants: {
      presentation: {
        overlay: "inset-0 h-full w-full bg-transparent",
        window:
          "top-1/2 left-1/2 h-[min(40rem,calc(100%_-_2rem))] w-[min(56rem,calc(100%_-_2rem))] -translate-x-1/2 -translate-y-1/2 overflow-hidden rounded-(--radius-panel) border border-edge-1 bg-surface-2 shadow-menu",
      },
    },
    defaultVariants: { presentation: "overlay" },
  },
);

export interface LightboxProps
  extends Omit<React.HTMLAttributes<HTMLDivElement>, "children" | "onClose">,
    VariantProps<typeof lightboxVariants> {
  /** Controlled visibility: false renders nothing. */
  open?: boolean;
  /** Called on a click on empty stage ("stage") or on Escape ("escape"). The caller closes it. */
  onClose?: (reason: LightboxCloseReason) => void;
  /** The layer's accessible name (screen readers only). */
  label?: string;
  /** AttachmentPreview, wrapped in LightboxMedia, or EmptyState. */
  children?: React.ReactNode;
}

/** The lightbox: a Radix modal dialog over Scrim strong-blur, with a stage that holds its content. */
export const Lightbox = React.forwardRef<HTMLDivElement, LightboxProps>(function Lightbox(
  { open = false, onClose, presentation = "overlay", label = "Attachment", className, children, ...props },
  ref,
) {
  return (
    <DialogPrimitive.Root
      open={open}
      onOpenChange={(next) => {
        // Radix reports only Escape here: outside presses are prevented below.
        if (!next) onClose?.("escape");
      }}
    >
      <DialogPrimitive.Portal>
        <Scrim
          dim="strong-blur"
          data-state={open ? "open" : "closed"}
          className="pointer-events-auto starting:opacity-0 transition-opacity duration-(--duration-fast) ease-standard data-[state=closed]:animate-[fade-in_var(--duration-fast)_var(--ease-exit)_reverse_both]"
        />
        <DialogPrimitive.Content
          ref={ref}
          data-slot="lightbox"
          aria-modal="true"
          aria-describedby={undefined}
          onPointerDownOutside={(event) => event.preventDefault()}
          onInteractOutside={(event) => event.preventDefault()}
          className={cn(lightboxVariants({ presentation }), className)}
          {...props}
        >
          <OverlayTitle className="sr-only">{label}</OverlayTitle>
          <div
            data-slot="stage"
            className="flex min-h-0 flex-1 flex-col px-16 pb-8"
            onClick={(event) => {
              // Only a press on the stage itself closes: children (the media wrapper) keep it open.
              if (event.target === event.currentTarget) onClose?.("stage");
            }}
          >
            {children}
          </div>
        </DialogPrimitive.Content>
      </DialogPrimitive.Portal>
    </DialogPrimitive.Root>
  );
});

/** The media wrapper: holds AttachmentPreview. A press on it never closes the lightbox. */
export const LightboxMedia = React.forwardRef<HTMLDivElement, React.HTMLAttributes<HTMLDivElement>>(
  function LightboxMedia({ className, ...props }, ref) {
    return (
      <div
        ref={ref}
        data-slot="media"
        className={cn("flex min-h-0 flex-1 items-center justify-center", className)}
        {...props}
      />
    );
  },
);
