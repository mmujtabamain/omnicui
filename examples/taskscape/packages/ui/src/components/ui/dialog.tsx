import * as React from "react";
import * as DialogPrimitive from "@radix-ui/react-dialog";
import { cva } from "class-variance-authority";
import { cn } from "../../lib/utils";
import { Button } from "./button";
import { Footer } from "./footer";
import { Input } from "./input";
import { Overlay, OverlayTitle, type OverlayPlacement } from "./overlay";

/**
 * The card. Its base is the Overlay layer (surface 2, menu elevation, 0.75rem corners). Sheet is a
 * centred 26rem card; palette hangs from the top at 35rem; fullscreen and the desktop window fill
 * the window. The window keeps the sheet width, centred, inside its own window.
 */
export const dialogCardVariants = cva("min-w-0", {
  variants: {
    size: {
      sheet: "w-[min(26rem,100%)] overflow-hidden",
      palette: "top-[6.5rem] max-h-[60vh] w-[min(35rem,calc(100%_-_2rem))] overflow-y-auto",
      fullscreen: "overflow-y-auto rounded-none border-0 bg-surface-2 shadow-none",
    },
    presentation: {
      overlay: "",
      window: "flex flex-col justify-center rounded-none border-0 bg-surface-2 shadow-none",
    },
  },
  compoundVariants: [{ presentation: "window", size: "sheet", className: "w-full overflow-hidden" }],
  defaultVariants: { size: "sheet", presentation: "overlay" },
});

export type DialogKind = "name" | "confirm";
export type DialogTone = "" | "default" | "destructive";
export type DialogPresentation = "overlay" | "window";
export type DialogSize = "sheet" | "palette" | "fullscreen";

export interface DialogProps {
  /** Controlled visibility, passed to Overlay. false renders nothing. */
  open?: boolean;
  /** Called with false when the card asks to close (Cancel, Escape, or an outside press on web). */
  onOpenChange?: (open: boolean) => void;
  /** Runs the accept action: Accept pressed, Enter or Space on Accept, or Enter in the field. */
  onAccept?: () => void;
  /** name: the field takes focus. confirm: Cancel takes focus, role alertdialog, no field. */
  kind?: DialogKind;
  /** Accept colour. "" derives it from kind (name default, confirm destructive). */
  tone?: DialogTone;
  presentation?: DialogPresentation;
  size?: DialogSize;
  /** Badge text at the top of the card. "" hides the badge. */
  badge?: string;
  /** Headline, rendered as the dialog's title. */
  title?: string;
  /** Detail copy. "" hides it and leaves aria-describedby unset. */
  description?: string;
  /** Text of the footer's left slot. "" hides it. */
  instead?: string;
  acceptLabel?: string;
  cancelLabel?: string;
  className?: string;
}

/**
 * The name card's body field. It is mounted only while the card is open, so each opening starts
 * empty. Enter submits (Input does not insert a line break) and accepts the card.
 */
function DialogField({ label, onAccept }: { label: string; onAccept?: () => void }) {
  const [text, setText] = React.useState("");
  return (
    <Input
      ariaLabel={label}
      mode="bare"
      size="md"
      value={text}
      onValueChange={setText}
      onSubmit={() => onAccept?.()}
      className="h-[1.75rem] rounded-(--radius-field) border border-edge-2 bg-surface-3 px-[0.571rem] text-[0.929rem] focus:border-accent-500 focus:ring-2 focus:ring-focus-1"
    />
  );
}

/**
 * Modal card built on Radix Dialog through Overlay (the single Radix root). Body and footer
 * follow the spec's name and confirm cards.
 */
export const Dialog = React.forwardRef<HTMLDivElement, DialogProps>(function Dialog(
  {
    open = false,
    onOpenChange,
    onAccept,
    kind = "name",
    tone = "",
    presentation = "overlay",
    size = "sheet",
    badge = "",
    title = "",
    description = "",
    instead = "",
    acceptLabel = "",
    cancelLabel = "Cancel",
    className,
  },
  ref,
) {
  const descriptionId = React.useId();
  const confirm = kind === "confirm";
  const destructive = tone === "destructive" || (tone === "" && confirm);
  const placement: OverlayPlacement =
    size === "palette" ? "top" : size === "fullscreen" || presentation === "window" ? "fill" : "center";

  return (
    <Overlay
      ref={ref}
      data-slot="card"
      open={open}
      onOpenChange={onOpenChange}
      placement={placement}
      dismissOn={presentation === "window" ? "none" : "outside"}
      role={confirm ? "alertdialog" : "dialog"}
      aria-describedby={description ? descriptionId : undefined}
      className={cn(dialogCardVariants({ size, presentation }), className)}
    >
      <div
        className={cn(
          "flex min-w-0 flex-col gap-[0.571rem] pt-[1.071rem] pr-[0.714rem] pb-[0.714rem] pl-[1.071rem]",
          presentation === "window" && size === "sheet" && "mx-auto w-[min(26rem,100%)]",
        )}
      >
        {badge !== "" && (
          <span
            data-slot="badge"
            className="self-start rounded-full bg-wash-2 px-[0.5rem] py-[0.143rem] text-[0.786rem] leading-[1.2] font-medium text-content-2"
          >
            {badge}
          </span>
        )}
        <OverlayTitle>{title}</OverlayTitle>
        {description !== "" && (
          <DialogPrimitive.Description
            id={descriptionId}
            data-slot="description"
            className="text-[0.929rem] leading-[1.4] text-content-2"
          >
            {description}
          </DialogPrimitive.Description>
        )}
        {!confirm && <DialogField label={title} onAccept={onAccept} />}
        <Footer density="row" edge="none" className="min-w-0 gap-[0.429rem] px-0 py-0">
          {instead !== "" && (
            <span data-slot="instead" className="min-w-0 text-[0.857rem] leading-[1.3] text-content-3">
              {instead}
            </span>
          )}
          <div className="ml-auto flex shrink-0 items-center gap-[0.429rem]">
            <Button
              data-slot="cancel"
              variant="neutral"
              width="content"
              label={cancelLabel}
              onClick={() => onOpenChange?.(false)}
            />
            <Button
              data-slot="accept"
              variant={destructive ? "danger" : "primary"}
              width="content"
              label={acceptLabel}
              onClick={() => onAccept?.()}
            />
          </div>
        </Footer>
      </div>
    </Overlay>
  );
});
