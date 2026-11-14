import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/utils";
import { Button, type ButtonProps } from "./button";

/**
 * Colour per tone; the confirming state adds a border cue. The button is the same control in
 * both states: only its label and these classes change.
 */
export const confirmButtonVariants = cva("", {
  variants: {
    tone: {
      default: "text-content-1 hover:text-content-1",
      danger: "text-danger-500 hover:text-danger-600",
    },
    confirming: {
      false: "",
      true: "",
    },
  },
  compoundVariants: [
    { tone: "default", confirming: true, className: "border-edge-3" },
    { tone: "danger", confirming: true, className: "border-danger-500" },
  ],
  defaultVariants: { tone: "default", confirming: false },
});

type Variants = VariantProps<typeof confirmButtonVariants>;

export interface ConfirmButtonProps extends Omit<ButtonProps, "label" | "tone" | "variant" | "size" | "onClick"> {
  /** "default" is an ordinary action; "danger" is destructive and Enter does not confirm it. */
  tone?: Variants["tone"];
  /** Idle label. */
  label?: string;
  /** Label while confirming. */
  confirmLabel?: string;
  /** Runs on the second activation. */
  onConfirm?: () => void;
}

export const ConfirmButton = React.forwardRef<HTMLButtonElement, ConfirmButtonProps>(function ConfirmButton(
  {
    tone = "default",
    label = "Reset all settings…",
    confirmLabel = "Confirm reset",
    onConfirm,
    className,
    onKeyDown,
    onBlur,
    ...props
  },
  ref,
) {
  const [confirming, setConfirming] = React.useState(false);
  const danger = tone === "danger";

  const handleClick = () => {
    if (!confirming) {
      setConfirming(true);
      return;
    }
    setConfirming(false);
    onConfirm?.();
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLButtonElement>) => {
    onKeyDown?.(e);
    if (e.defaultPrevented || !confirming) return;
    if (e.key === "Escape") {
      e.preventDefault();
      e.stopPropagation();
      setConfirming(false);
      return;
    }
    // Destructive tone: Enter arms (above) but never confirms; Space or a click confirms.
    if (e.key === "Enter" && danger) e.preventDefault();
  };

  const handleBlur = (e: React.FocusEvent<HTMLButtonElement>) => {
    onBlur?.(e);
    setConfirming(false);
  };

  return (
    <Button
      ref={ref}
      data-part="ui.confirmbutton.button"
      variant="outline"
      size="lg"
      label={confirming ? confirmLabel : label}
      className={cn(confirmButtonVariants({ tone, confirming }), className)}
      onClick={handleClick}
      onKeyDown={handleKeyDown}
      onBlur={handleBlur}
      {...props}
    />
  );
});
