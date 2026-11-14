import * as React from "react";
import { cn } from "../../lib/utils";

export interface IconProps extends React.HTMLAttributes<HTMLSpanElement> {
  /** Material Symbols ligature name, e.g. "add". */
  name: string;
  /** Glyph size in px. */
  size?: number;
  filled?: boolean;
  /** Font weight axis; defaults to the theme's --icon-wght. */
  weight?: number;
}

/** A Material Symbols (Outlined) glyph. Decorative: hidden from assistive technology. */
export function Icon({ name, size = 16, filled = false, weight, className, style, ...props }: IconProps) {
  return (
    <span
      data-slot="icon"
      aria-hidden
      className={cn("inline-block shrink-0 select-none leading-none whitespace-nowrap font-normal not-italic normal-case tracking-normal [font-family:'Material_Symbols_Outlined'] [font-feature-settings:'liga'] [direction:ltr]", className)}
      style={{ fontSize: size, width: size, height: size, fontVariationSettings: `'FILL' ${filled ? 1 : 0}, 'wght' ${weight ?? "var(--icon-wght, 350)"}`, ...style }}
      {...props}
    >
      {name}
    </span>
  );
}
