import * as React from "react";
import { Slot } from "@radix-ui/react-slot";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/utils";
import { formatAccel } from "../../lib/platform";
import { Icon } from "./icon";
import { Tooltip } from "./tooltip";

/** Tone is a text colour; the filled variants (primary, neutral, danger) ignore it. */
const buttonVariants = cva(
  [
    "group/button relative inline-flex min-w-0 items-center justify-center gap-1.5 select-none whitespace-nowrap font-medium",
    "outline-none transition-[color,background-color,border-color,opacity] duration-(--duration-fast) ease-standard",
    "focus-visible:ring-2 focus-visible:ring-focus-1 focus-visible:ring-offset-0",
    "disabled:pointer-events-none disabled:opacity-45 aria-disabled:pointer-events-none",
  ],
  {
    variants: {
      variant: {
        primary: "bg-accent-500 text-on-accent hover:bg-accent-600 active:bg-accent-700",
        neutral: "bg-surface-0 text-content-1 hover:bg-wash-2",
        outline: "border border-edge-2 bg-surface-3 hover:border-edge-3",
        ghost: "bg-transparent",
        danger: "bg-danger-500 text-on-accent hover:bg-danger-600",
        strong: "bg-transparent text-content-1 font-semibold",
        filled: "bg-surface-0 hover:bg-wash-2",
        dashed: "border border-dashed border-edge-2 bg-transparent hover:border-edge-3 hover:bg-wash-1",
        link: "bg-transparent px-0 underline-offset-2 hover:underline",
      },
      tone: {
        default: "text-content-2 hover:text-content-1",
        neutral: "text-content-2 hover:text-content-1",
        strong: "text-content-1",
        accent: "text-accent-500 hover:text-accent-600",
        danger: "text-danger-500 hover:text-danger-600",
        done: "text-done-lamp-1",
        muted: "text-content-3 hover:text-content-2",
      },
      size: {
        sm: "h-[1.5rem] px-1.5 text-[0.857rem] rounded-(--radius-field)",
        md: "h-[1.75rem] px-2 text-[0.929rem] rounded-(--radius-field)",
        lg: "h-[2rem] px-2.5 text-[1rem] rounded-(--radius-field)",
        default: "h-[1.5rem] px-1.5 text-[0.929rem] rounded-(--radius-field)",
        footer: "h-[1.357rem] px-1 text-[0.857rem] rounded-(--radius-field)",
        xl: "h-[2rem] px-2 text-[1rem] rounded-(--radius-control)",
        step: "h-[2.786rem] px-3 text-[1rem] rounded-full",
        dashedShort: "h-[2.5rem] px-3 text-[0.929rem] rounded-(--radius-control)",
        dashedTall: "h-[6rem] px-3 text-[0.929rem] rounded-(--radius-control)",
      },
      shape: { text: "", icon: "aspect-square px-0", square: "aspect-square px-0", round: "aspect-square px-0 rounded-full" },
      layout: { inline: "", stacked: "aspect-square h-auto flex-col gap-1 py-2" },
      width: { full: "w-full", half: "w-1/2", content: "w-auto" },
      hover: { text: "", wash: "hover:bg-wash-1" },
      reveal: {
        always: "",
        row: "opacity-0 hover:opacity-100 focus-visible:opacity-100 group-hover/row:opacity-100 group-focus-within/row:opacity-100 group-data-[selected=true]/row:opacity-100",
      },
      surface: { default: "", media: "" },
    },
    compoundVariants: [
      // Filled variants carry their own text colour, whatever the tone.
      { variant: ["primary", "danger"], tone: ["default", "neutral", "strong", "accent", "danger", "done", "muted"], className: "text-on-accent hover:text-on-accent" },
      { variant: "neutral", tone: ["default", "neutral", "strong", "accent", "danger", "done", "muted"], className: "text-content-1 hover:text-content-1" },
      { variant: "strong", tone: "default", className: "text-content-1" },
      // Icon-only shapes are square at the size's height.
      { shape: ["icon", "square", "round"], width: ["full", "half"], className: "w-auto" },
      // Over photos and the lightbox: white glyphs, a white wash.
      { surface: "media", className: "text-white/90 hover:text-white" },
      { surface: "media", variant: "ghost", className: "hover:bg-white/15" },
      { surface: "media", variant: "filled", className: "bg-white/10 hover:bg-white/20" },
      { surface: "media", tone: "danger", className: "hover:bg-danger-500 hover:text-white" },
    ],
    defaultVariants: { variant: "ghost", tone: "default", size: "md", shape: "text", layout: "inline", width: "full", hover: "text", reveal: "always", surface: "default" },
  },
);

const ICON_SIZE: Record<string, number> = { sm: 14, md: 16, default: 16, lg: 16, footer: 14, xl: 19, step: 26, dashedShort: 16, dashedTall: 18 };

type Variants = VariantProps<typeof buttonVariants>;

export interface ButtonProps extends Omit<React.ButtonHTMLAttributes<HTMLButtonElement>, "type"> {
  variant?: Variants["variant"];
  size?: Variants["size"];
  tone?: Variants["tone"];
  shape?: Variants["shape"];
  layout?: Variants["layout"];
  surface?: Variants["surface"];
  reveal?: Variants["reveal"];
  width?: Variants["width"];
  hover?: Variants["hover"];
  icon?: string;
  trailingIcon?: string;
  label?: string;
  ariaLabel?: string;
  /** An accelerator such as "Mod+S", shown in the tooltip only. */
  shortcut?: string;
  /** A spinner replaces the icon, and the button is disabled. */
  busy?: boolean;
  /** Shown in place of label and icon while non-empty; the caller owns the timer. */
  feedbackLabel?: string;
  feedbackIcon?: string;
  /** Red text and an error glyph; the text becomes the tooltip. */
  errorText?: string;
  /** A wash while a mark is on; exposed as selected. */
  active?: boolean;
  /** false takes the button out of the Tab order. */
  tabbable?: boolean;
  type?: "button" | "submit" | "reset";
  asChild?: boolean;
  iconSize?: number;
  iconWeight?: number;
  /** Corner radius in px; 0 keeps the size's own. */
  radius?: number;
  /** Pointer presses do not move focus away from a focused text field. */
  keepFocusOnPress?: boolean;
  /** Mouse down never takes focus. */
  retainFocus?: boolean;
  truncate?: boolean;
  mono?: boolean;
  /** Full-row hint style: left aligned and padded. */
  block?: boolean;
  /** Shown, muted, when label is empty. */
  emptyLabel?: string;
  /** Tooltip text. */
  description?: string;
}

export const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(function Button(
  {
    variant, size = "md", tone, shape = "text", layout, surface, reveal, width, hover,
    icon = "", trailingIcon = "", label = "", ariaLabel = "", shortcut = "", busy = false,
    feedbackLabel = "", feedbackIcon = "", errorText = "", active = false, tabbable = true,
    type = "button", asChild = false, iconSize = 0, iconWeight = 300, radius = 0,
    keepFocusOnPress = false, retainFocus = false, truncate = true, mono = false, block = false,
    emptyLabel = "", description = "", disabled = false, className, style, children, onMouseDown, ...props
  },
  ref,
) {
  const feedback = feedbackLabel !== "";
  const error = errorText !== "";
  const glyph = busy ? "" : error ? "error" : feedback && feedbackIcon ? feedbackIcon : icon;
  const glyphSize = iconSize || (surface === "media" && size === "xl" ? 17 : ICON_SIZE[size ?? "md"]);
  const text = feedback ? feedbackLabel : label;
  const tip = [error ? errorText : description, shortcut && formatAccel(shortcut)].filter(Boolean).join(" ");
  const Comp = asChild ? Slot : "button";

  const content = asChild ? (
    children
  ) : (
    <>
      {busy && <Icon data-slot="spinner" name="progress_activity" size={glyphSize} weight={iconWeight} tone="inherit" className="animate-spin" />}
      {glyph && <Icon data-slot="icon" name={glyph} size={glyphSize} weight={iconWeight} tone="inherit" />}
      {shape === "text" && (text !== "" || emptyLabel !== "") && (
        <span data-slot="label" className={cn("min-w-0", truncate ? "truncate" : "whitespace-normal text-left", text === "" && "text-content-3")}>
          {text !== "" ? text : emptyLabel}
        </span>
      )}
      {trailingIcon && <Icon data-slot="trailingIcon" name={trailingIcon} size={glyphSize} weight={iconWeight} tone="inherit" className="-mr-0.5" />}
      {feedback && <span data-slot="feedback" role="status" className="sr-only">{feedbackLabel}</span>}
    </>
  );

  return (
    <Tooltip content={tip}>
      <Comp
        ref={ref}
        data-slot={asChild ? "child" : "button"}
        type={asChild ? undefined : type}
        aria-label={ariaLabel || undefined}
        aria-selected={active || undefined}
        aria-busy={busy || undefined}
        data-active={active || undefined}
        disabled={asChild ? undefined : disabled || busy}
        aria-disabled={asChild && (disabled || busy) ? true : undefined}
        tabIndex={tabbable ? undefined : -1}
        onMouseDown={(e: React.MouseEvent<HTMLButtonElement>) => {
          if (retainFocus || (keepFocusOnPress && document.activeElement instanceof HTMLElement && document.activeElement.matches("input, textarea, [contenteditable=true]")))
            e.preventDefault();
          onMouseDown?.(e);
        }}
        className={cn(
          buttonVariants({ variant, size, tone, shape, layout, surface, reveal, width, hover }),
          active && "bg-wash-2",
          error && "text-danger-500 hover:text-danger-600",
          mono && "font-mono text-[0.821rem] leading-[1.232rem] font-normal",
          block && "justify-start px-2 text-left font-normal",
          className,
        )}
        style={radius ? { borderRadius: radius, ...style } : style}
        {...props}
      >
        {content}
      </Comp>
    </Tooltip>
  );
});

export { buttonVariants };
