import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/utils";

/**
 * Titled is the full app window: h-screen, w-screen, a TitleBar above the body.
 * Frameless is the desktop capture card: rounded panel with a 1px border, width of its window, no TitleBar.
 */
export const windowFrameVariants = cva(
  "relative flex min-h-0 min-w-0 flex-col overflow-hidden text-content-1",
  {
    variants: {
      chrome: {
        titled: "h-screen w-screen bg-surface-1",
        frameless: "w-screen rounded-(--radius-panel) border border-edge-2 bg-surface-2",
      },
    },
    defaultVariants: { chrome: "titled" },
  },
);

export interface WindowFrameProps
  extends React.HTMLAttributes<HTMLDivElement>,
    VariantProps<typeof windowFrameVariants> {}

/** The root of one window. Each desktop window is its own document, so it has no role. */
export const WindowFrame = React.forwardRef<HTMLDivElement, WindowFrameProps>(function WindowFrame(
  { chrome, className, ...props },
  ref,
) {
  return <div ref={ref} data-slot="frame" className={cn(windowFrameVariants({ chrome }), className)} {...props} />;
});

/** Titled only: the row above the body. Its own TitleBar widget nests inside. */
export function WindowFrameTitleBar({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      role="banner"
      data-slot="titleBar"
      data-part="ui.windowframe.titleBar"
      className={cn("flex shrink-0 min-w-0 items-center", className)}
      {...props}
    />
  );
}

/** Titled only: the row holding the PaneGroup. */
export function WindowFrameBody({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return <div data-slot="body" className={cn("flex min-h-0 min-w-0 flex-1 overflow-hidden", className)} {...props} />;
}

/** Titled only: the layer above the body for sheets and toasts. Clicks pass through it. */
export function WindowFrameOverlays({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      data-slot="overlays"
      className={cn("pointer-events-none absolute inset-0 z-(--z-index-overlay)", className)}
      {...props}
    />
  );
}

/** Frameless only: the single-line title field of the capture bar. */
export const WindowFrameCaptureTitle = React.forwardRef<HTMLInputElement, React.InputHTMLAttributes<HTMLInputElement>>(
  function WindowFrameCaptureTitle({ className, ...props }, ref) {
    return (
      <input
        ref={ref}
        type="text"
        data-slot="captureTitle"
        aria-label="Title"
        autoComplete="off"
        spellCheck={false}
        className={cn(
          "block h-[2.25rem] w-full min-w-0 bg-transparent px-3 text-[1.143rem] font-medium text-content-1 outline-none",
          "placeholder:text-content-3 focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-focus-1",
          className,
        )}
        {...props}
      />
    );
  },
);

/** Frameless only: the notes field of the capture bar, under the title. */
export const WindowFrameCaptureNotes = React.forwardRef<
  HTMLTextAreaElement,
  React.TextareaHTMLAttributes<HTMLTextAreaElement>
>(function WindowFrameCaptureNotes({ className, ...props }, ref) {
  return (
    <textarea
      ref={ref}
      data-slot="captureNotes"
      aria-label="Notes"
      rows={3}
      spellCheck={false}
      className={cn(
        "block min-h-[4.5rem] w-full min-w-0 resize-none border-t border-edge-1 bg-transparent px-3 py-2 text-[0.929rem] leading-[1.357rem] text-content-2 outline-none",
        "placeholder:text-content-3 focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-focus-1",
        className,
      )}
      {...props}
    />
  );
});
