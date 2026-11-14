import * as React from "react";
import * as TogglePrimitive from "@radix-ui/react-toggle";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/utils";
import { formatAccel } from "../../lib/platform";
import { Icon } from "./icon";
import { Tooltip } from "./tooltip";

/** Off is content-2 with a wash on hover; on (data-state=on) is a wash-2 fill with strong text. */
export const toggleVariants = cva(
  [
    "group/toggle relative inline-flex min-w-0 shrink-0 select-none items-center justify-center gap-1 whitespace-nowrap",
    "outline-none transition-[color,background-color,opacity] duration-(--duration-fast) ease-standard",
    "text-content-2 hover:bg-wash-1 hover:text-content-1",
    "data-[state=on]:bg-wash-2 data-[state=on]:text-content-1 data-[state=on]:hover:bg-wash-2",
    "focus-visible:ring-2 focus-visible:ring-focus-1 focus-visible:ring-offset-0",
    "disabled:pointer-events-none disabled:opacity-45",
  ],
  {
    variants: {
      size: {
        sm: "h-[1.5rem] text-[0.857rem] leading-[1.286rem] rounded-(--radius-field)",
        md: "h-[1.75rem] rounded-(--radius-field)",
        lg: "h-[2rem] rounded-(--radius-control)",
      },
      shape: {
        icon: "aspect-square px-0",
        text: "px-1.5 font-semibold",
      },
    },
    defaultVariants: { size: "sm", shape: "icon" },
  },
);

/** Glyph size in px per toggle size (1rem is 14px). */
const GLYPH_PX = { sm: 15, md: 14, lg: 19 } as const;

export interface ToggleProps
  extends Omit<React.ButtonHTMLAttributes<HTMLButtonElement>, "type" | "defaultValue">,
    VariantProps<typeof toggleVariants> {
  /** Controlled pressed state. Absent: the toggle keeps its own state from defaultPressed. */
  pressed?: boolean;
  /** Uncontrolled starting state. */
  defaultPressed?: boolean;
  /** Asks for the next state on click, Enter or Space; controlled use decides whether it changes. */
  onPressedChange?: (pressed: boolean) => void;
  /** Glyph shown while off. */
  icon?: string;
  /** Glyph shown while pressed; empty keeps icon in both states. */
  pressedIcon?: string;
  /** Visible text for the text shape. */
  label?: string;
  /** Accessible name; constant, does not change with pressed. */
  ariaLabel?: string;
  /** Accelerator shown in the tooltip only, such as "Meta+Backslash". */
  shortcut?: string;
  /** Prevents default on mousedown so a focused editor keeps focus. */
  retainFocus?: boolean;
  disabled?: boolean;
}

export const Toggle = React.forwardRef<HTMLButtonElement, ToggleProps>(function Toggle(
  {
    pressed, defaultPressed = false, onPressedChange, size = "sm", shape = "icon", icon = "", pressedIcon = "",
    label = "", ariaLabel = "", shortcut = "", retainFocus = false, disabled = false, className, onMouseDown,
    ...props
  },
  ref,
) {
  const [inner, setInner] = React.useState(defaultPressed);
  const isPressed = pressed ?? inner;
  const glyph = isPressed && pressedIcon !== "" ? pressedIcon : icon;
  const name = ariaLabel !== "" ? ariaLabel : shape === "text" ? label : "";
  const tip = shortcut !== "" ? formatAccel(shortcut) : "";

  return (
    <Tooltip content={tip}>
      <TogglePrimitive.Root
        ref={ref}
        data-slot="toggle"
        {...props}
        pressed={isPressed}
        disabled={disabled}
        aria-pressed={isPressed}
        aria-label={name || undefined}
        onPressedChange={(next: boolean) => {
          if (pressed === undefined) setInner(next);
          onPressedChange?.(next);
        }}
        onMouseDown={(e: React.MouseEvent<HTMLButtonElement>) => {
          if (retainFocus) e.preventDefault();
          onMouseDown?.(e);
        }}
        className={cn(toggleVariants({ size, shape }), className)}
      >
        {glyph !== "" && <Icon name={glyph} size={GLYPH_PX[size ?? "sm"]} tone="inherit" />}
        {shape === "text" && label !== "" && <span data-slot="label" className="min-w-0 truncate">{label}</span>}
      </TogglePrimitive.Root>
    </Tooltip>
  );
});
