import * as React from "react";
import { cva } from "class-variance-authority";
import { cn } from "../../lib/utils";
import { Icon } from "./icon";

/** Sizes are the measured ones (1rem = 14px). Tone inherit takes the parent's text colour. */
export const hotkeyHintVariants = cva(
  "inline-flex items-center whitespace-nowrap font-sans font-light select-none",
  {
    variants: {
      size: {
        sm: "text-[0.857rem] leading-[0.857rem]",
        md: "text-[0.929rem] leading-[1.286rem]",
      },
      tone: {
        inherit: "",
        muted: "text-content-3",
      },
    },
    defaultVariants: { size: "sm", tone: "inherit" },
  },
);

export type HotkeyHintOs = "macos" | "windows" | "other";

export interface HotkeyHintProps extends React.HTMLAttributes<HTMLElement> {
  /** Canonical accelerator, modifiers then key, joined with "+". Empty renders nothing. */
  accel?: string;
  size?: "sm" | "md";
  tone?: "inherit" | "muted";
  /** "" resolves the host OS from the browser. */
  platform?: HotkeyHintOs | "";
}

type Glyph = { icon?: string; text?: string; label: string };

const GLYPH_SIZE = { sm: 12, md: 13 } as const;

/** Host OS for platform="". The web follows the Windows rules; Linux is "other". */
function hostPlatform(): HotkeyHintOs {
  if (typeof navigator === "undefined") return "windows";
  const id = navigator.platform || navigator.userAgent || "";
  if (/Mac|iPhone|iPad/.test(id)) return "macos";
  if (/Linux/.test(id) && !/Android/.test(id)) return "other";
  return "windows";
}

function modifierGlyph(name: string, os: HotkeyHintOs): Glyph | null {
  switch (name) {
    case "Meta":
      if (os === "macos") return { icon: "keyboard_command_key", label: "Command" };
      if (os === "windows") return { text: "Ctrl", label: "Control" };
      return { text: "Meta", label: "Meta" };
    case "Alt":
      if (os === "macos") return { icon: "keyboard_option_key", label: "Option" };
      return { text: "Alt", label: "Alt" };
    case "Control":
      return { text: "Ctrl", label: "Control" };
    case "Shift":
      if (os === "macos") return { icon: "arrow_upward", label: "Shift" };
      return { text: "Shift", label: "Shift" };
    default:
      return null;
  }
}

const KEY_ICONS: Record<string, string> = {
  Enter: "keyboard_return",
  Backspace: "backspace",
  ArrowUp: "arrow_upward",
  ArrowLeft: "arrow_back",
};

const KEY_TEXT: Record<string, { text: string; label: string }> = {
  Comma: { text: ",", label: "comma" },
  Backslash: { text: "\\", label: "backslash" },
  Minus: { text: "-", label: "minus" },
  Escape: { text: "Esc", label: "Escape" },
  Space: { text: "Space", label: "Space" },
  Tab: { text: "Tab", label: "Tab" },
};

function keyGlyph(key: string): Glyph {
  if (KEY_ICONS[key]) return { icon: KEY_ICONS[key], label: key };
  if (KEY_TEXT[key]) return KEY_TEXT[key];
  return { text: key.length === 1 ? key.toUpperCase() : key, label: key.length === 1 ? key.toUpperCase() : key };
}

export const HotkeyHint = React.forwardRef<HTMLElement, HotkeyHintProps>(function HotkeyHint(
  { accel = "", size = "sm", tone = "inherit", platform = "", className, ...props },
  ref,
) {
  const names = accel.split("+").map((s) => s.trim()).filter(Boolean);
  if (names.length === 0)
    // Nothing to show: the hint stays in the tree as an empty, hidden kbd.
    return <kbd ref={ref} {...props} data-slot="hint" hidden />;

  const os: HotkeyHintOs = platform === "" ? hostPlatform() : platform;
  const key = names[names.length - 1];
  const glyphs: Glyph[] = [
    ...names.slice(0, -1).map((name) => modifierGlyph(name, os)).filter((g): g is Glyph => g !== null),
    keyGlyph(key),
  ];
  const spoken = glyphs.map((g) => g.label).join(" ");
  const glyphSize = GLYPH_SIZE[size === "md" ? "md" : "sm"];

  return (
    <kbd
      ref={ref}
      {...props}
      data-slot="hint"
      className={cn(hotkeyHintVariants({ size, tone }), className)}
    >
      {glyphs.map((g, i) => (
        <span
          key={i}
          aria-hidden="true"
          className={cn("inline-flex items-center justify-center", g.text !== undefined && "px-px")}
        >
          {g.icon ? <Icon name={g.icon} size={glyphSize} weight={300} tone="inherit" /> : g.text}
        </span>
      ))}
      <span className="sr-only">{spoken}</span>
    </kbd>
  );
});
