import * as React from "react";
import * as ToastPrimitive from "@radix-ui/react-toast";
import { cva } from "class-variance-authority";
import { cn } from "../../lib/utils";
import { Button } from "./button";

/** Why the toast closed. The widget reports it with onDismissed. */
export type ToastDismissReason = "timeout" | "swipe" | "close" | "escape" | "action";
export type ToastSwipeDirection = "up" | "down" | "left" | "right";

/**
 * Radix treats an empty hotkey list as "every key pressed", so an empty focusHotkey
 * is replaced by a key name no keyboard event carries, which turns the shortcut off.
 */
const NO_HOTKEY = ["NoHotkey"];

/** Bottom-edge list: full width, centred, 20px (1.429rem) above the window bottom. */
export const toastViewportVariants = cva(
  "pointer-events-none fixed inset-x-0 z-(--z-index-popover) m-0 flex list-none flex-col items-center p-0 outline-none",
  {
    variants: {
      position: {
        "bottom-centre": "bottom-[1.429rem]",
      },
    },
    defaultVariants: { position: "bottom-centre" },
  },
);

/** The pill: 41px (2.929rem) tall, a popover tone with the lift shadow and the control radius. */
export const toastVariants = cva(
  [
    "pointer-events-auto relative flex h-[2.929rem] max-w-[calc(100%-2.286rem)] items-center gap-2 border pl-3.5 pr-1.5 select-none touch-none",
    "rounded-(--radius-control) outline-none focus-visible:ring-2 focus-visible:ring-focus-1 data-[state=closed]:hidden",
    "data-[swipe=move]:translate-x-(--radix-toast-swipe-move-x) data-[swipe=move]:translate-y-(--radix-toast-swipe-move-y)",
    "data-[swipe=cancel]:translate-x-0 data-[swipe=cancel]:translate-y-0",
  ],
  {
    variants: {
      tone: {
        neutral: "border-edge-2 bg-surface-3 text-content-1 shadow-lift",
      },
    },
    defaultVariants: { tone: "neutral" },
  },
);

export interface ToastViewportProps extends React.ComponentPropsWithoutRef<typeof ToastPrimitive.Viewport> {
  position?: "bottom-centre";
  /** Key that moves focus into the toast list; "" turns the shortcut off. */
  focusHotkey?: string;
}

/** The list that holds the toasts: one per window. It has no size while empty. */
export const ToastViewport = React.forwardRef<HTMLOListElement, ToastViewportProps>(function ToastViewport(
  { position = "bottom-centre", focusHotkey = "F8", className, ...props },
  ref,
) {
  return (
    <ToastPrimitive.Viewport
      ref={ref}
      data-slot="viewport"
      hotkey={focusHotkey ? [focusHotkey] : NO_HOTKEY}
      className={cn(toastViewportVariants({ position }), className)}
      {...props}
    />
  );
});

export interface ToastRootProps extends React.ComponentPropsWithoutRef<typeof ToastPrimitive.Root> {
  tone?: "neutral";
  /** true: no rise on entry (the app's Reduced motion setting, resolved). */
  reducedMotion?: boolean;
}

/** The notice itself. Polite (background) announcement. */
export const ToastRoot = React.forwardRef<HTMLLIElement, ToastRootProps>(function ToastRoot(
  { tone = "neutral", reducedMotion = false, className, ...props },
  ref,
) {
  return (
    <ToastPrimitive.Root
      ref={ref}
      role="status"
      type="background"
      data-slot="toast"
      className={cn(toastVariants({ tone }), !reducedMotion && "animate-rise", className)}
      {...props}
    />
  );
});

/** The notice text. */
export const ToastMessage = React.forwardRef<HTMLSpanElement, React.ComponentPropsWithoutRef<"span">>(function ToastMessage(
  { className, ...props },
  ref,
) {
  return (
    <span
      ref={ref}
      data-slot="message"
      className={cn("min-w-0 truncate text-[0.929rem] leading-[1.286rem]", className)}
      {...props}
    />
  );
});

/** Enter and Space press the focused control: the press is explicit, so it does not rely on the native key-to-click path. */
function pressOnKey(event: React.KeyboardEvent<HTMLButtonElement>) {
  if (event.key === "Enter" || event.key === " " || event.key === "Space") {
    event.preventDefault();
    event.currentTarget.click();
  }
}

/** A native Space click fires on keyup; cancelling it keeps one Space press to one click. */
function cancelSpaceKeyUp(event: React.KeyboardEvent<HTMLButtonElement>) {
  if (event.key === " " || event.key === "Space") event.preventDefault();
}

export interface ToastProps {
  /** Controlled visibility. false is the idle state: nothing is mounted. */
  open?: boolean;
  message?: string;
  actionLabel?: string;
  showAction?: boolean;
  /** The undo failed: errorMessage shows and the action reads "Try again". */
  failed?: boolean;
  errorMessage?: string;
  /** Auto-dismiss in ms. Paused while hovered or focused. */
  duration?: number;
  /** Key that moves focus into the toast list; "" turns it off. */
  focusHotkey?: string;
  /** true: no rise on entry. */
  reducedMotion?: boolean;
  swipeDirection?: ToastSwipeDirection;
  position?: "bottom-centre";
  tone?: "neutral";
  /** Fires whenever the widget asks for a new visibility. */
  onOpenChange?: (open: boolean) => void;
  /** Fires once per close. */
  onDismissed?: (reason: ToastDismissReason) => void;
  /** The action was pressed; label is the text shown at that moment. */
  onActioned?: (label: string) => void;
}

/**
 * Transient, non-blocking notice after an undoable action. Renders its own Radix
 * Provider and Viewport, so one Toast mounted per window is the whole notice system.
 */
export function Toast({
  open = false,
  message = "",
  actionLabel = "Undo",
  showAction = true,
  failed = false,
  errorMessage = "Could not undo",
  duration = 5000,
  focusHotkey = "F8",
  reducedMotion = false,
  swipeDirection = "down",
  position = "bottom-centre",
  tone = "neutral",
  onOpenChange,
  onDismissed,
  onActioned,
}: ToastProps) {
  const reason = React.useRef<ToastDismissReason | null>(null);
  const rootRef = React.useRef<HTMLLIElement>(null);
  const label = failed ? "Try again" : actionLabel;

  return (
    <ToastPrimitive.Provider swipeDirection={swipeDirection} label="Notifications">
      <ToastViewport position={position} focusHotkey={focusHotkey} />
      <ToastRoot
        ref={rootRef}
        forceMount
        open={open}
        duration={duration}
        tone={tone}
        reducedMotion={reducedMotion}
        onOpenChange={(next) => {
          if (!next) {
            onDismissed?.(reason.current ?? "timeout");
            reason.current = null;
          }
          onOpenChange?.(next);
        }}
        onEscapeKeyDown={(event) => {
          // Escape dismisses only while focus is inside this toast.
          if (!rootRef.current?.contains(document.activeElement)) {
            event.preventDefault();
            return;
          }
          reason.current = "escape";
        }}
        onSwipeEnd={() => {
          reason.current = "swipe";
        }}
      >
        <ToastMessage>{failed ? errorMessage : message}</ToastMessage>
        {showAction && (
          <ToastPrimitive.Action
            asChild
            altText={label}
            onClick={() => {
              reason.current = "action";
              onActioned?.(label);
            }}
          >
            <Button
              data-slot="action"
              variant="ghost"
              tone="strong"
              hover="wash"
              size="sm"
              width="content"
              label={label}
              onKeyDown={pressOnKey}
              onKeyUp={cancelSpaceKeyUp}
            />
          </ToastPrimitive.Action>
        )}
        <ToastPrimitive.Close
          asChild
          onClick={() => {
            reason.current = "close";
          }}
        >
          <Button
            data-slot="close"
            variant="ghost"
            tone="muted"
            hover="wash"
            size="sm"
            shape="icon"
            width="content"
            icon="close"
            iconSize={16}
            ariaLabel="Close"
            onKeyDown={pressOnKey}
            onKeyUp={cancelSpaceKeyUp}
          />
        </ToastPrimitive.Close>
      </ToastRoot>
    </ToastPrimitive.Provider>
  );
}
