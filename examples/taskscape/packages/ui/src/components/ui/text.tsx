import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/utils";
import { Tooltip } from "./tooltip";

/** The one general text label. Sizes are rem at 1rem = 14px; colours are tokens only. */
export const textVariants = cva("min-w-0", {
  variants: {
    size: {
      xs: "text-[0.786rem]",
      sm: "text-[0.857rem]",
      md: "text-[0.929rem]",
      base: "text-[1rem]",
      inherit: "text-[length:inherit]",
      title: "text-[1rem] leading-[1.1875rem]",
      caption: "text-[0.893rem] leading-4.25",
    },
    weight: {
      regular: "font-normal",
      medium: "font-medium",
      semibold: "font-semibold",
    },
    tone: {
      primary: "text-content-1",
      secondary: "text-content-2",
      muted: "text-content-3",
      danger: "text-danger-500",
      onMedia: "text-white",
      onMediaMuted: "text-white/60",
      inherit: "text-[color:inherit]",
    },
    font: {
      sans: "",
      mono: "font-mono",
    },
    overflow: {
      wrap: "whitespace-normal break-words",
      truncate: "block truncate",
    },
    case: {
      normal: "",
      uppercase: "uppercase tracking-[0.08em]",
    },
    numeric: {
      off: "",
      tabular: "tabular-nums",
    },
    fontStyle: {
      normal: "not-italic",
      italic: "italic",
    },
    done: {
      off: "",
      struck: "line-through",
      faded: "opacity-55",
    },
    align: {
      start: "text-left",
      center: "text-center",
      end: "text-right",
    },
  },
  defaultVariants: {
    size: "md",
    tone: "primary",
    font: "sans",
    overflow: "wrap",
    case: "normal",
    numeric: "off",
    fontStyle: "normal",
    done: "off",
    align: "start",
  },
});

type Variants = VariantProps<typeof textVariants>;

export interface TextProps extends React.HTMLAttributes<HTMLElement>, Omit<Variants, "weight"> {
  /** Element. span by default; p for helper and error lines; div for block rows. */
  as?: "span" | "p" | "div";
  /** Medium for titles, semibold for stats; regular otherwise. */
  weight?: Variants["weight"];
  /** Lets the user select and copy the text. Chrome labels are not selectable. */
  selectable?: boolean;
  /** Full value shown on hover and keyboard focus. */
  tooltip?: string;
  /** Announces the text when it appears. */
  live?: "off" | "polite" | "assertive";
  children?: React.ReactNode;
}

export const Text = React.forwardRef<HTMLElement, TextProps>(function Text(
  {
    as: Comp = "span",
    size = "md",
    weight = size === "title" ? "medium" : "regular",
    tone = "primary",
    font = "sans",
    overflow = "wrap",
    case: textCase = "normal",
    numeric = "off",
    fontStyle = "normal",
    done = "off",
    align = "start",
    selectable = false,
    tooltip = "",
    live = "off",
    className,
    children,
    ...props
  },
  ref,
) {
  // A done title is always muted, whatever tone the caller asked for.
  const colour = done === "struck" ? "muted" : tone;

  const node = React.createElement(
    Comp,
    {
      ref,
      "data-slot": "text",
      "aria-live": live === "off" ? undefined : live,
      tabIndex: tooltip ? 0 : undefined,
      className: cn(
        textVariants({ size, weight: weight ?? "regular", tone: colour, font, overflow, case: textCase, numeric, fontStyle, done, align }),
        selectable ? "select-text" : "select-none",
        // A tooltip makes the text focusable (tabIndex 0): show the focus ring from the tokens.
        tooltip && "outline-none focus-visible:ring-2 focus-visible:ring-focus-1",
        className,
      ),
      ...props,
    },
    children,
  );

  return tooltip ? <Tooltip content={tooltip}>{node}</Tooltip> : node;
});
