import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/utils";

/**
 * Inline @mention chip for an attachment inside a note body. It is an atom: contenteditable is
 * false, it cannot be typed into, and Backspace or Delete removes it as one unit. It is a native
 * button, so Enter and Space open it too (WAI-ARIA APG button). The source changes what a press
 * does, not how the chip looks.
 */
export const mentionChipVariants = cva(
  [
    "inline-flex min-w-0 max-w-full items-center align-middle select-none whitespace-nowrap",
    "h-[1.5rem] px-1.5 rounded-(--radius-field) text-[0.857rem] leading-none font-medium",
    "bg-selection-1 text-accent-600 cursor-pointer outline-none",
    "transition-[color,background-color] duration-(--duration-fast) ease-standard hover:text-accent-700",
    "focus-visible:ring-2 focus-visible:ring-focus-1 focus-visible:ring-offset-0",
  ],
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
  extends Omit<React.ButtonHTMLAttributes<HTMLButtonElement>, "type">,
    VariantProps<typeof mentionChipVariants> {
  /** Shown as @name. */
  name?: string;
  /** Press, Enter or Space on the chip. */
  onOpen?: () => void;
  /** Backspace or Delete on the focused chip. */
  onRemove?: () => void;
}

export const MentionChip = React.forwardRef<HTMLButtonElement, MentionChipProps>(function MentionChip(
  { source = "local", name = "attachment", onOpen, onRemove, onClick, onKeyDown, className, ...props },
  ref,
) {
  return (
    <button
      ref={ref}
      type="button"
      data-slot="chip"
      data-source={source}
      contentEditable={false}
      className={cn(mentionChipVariants({ source }), className)}
      onClick={(e) => {
        onClick?.(e);
        onOpen?.();
      }}
      onKeyDown={(e) => {
        onKeyDown?.(e);
        if (e.defaultPrevented) return;
        // Enter and Space open the chip here; preventDefault stops the native click so it opens once.
        if (e.key === "Enter" || e.key === " " || e.key === "Spacebar") {
          e.preventDefault();
          onOpen?.();
        } else if (e.key === "Backspace" || e.key === "Delete") {
          e.preventDefault();
          onRemove?.();
        }
      }}
      {...props}
    >
      <span className="min-w-0 truncate">@{name}</span>
    </button>
  );
});
