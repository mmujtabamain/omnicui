import * as React from "react";
import { cva } from "class-variance-authority";
import { cn } from "../../lib/utils";
import { HotkeyHint } from "./hotkey-hint";
import { Icon } from "./icon";

const isMac = typeof navigator !== "undefined" && /Macintosh|Mac OS X/.test(navigator.userAgent);

/** Mod is Command on macOS and Control elsewhere; HotkeyHint takes the canonical modifier names. */
function canonicalAccel(shortcut: string): string {
  return shortcut
    .split("+")
    .map((part) => {
      const name = part.trim();
      return name === "Mod" ? (isMac ? "Meta" : "Control") : name;
    })
    .join("+");
}

/**
 * One option row in a listbox whose DOM focus stays in a text field (palette search input or
 * the mention editor). Geometry at 1rem = 14px: palette row 30px high (32px with a hotkey),
 * 9px side padding, 16px icon, 7px gap, 13px label; mention icon 15px, weight 300.
 */
export const comboboxItemVariants = cva(
  "relative flex w-full min-w-0 cursor-default select-none items-center gap-[0.5rem] rounded-(--radius-field) px-[0.643rem] text-[0.929rem] leading-[1.393rem] font-normal outline-none",
  {
    variants: {
      context: {
        palette: "text-content-1",
        mention: "text-content-2",
      },
      shortcut: {
        none: "h-[2.143rem]",
        hotkey: "h-[2.286rem]",
      },
      active: {
        true: "bg-selection-1",
        false: "",
      },
    },
    defaultVariants: { context: "palette", shortcut: "none", active: false },
  },
);

export interface ComboboxItemProps extends Omit<React.HTMLAttributes<HTMLDivElement>, "id" | "onSelect"> {
  /** palette | mention: sets geometry, highlight token and commit behaviour. */
  context?: "palette" | "mention";
  /** Leading Material Symbols name: 16px in palette, 15px weight 300 in mention. */
  icon?: string;
  /** Row text; truncates. */
  label?: string;
  /** Text the palette filter matches; never rendered on the row. */
  value?: string;
  /** Option id, referenced by aria-activedescendant. */
  id?: string;
  /** e.g. Mod+Shift+Period; palette only. Empty means no trailing element. */
  shortcut?: string;
  /** The highlighted row. */
  active?: boolean;
  /** Palette: runs the command or jump. Mention: inserts the attachment chip and trailing space. */
  onSelect?: () => void;
  /** Pointer movement over an inactive row asks the list to make it active. */
  onHighlight?: () => void;
}

export const ComboboxItem = React.forwardRef<HTMLDivElement, ComboboxItemProps>(function ComboboxItem(
  {
    context = "palette",
    icon = "",
    label = "",
    value: _value,
    id = "",
    shortcut = "",
    active = false,
    onSelect,
    onHighlight,
    className,
    onMouseDown,
    onClick,
    onPointerMove,
    ...props
  },
  ref,
) {
  const hasShortcut = context === "palette" && shortcut !== "";
  const mention = context === "mention";

  return (
    <div
      ref={ref}
      role="option"
      aria-selected={active}
      aria-label={label || undefined}
      id={id || undefined}
      data-slot="row"
      data-context={context}
      data-active={active || undefined}
      onMouseDown={(event: React.MouseEvent<HTMLDivElement>) => {
        // Pressing keeps focus in the text field; mention commits on press.
        event.preventDefault();
        onMouseDown?.(event);
        if (mention) onSelect?.();
      }}
      onClick={(event: React.MouseEvent<HTMLDivElement>) => {
        onClick?.(event);
        if (!mention) onSelect?.();
      }}
      onPointerMove={(event: React.PointerEvent<HTMLDivElement>) => {
        onPointerMove?.(event);
        if (!active) onHighlight?.();
      }}
      className={cn(
        comboboxItemVariants({
          context,
          shortcut: hasShortcut ? "hotkey" : "none",
          active,
        }),
        mention && active && "text-content-1",
        className,
      )}
      {...props}
    >
      {icon !== "" && (
        <Icon
          data-slot="icon"
          name={icon}
          size={mention ? 15 : 16}
          weight={mention ? 300 : 350}
          tone="inherit"
          className="shrink-0"
        />
      )}
      <span data-slot="label" className="min-w-0 flex-1 truncate">
        {label}
      </span>
      {hasShortcut && (
        <span data-slot="shortcut" className="shrink-0">
          <HotkeyHint
            accel={canonicalAccel(shortcut)}
            size="sm"
            tone="muted"
            platform={isMac ? "macos" : "other"}
          />
        </span>
      )}
    </div>
  );
});
