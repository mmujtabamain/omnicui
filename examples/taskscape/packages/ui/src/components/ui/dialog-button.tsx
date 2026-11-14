import * as React from "react";
import * as AlertDialogPrimitive from "@radix-ui/react-alert-dialog";
import { cva } from "class-variance-authority";
import { cn } from "../../lib/utils";
import { Button } from "./button";

/**
 * The footer minimum: 84px is the measured Cancel width (6rem at 1rem = 14px). Every role takes it,
 * so Cancel and the accept buttons line up in a sheet footer. There is no tone axis; the look is the
 * Button variant.
 */
export const dialogButtonVariants = cva("min-w-[6rem]");

export type DialogButtonRole = "accept" | "cancel" | "instead";
export type DialogButtonTone = "primary" | "neutral" | "danger";

export interface DialogButtonProps extends Omit<React.ButtonHTMLAttributes<HTMLButtonElement>, "role"> {
  /** accept (Radix AlertDialog Action), cancel (AlertDialog Cancel) or instead (plain button). "" plays no role. */
  role: DialogButtonRole | "";
  /** The id instead resolves with. Read only when role is instead. */
  insteadId?: string;
  /** primary | neutral | danger, passed to Button. "" and unknown values take neutral (the recorded default). */
  variant?: DialogButtonTone | "";
  /** Visible text, also the accessible name. */
  label?: string;
  /** Passed to Button. */
  disabled?: boolean;
  /** accept: called on every press. The caller's onClick may preventDefault to keep the sheet open. */
  onActivate?: () => void;
  /** cancel: called on press, then the Radix Cancel closes the sheet. */
  onDismiss?: () => void;
  /** instead: called with insteadId on press. */
  onResolve?: (insteadId: string) => void;
}

export const DialogButton = React.forwardRef<HTMLButtonElement, DialogButtonProps>(function DialogButton(
  { role, insteadId = "", variant = "", label = "", disabled = false, onActivate, onDismiss, onResolve, onClick, className, ...props },
  ref,
) {
  const tone: DialogButtonTone = variant === "primary" || variant === "danger" ? variant : "neutral";
  const look = {
    size: "lg" as const,
    width: "content" as const,
    variant: tone,
    label,
    disabled,
    className: cn(dialogButtonVariants(), className),
  };

  // Radix closes the sheet after the press unless the press was defaultPrevented (the caller's
  // empty-field guard). Activate is emitted on every press, guard or not.
  if (role === "accept") {
    const press = (event: React.MouseEvent<HTMLButtonElement>) => {
      onClick?.(event);
      onActivate?.();
    };
    return (
      <AlertDialogPrimitive.Action asChild onClick={press}>
        <Button ref={ref} data-part="ui.dialogbutton.button" {...look} {...props} />
      </AlertDialogPrimitive.Action>
    );
  }

  if (role === "cancel") {
    const dismiss = (event: React.MouseEvent<HTMLButtonElement>) => {
      onClick?.(event);
      onDismiss?.();
    };
    return (
      <AlertDialogPrimitive.Cancel asChild onClick={dismiss}>
        <Button ref={ref} data-part="ui.dialogbutton.button" {...look} {...props} />
      </AlertDialogPrimitive.Cancel>
    );
  }

  const resolve = (event: React.MouseEvent<HTMLButtonElement>) => {
    onClick?.(event);
    onResolve?.(insteadId);
  };
  return (
    <Button
      ref={ref}
      data-part="ui.dialogbutton.button"
      {...look}
      {...props}
      onClick={role === "instead" ? resolve : onClick}
    />
  );
});
