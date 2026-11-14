import * as React from "react";
import { cva } from "class-variance-authority";
import { cn } from "../../lib/utils";

/** Named sizes. The px value is the measured glyph size; 1rem is 14px. */
export const ICON_SIZES = { xs: 12, sm: 14, md: 16, lg: 18, xl: 20, display: 44 } as const;
export type IconSize = keyof typeof ICON_SIZES;
export type IconTone = "inherit" | "primary" | "secondary" | "muted" | "accent" | "danger" | "onMedia" | "onMediaMuted";
export type IconTint = "none" | "accent" | "danger";
export type IconReveal = "always" | "groupHover";
export type IconRotate = "none" | "ccw90";

const WEIGHTS = [200, 300, 400] as const;
type IconWeightKey = "200" | "300" | "400";

/** The step closest to value; a tie goes to the larger step (13 to 14, 15 to 16, 19 to 20). */
function nearest<T extends number>(value: number, steps: readonly T[]): T {
  return steps.reduce((best, step) => {
    const d = Math.abs(step - value);
    const bd = Math.abs(best - value);
    return d < bd || (d === bd && step > best) ? step : best;
  });
}

/** Accepts a named size or a px number, which snaps to the nearest named step. */
export function resolveIconSize(size: IconSize | number): IconSize {
  if (typeof size === "number") {
    const px = nearest(size, Object.values(ICON_SIZES));
    return (Object.keys(ICON_SIZES) as IconSize[]).find((key) => ICON_SIZES[key] === px) ?? "md";
  }
  if (!Object.prototype.hasOwnProperty.call(ICON_SIZES, size))
    throw new Error(`Icon size "${String(size)}" is not one of ${Object.keys(ICON_SIZES).join(", ")}`);
  return size;
}

export const iconVariants = cva(
  [
    "material-symbols-outlined [font-family:'Material_Symbols_Outlined'] pointer-events-none inline-flex shrink-0 select-none items-center justify-center",
    "whitespace-nowrap normal-case tracking-normal leading-none",
    "transition-[opacity,transform,rotate] duration-(--duration-fast) ease-standard motion-reduce:transition-none",
  ],
  {
    variants: {
      size: {
        xs: "text-[0.857rem]",
        sm: "text-[1rem]",
        md: "text-[1.143rem]",
        lg: "text-[1.286rem]",
        xl: "text-[1.429rem]",
        display: "text-[3.143rem]",
      },
      weight: {
        "200": "[font-variation-settings:'wght'_200]",
        "300": "[font-variation-settings:'wght'_300]",
        "400": "[font-variation-settings:'wght'_400]",
      },
      tone: {
        inherit: "",
        primary: "text-content-1",
        secondary: "text-content-2",
        muted: "text-content-3",
        accent: "text-accent-500",
        danger: "text-danger-500",
        onMedia: "text-white",
        onMediaMuted: "text-white/60",
      },
      reveal: {
        always: "",
        groupHover: "opacity-0 group-hover/tile:opacity-100 group-focus-within/tile:opacity-100",
      },
      rotate: {
        none: "",
        ccw90: "-rotate-90 duration-[100ms]",
      },
    },
    defaultVariants: { size: "md", weight: "300", tone: "muted", reveal: "always", rotate: "none" },
  },
);

/** The 32px tint box behind a sheet glyph. */
export const iconBadgeVariants = cva(
  "pointer-events-none inline-flex h-[2.2857rem] w-[2.2857rem] shrink-0 items-center justify-center rounded-(--radius-control)",
  {
    variants: {
      tint: {
        none: "",
        accent: "bg-selection-1",
        danger: "bg-danger-100",
      },
    },
    defaultVariants: { tint: "none" },
  },
);

export interface IconProps extends React.HTMLAttributes<HTMLSpanElement> {
  /** Material Symbols ligature name. Empty renders nothing. */
  name?: string;
  /** Given, the glyph is role img with this name; otherwise it is decorative. */
  label?: string;
  /** Named size (default md) or an exact px number (for a measured glyph). A tint fixes the glyph at lg (18px). */
  size?: IconSize | number;
  /** 200, 300 or 400 (default 300); another number is used as it is (a measured weight). */
  weight?: number;
  /** Glyph colour; ignored when tint is not none. */
  tone?: IconTone;
  /** A 32px box behind the glyph; the glyph takes the tint colour and an 18px size. */
  tint?: IconTint;
  reveal?: IconReveal;
  rotate?: IconRotate;
  /** Replaces the default data-slot of the root ("glyph", or "badge" with a tint). */
  "data-slot"?: string;
}

export const Icon = React.forwardRef<HTMLSpanElement, IconProps>(function Icon(
  {
    name = "", label = "", size, weight = 300, tone = "muted", tint = "none", reveal = "always", rotate = "none",
    className, "data-slot": dataSlot, ...props
  },
  ref,
) {
  if (name === "") return null;

  // A tint fixes the glyph at 18px (lg) inside its 32px box; the size is still validated.
  const requested = resolveIconSize(size ?? "md");
  const glyphSize: IconSize = tint === "none" ? requested : "lg";
  const glyphTone = tint === "none" ? tone : tint;
  const glyphWeight = String(nearest(weight, WEIGHTS)) as IconWeightKey;
  const glyphClass = iconVariants({ size: glyphSize, weight: glyphWeight, tone: glyphTone, reveal, rotate });
  // A measured px size or weight that is not one of the steps is kept exactly, inline.
  const exact: React.CSSProperties = {
    ...(typeof size === "number" && tint === "none" && size !== ICON_SIZES[glyphSize] ? { fontSize: size } : {}),
    ...(!(WEIGHTS as readonly number[]).includes(weight) ? { fontVariationSettings: `'wght' ${weight}` } : {}),
  };
  // The glyph part stays a generic box; a label is announced by an img inside it, and no label hides it.
  const content = label ? (
    <span role="img" aria-label={label}>
      {name}
    </span>
  ) : (
    name
  );

  if (tint === "none") {
    return (
      <span ref={ref} data-slot={dataSlot ?? "glyph"} aria-hidden={label ? undefined : true} className={cn(glyphClass, className)} {...props} style={{ ...exact, ...props.style }}>
        {content}
      </span>
    );
  }

  return (
    <span ref={ref} data-slot={dataSlot ?? "badge"} className={cn(iconBadgeVariants({ tint }), className)} {...props}>
      <span data-slot="glyph" aria-hidden={label ? undefined : true} className={glyphClass} style={exact}>
        {content}
      </span>
    </span>
  );
});
