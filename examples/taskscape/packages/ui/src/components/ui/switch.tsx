import * as React from "react";
import * as SwitchPrimitive from "@radix-ui/react-switch";
import { cn } from "../../lib/utils";

/**
 * An on/off setting that applies at once. It has no visible label of its own: the row title
 * names it through ariaLabel. Forced on = checked plus disabled; the reason is shown only then.
 */
export interface SwitchProps
  extends Omit<React.ComponentPropsWithoutRef<typeof SwitchPrimitive.Root>, "defaultChecked"> {
  checked?: boolean;
  disabled?: boolean;
  /** Visible text shown only while forced on (checked and disabled). "" = none. */
  reason?: string;
  /** The row title text; becomes the accessible name. */
  ariaLabel?: string;
  onCheckedChange?: (checked: boolean) => void;
}

export const Switch = React.forwardRef<React.ComponentRef<typeof SwitchPrimitive.Root>, SwitchProps>(
  function Switch(
    { checked = false, disabled = false, reason = "", ariaLabel = "", onCheckedChange, className, ...props },
    ref,
  ) {
    const reasonId = React.useId();
    const showReason = checked && disabled && reason !== "";

    return (
      <span className="inline-flex max-w-full flex-col items-end gap-[0.35rem]">
        <SwitchPrimitive.Root
          ref={ref}
          data-slot="switch"
          aria-label={ariaLabel}
          aria-describedby={showReason ? reasonId : undefined}
          checked={checked}
          disabled={disabled}
          onCheckedChange={onCheckedChange}
          className={cn(
            "group/switch relative inline-flex h-[1.357rem] w-[2.5rem] shrink-0 items-center rounded-full border border-transparent p-[0.1rem] outline-none",
            "bg-edge-3 transition-[background-color] duration-(--duration-fast) ease-standard",
            "data-[state=checked]:bg-accent-500 hover:data-[state=unchecked]:bg-wash-2",
            "focus-visible:ring-2 focus-visible:ring-focus-1 focus-visible:ring-offset-0",
            "disabled:pointer-events-none disabled:opacity-45",
            className,
          )}
          {...props}
        >
          <SwitchPrimitive.Thumb
            data-slot="thumb"
            className={cn(
              "pointer-events-none block h-[1rem] w-[1rem] translate-x-0 rounded-full bg-on-accent shadow-lift",
              "transition-transform duration-(--duration-fast) ease-standard",
              "data-[state=checked]:translate-x-[1.3rem]",
            )}
          />
        </SwitchPrimitive.Root>
        {showReason && (
          <span
            data-slot="reason"
            id={reasonId}
            className="max-w-[18rem] text-right text-[0.786rem] leading-[1.1rem] text-content-2"
          >
            {reason}
          </span>
        )}
      </span>
    );
  },
);
