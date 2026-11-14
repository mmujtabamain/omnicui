import * as React from "react";
import { cva } from "class-variance-authority";
import { cn } from "../../lib/utils";
import { Button } from "./button";

/**
 * Inline @mention chip for an attachment in a note body. It is an atom: contenteditable="false",
 * it cannot be typed into, and Backspace or Delete removes it as one unit. It is a button because
 * it is focusable, so Enter and Space press it (WAI-ARIA APG button); a native button does that.
 * Local and remote look the same; the source only decides where the press goes (the caller's onOpen).
 */
export const mentionChipVariants = cva(
  "max-w-full select-none rounded-full bg-selection-1 font-medium hover:bg-selection-1",
  {
    variants: {
      source: {
        local: "",
        remote: "",
      },
    },
    defaultVariants: { source: "local" },
  },
);

export interface MentionChipProps
  extends Omit<React.ComponentPropsWithoutRef<typeof Button>, "label" | "children" | "variant" | "size" | "width"> {
  /** local: press opens the attachment in the lightbox. remote: press calls the host's api.openAttachment. */
  source?: "local" | "remote";
  /** Shown as @name. */
  name?: string;
  /** Press, Enter or Space on the chip. */
  onOpen?: () => void;
  /** Backspace or Delete on the focused chip. */
  onRemove?: () => void;
}

export const MentionChip = React.forwardRef<HTMLButtonElement, MentionChipProps>(function MentionChip(
  { source = "local", name = "attachment", onOpen, onRemove, className, onClick, onKeyDown, ...props },
  ref,
) {
  return (
    <Button
      ref={ref}
      data-slot="chip"
      variant="ghost"
      tone="accent"
      size="sm"
      width="content"
      contentEditable={false}
      label={`@${name}`}
      className={cn(mentionChipVariants({ source }), className)}
      onClick={(e) => {
        onClick?.(e);
        if (!e.defaultPrevented) onOpen?.();
      }}
      onKeyDown={(e) => {
        onKeyDown?.(e);
        if (!e.defaultPrevented && (e.key === "Backspace" || e.key === "Delete")) {
          e.preventDefault();
          onRemove?.();
        }
      }}
      {...props}
    />
  );
});
