import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/utils";
import { Icon } from "./icon";

/**
 * Display-only glyph sequence for one accelerator (shadcn Kbd pattern, no Radix primitive).
 * Modifiers then key, edge to edge. Sizes at 1rem = 14px: sm 12px glyphs and text, md 13px.
 */
const hotkeyHintVariants = cva("inline-flex items-center whitespace-nowrap font-sans font-light select-none", {
  variants: {
    size: {
      sm: "text-[0.857rem] leading-[0.857rem]",
      md: "text-[0.929rem] leading-[1.286rem]",
    },
    /** inherit takes the parent's text colour (currentColor); muted is the muted-foreground token. */
    tone: {
      inherit: "",
      muted: "text-content-3",
    },
  },
  defaultVariants: { size: "sm", tone: "inherit" },
});

type Variants = VariantProps<typeof hotkeyHintVariants>;
type Platform = "macos" | "windows" | "other";

/** A glyph (Material Symbols name) or, where no glyph is asserted, a text label. */
interface Token {
  glyph?: string;
  text?: string;
  /** Spoken equivalent for the visually hidden text. */
  label: string;
}

const MODIFIERS: Record<Platform, Record<string, Token>> = {
  macos: {
    Meta: { glyph: "keyboard_command_key", label: "Command" },
    Alt: { glyph: "keyboard_option_key", label: "Option" },
    Control: { glyph: "keyboard_control_key", label: "Control" },
    Shift: { glyph: "arrow_upward", label: "Shift" },
  },
  // Windows and web: Meta is the Control glyph, Alt is the Alt label.
  windows: {
    Meta: { glyph: "keyboard_control_key", label: "Control" },
    Alt: { text: "Alt", label: "Alt" },
    Control: { glyph: "keyboard_control_key", label: "Control" },
    Shift: { glyph: "arrow_upward", label: "Shift" },
  },
  // Linux: the glyph map is not in the brief, so every modifier is a text label.
  other: {
    Meta: { text: "Super", label: "Super" },
    Alt: { text: "Alt", label: "Alt" },
    Control: { text: "Ctrl", label: "Control" },
    Shift: { text: "Shift", label: "Shift" },
  },
};

const KEYS: Record<string, Token> = {
  Enter: { glyph: "keyboard_return", label: "Return" },
  Return: { glyph: "keyboard_return", label: "Return" },
  Backspace: { glyph: "backspace", label: "Backspace" },
  ArrowUp: { glyph: "arrow_upward", label: "Up" },
  ArrowLeft: { glyph: "arrow_back", label: "Left" },
  Comma: { text: ",", label: "Comma" },
  Backslash: { text: "\\", label: "Backslash" },
  Minus: { text: "-", label: "Minus" },
};

function resolvePlatform(platform: string): Platform {
  if (platform === "macos" || platform === "windows" || platform === "other") return platform;
  // Empty: resolve from the host. The web follows the Windows rule, so only Apple hosts differ.
  if (typeof navigator !== "undefined" && /Macintosh|Mac OS X/.test(navigator.userAgent)) return "macos";
  return "windows";
}

function modifierToken(name: string, platform: Platform): Token {
  return MODIFIERS[platform][name] ?? { text: name, label: name };
}

function keyToken(key: string): Token {
  if (KEYS[key]) return KEYS[key];
  const text = key.length === 1 ? key.toUpperCase() : key;
  return { text, label: text };
}

/** Glyphs run edge to edge; a text modifier (Alt, Shift) gets a small trailing gap so it reads as a word. */
function Glyph({ token, size, word = false }: { token: Token; size: number; word?: boolean }) {
  return (
    <span aria-hidden="true" className={cn("inline-flex items-center", word && token.text && "mr-[0.214rem]")}>
      {token.glyph ? (
        <Icon name={token.glyph} size={size} weight={300} tone="inherit" />
      ) : (
        <span>{token.text}</span>
      )}
    </span>
  );
}

export interface HotkeyHintProps extends React.HTMLAttributes<HTMLElement> {
  /** Canonical accelerator, modifiers then key, joined with "+", for example Meta+Alt+N. Empty renders nothing. */
  accel?: string;
  size?: Variants["size"];
  tone?: Variants["tone"];
  /** Host OS for glyph resolution; "" resolves from the app context. */
  platform?: Platform | "";
}

export const HotkeyHint = React.forwardRef<HTMLElement, HotkeyHintProps>(function HotkeyHint(
  { accel = "", size = "sm", tone = "inherit", platform = "", className, ...props },
  ref,
) {
  const tokens = accel.split("+").map((part) => part.trim()).filter(Boolean);
  if (tokens.length === 0) return null;

  const host = resolvePlatform(platform);
  const key = tokens[tokens.length - 1];
  const modifiers = tokens.slice(0, -1).map((name) => modifierToken(name, host));
  const keyTok = keyToken(key);
  const glyphSize = size === "md" ? 13 : 12;
  const spoken = [...modifiers.map((token) => token.label), keyTok.label].join(" ");

  return (
    <kbd
      ref={ref}
      {...props}
      data-slot="hint"
      className={cn(hotkeyHintVariants({ size, tone }), className)}
    >
      <span className="sr-only">{spoken}</span>
      {modifiers.map((token, index) => (
        <Glyph key={index} token={token} size={glyphSize} word />
      ))}
      <Glyph token={keyTok} size={glyphSize} />
    </kbd>
  );
});

export { hotkeyHintVariants };
