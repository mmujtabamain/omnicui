import * as React from "react";
import * as CheckboxPrimitive from "@radix-ui/react-checkbox";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/utils";
import { Icon } from "./icon";

/**
 * Sizes are measured in px at 1rem = 14px: sm is a 16px box, md an 18px box.
 * Intent sets the checked colour: done uses the done-lamp token, select the selection accent.
 * The unchecked outline is one token (content-1) for both intents.
 */
export const checkboxVariants = cva(
  [
    "relative inline-flex shrink-0 items-center justify-center border bg-surface-3 text-on-accent select-none outline-none",
    "transition-[background-color,border-color,box-shadow] duration-(--duration-fast) ease-standard",
    "focus-visible:ring-2 focus-visible:ring-focus-1 focus-visible:ring-offset-0",
    "disabled:pointer-events-none disabled:opacity-45",
  ],
  {
    variants: {
      size: {
        sm: "size-[1.143rem]",
        md: "size-[1.286rem]",
      },
      shape: {
        square: "rounded-(--radius-field)",
        round: "rounded-full",
      },
      intent: {
        done: "border-content-1 data-[state=checked]:border-done-lamp-1 data-[state=checked]:bg-done-lamp-1",
        select: "border-content-1 data-[state=checked]:border-accent-500 data-[state=checked]:bg-accent-500",
      },
    },
    defaultVariants: { size: "sm", shape: "square", intent: "done" },
  },
);

type Variants = VariantProps<typeof checkboxVariants>;

export interface CheckboxProps
  extends Omit<React.ComponentPropsWithoutRef<typeof CheckboxPrimitive.Root>, "checked" | "onCheckedChange">,
    Omit<Variants, "size" | "shape" | "intent"> {
  size?: Variants["size"];
  shape?: Variants["shape"];
  intent?: Variants["intent"];
  checked?: boolean;
  onCheckedChange?: (checked: boolean) => void;
  /** Required accessible name: "Mark done" for intent done, "Select" for intent select. */
  ariaLabel: string;
}

export const Checkbox = React.forwardRef<HTMLButtonElement, CheckboxProps>(function Checkbox(
  {
    className, size = "sm", shape = "square", intent = "done", checked = false, ariaLabel,
    onCheckedChange, disabled = false, onKeyDown, ...props
  },
  ref,
) {
  // Done carries a check glyph; select carries an inner white dot.
  const glyph =
    intent === "select" ? (
      <span aria-hidden="true" className="block size-[0.429rem] rounded-full bg-on-accent" />
    ) : (
      <Icon name="check" size={size === "md" ? 14 : 12} weight={size === "md" ? 700 : 300} tone="inherit" />
    );

  return (
    <CheckboxPrimitive.Root
      ref={ref}
      data-slot="control"
      aria-label={ariaLabel}
      checked={checked}
      disabled={disabled}
      onCheckedChange={(next) => onCheckedChange?.(next === true)}
      onKeyDown={(event: React.KeyboardEvent<HTMLButtonElement>) => {
        onKeyDown?.(event);
        if (event.defaultPrevented) return;
        // Space toggles here, so one press makes one change; the native click that Space would
        // otherwise send is suppressed by preventDefault.
        if (event.key === " " || event.key === "Spacebar" || event.key === "Space" || event.code === "Space") {
          event.preventDefault();
          if (!event.repeat && !disabled) onCheckedChange?.(!checked);
        }
      }}
      className={cn(checkboxVariants({ size, shape, intent }), className)}
      {...props}
    >
      {/* Always mounted; hidden (display none) while unchecked, so it is not visible. */}
      <CheckboxPrimitive.Indicator forceMount data-slot="indicator" className="flex items-center justify-center data-[state=unchecked]:hidden">
        {glyph}
      </CheckboxPrimitive.Indicator>
    </CheckboxPrimitive.Root>
  );
});

/** The task-row group that holds a round check. Its hover border is drawn here, not on the control. */
export function CheckboxRow({ className, ...props }: React.ComponentProps<"div">) {
  return (
    <div
      data-slot="row"
      className={cn(
        "group/row inline-flex items-center rounded-(--radius-field) border border-transparent transition-[border-color] duration-(--duration-fast) ease-standard hover:border-content-3",
        className,
      )}
      {...props}
    />
  );
}
