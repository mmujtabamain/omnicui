import * as React from "react";
import * as RadioGroupPrimitive from "@radix-ui/react-radio-group";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/utils";
import { Icon } from "./icon";

/** The group: a track for the surface tone and the surfaceThumb variant. */
const segmentedControlVariants = cva("inline-flex min-w-0 items-stretch rounded-(--radius-field)", {
  variants: {
    tone: {
      accent: "",
      surface: "bg-wash-2 p-[0.143rem]",
    },
    variant: {
      text: "",
      surfaceThumb: "bg-wash-2 p-[0.143rem]",
    },
  },
  defaultVariants: { tone: "accent", variant: "text" },
});

/** One segment. The selected look is driven by Radix's data-state, so the segment needs no state prop. */
const segmentVariants = cva(
  [
    "relative inline-flex min-w-0 shrink-0 select-none items-center justify-center gap-1 whitespace-nowrap rounded-(--radius-field) font-semibold text-content-2 outline-none",
    "transition-[color,background-color,box-shadow] duration-(--duration-fast) ease-standard",
    "data-[state=unchecked]:hover:bg-wash-1 focus-visible:ring-2 focus-visible:ring-focus-1 focus-visible:ring-offset-0",
  ],
  {
    variants: {
      tone: {
        accent: "data-[state=checked]:bg-accent-500 data-[state=checked]:text-on-accent data-[state=checked]:shadow-lift",
        surface: "data-[state=checked]:bg-surface-3 data-[state=checked]:text-content-1",
      },
      size: {
        default: "h-[1.643rem] text-[0.857rem] leading-[1.286rem]",
        sm: "h-[1.357rem] text-[0.786rem] leading-[1.179rem]",
      },
      shape: {
        text: "",
        icon: "",
      },
    },
    compoundVariants: [
      { shape: "text", size: "default", className: "px-[0.286rem]" },
      { shape: "text", size: "sm", className: "px-[0.143rem]" },
      { shape: "icon", size: "default", className: "w-[1.786rem]" },
      { shape: "icon", size: "sm", className: "w-[1.357rem]" },
    ],
    defaultVariants: { tone: "accent", size: "default", shape: "icon" },
  },
);

const SEGMENT_SLOTS = ["firstSegment", "secondSegment", "thirdSegment"] as const;

/** WAI-ARIA APG: Right and Down move to the next segment, Left and Up to the previous; both wrap. */
const NEXT_KEYS = new Set(["ArrowRight", "ArrowDown"]);
const PREVIOUS_KEYS = new Set(["ArrowLeft", "ArrowUp"]);

export interface SegmentedControlOption {
  value: string;
  label: string;
  /** Material Symbols name, shown for icon-only segments and the surfaceThumb variant. */
  icon?: string;
}

type RootVariants = VariantProps<typeof segmentedControlVariants>;
type SegmentVariants = VariantProps<typeof segmentVariants>;

export interface SegmentedControlProps
  extends Omit<React.ComponentPropsWithoutRef<typeof RadioGroupPrimitive.Root>, "value" | "defaultValue" | "onValueChange"> {
  options: SegmentedControlOption[];
  /** Selected value (controlled). Exactly one segment is always selected. */
  value: string;
  /** Fires only when the selection changes. */
  onValueChange?: (value: string) => void;
  tone?: SegmentVariants["tone"];
  variant?: RootVariants["variant"];
  shape?: SegmentVariants["shape"];
  size?: SegmentVariants["size"];
  disabled?: boolean;
  /** Required accessible name of the group. */
  ariaLabel?: string;
}

export const SegmentedControl = React.forwardRef<HTMLDivElement, SegmentedControlProps>(function SegmentedControl(
  {
    options,
    value,
    onValueChange,
    tone = "accent",
    variant = "text",
    shape = "icon",
    size = "default",
    disabled = false,
    ariaLabel,
    className,
    ...props
  },
  ref,
) {
  const showIcon = shape === "icon" || variant === "surfaceThumb";
  const showLabel = shape === "text";
  const select = (next: string) => {
    if (next !== value) onValueChange?.(next);
  };

  return (
    <RadioGroupPrimitive.Root
      ref={ref}
      data-slot="group"
      aria-label={ariaLabel}
      value={value}
      disabled={disabled}
      onValueChange={select}
      onKeyDownCapture={(event) => {
        // Arrows move focus and select the segment reached, wrapping at both ends; Radix's own
        // arrow handling does not select on keyboard focus here, so the root does both.
        const step = NEXT_KEYS.has(event.key) ? 1 : PREVIOUS_KEYS.has(event.key) ? -1 : 0;
        if (step === 0) return;
        const segments = Array.from(event.currentTarget.querySelectorAll<HTMLElement>('[role="radio"]'));
        const at = segments.indexOf(event.target as HTMLElement);
        if (at < 0) return;
        event.preventDefault();
        event.stopPropagation();
        const next = (at + step + segments.length) % segments.length;
        segments[next].focus();
        const option = options[next];
        if (option) select(option.value);
      }}
      className={cn(
        segmentedControlVariants({ tone, variant }),
        disabled && "pointer-events-none opacity-40",
        className,
      )}
      {...props}
    >
      {options.map((option, index) => (
        <RadioGroupPrimitive.Item
          key={option.value}
          value={option.value}
          aria-label={option.label}
          data-slot={SEGMENT_SLOTS[index]}
          className={segmentVariants({ tone, size, shape })}
        >
          {showIcon && option.icon && (
            // A fixed 15px box: a glyph name wider than the icon cannot widen the segment.
            <span className="inline-flex size-[1.071rem] shrink-0 items-center justify-center overflow-hidden">
              <Icon name={option.icon} size={15} tone="inherit" />
            </span>
          )}
          {showLabel && option.label}
        </RadioGroupPrimitive.Item>
      ))}
    </RadioGroupPrimitive.Root>
  );
});

export { segmentedControlVariants, segmentVariants };
